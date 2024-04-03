use crate::win;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseButton(pub win::mouse::Button);

impl FromStr for MouseButton {
    type Err = &'static str;

    fn from_str(word: &str) -> Result<Self, Self::Err> {
        Ok(Self(match word {
            "left" | "1" => win::mouse::Button::Left,
            "right" | "2" => win::mouse::Button::Right,
            "middle" | "3" => win::mouse::Button::Middle,
            _ => return Err("click can only be left, right or middle"),
        }))
    }
}

impl fmt::Display for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            win::mouse::Button::Left => "left",
            win::mouse::Button::Right => "right",
            win::mouse::Button::Middle => "middle",
        })
    }
}
