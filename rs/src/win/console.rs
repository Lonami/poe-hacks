#![cfg(windows)]

use std::mem::MaybeUninit;

use winapi::um::wincon::{
    FillConsoleOutputCharacterA, GetConsoleScreenBufferInfo, GetConsoleWindow,
    SetConsoleCursorPosition,
};

use winapi::um::winuser::{
    SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_DRAWFRAME, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW,
};

use winapi::um::processenv::GetStdHandle;

use winapi::um::winbase::STD_OUTPUT_HANDLE;

use winapi::um::wincontypes::COORD;

// TODO better error handling

/// Stick the console window to the top of the screen.
/// Returns whether there was a console to stick or not.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setwindowpos
pub fn stick() -> bool {
    unsafe {
        let handle = GetConsoleWindow();
        if handle.is_null() {
            false
        } else {
            SetWindowPos(
                handle,
                HWND_TOPMOST,
                0,
                0,
                600,
                100,
                SWP_DRAWFRAME | SWP_SHOWWINDOW,
            ) != 0
        }
    }
}

/// Un-stick the console window from the top of the screen. Reverts `stick`.
/// Returns whether there was a console to stick or not.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setwindowpos
pub fn unstick() -> bool {
    unsafe {
        let handle = GetConsoleWindow();
        if handle.is_null() {
            false
        } else {
            SetWindowPos(
                handle,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_DRAWFRAME | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            ) != 0
        }
    }
}

// TODO use static console handle
// TODO use features for these (many times not needed so they're just bloat)

/// Clear the console, resetting the cursor position to the top-left corner.
pub fn clear() {
    unsafe {
        let console = GetStdHandle(STD_OUTPUT_HANDLE);
        let pos = COORD { X: 0, Y: 0 };
        let (x, y) = size();
        let mut written = MaybeUninit::uninit();

        FillConsoleOutputCharacterA(
            console,
            b' ' as i8,
            (x * y) as u32,
            pos,
            written.as_mut_ptr(),
        );
        SetConsoleCursorPosition(console, pos);
    }
}

/// Get the console size as `(width, height)`.
pub fn size() -> (usize, usize) {
    unsafe {
        let console = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut screen = MaybeUninit::uninit();
        GetConsoleScreenBufferInfo(console, screen.as_mut_ptr());
        let screen = screen.assume_init();
        (screen.dwSize.X as usize, screen.dwSize.Y as usize)
    }
}

/// Set the cursor position.
pub fn set_cursor(x: usize, y: usize) {
    unsafe {
        let console = GetStdHandle(STD_OUTPUT_HANDLE);
        SetConsoleCursorPosition(
            console,
            COORD {
                X: x as i16,
                Y: y as i16,
            },
        );
    }
}
