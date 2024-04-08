use rshacks::win::proc::Process;

use super::action::{Action, TriggerResult};
use super::pre::{GameState, PreRequirement};
use super::{PostCondition, PostResult};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

pub struct ActionSet {
    actions: Vec<Action>,
    inhibit_key_presses: bool,
    created: Instant,
}

impl ActionSet {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, &'static str> {
        let actions: Vec<Action> = match File::open(path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Err("poe key file not found");
            }
            Err(_) => {
                return Err("failed to open poe key file, lack of permissions?");
            }
            Ok(file) => BufReader::new(file)
                .lines()
                .map(|line| line.expect("failed to read file"))
                .flat_map(|line| match Action::from_line(&line) {
                    Ok(action) => action,
                    Err(message) => {
                        eprintln!("warning: skipping '{}' because {}", line, message);
                        None
                    }
                })
                .collect(),
        };

        Ok(ActionSet {
            actions,
            inhibit_key_presses: false,
            created: Instant::now(),
        })
    }

    pub fn requires(&self, requirement: PreRequirement) -> bool {
        self.actions
            .iter()
            .any(|action| action.pre.iter().any(|pre| pre.requires(requirement)))
    }

    pub fn check_all(&mut self, state: &GameState, process: &Process) {
        let actions = &mut self.actions;
        let inhibit_key_presses = &mut self.inhibit_key_presses;
        let skip_key_presses = *inhibit_key_presses;
        let created = &self.created;
        actions
            .iter_mut()
            .map(|a| {
                a.try_toggle(state);
                a
            })
            .filter(|a| !(skip_key_presses && matches!(a.post, PostCondition::PressKey { .. })))
            .filter(|a| a.check(state))
            .for_each(|a| match a.try_trigger(process) {
                TriggerResult::Success(result) => {
                    if !a.silent {
                        eprintln!("[{:?}] note: ran successfully: {}", created.elapsed(), a);
                    }
                    match result {
                        PostResult::SetKeySuppression { suppress } => {
                            *inhibit_key_presses = suppress;
                        }
                        PostResult::None => {}
                    }
                }
                TriggerResult::Failed { reason } => {
                    eprintln!(
                        "[{:?}] warning: run failed: {}: {}",
                        created.elapsed(),
                        a,
                        reason
                    );
                }
                TriggerResult::Queued => {
                    if !a.silent {
                        eprintln!("[{:?}] note: queued action: {}", created.elapsed(), a);
                    }
                }
                TriggerResult::Delayed => {}
            });
    }
}

impl fmt::Display for ActionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} actions for:", self.actions.len(),)?;
        for action in self.actions.iter() {
            write!(f, "\n- {}", action)?;
        }
        Ok(())
    }
}
