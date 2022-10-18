use crate::win;
use std::fmt;
use winapi::um::winuser::VK_F1;

const POE_EXE: &'static str = "PathOfExile";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Percent(f32),
    Flat(i32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
}

pub fn parse_value(word: &str) -> Result<Value, &'static str> {
    let (percent, word) = if word.ends_with('%') {
        (true, word.trim_end_matches('%'))
    } else {
        (false, word)
    };

    match word.parse::<i32>() {
        Ok(value) => {
            if value < 0 {
                Err("the value can't be negative")
            } else if percent {
                if value > 100 {
                    Err("the percentage can't be bigger than 100")
                } else {
                    Ok(Value::Percent(value as f32 / 100.0))
                }
            } else {
                Ok(Value::Flat(value))
            }
        }
        Err(_) => Err("the value was not a valid number"),
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

pub fn parse_direction(word: &str) -> Result<Direction, &'static str> {
    Ok(match word {
        "up" => Direction::Up,
        "down" => Direction::Down,
        _ => return Err("direction can only be up or down"),
    })
}

pub fn open_poe() -> Option<win::proc::Process> {
    win::proc::Process::open_by_name(POE_EXE)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percent(percent) => write!(f, "{}%", (percent * 100.0) as i32),
            Self::Flat(flat) => flat.fmt(f),
        }
    }
}
