use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
}

impl FromStr for Direction {
    type Err = &'static str;

    fn from_str(word: &str) -> Result<Self, Self::Err> {
        Ok(match word {
            "up" => Self::Up,
            "down" => Self::Down,
            _ => return Err("direction can only be up or down"),
        })
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Up => "up",
            Self::Down => "down",
        })
    }
}
