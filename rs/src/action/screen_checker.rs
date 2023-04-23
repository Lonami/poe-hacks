use crate::win;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const DELAY: Duration = Duration::from_millis(50);

const CHAT_BORDER_THICKNESS: usize = 2;
const CHAT_APPROX_START_PCT: usize = 70;
const CHAT_APPROX_HEIGHT_PCT: usize = 5;
const CHAT_BORDER_COLOR: (u8, u8, u8) = (136, 98, 59);
const CHAT_CHECK_HEIGHT: usize = 32;

enum Message {
    Chat { open: bool },
}

pub struct ScreenChecker {
    rx: mpsc::Receiver<Message>,
    tx: mpsc::Sender<()>,
    handle: thread::JoinHandle<()>,
    chat_open: bool,
}

impl ScreenChecker {
    pub fn install() -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();
        let (kill_tx, kill_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let mut size = win::screen::size().unwrap();
            size.width = CHAT_BORDER_THICKNESS;
            size.top += (CHAT_APPROX_START_PCT * size.height) / 100;
            size.height = (CHAT_APPROX_HEIGHT_PCT * size.height) / 100;
            let mut screen = win::screen::Screen::capture_region(size).unwrap();

            loop {
                let start = Instant::now();
                screen.refresh().unwrap();

                let mut longest_run = 0;
                let mut msg = Message::Chat { open: false };
                for color in screen.screenshot().colors() {
                    if color == CHAT_BORDER_COLOR {
                        longest_run += 1;
                        if longest_run >= CHAT_CHECK_HEIGHT * CHAT_BORDER_THICKNESS {
                            msg = Message::Chat { open: true };
                            break;
                        }
                    } else {
                        longest_run = 0;
                    }
                }

                if msg_tx.send(msg).is_err() {
                    break;
                }

                match kill_rx.recv_timeout(DELAY.saturating_sub(start.elapsed())) {
                    Ok(_) => break,
                    Err(_) => continue,
                }
            }
        });

        Self {
            rx: msg_rx,
            tx: kill_tx,
            handle,
            chat_open: false,
        }
    }

    #[allow(dead_code)]
    pub fn uninstall(self) -> thread::Result<()> {
        let _ = self.tx.send(());
        self.handle.join()
    }

    pub fn chat_open(&mut self) -> bool {
        loop {
            match self.rx.try_recv() {
                Ok(Message::Chat { open }) => self.chat_open = open,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => self.chat_open = false,
            }
        }

        self.chat_open
    }
}
