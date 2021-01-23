use super::{
    Checker, PostCondition, PreCondition, ScreenChecker, LIFE_PERCENT_UNSAFE, MANA_PERCENT_UNSAFE,
};
use crate::utils;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

// In-memory structures
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Health {
    hp: i32,
    max_hp: i32,
    unreserved_hp: i32,
    es: i32,
    max_es: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Mana {
    mana: i32,
    max_mana: i32,
    unreserved_mana: i32,
}

struct Action {
    pre: PreCondition,
    post: PostCondition,
    last_trigger: Instant,
    delay: Duration,
    display: String,
}

pub struct ActionSet {
    pub checker: ScreenChecker,
    actions: Vec<Action>,
}

impl Action {
    fn from_line(line: &str) -> Result<Option<Action>, String> {
        if line.starts_with("//") || line.chars().all(|c| c.is_whitespace()) {
            return Ok(None);
        }

        let mut pre: Option<PreCondition> = None;
        let mut post: Option<PostCondition> = None;
        let mut delay = Duration::from_millis(0);

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
        };

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
                    let percent = utils::parse_percentage(word)?;
                    if percent - 0.005 < LIFE_PERCENT_UNSAFE {
                        eprintln!(
                            "\x07warning: the life percentage {}% is too low and may not work",
                            (percent * 100.0) as usize
                        );
                    }

                    pre = Some(PreCondition::LifeBelow { percent });
                    WaitKeyword
                }
                WaitEsValue => {
                    let percent = utils::parse_percentage(word)?;
                    pre = Some(PreCondition::EnergyBelow { percent });
                    WaitKeyword
                }
                WaitManaValue => {
                    let percent = utils::parse_percentage(word)?;
                    if percent - 0.005 < MANA_PERCENT_UNSAFE {
                        eprintln!(
                            "\x07warning: the mana percentage {}% is too low and may not work",
                            (percent * 100.0) as usize
                        );
                    }

                    pre = Some(PreCondition::ManaBelow { percent });
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
            display: line,
        }))
    }

    fn check(&self, checker: &ScreenChecker) -> bool {
        self.pre.is_valid(checker) && self.last_trigger.elapsed() > self.delay
    }

    fn try_trigger(&mut self) -> Result<(), &'static str> {
        self.last_trigger = Instant::now();
        self.post.act()
    }

    fn trigger(&mut self) {
        if let Err(message) = self.try_trigger() {
            eprintln!("warning: run failed: {}: {}", self.display, message);
        } else {
            eprintln!("note: ran successfully: {}", self.display);
        }
    }
}

impl ActionSet {
    pub fn from_file<P: AsRef<Path>>(path: P, mut screen: rshacks::win::screen::Screen) -> Result<Self, &'static str> {
        screen.refresh()?;
        let checker = ScreenChecker::new(screen);

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
            write!(f, "\n- {}", action.display)?;
        }
        Ok(())
    }
}
