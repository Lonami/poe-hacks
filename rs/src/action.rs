use crate::https;

use rshacks::{globals, input, proc};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use winapi::um::winuser::{VK_F1, VK_HOME, VK_RETURN, VK_RIGHT};

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

// Where to click to enable/disable downscaling
const PARTY_X: f64 = 350.0 / 1920.0;
const PARTY_Y: f64 = 185.0 / 1080.0;

const DOWNSCALING_SELECT_X: f64 = 500.0 / 1920.0;
const DOWNSCALING_SELECT_Y: f64 = 800.0 / 1080.0;
const DOWNSCALING_ENABLE_Y: f64 = 830.0 / 1080.0;
const DOWNSCALING_DISABLE_Y: f64 = 860.0 / 1080.0;

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


// The color distance threshold after which we consider it to have changed.
// Tested on all ES ranges with all life reserved (30 disconnects, 40 doesn't),
// going in and out of town (having no life works fine too).
const ES_COLOR_THRESHOLD_SQ: i32 = 40 * 40;

const POE_EXE: &'static str = "PathOfExile";
const DISCONNECT_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
struct ScreenPoint {
    x: usize,
    y: usize,
    rgb: (u8, u8, u8),
    distance: i32,
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
    ShowPrice,
    InviteLast,
    Destroy,
    Downscaling { enable: bool },
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
    fn new(x: usize, y: usize) -> Self {
        let rgb = globals::get_cached_color(x, y);
        Self {
            x,
            y,
            rgb,
            distance: 1,
        }
    }

    fn new_life(percent: f64, width: usize, height: usize) -> Self {
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

    fn new_es(percent: f64, width: usize, height: usize) -> Self {
        // x²/a² + y²/b² = 1
        // x = √(a² * (1 - y²/b²))
        let a = LIFE_RX;
        let b = LIFE_RY;
        let y = b * 2.0 * (0.5 - percent);
        let x = f64::sqrt(a.powi(2) * (1.0 - y.powi(2) / b.powi(2)));

        let x = (width as f64 * (LIFE_CX + x)) as usize;
        let y = (height as f64 * (LIFE_CY + y)) as usize;
        let rgb = globals::get_cached_color(x, y);
        // Only ES needs a threshold because life can be reserved. The colors of everything else
        // must match exactly. It is risky to use the threshold anywhere else because the ground
        // may be close enough (e.g. mana).
        Self {
            x,
            y,
            rgb,
            distance: ES_COLOR_THRESHOLD_SQ,
        }
    }

    fn new_mana(percent: f64, width: usize, height: usize) -> Self {
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

    fn new_deco1(width: usize, height: usize) -> Self {
        Self::new(
            (width as f64 * DECO_X0) as usize,
            (height as f64 * DECO_Y0) as usize,
        )
    }

    fn new_deco2(width: usize, height: usize) -> Self {
        Self::new(
            (width as f64 * DECO_X1) as usize,
            (height as f64 * DECO_Y1) as usize,
        )
    }

    fn changed(&self) -> bool {
        let rgb = globals::get_cached_color(self.x, self.y);
        self.rgb != rgb
    }

    fn different(&self) -> bool {
        if self.distance == 1 {
            self.changed()
        } else {
            let rgb = globals::get_cached_color(self.x, self.y);
            (self.rgb.0 as i32 - rgb.0 as i32).pow(2)
                + (self.rgb.1 as i32 - rgb.1 as i32).pow(2)
                + (self.rgb.2 as i32 - rgb.2 as i32).pow(2)
                >= self.distance
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
    fn is_valid(&self) -> bool {
        match self {
            Self::ScreenChange { point } => point.different(),
            Self::KeyPress { vk } => input::keyboard::is_down(*vk),
        }
    }
}

impl PostCondition {
    fn act(&self, width: usize, height: usize) -> Result<(), &'static str> {
        match self {
            Self::PressKey { vk } => {
                input::keyboard::press(*vk);
                Ok(())
            }
            Self::Disconnect => match proc::Process::open_by_name(POE_EXE) {
                None => Err("could not find poe running"),
                Some(proc) => match proc::kill_network(proc.pid) {
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
            Self::ShowPrice => {
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
            Self::InviteLast => {
                input::keyboard::ctrl_press(VK_RETURN as u16);
                input::keyboard::press(VK_HOME as u16);
                input::keyboard::shift_press(VK_RIGHT as u16);
                input::keyboard::type_string("/invite ");
                input::keyboard::ctrl_press(VK_RETURN as u16);
                Ok(())
            }
            Self::Destroy => {
                input::mouse::click(input::mouse::Button::Left);
                input::keyboard::ctrl_press(VK_RETURN as u16);
                input::keyboard::type_string("/destroy");
                input::keyboard::ctrl_press(VK_RETURN as u16);
                Ok(())
            }
            Self::Downscaling { enable } => {
                let rel_click = |x, y| -> Result<(), &'static str> {
                    input::mouse::set((x * width as f64) as usize, (y * height as f64) as usize)
                        .map_err(|_| "failed to move mouse")?;

                    sleep(Duration::from_millis(64));
                    input::mouse::click(input::mouse::Button::Left);
                    Ok(())
                };

                let downscaling_select_y = if *enable {
                    DOWNSCALING_ENABLE_Y
                } else {
                    DOWNSCALING_DISABLE_Y
                };

                let (old_x, old_y) =
                    input::mouse::get().map_err(|_| "failed to get original mouse pos")?;

                input::keyboard::press(b'S' as u16);
                sleep(Duration::from_millis(128));
                rel_click(PARTY_X, PARTY_Y)?;
                rel_click(DOWNSCALING_SELECT_X, DOWNSCALING_SELECT_Y)?;
                rel_click(DOWNSCALING_SELECT_X, downscaling_select_y)?;
                input::keyboard::press(b'S' as u16);

                input::mouse::set(old_x, old_y)
                    .map_err(|_| "failed to restore original mouse pos")?;

                Ok(())
            }
        }
    }
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
                    let percent = parse_percentage(word)?;
                    pre = Some(PreCondition::ScreenChange {
                        point: ScreenPoint::new_life(percent, width, height),
                    });
                    WaitKeyword
                }
                WaitEsValue => {
                    let percent = parse_percentage(word)?;
                    pre = Some(PreCondition::ScreenChange {
                        point: ScreenPoint::new_es(percent, width, height),
                    });
                    WaitKeyword
                }
                WaitManaValue => {
                    let percent = parse_percentage(word)?;
                    pre = Some(PreCondition::ScreenChange {
                        point: ScreenPoint::new_mana(percent, width, height),
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
