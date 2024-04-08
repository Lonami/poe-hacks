use crate::win;
use std::io::Error;
use std::rc::Rc;
use win::proc::Process;

pub struct FocusChecker {
    process: Rc<Process>,
}

pub struct FocusState {
    pub in_foreground: bool,
}

impl FocusChecker {
    pub fn new(process: Rc<Process>) -> Self {
        Self { process }
    }

    pub fn check(&self) -> Result<FocusState, Error> {
        win::screen::get_foreground_pid().map(|pid| FocusState {
            in_foreground: pid == self.process.pid,
        })
    }
}
