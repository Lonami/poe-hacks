use super::{AreaStatus, MouseStatus, PlayerStats};
use crate::win;
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

impl PreCondition {
    pub fn is_valid(
        &self,
        checker: &PlayerStats,
        mouse_status: MouseStatus,
        area_status: AreaStatus,
    ) -> bool {
        match self {
            Self::LifeBelow { threshold } => checker.life_below(*threshold),
            Self::EnergyBelow { threshold } => checker.es_below(*threshold),
            Self::ManaBelow { threshold } => checker.mana_below(*threshold),
            Self::KeyPress { vk } => win::keyboard::is_down(vk.0),
            Self::MouseWheel { dir } => match dir {
                Direction::Up => mouse_status.scrolled_up,
                Direction::Down => mouse_status.scrolled_down,
            },
            Self::InArea { town } => area_status.in_town.map_or(false, |x| *town == x),
            Self::JustTransitioned => area_status.just_transitioned,
            Self::Chat { open } => *open == area_status.chat_open,
            Self::WindowFocus => area_status.in_foreground,
            Self::WindowBlur => !area_status.in_foreground,
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
