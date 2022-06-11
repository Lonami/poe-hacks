use super::{Checker as _, MemoryChecker};
use crate::utils::Value;
use crate::win;

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
