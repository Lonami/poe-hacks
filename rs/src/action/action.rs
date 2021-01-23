use super::{PostCondition, PreCondition, ScreenPoint};
use crate::utils;
use rshacks::win;
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
    actions: Vec<Action>,
    width: usize,
    height: usize,
    decorations: [ScreenPoint; 2],
}

impl Action {
    fn from_line(line: &str, screen_size: (usize, usize)) -> Result<Option<Action>, String> {
        if line.starts_with("//") || line.chars().all(|c| c.is_whitespace()) {
            return Ok(None);
        }

        let (width, height) = screen_size;
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
                    pre = Some(PreCondition::ScreenChange {
                        point: ScreenPoint::new_life(percent, width, height),
                    });
                    WaitKeyword
                }
                WaitEsValue => {
                    let percent = utils::parse_percentage(word)?;
                    pre = Some(PreCondition::ScreenChange {
                        point: ScreenPoint::new_es(percent, width, height),
                    });
                    WaitKeyword
                }
                WaitManaValue => {
                    let percent = utils::parse_percentage(word)?;
                    pre = Some(PreCondition::ScreenChange {
                        point: ScreenPoint::new_mana(percent, width, height),
                    });
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

    fn check(&self) -> bool {
        self.pre.is_valid() && self.last_trigger.elapsed() > self.delay
    }

    fn try_trigger(&mut self, width: usize, height: usize) -> Result<(), &'static str> {
        self.last_trigger = Instant::now();
        self.post.act(width, height)
    }

    fn trigger(&mut self, width: usize, height: usize) {
        if let Err(message) = self.try_trigger(width, height) {
            eprintln!("warning: run failed: {}: {}", self.display, message);
        } else {
            eprintln!("note: ran successfully: {}", self.display);
        }
    }
}

impl ActionSet {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, &'static str> {
        let (width, height) = match win::screen::size() {
            Ok(value) => value,
            Err(_) => return Err("failed to get screen size"),
        };

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
                .flat_map(|line| match Action::from_line(&line, (width, height)) {
                    Ok(action) => action,
                    Err(message) => {
                        eprintln!("warning: skipping '{}' because {}", line, message);
                        None
                    }
                })
                .collect(),
        };

        let decorations = [
            ScreenPoint::new_deco1(width, height),
            ScreenPoint::new_deco2(width, height),
        ];

        Ok(ActionSet {
            actions,
            width,
            height,
            decorations,
        })
    }

    pub fn check_all(&mut self) {
        let (width, height) = (self.width, self.height);
        if self.decorations.iter().all(|d| !d.changed()) {
            self.actions
                .iter_mut()
                .filter(|a| a.check())
                .for_each(|a| a.trigger(width, height));
        }
    }
}

impl fmt::Display for ActionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} actions for a {}x{} screen:",
            self.actions.len(),
            self.width,
            self.height
        )?;
        for action in self.actions.iter() {
            write!(f, "\n- {}", action.display)?;
        }
        Ok(())
    }
}
