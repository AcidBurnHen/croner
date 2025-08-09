use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::models::{Fanout, JobSpec};
use crate::parser::CronParser;
use crate::shell::get_command_as_os_str;

pub struct ConfigCache {
    pub jobs: Vec<JobSpec>,
    last_modified: Option<SystemTime>,
    file_size: Option<u64>,
}

impl ConfigCache {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            last_modified: None,
            file_size: None,
        }
    }

    /// Atomically reloads config if the file changed (mtime+size).
    /// Returns true if reloaded, false if unchanged.
    pub fn reload_if_changed(&mut self, path: &Path) -> Result<bool, String> {
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(format!("config open error: {}", e)),
        };
        let meta = match file.metadata() {
            Ok(m) => m,
            Err(e) => return Err(format!("config metadata error: {}", e)),
        };
        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let size = meta.len();

        if self.last_modified == Some(modified) && self.file_size == Some(size) {
            return Ok(false);
        }

        let new_jobs = load_config(path)?;
        self.jobs = new_jobs;
        self.last_modified = Some(modified);
        self.file_size = Some(size);
        Ok(true)
    }
}

// State for the current [job:<id>] section while parsing
struct JobBuilder<'a> {
    id: &'a str,
    schedule: Option<&'a str>,
    command: Option<&'a str>,
    fanout_int: Option<usize>,
    fanout_list: Vec<&'a str>,
    first_line: usize,
}

pub fn load_config(path: &Path) -> Result<Vec<JobSpec>, String> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => return Err(format!("failed to read config: {}", e)),
    };

    let text = match std::str::from_utf8(&bytes) {
        Ok(t) => t,
        Err(_) => return Err("config is not valid UTF-8".to_string()),
    };

    let mut jobs: Vec<JobSpec> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut cur: Option<JobBuilder> = None;
    let mut cron = CronParser::new();

    let data = text.as_bytes();
    let n = data.len();
    let mut i = 0usize;
    let mut lineno = 0usize;

    // Skip UTF-8 BOM if present
    if n >= 3 && data[0..3] == [0xEF, 0xBB, 0xBF] {
        i = 3;
    }

    while i < n {
        lineno += 1;

        let line_start = i;
        while i < n && data[i] != b'\n' {
            i += 1;
        }
        let mut line_end = i;

        if i < n && data[i] == b'\n' {
            i += 1;
        }

        if line_end > line_start && data[line_end - 1] == b'\r' {
            line_end -= 1;
        }

        let mut line = &data[line_start..line_end];

        // Strip comment
        if let Some(hash) = memchr(line, b'#') {
            line = &line[..hash];
        }

        line = trim_ascii(line);
        if line.is_empty() {
            continue;
        }

        // Section header
        if let Some(id_slice) = parse_section_header(line) {
            if let Some(prev) = cur.take() {
                let start_line = prev.first_line;
                let job = match finalize_job(&mut cron, prev) {
                    Ok(j) => j,
                    Err(e) => return Err(format!("line {}: {}", start_line, e)),
                };

                if !seen_ids.insert(job.id.clone()) {
                    return Err(format!("duplicate job id '{}'", job.id));
                }
                jobs.push(job);
            }

            cur = Some(JobBuilder {
                id: id_slice,
                schedule: None,
                command: None,
                fanout_int: None,
                fanout_list: Vec::new(),
                first_line: lineno,
            });

            continue;
        }

        // key = value
        let (key, value) = match parse_key_value(line) {
            Some(kv) => kv,
            None => return Err(format!("line {}: expected `key = value`", lineno)),
        };

        let Some(b) = cur.as_mut() else {
            return Err(format!(
                "line {}: key outside of [job:<id>] section",
                lineno
            ));
        };

        match key {
            b"schedule" => {
                if b.schedule.is_some() {
                    return Err(format!("line {}: duplicate `schedule`", lineno));
                }
                let v = trim_ascii(value);
                let s = match std::str::from_utf8(v) {
                    Ok(s) => s,
                    Err(_) => return Err(format!("line {}: invalid UTF-8 in schedule", lineno)),
                };
                b.schedule = Some(s);
            }
            b"command" => {
                if b.command.is_some() {
                    return Err(format!("line {}: duplicate `command`", lineno));
                }
                let v = trim_ascii(value);
                if v.is_empty() {
                    return Err(format!("line {}: command cannot be empty", lineno));
                }
                let s = match std::str::from_utf8(v) {
                    Ok(s) => s,
                    Err(_) => return Err(format!("line {}: invalid UTF-8 in command", lineno)),
                };
                b.command = Some(s);
            }
            b"fanout" => {
                if !b.fanout_list.is_empty() {
                    return Err(format!(
                        "line {}: `fanout` conflicts with `fanout[]`",
                        lineno
                    ));
                }
                if b.fanout_int.is_some() {
                    return Err(format!("line {}: duplicate `fanout`", lineno));
                }
                let s = match std::str::from_utf8(trim_ascii(value)) {
                    Ok(s) => s,
                    Err(_) => return Err(format!("line {}: invalid UTF-8 in fanout", lineno)),
                };
                let n: usize = match s.parse() {
                    Ok(n) => n,
                    Err(_) => return Err(format!("line {}: fanout must be an integer", lineno)),
                };
                b.fanout_int = Some(n);
            }
            b"fanout[]" => {
                if b.fanout_int.is_some() {
                    return Err(format!(
                        "line {}: `fanout[]` conflicts with `fanout`",
                        lineno
                    ));
                }
                let v = trim_ascii(value);
                if v.is_empty() {
                    return Err(format!("line {}: fanout[] value cannot be empty", lineno));
                }
                let s = match std::str::from_utf8(v) {
                    Ok(s) => s,
                    Err(_) => return Err(format!("line {}: invalid UTF-8 in fanout[]", lineno)),
                };
                b.fanout_list.push(s);
            }
            _ => {
                return Err(format!(
                    "line {}: unknown key {}",
                    lineno,
                    as_debug_str(key)
                ));
            }
        }
    }

    // finalize last section
    if let Some(prev) = cur.take() {
        let start_line = prev.first_line;
        let job = match finalize_job(&mut cron, prev) {
            Ok(j) => j,
            Err(e) => return Err(format!("line {}: {}", start_line, e)),
        };
        if !seen_ids.insert(job.id.clone()) {
            return Err(format!("duplicate job id '{}'", job.id));
        }
        jobs.push(job);
    }

    Ok(jobs)
}

fn finalize_job<'a>(cron: &mut CronParser, b: JobBuilder<'a>) -> Result<JobSpec, String> {
    let id = b.id.trim();
    if id.is_empty() {
        return Err("empty job id".into());
    }

    let schedule_str = match b.schedule {
        Some(s) => s,
        None => return Err(format!("job '{}': missing schedule", id)),
    };

    let schedule = match cron.parse(schedule_str) {
        Ok(s) => s,
        Err(e) => return Err(format!("job '{}': invalid schedule: {}", id, e)),
    };

    let command_str = match b.command {
        Some(c) => c,
        None => return Err(format!("job '{}': missing command", id)),
    };

    // Pre-parse base command once
    let base_cmd = get_command_as_os_str(command_str);

    // Build fanout plan
    let fanout = if let Some(n) = b.fanout_int {
        Fanout::Int(n)
    } else if !b.fanout_list.is_empty() {
        // Prepare full argv per fanout entry: base_cmd + parsed extras
        let extras: Vec<String> = b.fanout_list.into_iter().map(|s| s.to_string()).collect();
        let list = crate::models::JobSpec::build_fanout_list_from_strings(&base_cmd, &extras);
        Fanout::List(list)
    } else {
        Fanout::None
    };

    Ok(JobSpec {
        id: id.to_string(),
        schedule,
        base_cmd,
        fanout,
    })
}

#[inline]
fn trim_ascii(mut s: &[u8]) -> &[u8] {
    while let Some(&b) = s.first() {
        if !is_ascii_ws(b) {
            break;
        }
        s = &s[1..];
    }
    while let Some(&b) = s.last() {
        if !is_ascii_ws(b) {
            break;
        }
        s = &s[..s.len() - 1];
    }
    s
}

#[inline]
fn is_ascii_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
}

#[inline]
fn parse_section_header(line: &[u8]) -> Option<&str> {
    // Accept exactly: [job:<id>]
    if line.len() >= 7 && line.starts_with(b"[job:") && line.ends_with(b"]") {
        let inner = &line[5..line.len() - 1];
        let id_bytes = trim_ascii(inner);
        if id_bytes.is_empty() {
            return None;
        }
        return std::str::from_utf8(id_bytes).ok();
    }
    None
}

#[inline]
fn memchr(hay: &[u8], needle: u8) -> Option<usize> {
    for (i, &b) in hay.iter().enumerate() {
        if b == needle {
            return Some(i);
        }
    }
    None
}

#[inline]
fn parse_key_value(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let eq = memchr(line, b'=')?;
    let key = trim_ascii(&line[..eq]);
    let val = trim_ascii(&line[eq + 1..]);
    Some((key, val))
}

#[inline]
fn as_debug_str(key: &[u8]) -> String {
    match std::str::from_utf8(key) {
        Ok(s) => s.to_string(),
        Err(_) => format!("{:?}", key),
    }
}
