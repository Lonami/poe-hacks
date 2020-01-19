use crate::https;

use rshacks::{input, proc};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use winapi::um::winuser::{VK_F1, VK_RETURN};

// Measured in a 1920x1080 screen, life and mana fit in a 205px box.
// The bottom right corners are (16, 2) for life and (1704, 2) for mana.
// There is some decoration near the bottom in both (20px and 15px).
// It doesn't seem to consider the area, only the height to indicate values.
//
// These values start at bottom-left, but we need origin to be in top-left
// which is why we do `1.0 - (...)` for the Y coordinates.
//
// The unsafe zone contains decoration so points below it may not work.
const LIFE_CX: f64 = (16.0 + 100.0) / 1920.0;
const LIFE_CY: f64 = 1.0 - ((2.0 + 100.0) / 1080.0);
const LIFE_RX: f64 = 100.0 / 1920.0;
const LIFE_RY: f64 = 100.0 / 1080.0;
const LIFE_Y_UNSAFE: f64 = 1.0 - (26.0 / 1080.0);

const MANA_CX: f64 = (1704.0 + 100.0) / 1920.0;
const MANA_CY: f64 = 1.0 - ((2.0 + 100.0) / 1080.0);
//const MANA_RX: f64 = 100.0 / 1920.0;
const MANA_RY: f64 = 100.0 / 1080.0;
const MANA_Y_UNSAFE: f64 = 1.0 - (16.0 / 1080.0);

// There are plenty of places where we can look for decorations,
// but we just pick a few around the bottom-left side of the screen.
const DECO_X0: f64 = 8.0 / 1920.0;
const DECO_Y0: f64 = 1.0 - (130.0 / 1080.0);

const DECO_X1: f64 = 69.0 / 1920.0;
const DECO_Y1: f64 = 1.0 - (44.0 / 1080.0);

// The color distance threshold after which we consider it to have changed.
// Experimental values (blood magic + auras with some ES to test at 80%).
const COLOR_DISTANCE_SQ: i32 = 70 * 70;

const POE_EXE: &'static str = "PathOfExile";
const DISCONNECT_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone)]
struct ScreenPoint {
    x: usize,
    y: usize,
    rgb: (u8, u8, u8),
}

enum PreCondition {
    ScreenChange { point: ScreenPoint },
    KeyPress { vk: u16 },
}

#[derive(PartialEq)]
enum PostCondition {
    PressKey { vk: u16 },
    Disconnect,
    Type { string: String },
    TypePrice,
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

impl ScreenPoint {
    fn new(x: usize, y: usize) -> Option<Self> {
        if let Ok(rgb) = input::screen::color(x, y) {
            Some(Self { x, y, rgb })
        } else {
            None
        }
    }

    fn new_life(percent: f64, width: usize, height: usize) -> Option<Self> {
        let y = LIFE_CY + LIFE_RY * 2.0 * (0.5 - percent);
        if y > LIFE_Y_UNSAFE {
            eprintln!(
                "\x07warning: the life percentage {}% is too low and may not work",
                (percent * 100.0) as usize
            );
        }
        Self::new(
            (width as f64 * LIFE_CX) as usize,
            (height as f64 * y) as usize,
        )
    }

    fn new_es(percent: f64, width: usize, height: usize) -> Option<Self> {
        // x²/a² + y²/b² = 1
        // x = √(a² * (1 - y²/b²))
        let a = LIFE_RX;
        let b = LIFE_RY;
        let y = b * 2.0 * (0.5 - percent);
        let x = f64::sqrt(a.powi(2) * (1.0 - y.powi(2) / b.powi(2)));
        Self::new(
            (width as f64 * (LIFE_CX + x)) as usize,
            (height as f64 * (LIFE_CY + y)) as usize,
        )
    }

    fn new_mana(percent: f64, width: usize, height: usize) -> Option<Self> {
        let y = MANA_CY + MANA_RY * 2.0 * (0.5 - percent);
        if y > MANA_Y_UNSAFE {
            eprintln!(
                "\x07warning: the mana percentage {}% is too low and may not work",
                (percent * 100.0) as usize
            );
        }
        Self::new(
            (width as f64 * MANA_CX) as usize,
            (height as f64 * y) as usize,
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

    fn different(&self) -> Option<bool> {
        // It's a constant so the compiler should optimize this branch
        if COLOR_DISTANCE_SQ == 1 {
            self.changed()
        } else if let Ok(rgb) = input::screen::color(self.x, self.y) {
            Some(
                (self.rgb.0 as i32 - rgb.0 as i32).pow(2)
                    + (self.rgb.1 as i32 - rgb.1 as i32).pow(2)
                    + (self.rgb.2 as i32 - rgb.2 as i32).pow(2)
                    >= COLOR_DISTANCE_SQ,
            )
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
        if word.starts_with("f") {
            match word[1..].parse::<u8>() {
                Ok(n) => Ok(((VK_F1 - 1) + n as i32) as u16),
                Err(_) => Err("invalid integer value for fn key"),
            }
        } else {
            Err("cannot map more than one character to a virtual key code unless it's a fn key")
        }
    } else {
        Ok(input::keyboard::get_vk(word.as_bytes()[0]))
    }
}

impl PreCondition {
    fn is_valid(&self) -> Result<bool, &'static str> {
        match self {
            Self::ScreenChange { point } => {
                if let Some(changed) = point.different() {
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
                    Ok(n) => {
                        if n > 0 {
                            sleep(DISCONNECT_DELAY);
                        }
                        Ok(())
                    }
                },
            },
            Self::Type { string } => {
                input::keyboard::press(VK_RETURN as u16);
                input::keyboard::type_string(&string);
                input::keyboard::press(VK_RETURN as u16);
                Ok(())
            }
            Self::TypePrice => {
                // Press Ctrl+C
                sleep(Duration::from_millis(200));
                input::keyboard::ctrl_press(b'C' as u16);
                sleep(Duration::from_millis(200));

                // Extract name from clipboard
                let clipboard = input::clipboard::get()?;
                let name = {
                    let mut it = clipboard.split("\r\n");
                    it.next();
                    match it.next() {
                        Some(x) => x,
                        None => return Err("copied data does not contain item name"),
                    }
                };

                // Search for this item in poe.trade
                let prices = {
                    let _tooltip =
                        input::screen::create_tooltip(&format!("Checking price for {}...", name))
                            .map_err(|_| "failed to show loading tooltip")?;

                    https::find_unique_prices(name).map_err(|_| "failed to fetch prices")?
                };

                // Average the first few results
                let first_results = &prices[..prices.len().min(5)];
                let avg_price = first_results.iter().sum::<f64>() / first_results.len() as f64;

                // Show a tooltip until the mouse is moved
                {
                    let _tooltip = input::screen::create_tooltip(&format!(
                        "{} is worth {:.1}c",
                        name, avg_price
                    ))
                    .map_err(|_| "failed to show price tooltip")?;

                    let mouse = input::mouse::get().map_err(|_| "failed to detect mouse")?;
                    while mouse == input::mouse::get().map_err(|_| "failed to detect mouse")? {
                        sleep(Duration::from_millis(10));
                    }
                }

                Ok(())
            }
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
                WaitEsValue => {
                    let percent = parse_percentage(word)?;
                    pre = Some(PreCondition::ScreenChange {
                        point: match ScreenPoint::new_es(percent, width, height) {
                            Some(value) => value,
                            None => {
                                return Err(format!("could not read es pixel at {:.2}", percent))
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
                    "type" => {
                        post = Some(PostCondition::Type {
                            string: String::new(),
                        });
                        WaitPostRemaining
                    }
                    "price" => {
                        post = Some(PostCondition::TypePrice);
                        WaitKeyword
                    }
                    _ => return Err(format!("found unknown action '{}'", word)),
                },
                WaitPostValue => {
                    post = Some(PostCondition::PressKey {
                        vk: parse_vk(word)?,
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

    fn special(&self) -> bool {
        self.post == PostCondition::Disconnect
            && match self.pre {
                PreCondition::KeyPress { .. } => true,
                _ => false,
            }
    }

    fn try_check(&self) -> Result<bool, &'static str> {
        Ok(self.pre.is_valid()? && self.last_trigger.elapsed() > self.delay)
    }

    fn check(&self) -> bool {
        match self.try_check() {
            Ok(x) => x,
            Err(message) => {
                eprintln!("warning: checking action failed: {}", message);
                false
            }
        }
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
        // First try inconditional actions
        self.actions
            .iter_mut()
            .filter(|a| a.special() && a.check())
            .for_each(|a| a.trigger());

        // This decoration check can't be a `fn(&self)` because that takes
        // an immutable reference (and `actions_to_trigger` has mutable)
        // which wouldn't work. However, the lambda seems to be fine.
        let decorations = &self.decorations;
        let deco_check = || {
            decorations.iter().all(|decoration| {
                if let Some(changed) = decoration.changed() {
                    !changed
                } else {
                    eprintln!("warning: failed to check decoration pixel");
                    false
                }
            })
        };

        // Then, check decorations before determining other actions and
        // also after. This is important because loading screens somehow
        // trip us up if we skip either decoration check (before or after).
        if deco_check() {
            // Note: the collect is important because we want to check the
            //       decorations immediately, not lazily! Otherwise, both
            //       decoration checks would happen *before* and none after.
            let actions_to_trigger: Vec<&mut Action> = self
                .actions
                .iter_mut()
                .filter(|a| !a.special() && a.check())
                .collect();

            if deco_check() {
                actions_to_trigger.into_iter().for_each(|a| a.trigger());
            }
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
