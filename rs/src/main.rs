mod action;

use crate::action::{ActionSet, GameState, PreRequirement};
use rshacks::checker::{
    FocusChecker, LogChecker, MemoryChecker, MemoryState, MouseChecker, ScreenChecker,
};
use rshacks::win;
use std::fs;
use std::io::{self, Write as _};
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

const DELAY: Duration = Duration::from_millis(10);
const TOO_LONG: Duration = Duration::from_millis(100);
const PTR_MAP_FILE: &str = "ptr.map";

const SUSPICIOUS_MAX_HEALTH: i32 = 12000;
const SUSPICIOUS_MAX_ENERGY: i32 = 15000;
const SUSPICIOUS_MAX_MANA: i32 = 12000;

// Since PoE 3.14 and 3.18 the base addresses have been 0x025F5988 to 0x02C51FA8,
// and all of them end at 0x8. Subsequent offsets have never changed since 3.16,
// but if they do, they would need to be found manually with other external tools.
// (They may have also worked before 3.16 but a different offset had been used then).
const MAX_BASE_ADDR_NUDGE: isize = 0x01000000;
const BASE_ADDR_NUDGE_STEP: isize = 0x10;
const REPORT_PROGRESS_EVERY: isize = 256;

/// Return true if the health, energy shield or mana values seem abnormal
/// (such as the life being negative or too much mana reservation).
fn suspicious_hp_or_mana(MemoryState { health, mana }: &MemoryState) -> bool {
    health.hp < 1
        || health.max_hp > SUSPICIOUS_MAX_HEALTH
        || health.hp > health.max_hp
        || health.unreserved_hp < 1
        || health.unreserved_hp > health.max_hp
        || health.es < 0
        || health.max_es > SUSPICIOUS_MAX_ENERGY
        || health.es > health.max_es
        || mana.mana < 0
        || mana.max_mana > SUSPICIOUS_MAX_MANA
        || mana.mana > mana.max_mana
        || mana.unreserved_mana < 0
        || mana.unreserved_mana > mana.max_mana
}

/// Try to obtain the health and mana values with the pointer map in use.
/// If possible and the values are not suspicious, ask the user if they match.
/// Return true if and only if the user interactively confirms these values.
fn try_confirm_valid_hp_or_mana(checker: &MemoryChecker) -> bool {
    let state = match checker.check() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if !suspicious_hp_or_mana(&state) {
        return win::prompt::ask(
            "possible values found",
            &format!("is this ok?: {:#?}", state),
        )
        .expect("search for new working ptr.map cancelled");
    }

    false
}

fn main() {
    match std::panic::catch_unwind(run) {
        Ok(()) => {}
        Err(err) => {
            if let Some(msg) = err.downcast_ref::<String>() {
                win::prompt::error("poe-hacks crashed!", msg);
            } else if let Some(msg) = err.downcast_ref::<&str>() {
                win::prompt::error("poe-hacks crashed!", msg);
            } else {
                win::prompt::error(
                    "poe-hacks crashed!",
                    "sorry, but there is no error information",
                );
            }

            std::process::exit(101);
        }
    }
}

const POE_EXE: &'static str = "PathOfExile";

fn run() {
    let mut args = std::env::args();
    let _program = args.next();
    let file = args.next().unwrap_or_else(|| "poe.key".into());

    let mut actions =
        ActionSet::from_file(&file).expect(&format!("failed to load action set from '{}'", file));
    eprintln!("loaded action set from '{}'", file);
    eprintln!("loaded {}", actions);

    eprintln!("waiting for right click...");
    while !win::keyboard::is_down(0x02) {
        sleep(DELAY);
    }
    while win::keyboard::is_down(0x02) {
        sleep(DELAY);
    }

    let process =
        Rc::new(win::proc::Process::open_by_name(POE_EXE).expect("could not find poe running"));

    let mut area_checker = if actions.requires(PreRequirement::Area) {
        eprintln!("initializing log checker");
        Some(LogChecker::new(Rc::clone(&process)))
    } else {
        None
    };
    let mut focus_checker = if actions.requires(PreRequirement::Focus) {
        eprintln!("initializing focus checker");
        Some(FocusChecker::new(Rc::clone(&process)))
    } else {
        None
    };
    let mut mouse_checker = if actions.requires(PreRequirement::Mouse) {
        eprintln!("initializing mouse checker");
        Some(MouseChecker::new())
    } else {
        None
    };
    let mut player_checker = if actions.requires(PreRequirement::Player) {
        eprintln!("initializing memory checker");

        let mut ptr_map = std::env::current_exe().expect("could not locate self file location");
        ptr_map.set_file_name(PTR_MAP_FILE);
        let mut checker = match MemoryChecker::load_ptr_map(&ptr_map, process.clone()) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                panic!("pointer .map file not found at: {}\n\nthe file must exist for the program to read the in-game ehp value", ptr_map.to_string_lossy());
            }
            Err(err) => {
                panic!("failed to initialize memory checker: {}", err);
            }
            Ok(checker) => checker,
        };

        match checker.check() {
            Ok(state) => {
                // pointer-map seems to work but may have been chance (unlikely) so check for abnormal values.
                // if abnormal values are found, crash (manually finding the new addresses is required).
                if suspicious_hp_or_mana(&state) {
                    panic!("current ptr.map did not fail but the values look wrong, manual fix required: {:#?}", state);
                }
            }
            Err(e) => {
                // pointer-map no longer works, try to fix the base address
                win::prompt::warn("current ptr.map is invalid", &format!("the current ptr.map does not work, so poe-hacks will try to fix it:\n{e}\n\nDO NOT CHANGE AREA WHILE THIS PROCESS RUNS!\n\nanother alert will show once the process completes (this can take a few minutes)"));
                write!(io::stderr(), "scanning for new base address...        \r").unwrap();

                let mut nudge_amount = 0;
                while nudge_amount < MAX_BASE_ADDR_NUDGE {
                    if nudge_amount % (REPORT_PROGRESS_EVERY * BASE_ADDR_NUDGE_STEP) == 0 {
                        write!(
                            io::stderr(),
                            "scaning for new base address... {:.2}% \r",
                            100.0 * (nudge_amount as f32 / MAX_BASE_ADDR_NUDGE as f32)
                        )
                        .unwrap();
                    }
                    // nudge the address in a "zig-zag" kind of way until we manage to read all the way through
                    nudge_amount += BASE_ADDR_NUDGE_STEP;
                    checker.nudge_map_base_addr(nudge_amount);
                    if try_confirm_valid_hp_or_mana(&checker) {
                        break;
                    }
                    checker.nudge_map_base_addr(nudge_amount * -2);
                    if try_confirm_valid_hp_or_mana(&checker) {
                        break;
                    }

                    // reset the base address to its original value after every iteration
                    // so it's left unmodified if things don't work out
                    checker.nudge_map_base_addr(nudge_amount);
                }
                write!(io::stderr(), "scanning for new base address... complete\n").unwrap();

                if nudge_amount < MAX_BASE_ADDR_NUDGE {
                    let timestamp = chrono::Local::now().format("%Y%m%d.%H%M%S.map").to_string();
                    win::prompt::info("new base address found", &format!("a new working base address was found at an offset of {:08X}\n\na copy of the ptr.map will be saved to ptr.{}, and the current one will be updated", nudge_amount, timestamp));
                    fs::rename(&ptr_map, ptr_map.with_extension(timestamp))
                        .expect("failed to backup existing ptr.map");
                    checker
                        .save_ptr_map(ptr_map)
                        .expect("failed to save updated ptr.map");
                } else {
                    panic!("ptr.map could not be updated, manual fix required");
                }
            }
        }

        Some(checker)
    } else {
        None
    };
    let mut screen_checker = if actions.requires(PreRequirement::Screen) {
        eprintln!("initializing checker checker");
        Some(ScreenChecker::new())
    } else {
        None
    };

    println!("poe-hacks is now running");
    let mut last = Instant::now();
    loop {
        let now = Instant::now();
        if (now - last) > TOO_LONG {
            eprintln!("warning: check is taking too long: {:?}", now - last);
        }
        last = now;
        sleep(DELAY);

        // TODO could skip checkers that are disabled (say, chat screen only needed sometimes)
        let state = GameState {
            area: area_checker.as_mut().and_then(|checker| {
                checker
                    .check()
                    .inspect_err(|e| eprintln!("warning: failed to refresh area checker: {e}"))
                    .ok()
            }),
            focus: focus_checker.as_mut().and_then(|checker| {
                checker
                    .check()
                    .inspect_err(|e| eprintln!("warning: failed to refresh focus checker: {e}"))
                    .ok()
            }),
            mouse: mouse_checker.as_mut().map(|checker| checker.check()),
            player: player_checker.as_mut().and_then(|checker| {
                checker
                    .check()
                    .inspect_err(|e| eprintln!("warning: failed to refresh player checker: {e}"))
                    .ok()
            }),
            screen: screen_checker.as_mut().map(|checker| checker.check()),
        };

        actions.check_all(&state, process.as_ref());
    }
}
