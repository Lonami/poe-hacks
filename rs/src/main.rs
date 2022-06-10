mod action;
mod https;
mod utils;

use crate::action::{ActionSet, Checker as _, MemoryChecker};
use rshacks::win;
use std::io;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[macro_use]
extern crate lazy_static;

const DELAY: Duration = Duration::from_millis(10);
const TOO_LONG: Duration = Duration::from_millis(100);
const PTR_MAP_FILE: &str = "ptr.map";

fn main() {
    win::screen::register_window_class().expect("failed to register window class for tooltips");

    let checker = match std::env::current_exe() {
        Ok(mut file) => {
            file.set_file_name(PTR_MAP_FILE);
            match MemoryChecker::load_ptr_map(&file) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    panic!("no ptr.map file found");
                }
                Err(err) => {
                    panic!("failed to read ptr.map file: {}", err);
                }
                Ok(checker) => checker,
            }
        }
        Err(_) => {
            panic!("failed to detect current exe");
        }
    };

    eprintln!("waiting for right click...");
    while !win::keyboard::is_down(0x02) {
        sleep(DELAY);
    }
    while win::keyboard::is_down(0x02) {
        sleep(DELAY);
    }

    let mut args = std::env::args();
    let _program = args.next();
    let file = args.next().unwrap_or_else(|| "poe.key".into());

    // TODO make an audible noise or show a window if this fails to let the user know that poehacks is NOT running
    let mut actions = ActionSet::from_file(&file, checker)
        .expect(&format!("failed to load action set from '{}'", file));
    eprintln!("loaded action set from '{}'", file);

    eprintln!("loaded {}", actions);
    println!("poe-hacks is now running");
    let mut last = Instant::now();
    loop {
        let now = Instant::now();
        if (now - last) > TOO_LONG {
            eprintln!("warning: check is taking too long: {:?}", now - last);
        }
        last = now;
        // Taking a new "screenshot" of the entire screen takes ~30ms...
        // But if we don't sleep at all we use too much CPU.
        // 30ms is still better than 16ms **per point** we had with `GetPixel` anyway.
        sleep(DELAY);
        match actions.checker.refresh() {
            Ok(_) => actions.check_all(),
            Err(e) => eprintln!("warning: failed to refresh checker: {}", e),
        }
    }
}
