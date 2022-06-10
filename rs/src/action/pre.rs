use crate::{utils, win};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use win::proc::{Process, PtrMap};

// In-memory structures for the memory checker.
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

pub trait Checker {
    fn refresh(&mut self) -> Result<(), &'static str>;
    fn can_check(&self) -> bool;
    fn life_below(&self, percent: f64) -> bool;
    fn es_below(&self, percent: f64) -> bool;
    fn mana_below(&self, percent: f64) -> bool;
}

pub struct MemoryChecker {
    process: Process,
    life_es_map: win::proc::PtrMap,
    mana_map: win::proc::PtrMap,
}

pub enum PreCondition {
    LifeBelow { percent: f64 },
    EnergyBelow { percent: f64 },
    ManaBelow { percent: f64 },
    KeyPress { vk: u16 },
}

impl MemoryChecker {
    pub fn load_ptr_map<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let life_es_map = lines.next().ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "life+es ptr map line missing",
        ))??;
        let mana_map = lines.next().ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "mana ptr map line missing",
        ))??;

        let life_es_map = life_es_map
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mana_map = mana_map
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let process = utils::open_poe().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "could not find poe running")
        })?;

        Ok(Self {
            process,
            life_es_map,
            mana_map,
        })
    }

    fn read<T>(&self, map: &PtrMap) -> Option<T> {
        match self.process.deref::<T>(map) {
            Ok(t) => Some(t),
            Err(err) => {
                eprintln!("failed to follow pointer map {:?}: {}", map, err);
                None
            }
        }
    }
}

impl Checker for MemoryChecker {
    fn refresh(&mut self) -> Result<(), &'static str> {
        if self
            .process
            .running()
            .expect("failed to check for process status")
        {
            Ok(())
        } else if let Some(proc) = utils::open_poe() {
            self.process = proc;
            Ok(())
        } else {
            Err("could not find poe running")
        }
    }

    fn can_check(&self) -> bool {
        self.process.running().unwrap_or(false)
    }

    fn life_below(&self, percent: f64) -> bool {
        if let Some(health) = self.read::<Health>(&self.life_es_map) {
            let hp = health.hp as f64 / health.max_hp as f64;
            hp < percent
        } else {
            false
        }
    }

    fn es_below(&self, percent: f64) -> bool {
        if let Some(health) = self.read::<Health>(&self.life_es_map) {
            let es = health.es as f64 / health.max_es as f64;
            es < percent
        } else {
            false
        }
    }

    fn mana_below(&self, percent: f64) -> bool {
        if let Some(mana) = self.read::<Mana>(&self.mana_map) {
            let mp = mana.mana as f64 / mana.max_mana as f64;
            mp < percent
        } else {
            false
        }
    }
}

impl PreCondition {
    pub fn is_valid(&self, checker: &MemoryChecker) -> bool {
        match self {
            Self::LifeBelow { percent } => checker.life_below(*percent),
            Self::EnergyBelow { percent } => checker.es_below(*percent),
            Self::ManaBelow { percent } => checker.mana_below(*percent),
            Self::KeyPress { vk } => win::keyboard::is_down(*vk),
        }
    }
}
