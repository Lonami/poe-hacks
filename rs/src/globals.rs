use crate::input::screen::Screen;
use winapi::shared::windef::HDC;
use winapi::um::winuser::GetDC;

// TODO: Worth releasing prior to closing the app?
static mut DESKTOP_DC: HDC = std::ptr::null_mut();
static mut DESKTOP: Option<Screen> = None;

pub unsafe fn get_desktop_dc() -> Option<HDC> {
    if DESKTOP_DC.is_null() {
        DESKTOP_DC = GetDC(std::ptr::null_mut());
    }
    if DESKTOP_DC.is_null() {
        None
    } else {
        Some(DESKTOP_DC)
    }
}

// WARNING: NOT THREAD SAFE, BUT THE PROGRAM DOESN'T USE THREADS FOR NOW

pub fn new_screen() {
    unsafe {
        assert!(DESKTOP.is_none());
        DESKTOP = Some(Screen::new().unwrap());
    }
}

pub fn refresh_screen() -> Result<(), &'static str> {
    unsafe { DESKTOP.as_mut().unwrap().refresh() }
}

pub fn get_cached_color(x: usize, y: usize) -> (u8, u8, u8) {
    unsafe { DESKTOP.as_ref().unwrap().color(x, y) }
}
