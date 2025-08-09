use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::cli_colors::CliColorPicker;
use crate::loader::ConfigCache;
use crate::models::JobSpec;
use crate::parser::CronSchedule;
use crate::printer::Printer;

pub struct Scheduler {
    queue: BinaryHeap<ScheduledJob>,
    cache: ConfigCache,
    printer: Printer,
    colors: CliColorPicker,
}

#[derive(Clone)]
struct ScheduledJob {
    when: Instant,
    job: Arc<JobSpec>,
}

impl PartialEq for ScheduledJob {
    fn eq(&self, other: &Self) -> bool {
        self.when.eq(&other.when)
    }
}
impl Eq for ScheduledJob {}
impl PartialOrd for ScheduledJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.when.cmp(&self.when))
    }
}
impl Ord for ScheduledJob {
    fn cmp(&self, other: &Self) -> Ordering {
        other.when.cmp(&self.when)
    }
}

impl Scheduler {
    pub fn new(cache: ConfigCache, printer: Printer) -> Self {
        Self {
            queue: BinaryHeap::new(),
            cache,
            printer,
            colors: CliColorPicker::new(),
        }
    }

    pub fn init(&mut self) {
        self.queue.clear();
        for job in &self.cache.jobs {
            self.queue.push(ScheduledJob {
                when: compute_next_run(&job.schedule),
                job: Arc::new(job.clone()),
            });
        }
    }

    pub fn run(&mut self, config_path: &Path) {
        loop {
            if let Ok(true) = self.cache.reload_if_changed(config_path) {
                self.init();
            }

            if let Some(sched_job) = self.queue.pop() {
                let now = Instant::now();
                if sched_job.when <= now {
                    self.run_job(&sched_job.job);
                    self.queue.push(ScheduledJob {
                        when: compute_next_run(&sched_job.job.schedule),
                        job: sched_job.job.clone(),
                    });
                } else {
                    let sleep_dur = sched_job.when.saturating_duration_since(now);
                    thread::sleep(sleep_dur);
                    self.queue.push(sched_job);
                }
            } else {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    fn run_job(&mut self, job: &Arc<JobSpec>) {
        let instances = job.expand();
        let color_code = self.colors.get(hash_id(&job.id));

        for instance in instances {
            // Join all parts of the command into a single string
            let full_cmd = instance
                .command
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            #[cfg(unix)]
            let mut cmd = Command::new("sh");
            #[cfg(unix)]
            cmd.arg("-c").arg(&full_cmd);

            #[cfg(windows)]
            let mut cmd = Command::new("cmd");
            #[cfg(windows)]
            cmd.arg("/C").arg(&full_cmd);

            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            let job_id = instance.id.clone();
            let printer = self.printer.clone();
            let color = color_code.to_string();

            match cmd.spawn() {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let p = printer.clone();
                        let c = color.clone();
                        let jid = job_id.clone();
                        thread::spawn(move || {
                            use std::io::{BufRead, BufReader};
                            for line in BufReader::new(stdout).lines().flatten() {
                                p.write(format!("{}[{}]\u{1b}[0m {}", c, jid, line));
                            }
                        });
                    }
                    if let Some(stderr) = child.stderr.take() {
                        let p = printer.clone();
                        let c = color.clone();
                        let jid = job_id.clone();
                        thread::spawn(move || {
                            use std::io::{BufRead, BufReader};
                            for line in BufReader::new(stderr).lines().flatten() {
                                p.write(format!("{}[{}]\u{1b}[0m {}", c, jid, line));
                            }
                        });
                    }
                }
                Err(e) => {
                    self.printer.write(format!(
                        "{}[{}]\u{1b}[0m failed to start: {}",
                        color, job_id, e
                    ));
                }
            }
        }
    }
}

/// Very fast hash for job IDs → color slot
pub fn hash_id(id: &str) -> usize {
    let mut hash = 0usize;
    for b in id.as_bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(*b as usize);
    }
    hash
}

pub fn compute_next_run(schedule: &CronSchedule) -> Instant {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();
    let minutes = secs / 60;
    let mut next_minutes = minutes + 1;

    loop {
        let total_minutes = next_minutes;
        let minute = (total_minutes % 60) as u8;
        let hour = ((total_minutes / 60) % 24) as u8;
        let days_since_epoch = total_minutes / (60 * 24);

        let weekday = ((days_since_epoch + 4) % 7) as u8;

        if (schedule.minute & (1 << minute) != 0)
            && (schedule.hour & (1 << hour) != 0)
            && (schedule.weekday & (1 << weekday) != 0)
        {
            let delta_secs = (next_minutes - minutes) * 60;
            return Instant::now() + Duration::from_secs(delta_secs);
        }

        next_minutes += 1;
    }
}
