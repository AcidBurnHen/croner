use std::ffi::OsString;
use std::fmt::format;

use crate::parser::CronSchedule;
use crate::shell::get_command_as_os_str;

#[derive(Debug, Clone)]
pub struct JobInstance {
    pub id: String,
    pub command: Vec<OsString>,
}

#[derive(Debug, Clone)]
pub enum Fanout {
    Int(usize),
    List(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct JobSpec {
    pub id: String,
    pub schedule: CronSchedule,
    pub command: String,
    pub fanout: Option<Fanout>,
}

impl JobSpec {
    pub fn expand(&self) -> Vec<JobInstance> {
        match &self.fanout {
            None => {
                let cmd = get_command_as_os_str(&self.command);

                vec![JobInstance {
                    id: self.id.clone(),
                    command: cmd,
                }]
            }

            Some(Fanout::List(list)) => list
                .iter()
                .enumerate()
                .map(|(i, args)| {
                    let mut combined = String::with_capacity(self.command.len() + 1 + args.len());
                    combined.push_str(&self.command);
                    combined.push(' ');
                    combined.push_str(args);

                    let cmd = get_command_as_os_str(&combined);
                    JobInstance {
                        id: format!("{}-{}", self.id, i),
                        command: cmd,
                    }
                })
                .collect(),

            Some(Fanout::Int(n)) => {
                let cmd = get_command_as_os_str(&self.command);
                (0..*n)
                    .map(|i| JobInstance {
                        id: format!("{}-{}", self.id, i),
                        command: cmd.clone(),
                    })
                    .collect()
            }
        }
    }
}
