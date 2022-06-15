use super::{Checker as _, MemoryChecker};
use crate::utils::Value;
use crate::win;
use std::fmt;

pub enum PreCondition {
    LifeBelow { threshold: Value },
    EnergyBelow { threshold: Value },
    ManaBelow { threshold: Value },
    KeyPress { vk: u16 },
}

impl PreCondition {
    pub fn is_valid(&self, checker: &MemoryChecker) -> bool {
        match self {
            Self::LifeBelow { threshold } => checker.life_below(*threshold),
            Self::EnergyBelow { threshold } => checker.es_below(*threshold),
            Self::ManaBelow { threshold } => checker.mana_below(*threshold),
            Self::KeyPress { vk } => win::keyboard::is_down(*vk),
        }
    }
}

impl fmt::Display for PreCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LifeBelow { threshold } => write!(f, "life {}", threshold),
            Self::EnergyBelow { threshold } => write!(f, "es {}", threshold),
            Self::ManaBelow { threshold } => write!(f, "mana {}", threshold),
            Self::KeyPress { vk } => write!(f, "key 0x{:02X}", vk),
        }
    }
}
