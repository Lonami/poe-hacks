use crate::win;
use std::fs::{self, File};
use std::io::{self, BufRead as _, BufReader};
use std::path::Path;
use std::rc::Rc;
use win::proc::{Process, PtrMap};

pub struct MemoryChecker {
    process: Rc<Process>,
    life_es_map: win::proc::PtrMap,
    mana_map: win::proc::PtrMap,
}

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

#[derive(Debug)]
pub struct MemoryState {
    pub health: Health,
    pub mana: Mana,
}

impl MemoryChecker {
    pub fn load_ptr_map<P: AsRef<Path>>(path: P, process: Rc<Process>) -> io::Result<Self> {
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

        Ok(Self {
            process,
            life_es_map,
            mana_map,
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

    fn health(&self) -> Option<Health> {
        self.read(&self.life_es_map)
    }

    fn mana(&self) -> Option<Mana> {
        self.read(&self.mana_map)
    }

    pub fn check(&self) -> Result<MemoryState, &'static str> {
        Ok(MemoryState {
            health: self.health().ok_or("could not read health")?,
            mana: self.mana().ok_or("could not read mana")?,
        })
    }

    pub fn nudge_map_base_addr(&mut self, delta: isize) {
        self.life_es_map.nudge_base(delta);
        self.mana_map.nudge_base(delta);
    }
}
