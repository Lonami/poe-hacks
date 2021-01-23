use crate::{https, win};
use std::thread::sleep;
use std::time::Duration;
use winapi::um::winuser::{VK_HOME, VK_RETURN, VK_RIGHT};

// Where to click to enable/disable downscaling
const PARTY_X: f64 = 350.0 / 1920.0;
const PARTY_Y: f64 = 185.0 / 1080.0;

const DOWNSCALING_SELECT_X: f64 = 500.0 / 1920.0;
const DOWNSCALING_SELECT_Y: f64 = 800.0 / 1080.0;
const DOWNSCALING_ENABLE_Y: f64 = 830.0 / 1080.0;
const DOWNSCALING_DISABLE_Y: f64 = 860.0 / 1080.0;

const POE_EXE: &'static str = "PathOfExile";
const DISCONNECT_DELAY: Duration = Duration::from_secs(1);

#[derive(PartialEq)]
pub enum PostCondition {
    PressKey { vk: u16 },
    Disconnect,
    Type { string: String },
    ShowPrice,
    InviteLast,
    Destroy,
    Downscaling { enable: bool },
}

impl PostCondition {
    pub fn act(&self, width: usize, height: usize) -> Result<(), &'static str> {
        match self {
            Self::PressKey { vk } => {
                win::keyboard::press(*vk);
                Ok(())
            }
            Self::Disconnect => match win::proc::Process::open_by_name(POE_EXE) {
                None => Err("could not find poe running"),
                Some(proc) => match win::proc::kill_network(proc.pid) {
                    Err(_) => Err("failed to kill poe network"),
                    Ok(n) => {
                        if n > 0 {
                            sleep(DISCONNECT_DELAY);
                        }
                        Ok(())
                    }
                },
            },
            Self::Type { string } => {
                win::keyboard::press(VK_RETURN as u16);
                win::keyboard::type_string(&string);
                win::keyboard::press(VK_RETURN as u16);
                Ok(())
            }
            Self::ShowPrice => {
                // Press Ctrl+C
                sleep(Duration::from_millis(200));
                win::keyboard::ctrl_press(b'C' as u16);
                sleep(Duration::from_millis(200));

                // Extract name from clipboard
                let clipboard = win::clipboard::get()?;
                let name = {
                    let mut it = clipboard.split("\r\n");
                    it.next();
                    match it.next() {
                        Some(x) => x,
                        None => return Err("copied data does not contain item name"),
                    }
                };

                // Search for this item in poe.trade
                let prices = {
                    let _tooltip =
                        win::screen::create_tooltip(&format!("Checking price for {}...", name))
                            .map_err(|_| "failed to show loading tooltip")?;

                    https::find_unique_prices(name).map_err(|_| "failed to fetch prices")?
                };

                // Average the first few results
                let first_results = &prices[..prices.len().min(5)];
                let avg_price = first_results.iter().sum::<f64>() / first_results.len() as f64;

                // Show a tooltip until the mouse is moved
                {
                    let _tooltip = win::screen::create_tooltip(&format!(
                        "{} is worth {:.1}c",
                        name, avg_price
                    ))
                    .map_err(|_| "failed to show price tooltip")?;

                    let mouse = win::mouse::get().map_err(|_| "failed to detect mouse")?;
                    while mouse == win::mouse::get().map_err(|_| "failed to detect mouse")? {
                        sleep(Duration::from_millis(10));
                    }
                }

                Ok(())
            }
            Self::InviteLast => {
                win::keyboard::ctrl_press(VK_RETURN as u16);
                win::keyboard::press(VK_HOME as u16);
                win::keyboard::shift_press(VK_RIGHT as u16);
                win::keyboard::type_string("/invite ");
                win::keyboard::ctrl_press(VK_RETURN as u16);
                Ok(())
            }
            Self::Destroy => {
                win::mouse::click(win::mouse::Button::Left);
                win::keyboard::ctrl_press(VK_RETURN as u16);
                win::keyboard::type_string("/destroy");
                win::keyboard::ctrl_press(VK_RETURN as u16);
                Ok(())
            }
            Self::Downscaling { enable } => {
                let rel_click = |x, y| -> Result<(), &'static str> {
                    win::mouse::set((x * width as f64) as usize, (y * height as f64) as usize)
                        .map_err(|_| "failed to move mouse")?;

                    sleep(Duration::from_millis(64));
                    win::mouse::click(win::mouse::Button::Left);
                    Ok(())
                };

                let downscaling_select_y = if *enable {
                    DOWNSCALING_ENABLE_Y
                } else {
                    DOWNSCALING_DISABLE_Y
                };

                let (old_x, old_y) =
                    win::mouse::get().map_err(|_| "failed to get original mouse pos")?;

                win::keyboard::press(b'S' as u16);
                sleep(Duration::from_millis(128));
                rel_click(PARTY_X, PARTY_Y)?;
                rel_click(DOWNSCALING_SELECT_X, DOWNSCALING_SELECT_Y)?;
                rel_click(DOWNSCALING_SELECT_X, downscaling_select_y)?;
                win::keyboard::press(b'S' as u16);

                win::mouse::set(old_x, old_y)
                    .map_err(|_| "failed to restore original mouse pos")?;

                Ok(())
            }
        }
    }
}
