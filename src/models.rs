use std::ffi::OsString;

use crate::parser::CronSchedule;
use crate::shell::get_command_as_os_str;

#[derive(Debug, Clone)]
pub struct JobInstance {
    pub id: String,
    pub command: Vec<OsString>,
}

/// Fanout is prepared for runtime:
/// - None: single run with base_cmd
/// - Int(n): run n times with base_cmd
/// - List(cmds): each entry is a full prebuilt command (base + extras)
#[derive(Debug, Clone)]
pub enum Fanout {
    None,
    Int(usize),
    List(Vec<Vec<OsString>>),
}

#[derive(Debug, Clone)]
pub struct JobSpec {
    pub id: String,
    pub schedule: CronSchedule,

    /// Pre-parsed base command tokens
    pub base_cmd: Vec<OsString>,

    /// Prepared fanout plan.
    pub fanout: Fanout,
}

impl JobSpec {
    /// Expand into concrete instances with already prepared argv vectors.
    pub fn expand(&self) -> Vec<JobInstance> {
        match &self.fanout {
            Fanout::None => {
                vec![JobInstance {
                    id: self.id.clone(),
                    command: self.base_cmd.clone(),
                }]
            }
            Fanout::Int(n) => {
                let mut out = Vec::with_capacity(*n);
                for i in 0..*n {
                    out.push(JobInstance {
                        id: format!("{}-{}", self.id, i),
                        command: self.base_cmd.clone(),
                    });
                }
                out
            }
            Fanout::List(cmds) => {
                let mut out = Vec::with_capacity(cmds.len());
                for (i, argv) in cmds.iter().enumerate() {
                    out.push(JobInstance {
                        id: format!("{}-{}", self.id, i),
                        command: argv.clone(),
                    });
                }
                out
            }
        }
    }

    /// Helper used by loader to build list fanouts efficiently (base + extra args).
    pub fn build_fanout_list_from_strings(
        base_cmd: &[OsString],
        extras: &[String],
    ) -> Vec<Vec<OsString>> {
        let mut out = Vec::with_capacity(extras.len());
        for e in extras {
            let extra_tokens = get_command_as_os_str(e);
            let mut combined = Vec::with_capacity(base_cmd.len() + extra_tokens.len());

            combined.extend(base_cmd.iter().cloned());
            combined.extend(extra_tokens);
            out.push(combined);
        }

        out
    }
}
