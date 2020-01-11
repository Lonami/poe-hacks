#![cfg(windows)]

use winapi::um::winbase::{GlobalLock, GlobalUnlock};
use winapi::um::winuser::{CloseClipboard, GetClipboardData, OpenClipboard, CF_TEXT};

pub fn get() -> Result<String, &'static str> {
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("failed to open clipboard");
        }

        let clipboard = GetClipboardData(CF_TEXT);
        if clipboard.is_null() {
            CloseClipboard();
            return Err("failed to get clipboard data");
        }

        let data = GlobalLock(clipboard);
        if data.is_null() {
            CloseClipboard();
            return Err("failed to acquire lock for the clipboard");
        }

        let result = std::ffi::CStr::from_ptr(data as *const i8)
            .to_string_lossy()
            .to_string();

        GlobalUnlock(clipboard);
        CloseClipboard();

        Ok(result)
    }
}
