#![cfg(windows)]

use std::io::Error;
use std::mem::MaybeUninit;

use winapi::um::winuser::{GetDC, GetDesktopWindow, GetWindowRect};

use winapi::um::wingdi::{GetPixel, CLR_INVALID};

/// Gets the primary screen's size as `(width, height)`.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getdesktopwindow
pub fn size() -> Result<(usize, usize), Error> {
    unsafe {
        let mut desktop = MaybeUninit::uninit();
        // TODO cache this handle (C used static)
        let handle = GetDesktopWindow();
        if GetWindowRect(handle, desktop.as_mut_ptr()) == 0 {
            Err(Error::last_os_error())
        } else {
            let desktop = desktop.assume_init();
            Ok((desktop.right as usize, desktop.bottom as usize))
        }
    }
}

/// Gets an on-screen color as `(r, g, b)`.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/wingdi/nf-wingdi-getpixel
pub fn color(x: usize, y: usize) -> Option<(u8, u8, u8)> {
    // TODO cache this handle (C used static)
    unsafe {
        let dc = GetDC(std::ptr::null_mut());
        let color = GetPixel(dc, x as i32, y as i32);
        if color == CLR_INVALID {
            None
        } else {
            Some((
                ((color >> 0) & 0xff) as u8,
                ((color >> 2) & 0xff) as u8,
                ((color >> 4) & 0xff) as u8,
            ))
        }
    }
}
