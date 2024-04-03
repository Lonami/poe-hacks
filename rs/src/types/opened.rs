use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Opened {
    #[default]
    Closed,
    Open,
}

impl FromStr for Opened {
    type Err = &'static str;

    fn from_str(word: &str) -> Result<Self, Self::Err> {
        Ok(match word {
            "open" | "opened" => Self::Open,
            "close" | "closed" => Self::Closed,
            _ => return Err("opened can only be open or closed"),
        })
    }
}

impl fmt::Display for Opened {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Open => "opened",
            Self::Closed => "closed",
        })
    }
}
