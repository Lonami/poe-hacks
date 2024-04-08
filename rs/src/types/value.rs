use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Percent(f32),
    Flat(i32),
}

impl Value {
    pub fn above(&self, current: i32, max: i32) -> bool {
        match self {
            Value::Percent(percent) => current <= (percent * max as f32) as i32,
            Value::Flat(flat) => current <= *flat,
        }
    }
}

impl FromStr for Value {
    type Err = &'static str;

    fn from_str(word: &str) -> Result<Self, Self::Err> {
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
                        Ok(Self::Percent(value as f32 / 100.0))
                    }
                } else {
                    Ok(Self::Flat(value))
                }
            }
            Err(_) => Err("the value was not a valid number"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percent(percent) => write!(f, "{}%", (percent * 100.0) as i32),
            Self::Flat(flat) => flat.fmt(f),
        }
    }
}
