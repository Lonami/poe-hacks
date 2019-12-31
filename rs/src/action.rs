use rshacks::{input, proc};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

const LIFE_X: f64 = 0.06;
const MANA_X: f64 = 0.94;

const LIFE_Y1: f64 = 0.813;
const LIFE_Y0: f64 = 0.974;
const MANA_Y1: f64 = 0.809;
const MANA_Y0: f64 = 0.981;

const DECO_X0: f64 = 0.004;
const DECO_Y0: f64 = 0.880;

const DECO_X1: f64 = 0.036;
const DECO_Y1: f64 = 0.960;

const POE_EXE: &'static str = "PathOfExile";
const DISCONNECT_DELAY: Duration = Duration::from_secs(1);

struct ScreenPoint {
    x: usize,
    y: usize,
    rgb: (u8, u8, u8),
}

enum PreCondition {
    ScreenChange { point: ScreenPoint },
    KeyPress { vk: u16 },
}

enum PostCondition {
    PressKey { vk: u16 },
    Disconnect,
}

struct Action {
    pre: PreCondition,
    post: PostCondition,
    last_trigger: Instant,
    delay: Duration,
}

pub struct ActionSet {
    actions: Vec<Action>,
    width: usize,
    height: usize,
    decorations: [ScreenPoint; 2],
}

impl ScreenPoint {
    fn new(x: usize, y: usize) -> Option<Self> {
        if let Ok(rgb) = input::screen::color(x, y) {
            Some(Self { x, y, rgb })
        } else {
            None
        }
    }

    fn new_life(percent: f64, width: usize, height: usize) -> Option<Self> {
        Self::new(
            (width as f64 * LIFE_X) as usize,
            (height as f64 * (LIFE_Y0 + (LIFE_Y1 - LIFE_Y0) * percent)) as usize,
        )
    }

    fn new_mana(percent: f64, width: usize, height: usize) -> Option<Self> {
        Self::new(
            (width as f64 * MANA_X) as usize,
            (height as f64 * (MANA_Y0 + (MANA_Y1 - MANA_Y0) * percent)) as usize,
        )
    }

    fn new_deco1(width: usize, height: usize) -> Option<Self> {
        Self::new(
            (width as f64 * DECO_X0) as usize,
            (height as f64 * DECO_Y0) as usize,
        )
    }

    fn new_deco2(width: usize, height: usize) -> Option<Self> {
        Self::new(
            (width as f64 * DECO_X1) as usize,
            (height as f64 * DECO_Y1) as usize,
        )
    }

    fn changed(&self) -> Option<bool> {
        if let Ok(rgb) = input::screen::color(self.x, self.y) {
            Some(self.rgb != rgb)
        } else {
            None
        }
    }
}

fn parse_percentage(word: &str) -> Result<f64, &'static str> {
    match word.trim_end_matches('%').parse::<isize>() {
        Ok(value) => {
            if value < 0 {
                Err("the percentage can't be negative")
            } else if value > 100 {
                Err("the percentage can't be bigger than 100")
            } else {
                Ok(value as f64 / 100.0)
            }
        }
        Err(_) => Err("the percentage was not a valid number"),
    }
}

fn parse_vk(word: &str) -> Result<u16, &'static str> {
    if word.starts_with("0x") {
        u16::from_str_radix(&word[2..], 16).map_err(|_| "got invalid hex virtual key code")
    } else if word.len() != 1 {
        Err("cannot map more than one character to a virtual key code")
    } else {
        Ok(input::keyboard::get_vk(word.as_bytes()[0]))
    }
}

impl PreCondition {
    fn is_valid(&self) -> Result<bool, &'static str> {
        match self {
            Self::ScreenChange { point } => {
                if let Some(changed) = point.changed() {
                    Ok(changed)
                } else {
                    Err("failed to detect color change")
                }
            }
            Self::KeyPress { vk } => Ok(input::keyboard::is_down(*vk)),
        }
    }
}

impl PostCondition {
    fn act(&self) -> Result<(), &'static str> {
        match self {
            Self::PressKey { vk } => {
                input::keyboard::press(*vk);
                Ok(())
            }
            Self::Disconnect => match proc::find_proc(POE_EXE) {
                None => Err("could not find poe running"),
                Some(pid) => match proc::kill_network(pid) {
                    Err(_) => Err("failed to kill poe network"),
                    Ok(_) => Ok(()),
                },
            },
        }
    }
}

impl Action {
    fn from_line(line: &str, screen_size: (usize, usize)) -> Result<Option<Action>, String> {
        if line.starts_with("//") {
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
            WaitManaValue,
            WaitKeyValue,

            WaitPostKind,
            WaitPostValue,

            WaitDelayValue,
        };

        let mut state = State::WaitKeyword;
        for word in line.to_lowercase().split_whitespace() {
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
                    "mana" => WaitManaValue,
                    "key" => WaitKeyValue,
                    _ => return Err(format!("found unknown condition '{}'", word)),
                },
                WaitLifeValue => {
                    let percent = parse_percentage(word)?;
                    pre = Some(PreCondition::ScreenChange {
                        point: match ScreenPoint::new_life(percent, width, height) {
                            Some(value) => value,
                            None => {
                                return Err(format!("could not read life pixel at {:.2}", percent))
                            }
                        },
                    });
                    WaitKeyword
                }
                WaitManaValue => {
                    let percent = parse_percentage(word)?;
                    pre = Some(PreCondition::ScreenChange {
                        point: match ScreenPoint::new_mana(percent, width, height) {
                            Some(value) => value,
                            None => {
                                return Err(format!("could not read mana pixel at {:.2}", percent))
                            }
                        },
                    });
                    WaitKeyword
                }
                WaitKeyValue => {
                    pre = Some(PreCondition::KeyPress {
                        vk: parse_vk(word)?,
                    });
                    WaitKeyword
                }

                WaitPostKind => match word {
                    "disconnect" => {
                        post = Some(PostCondition::Disconnect);
                        WaitKeyword
                    }
                    "flask" => WaitPostValue,
                    _ => return Err(format!("found unknown action '{}'", word)),
                },
                WaitPostValue => {
                    post = Some(PostCondition::PressKey {
                        vk: parse_vk(word)?,
                    });
                    WaitKeyword
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
        }))
    }

    fn check(&mut self) -> Result<bool, &'static str> {
        if self.pre.is_valid()? && self.last_trigger.elapsed() > self.delay {
            self.last_trigger = Instant::now();
            self.post.act()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl ActionSet {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, &'static str> {
        let (width, height) = match input::screen::size() {
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
            match ScreenPoint::new_deco1(width, height) {
                Some(value) => value,
                None => return Err("could not read deco 1 pixel"),
            },
            match ScreenPoint::new_deco2(width, height) {
                Some(value) => value,
                None => return Err("could not read deco 2 pixel"),
            },
        ];

        Ok(ActionSet {
            actions,
            width,
            height,
            decorations,
        })
    }

    pub fn check_all(&mut self) {
        if self.decorations.iter().all(|decoration| {
            if let Some(changed) = decoration.changed() {
                !changed
            } else {
                eprintln!("failed to check decoration pixel");
                std::thread::sleep(std::time::Duration::from_millis(1000));
                false
            }
        }) {
            self.actions
                .iter_mut()
                .for_each(|action| match action.check() {
                    Ok(_ran) => {
                    }
                    Err(message) => {
                        eprintln!("warning: checking action failed: {}", message);
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                });
        }
    }
}

impl fmt::Display for ActionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} actions for a {}x{} screen:\n",
            self.actions.len(),
            self.width,
            self.height
        )
    }
}
