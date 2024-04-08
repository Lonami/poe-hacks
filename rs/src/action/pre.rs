use crate::win;
use rshacks::checker::{FocusState, LogState, MemoryState, MouseState, ScreenState};
use rshacks::types::{Direction, Opened, Value, Vk};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PreCondition {
    LifeBelow { threshold: Value },
    EnergyBelow { threshold: Value },
    ManaBelow { threshold: Value },
    KeyPress { vk: Vk },
    MouseWheel { dir: Direction },
    InArea { town: bool },
    JustTransitioned,
    Chat { open: Opened },
    WindowFocus,
    WindowBlur,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PreRequirement {
    Area,
    Focus,
    Mouse,
    Player,
    Screen,
}

pub struct GameState {
    pub area: Option<LogState>,
    pub focus: Option<FocusState>,
    pub mouse: Option<MouseState>,
    pub player: Option<MemoryState>,
    pub screen: Option<ScreenState>,
}

impl PreCondition {
    pub fn is_valid(&self, state: &GameState) -> bool {
        fn ok() {}
        (|| match self {
            Self::LifeBelow { threshold } => threshold
                .above(
                    state.player.as_ref()?.health.hp,
                    state.player.as_ref()?.health.max_hp,
                )
                .then(ok),
            Self::EnergyBelow { threshold } => threshold
                .above(
                    state.player.as_ref()?.health.es,
                    state.player.as_ref()?.health.max_es,
                )
                .then(ok),
            Self::ManaBelow { threshold } => threshold
                .above(
                    state.player.as_ref()?.mana.mana,
                    state.player.as_ref()?.mana.max_mana,
                )
                .then(ok),
            Self::KeyPress { vk } => win::keyboard::is_down(vk.0).then(ok),
            Self::MouseWheel { dir } => match dir {
                Direction::Up => state.mouse.as_ref()?.scrolled_up.then(ok),
                Direction::Down => state.mouse.as_ref()?.scrolled_down.then(ok),
            },
            Self::InArea { town } => state
                .area
                .as_ref()?
                .in_town
                .map_or(false, |x| *town == x)
                .then(ok),
            Self::JustTransitioned => state.area.as_ref()?.just_transitioned.then(ok),
            Self::Chat { open } => (*open == state.screen.as_ref()?.chat_open).then(ok),
            Self::WindowFocus => state.focus.as_ref()?.in_foreground.then(ok),
            Self::WindowBlur => (!state.focus.as_ref()?.in_foreground).then(ok),
        })()
        .is_some()
    }

    pub fn requires(&self, requirement: PreRequirement) -> bool {
        match self {
            Self::LifeBelow { .. } | Self::EnergyBelow { .. } | Self::ManaBelow { .. } => {
                requirement == PreRequirement::Player
            }
            Self::KeyPress { .. } => false,
            Self::MouseWheel { .. } => requirement == PreRequirement::Mouse,
            Self::InArea { .. } | Self::JustTransitioned => requirement == PreRequirement::Area,
            Self::Chat { .. } => requirement == PreRequirement::Screen,
            Self::WindowFocus | Self::WindowBlur => requirement == PreRequirement::Focus,
        }
    }
}

impl fmt::Display for PreCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LifeBelow { threshold } => write!(f, "life {}", threshold),
            Self::EnergyBelow { threshold } => write!(f, "es {}", threshold),
            Self::ManaBelow { threshold } => write!(f, "mana {}", threshold),
            Self::KeyPress { vk } => write!(f, "key 0x{:02X}", vk.0),
            Self::MouseWheel { dir } => write!(
                f,
                "wheel {}",
                match dir {
                    Direction::Up => "up",
                    Direction::Down => "down",
                }
            ),
            Self::InArea { town } => write!(f, "{}", if *town { "town" } else { "map" }),
            Self::JustTransitioned => write!(f, "transition"),
            Self::Chat { open } => write!(f, "chat {open}"),
            Self::WindowFocus => write!(f, "focus"),
            Self::WindowBlur => write!(f, "blur"),
        }
    }
}
