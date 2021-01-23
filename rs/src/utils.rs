use crate::win;
use winapi::um::winuser::VK_F1;

const POE_EXE: &'static str = "PathOfExile";

pub fn parse_percentage(word: &str) -> Result<f64, &'static str> {
    match word.trim_end_matches('%').parse::<isize>() {
        Ok(value) => {
            if value < 0 {
                Err("the percentage can't be negative")
            } else if value > 100 {
                Err("the percentage can't be bigger than 100")
            } else {
                Ok(value as f64 / 100.0)
            }
        }
        Err(_) => Err("the percentage was not a valid number"),
    }
}

pub fn parse_vk(word: &str) -> Result<u16, &'static str> {
    if word.starts_with("0x") {
        u16::from_str_radix(&word[2..], 16).map_err(|_| "got invalid hex virtual key code")
    } else if word.len() != 1 {
        if word.starts_with("f") {
            match word[1..].parse::<u8>() {
                Ok(n) => Ok(((VK_F1 - 1) + n as i32) as u16),
                Err(_) => Err("invalid integer value for fn key"),
            }
        } else {
            Err("cannot map more than one character to a virtual key code unless it's a fn key")
        }
    } else {
        Ok(win::keyboard::get_vk(word.as_bytes()[0]))
    }
}

pub fn open_poe() -> Option<win::proc::Process> {
    win::proc::Process::open_by_name(POE_EXE)
}
