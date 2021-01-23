mod action;
mod https;
mod utils;

use crate::action::{ActionSet, Checker};
use rshacks::{win};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[macro_use]
extern crate lazy_static;

const DELAY: Duration = Duration::from_millis(10);
const TOO_LONG: Duration = Duration::from_millis(100);

fn main() {
    let screen = win::screen::Screen::new().expect("failed to open screen");
    win::screen::register_window_class().expect("failed to register window class for tooltips");

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

    let mut actions = ActionSet::from_file(&file, screen).expect(&format!("failed to load action set from '{}'", file));
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
            Err(_) => eprintln!("warning: failed to refresh screen"),
        }
    }
}
