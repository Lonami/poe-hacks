mod action;
mod https;

use crate::action::ActionSet;
use rshacks::{globals, input};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[macro_use]
extern crate lazy_static;

const DELAY: Duration = Duration::from_millis(10);
const TOO_LONG: Duration = Duration::from_millis(100);

fn main() {
    globals::new_screen();
    if input::screen::color(0, 0).is_err() {
        eprintln!("cannot get color from screen");
        return;
    }
    if input::screen::register_window_class().is_err() {
        eprintln!("failed to register window class for tooltips");
        return;
    }

    eprintln!("waiting for right click...");
    while !input::keyboard::is_down(0x02) {
        sleep(DELAY);
    }
    while input::keyboard::is_down(0x02) {
        sleep(DELAY);
    }

    let mut args = std::env::args();
    let _program = args.next();
    let file = args.next().unwrap_or_else(|| "poe.key".into());

    globals::refresh_screen().expect("failed to refresh screen");
    let mut actions = match ActionSet::from_file(&file) {
        Ok(value) => {
            eprintln!("loaded action set from '{}'", file);
            value
        }
        Err(message) => {
            eprintln!("failed to load action set from '{}': {}", file, message);
            return;
        }
    };

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
        match globals::refresh_screen() {
            Ok(_) => actions.check_all(),
            Err(_) => eprintln!("warning: failed to refresh screen"),
        }
    }
}
