use crate::win;
use rshacks::types::{MouseButton, Vk};
use rshacks::win::proc::Process;
use std::fmt;
use std::thread::sleep;
use std::time::Duration;
use winapi::um::winuser::{VK_HOME, VK_RETURN, VK_RIGHT};

const DISCONNECT_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, PartialEq)]
pub enum PostCondition {
    PressKey { vk: Vk },
    Click { button: MouseButton },
    Disconnect,
    Type { string: String },
    InviteLast,
    Destroy,
    SetKeySuppression { suppress: bool },
}

#[derive(Debug, PartialEq)]
pub enum PostResult {
    None,
    SetKeySuppression { suppress: bool },
}

impl PostCondition {
    pub fn act(&self, process: &Process) -> Result<PostResult, &'static str> {
        match self {
            Self::PressKey { vk } => {
                win::keyboard::press(vk.0);
                Ok(PostResult::None)
            }
            Self::Click { button } => {
                win::mouse::click(button.0);
                Ok(PostResult::None)
            }
            Self::Disconnect => match win::proc::kill_network(process.pid) {
                Err(_) => Err("failed to kill poe network"),
                Ok(n) => {
                    if n > 0 {
                        sleep(DISCONNECT_DELAY);
                    }
                    Ok(PostResult::None)
                }
            },
            Self::Type { string } => {
                win::keyboard::press(VK_RETURN as u16);
                win::keyboard::type_string(&string);
                win::keyboard::press(VK_RETURN as u16);
                Ok(PostResult::None)
            }
            Self::InviteLast => {
                win::keyboard::ctrl_press(VK_RETURN as u16);
                win::keyboard::press(VK_HOME as u16);
                win::keyboard::shift_press(VK_RIGHT as u16);
                win::keyboard::type_string("/invite ");
                win::keyboard::ctrl_press(VK_RETURN as u16);

                Ok(PostResult::None)
            }
            Self::Destroy => {
                win::mouse::click(win::mouse::Button::Left);
                win::keyboard::ctrl_press(VK_RETURN as u16);
                win::keyboard::type_string("/destroy");
                win::keyboard::ctrl_press(VK_RETURN as u16);

                Ok(PostResult::None)
            }
            Self::SetKeySuppression { suppress } => Ok(PostResult::SetKeySuppression {
                suppress: *suppress,
            }),
        }
    }
}

impl fmt::Display for PostCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PressKey { vk } => write!(f, "press {vk}"),
            Self::Click { button } => write!(f, "press {button}"),
            Self::Disconnect => write!(f, "disconnect"),
            Self::Type { string } => write!(f, "type {}", string),
            Self::InviteLast => write!(f, "invite"),
            Self::Destroy => write!(f, "destroy"),
            Self::SetKeySuppression { suppress } => {
                if *suppress {
                    write!(f, "disable")
                } else {
                    write!(f, "enable")
                }
            }
        }
    }
}
