use crate::win;
use std::fmt;
use std::str::FromStr;
use winapi::um::winuser::VK_F1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vk(pub u16);

impl FromStr for Vk {
    type Err = &'static str;

    fn from_str(word: &str) -> Result<Self, Self::Err> {
        if word.starts_with("0x") {
            u16::from_str_radix(&word[2..], 16)
                .map(Self)
                .map_err(|_| "got invalid hex virtual key code")
        } else if word.len() != 1 {
            if word.starts_with("f") {
                match word[1..].parse::<u8>() {
                    Ok(n) => Ok(Self(((VK_F1 - 1) + n as i32) as u16)),
                    Err(_) => Err("invalid integer value for fn key"),
                }
            } else {
                Err("cannot map more than one character to a virtual key code unless it's a fn key")
            }
        } else {
            Ok(Self(win::keyboard::get_vk(word.as_bytes()[0])))
        }
    }
}

impl fmt::Display for Vk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}
