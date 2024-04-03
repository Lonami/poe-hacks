use crate::win;

const POE_EXE: &'static str = "PathOfExile";

pub fn open_poe() -> Option<win::proc::Process> {
    win::proc::Process::open_by_name(POE_EXE)
}
