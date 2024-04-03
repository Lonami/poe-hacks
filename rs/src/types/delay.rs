use std::fmt;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Delay(pub Duration);

impl FromStr for Delay {
    type Err = String;

    fn from_str(word: &str) -> Result<Self, Self::Err> {
        let (number, factor) = if word.ends_with("ms") {
            (&word[..word.len() - 2], 1)
        } else if word.ends_with("s") {
            (&word[..word.len() - 1], 1000)
        } else if word == "0" {
            (word, 0)
        } else {
            return Err(format!("found unknown duration '{}' without ms", word));
        };

        Ok(Self(Duration::from_millis(match number.parse::<u64>() {
            Ok(value) => value * factor,
            Err(_) => return Err(format!("found unknown duration value '{}'", word)),
        })))
    }
}

impl fmt::Display for Delay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.0.as_millis())
    }
}
