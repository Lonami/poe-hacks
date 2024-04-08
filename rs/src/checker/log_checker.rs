use crate::win::proc::Process;
use std::fs::File;
use std::io::{self, BufRead as _, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::rc::Rc;

trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}

pub struct LogChecker {
    process: Rc<Process>,
    log_buffer: String,
    log_reader: BufReader<Box<dyn ReadSeek>>,
}

pub struct LogState {
    pub in_town: Option<bool>,
    pub just_transitioned: bool,
}

impl LogChecker {
    pub fn new(process: Rc<Process>) -> Self {
        Self {
            process,
            log_buffer: String::new(),
            log_reader: BufReader::new(Box::new(io::empty())),
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
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
                    Err(e) => {
                        eprintln!("warning: could not open log file, log checks won't work: {e}")
                    }
                }
            }
            Err(e) => {
                eprintln!("warning: could not find log file, log checks won't work: {e}")
            }
        }

        Ok(())
    }

    pub fn check(&mut self) -> Result<LogState, &'static str> {
        let mut result = LogState {
            in_town: None,
            just_transitioned: false,
        };

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
                            let mut matcher = msg.match_indices('"');
                            if let Some((start, end)) = matcher.next().zip(matcher.next()) {
                                let level = &msg[start.0 + 1..end.0];
                                result.in_town = Some(level.ends_with("_town"));
                            }
                        } else if msg.starts_with("[SHADER] Delay: ON") {
                            result.just_transitioned = true; // and finished loading
                        }
                    }
                }
                Err(e) => {
                    eprintln!("warning: failed to read from log file: {e}");
                    break;
                }
            }
        }

        Ok(result)
    }
}
