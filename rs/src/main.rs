mod action;
mod https;
mod utils;

use crate::action::{ActionSet, Checker, MemoryChecker, ScreenChecker};
use rshacks::win;
use std::io;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[macro_use]
extern crate lazy_static;

const DELAY: Duration = Duration::from_millis(10);
const TOO_LONG: Duration = Duration::from_millis(100);
const PTR_MAP_FILE: &str = "ptr.map";

fn create_screen_checker() -> Box<dyn Checker> {
    let mut screen = win::screen::Screen::new().expect("failed to open screen");
    screen.refresh().expect("failed to refresh screen");
    Box::new(ScreenChecker::new(screen))
}

fn main() {
    win::screen::register_window_class().expect("failed to register window class for tooltips");

    let checker = match std::env::current_exe() {
        Ok(mut file) => {
            file.set_file_name(PTR_MAP_FILE);
            match MemoryChecker::load_ptr_map(&file) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    eprintln!("no ptr.map file, won't use memory checker");
                    create_screen_checker()
                }
                Err(err) => {
                    panic!(format!("failed to read ptr.map file: {}", err));
                }
                Ok(checker) => Box::new(checker),
            }
        }
        Err(_) => {
            eprintln!("could not detect current exe, won't use memory checker");
            create_screen_checker()
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
