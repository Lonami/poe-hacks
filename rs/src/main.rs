mod action;
use crate::action::ActionSet;
use rshacks::input;
use std::thread::sleep;
use std::time::Duration;

const DELAY: Duration = Duration::from_millis(10);

fn main() {
    eprintln!("waiting for right click...");
    while !input::keyboard::is_down(0x02) {
        sleep(DELAY);
    }
    while input::keyboard::is_down(0x02) {
        sleep(DELAY);
    }

    let mut actions = match ActionSet::from_file("poe.key") {
        Ok(value) => value,
        Err(message) => {
            eprintln!("failed to load action set: {}", message);
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
