use super::{Checker, MemoryChecker, PostCondition, PreCondition};
use crate::utils;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

// Avoid spamming actions by default,
// or the server may send "too many actions" on accident.
const DEFAULT_ACTION_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug, PartialEq)]
struct Action {
    pre: PreCondition,
    post: PostCondition,
    last_trigger: Instant,
    delay: Duration,
    _source: String,
}

pub struct ActionSet {
    pub checker: MemoryChecker,
    actions: Vec<Action>,
}

impl Action {
    fn from_line(line: &str) -> Result<Option<Action>, String> {
        if line.starts_with("//") || line.chars().all(|c| c.is_whitespace()) {
            return Ok(None);
        }

        let mut pre: Option<PreCondition> = None;
        let mut post: Option<PostCondition> = None;
        let mut delay = DEFAULT_ACTION_DELAY;

        enum State {
            WaitKeyword,

            WaitPreKind,
            WaitLifeValue,
            WaitEsValue,
            WaitManaValue,
            WaitKeyValue,

            WaitPostKind,
            WaitPostValue,
            WaitPostRemaining,

            WaitDelayValue,
        }

        let mut state = State::WaitKeyword;
        let line = line.to_lowercase();
        for word in line.split_whitespace() {
            use State::*;
            state = match &state {
                WaitKeyword => match word {
                    "on" => WaitPreKind,
                    "do" => WaitPostKind,
                    "every" => WaitDelayValue,
                    _ => return Err(format!("found unexpected keyword '{}'", word)),
                },

                WaitPreKind => match word {
                    "life" => WaitLifeValue,
                    "es" => WaitEsValue,
                    "mana" => WaitManaValue,
                    "flask" | "key" | "skill" => WaitKeyValue,
                    _ => return Err(format!("found unknown condition '{}'", word)),
                },
                WaitLifeValue => {
                    let threshold = utils::parse_value(word)?;
                    pre = Some(PreCondition::LifeBelow { threshold });
                    WaitKeyword
                }
                WaitEsValue => {
                    let threshold = utils::parse_value(word)?;
                    pre = Some(PreCondition::EnergyBelow { threshold });
                    WaitKeyword
                }
                WaitManaValue => {
                    let threshold = utils::parse_value(word)?;
                    pre = Some(PreCondition::ManaBelow { threshold });
                    WaitKeyword
                }
                WaitKeyValue => {
                    pre = Some(PreCondition::KeyPress {
                        vk: utils::parse_vk(word)?,
                    });
                    WaitKeyword
                }

                WaitPostKind => match word {
                    "disconnect" => {
                        post = Some(PostCondition::Disconnect);
                        WaitKeyword
                    }
                    "flask" | "key" | "skill" => WaitPostValue,
                    "type" => {
                        post = Some(PostCondition::Type {
                            string: String::new(),
                        });
                        WaitPostRemaining
                    }
                    "price" => {
                        post = Some(PostCondition::ShowPrice);
                        WaitKeyword
                    }
                    "invite" => {
                        post = Some(PostCondition::InviteLast);
                        WaitKeyword
                    }
                    "destroy" => {
                        post = Some(PostCondition::Destroy);
                        WaitKeyword
                    }
                    "downscale" => {
                        post = Some(PostCondition::Downscaling { enable: true });
                        WaitKeyword
                    }
                    "upscale" => {
                        post = Some(PostCondition::Downscaling { enable: false });
                        WaitKeyword
                    }
                    _ => return Err(format!("found unknown action '{}'", word)),
                },
                WaitPostValue => {
                    post = Some(PostCondition::PressKey {
                        vk: utils::parse_vk(word)?,
                    });
                    WaitKeyword
                }
                WaitPostRemaining => {
                    match post {
                        Some(PostCondition::Type { ref mut string }) => {
                            if !string.is_empty() {
                                string.push(' ');
                            }
                            string.push_str(word);
                        }
                        _ => return Err(format!("cannot parse remaining unless action is typing")),
                    }
                    WaitPostRemaining
                }

                WaitDelayValue => {
                    if !word.ends_with("ms") {
                        return Err(format!("found unknown duration '{}' without ms", word));
                    }
                    delay = Duration::from_millis(match word[..word.len() - 2].parse() {
                        Ok(value) => value,
                        Err(_) => return Err(format!("found unknown duration value '{}'", word)),
                    });
                    WaitKeyword
                }
            }
        }

        let pre = match pre {
            Some(pre) => pre,
            None => return Err("it has no trigger condition".into()),
        };
        let post = match post {
            Some(post) => post,
            None => return Err("it has no action to perform".into()),
        };

        Ok(Some(Action {
            pre,
            post,
            delay,
            last_trigger: Instant::now() - delay,
            _source: line,
        }))
    }

    fn check(&self, checker: &MemoryChecker) -> bool {
        self.pre.is_valid(checker) && self.last_trigger.elapsed() > self.delay
    }

    fn try_trigger(&mut self) -> Result<(), &'static str> {
        self.last_trigger = Instant::now();
        self.post.act()
    }

    fn trigger(&mut self) {
        if let Err(message) = self.try_trigger() {
            eprintln!("warning: run failed: {}: {}", self, message);
        } else {
            eprintln!("note: ran successfully: {}", self);
        }
    }
}

impl ActionSet {
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        checker: MemoryChecker,
    ) -> Result<Self, &'static str> {
        let actions: Vec<Action> = match File::open(path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Err("poe key file not found");
            }
            Err(_) => {
                return Err("failed to open poe key file, lack of permissions?");
            }
            Ok(file) => BufReader::new(file)
                .lines()
                .map(|line| line.expect("failed to read file"))
                .flat_map(|line| match Action::from_line(&line) {
                    Ok(action) => action,
                    Err(message) => {
                        eprintln!("warning: skipping '{}' because {}", line, message);
                        None
                    }
                })
                .collect(),
        };

        Ok(ActionSet { checker, actions })
    }

    pub fn check_all(&mut self) {
        if self.checker.can_check() {
            let checker = &self.checker;
            self.actions
                .iter_mut()
                .filter(|a| a.check(checker))
                .for_each(|a| a.trigger());
        }
    }
}

impl fmt::Display for ActionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} actions for:", self.actions.len(),)?;
        for action in self.actions.iter() {
            write!(f, "\n- {}", action)?;
        }
        Ok(())
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "on {} every {}ms do {}",
            self.pre,
            self.delay.as_millis(),
            self.post
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Value;

    fn action(line: &str) -> Action {
        Action::from_line(line).unwrap().unwrap()
    }

    #[test]
    fn empty_action() {
        assert_eq!(Action::from_line("\t  \n"), Ok(None));
    }

    #[test]
    fn comment_action() {
        assert_eq!(Action::from_line("// on key 0x1 do disconnect"), Ok(None));
    }

    #[test]
    fn pre_no_post() {
        assert!(Action::from_line("on key 0x1").is_err());
    }

    #[test]
    fn post_no_pre() {
        assert!(Action::from_line("do disconnect").is_err());
    }

    #[test]
    fn life_percent() {
        assert_eq!(
            action("on life 50% do disconnect").pre,
            PreCondition::LifeBelow {
                threshold: Value::Percent(0.5)
            }
        );
    }

    #[test]
    fn life_flat() {
        assert_eq!(
            action("on life 1000 do disconnect").pre,
            PreCondition::LifeBelow {
                threshold: Value::Flat(1000)
            }
        );
    }

    #[test]
    fn es_percent() {
        assert_eq!(
            action("on es 50% do disconnect").pre,
            PreCondition::EnergyBelow {
                threshold: Value::Percent(0.5)
            }
        );
    }

    #[test]
    fn es_flat() {
        assert_eq!(
            action("on es 1000 do disconnect").pre,
            PreCondition::EnergyBelow {
                threshold: Value::Flat(1000)
            }
        );
    }

    #[test]
    fn mana_percent() {
        assert_eq!(
            action("on mana 50% do disconnect").pre,
            PreCondition::ManaBelow {
                threshold: Value::Percent(0.5)
            }
        );
    }

    #[test]
    fn mana_flat() {
        assert_eq!(
            action("on mana 1000 do disconnect").pre,
            PreCondition::ManaBelow {
                threshold: Value::Flat(1000)
            }
        );
    }

    #[test]
    fn key() {
        assert_eq!(
            action("on key z do disconnect").pre,
            PreCondition::KeyPress { vk: 0x5A }
        );
        assert_eq!(
            action("on key Z do disconnect").pre,
            PreCondition::KeyPress { vk: 0x5A }
        );
        assert_eq!(
            action("on key 6 do disconnect").pre,
            PreCondition::KeyPress { vk: 0x36 }
        );
        assert_eq!(
            action("on key F11 do disconnect").pre,
            PreCondition::KeyPress { vk: 0x7A }
        );
        assert_eq!(
            action("on key 0x2 do disconnect").pre,
            PreCondition::KeyPress { vk: 0x02 }
        );
    }

    #[test]
    fn key_synonyms() {
        assert_eq!(
            action("on key Z do disconnect").pre,
            action("on flask Z do disconnect").pre
        );
        assert_eq!(
            action("on key Z do disconnect").pre,
            action("on skill Z do disconnect").pre
        );
    }

    #[test]
    fn parse_self_display() {
        fn parse_self(line: &str) {
            let parsed = action(line);
            let reparsed = action(&parsed.to_string());
            assert_eq!(parsed.pre, reparsed.pre);
            assert_eq!(parsed.post, reparsed.post);
            assert_eq!(parsed.delay, reparsed.delay);
        }

        parse_self("on life 50% do disconnect");
        parse_self("do disconnect on life 1000");
        parse_self("every 200ms on es 50% do disconnect");
        parse_self("on es 1000 every 200ms do disconnect");
        parse_self("on mana 50% do disconnect every 200ms");
        parse_self("do disconnect every 200ms on mana 1000");
        parse_self("every 200ms on key z do type test");
        parse_self("do destroy on key Z every 200ms");
    }

    #[test]
    fn display() {
        assert_eq!(
            action("on key Z do disconnect every 200ms").to_string(),
            "on key 0x5A every 200ms do disconnect"
        );

        assert_eq!(
            action("on key A every 200ms do disconnect on key Z every 300ms do type test")
                .to_string(),
            "on key 0x5A every 300ms do type test"
        );
    }
}
