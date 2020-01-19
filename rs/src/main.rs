mod action;
mod https;

use crate::action::ActionSet;
use rshacks::input;
use std::thread::sleep;
use std::time::Duration;

const DELAY: Duration = Duration::from_millis(10);

fn main() {
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
    loop {
        sleep(DELAY);
        actions.check_all();
    }
}
