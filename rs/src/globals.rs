use winapi::um::winuser::GetDC;
use winapi::shared::windef::HDC;

// TODO: Worth releasing prior to closing the app?
static mut DESKTOP_DC: HDC = std::ptr::null_mut();

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
