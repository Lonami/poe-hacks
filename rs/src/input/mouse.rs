#![cfg(windows)]

use std::io::Error;
use std::mem::MaybeUninit;

use winapi::um::winuser::{
    GetCursorPos, INPUT_u, SendInput, SetCursorPos, INPUT, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, WHEEL_DELTA,
};

pub enum Button {
    Left,
    Right,
    Middle,
}

/// Get the cursor's position as `(x, y)`.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getcursorpos
pub fn get() -> Result<(usize, usize), Error> {
    unsafe {
        let mut point = MaybeUninit::uninit();
        if GetCursorPos(point.as_mut_ptr()) == 0 {
            Err(Error::last_os_error())
        } else {
            let point = point.assume_init();
            Ok((point.x as usize, point.y as usize))
        }
    }
}

/// Set the cursor position on screen.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setcursorpos
pub fn set(x: usize, y: usize) -> Result<(), Error> {
    unsafe {
        if SetCursorPos(x as i32, y as i32) == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

/// Perform a mouse click (press down and release).
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-sendinput
pub fn click(button: Button) {
    unsafe {
        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: MaybeUninit::<INPUT_u>::zeroed().assume_init(),
        };

        input.u.mi_mut().dwFlags = match button {
            Button::Left => MOUSEEVENTF_LEFTDOWN,
            Button::Right => MOUSEEVENTF_RIGHTDOWN,
            Button::Middle => MOUSEEVENTF_MIDDLEDOWN,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);

        input.u.mi_mut().dwFlags = match button {
            Button::Left => MOUSEEVENTF_LEFTUP,
            Button::Right => MOUSEEVENTF_RIGHTUP,
            Button::Middle => MOUSEEVENTF_MIDDLEUP,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Scroll the mouse wheel a certain amount.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-sendinput
pub fn scroll(amount: isize) {
    unsafe {
        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: MaybeUninit::<INPUT_u>::zeroed().assume_init(),
        };

        input.u.mi_mut().mouseData = (WHEEL_DELTA as isize * amount) as u32;
        input.u.mi_mut().dwFlags = MOUSEEVENTF_WHEEL;

        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}
