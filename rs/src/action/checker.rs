use crate::utils::{self, Value};
use crate::win;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use win::proc::{Process, PtrMap};

// In-memory structures for the memory checker.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Health {
    pub hp: i32,
    pub max_hp: i32,
    pub unreserved_hp: i32,
    pub es: i32,
    pub max_es: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Mana {
    pub mana: i32,
    pub max_mana: i32,
    pub unreserved_mana: i32,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerStats {
    health: Health,
    mana: Mana,
}

pub trait Checker {
    fn refresh(&mut self) -> Result<(), &'static str>;
    fn can_check(&self) -> bool;
    fn life_below(&self, threshold: Value) -> bool;
    fn es_below(&self, threshold: Value) -> bool;
    fn mana_below(&self, threshold: Value) -> bool;
}

pub struct MemoryChecker {
    process: Process,
    life_es_map: win::proc::PtrMap,
    mana_map: win::proc::PtrMap,
    stats: Option<PlayerStats>,
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
            stats: None,
        })
    }

    pub fn save_ptr_map<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        fs::write(
            path,
            format!("{}\n{}\n", self.life_es_map, self.mana_map).as_bytes(),
        )
    }

    fn read<T>(&self, map: &PtrMap) -> Option<T> {
        match self.process.deref::<T>(map) {
            Ok(t) => Some(t),
            Err(_) => {
                // Observed errors:
                // * Invalid access to memory location. (os error 998)
                // * Only part of a ReadProcessMemory or WriteProcessMemory request was completed. (os error 299)
                // In either case this pointer map won't work.
                None
            }
        }
    }

    pub fn health(&self) -> Option<Health> {
        self.read(&self.life_es_map)
    }

    pub fn mana(&self) -> Option<Mana> {
        self.read(&self.mana_map)
    }

    pub fn nudge_map_base_addr(&mut self, delta: isize) {
        self.life_es_map.nudge_base(delta);
        self.mana_map.nudge_base(delta);
    }
}

impl Checker for MemoryChecker {
    fn refresh(&mut self) -> Result<(), &'static str> {
        if !self
            .process
            .running()
            .expect("failed to check for process status")
        {
            if let Some(proc) = utils::open_poe() {
                self.process = proc;
            } else {
                return Err("could not find poe running");
            }
        }

        self.stats = self
            .health()
            .zip(self.mana())
            .map(|(health, mana)| PlayerStats { health, mana });

        Ok(())
    }

    fn can_check(&self) -> bool {
        // if the hp is zero, the player is dead, so any further action is no longer meaningful.
        // for this reason, treat 0 as having infinite health and never being below the threshold.
        //
        // when logging in to town, poe seems to initialize the values to -1. for this reason,
        // `hp > 0` is used as opposed to `hp != 0` (it is meaningless to check for -1 health).
        if let Some(stats) = &self.stats {
            stats.health.hp > 0
        } else {
            false
        }
    }

    fn life_below(&self, threshold: Value) -> bool {
        self.stats
            .as_ref()
            .map(|stats| check_threshold(threshold, stats.health.hp, stats.health.max_hp))
            .unwrap_or(false)
    }

    fn es_below(&self, threshold: Value) -> bool {
        self.stats
            .as_ref()
            .map(|stats| check_threshold(threshold, stats.health.es, stats.health.max_es))
            .unwrap_or(false)
    }

    fn mana_below(&self, threshold: Value) -> bool {
        self.stats
            .as_ref()
            .map(|stats| check_threshold(threshold, stats.mana.mana, stats.mana.max_mana))
            .unwrap_or(false)
    }
}

fn check_threshold(threshold: Value, current: i32, max: i32) -> bool {
    // this check is already present in `can_check` but things may have changed since then
    if current == -1 {
        false
    } else {
        match threshold {
            Value::Percent(percent) => current <= (percent * max as f32) as i32,
            Value::Flat(flat) => current <= flat,
        }
    }
}
