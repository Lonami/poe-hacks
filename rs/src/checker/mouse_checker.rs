use crate::win;

pub struct MouseChecker {}

pub struct MouseState {
    pub scrolled_up: bool,
    pub scrolled_down: bool,
}

impl MouseChecker {
    pub fn new() -> Self {
        win::hook::install_mouse_hook();
        Self {}
    }

    pub fn check(&self) -> MouseState {
        MouseState {
            scrolled_up: win::hook::poll_mouse_wheel_up(),
            scrolled_down: win::hook::poll_mouse_wheel_down(),
        }
    }
}

impl Drop for MouseChecker {
    fn drop(&mut self) {
        win::hook::uninstall_mouse_hook()
    }
}
