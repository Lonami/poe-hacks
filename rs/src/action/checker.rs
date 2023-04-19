use crate::utils::{self, Value};
use crate::win;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use win::proc::{Process, PtrMap};

#[derive(Clone, Copy, Debug, Default)]
pub struct MouseStatus {
    pub scrolled_up: bool,
    pub scrolled_down: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AreaStatus {
    pub in_town: Option<bool>,
    pub just_transitioned: bool,
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

#[derive(Clone, Debug, Default)]
pub struct PlayerStats {
    pub health: Health,
    pub mana: Mana,
}

trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}

pub struct MemoryChecker {
    process: Process,
    life_es_map: win::proc::PtrMap,
    mana_map: win::proc::PtrMap,
    stats: Option<PlayerStats>,
    in_town: Option<bool>,
    just_transitioned: bool,
    log_buffer: String,
    log_reader: BufReader<Box<dyn ReadSeek>>,
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

        // Use the default (non-existing, non-running) process until it's later refreshed.
        // Otherwise checkers cannot be loaded until the actual desired process was running
        // (which makes sense, but doesn't matter since even then it could later die).
        let process = Process::default();

        Ok(Self {
            process,
            life_es_map,
            mana_map,
            stats: None,
            in_town: None,
            just_transitioned: false,
            log_buffer: String::new(),
            log_reader: BufReader::new(Box::new(io::empty())),
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

    pub fn refresh(&mut self) -> Result<(), &'static str> {
        if !self
            .process
            .running()
            .expect("failed to check for process status")
        {
            if let Some(proc) = utils::open_poe() {
                self.process = proc;
                match self.process.file_name() {
                    Ok(file) => {
                        let mut path = Path::new(&file).parent().unwrap().to_path_buf();
                        path.push("logs");
                        path.push("Client.txt");
                        match File::open(path) {
                            Ok(f) => {
                                self.log_reader = BufReader::new(Box::new(f));
                                if let Err(e) = self.log_reader.seek(SeekFrom::End(-16 * 1024)) {
                                    eprintln!(
                                    "warning: could not seek log file, may take a while to catch-up: {e}");
                                }
                            }
                            Err(e) => eprintln!(
                                "warning: could not open log file, log checks won't work: {e}"
                            ),
                        }
                    }
                    Err(e) => {
                        eprintln!("warning: could not find log file, log checks won't work: {e}")
                    }
                }
            } else {
                return Err("could not find poe running");
            }
        }

        self.stats = self
            .health()
            .zip(self.mana())
            .map(|(health, mana)| PlayerStats { health, mana });

        self.just_transitioned = false;

        loop {
            self.log_buffer.clear();
            match self.log_reader.read_line(&mut self.log_buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    if let Some(i) = self.log_buffer.find("]") {
                        let msg = self.log_buffer[i + 1..].trim();
                        if msg.starts_with("Generating level") {
                            self.just_transitioned = true;
                            let mut matcher = msg.match_indices('"');
                            if let Some((start, end)) = matcher.next().zip(matcher.next()) {
                                let level = &msg[start.0 + 1..end.0];
                                self.in_town = Some(level.ends_with("_town"));
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("warning: failed to read from log file: {e}");
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn player_stats(&self) -> Option<&PlayerStats> {
        // if the hp is zero, the player is dead, so any further action is no longer meaningful.
        // for this reason, treat 0 as having infinite health and never being below the threshold.
        //
        // when logging in to town, poe seems to initialize the values to -1. for this reason,
        // `hp > 0` is used as opposed to `hp != 0` (it is meaningless to check for -1 health).
        if let Some(stats) = self.stats.as_ref() {
            if stats.health.hp > 0 {
                return Some(stats);
            }
        }

        None
    }

    pub fn in_town(&self) -> Option<bool> {
        self.in_town
    }

    pub fn just_transitioned(&self) -> bool {
        self.just_transitioned
    }
}

impl PlayerStats {
    pub fn life_below(&self, threshold: Value) -> bool {
        check_threshold(threshold, self.health.hp, self.health.max_hp)
    }

    pub fn es_below(&self, threshold: Value) -> bool {
        check_threshold(threshold, self.health.es, self.health.max_es)
    }

    pub fn mana_below(&self, threshold: Value) -> bool {
        check_threshold(threshold, self.mana.mana, self.mana.max_mana)
    }
}

fn check_threshold(threshold: Value, current: i32, max: i32) -> bool {
    match threshold {
        Value::Percent(percent) => current <= (percent * max as f32) as i32,
        Value::Flat(flat) => current <= flat,
    }
}

impl fmt::Display for PlayerStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "hp: {}/{}, es: {}/{}, mana: {}/{}",
            self.health.hp,
            self.health.max_hp,
            self.health.es,
            self.health.max_es,
            self.mana.mana,
            self.mana.max_mana
        )
    }
}

pub fn poll_mouse_status() -> MouseStatus {
    MouseStatus {
        scrolled_up: win::hook::poll_mouse_wheel_up(),
        scrolled_down: win::hook::poll_mouse_wheel_down(),
    }
}
