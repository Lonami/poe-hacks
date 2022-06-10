use std::ptr;
use winapi::um::winuser::{
    MessageBoxExW, IDNO, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_ICONWARNING,
    MB_OK, MB_YESNOCANCEL,
};

fn as_wide_string(string: &str) -> Vec<u16> {
    let mut vec = Vec::with_capacity(string.len() * 2);
    vec.extend(string.encode_utf16());
    vec.push(0);
    vec
}

pub fn ask(title: &str, body: &str) -> Option<bool> {
    let answer = show(title, body, MB_ICONQUESTION, MB_YESNOCANCEL);
    if answer == IDYES {
        Some(true)
    } else if answer == IDNO {
        Some(false)
    } else {
        None
    }
}

pub fn info(title: &str, body: &str) {
    show(title, body, MB_ICONINFORMATION, MB_OK);
}

pub fn warn(title: &str, body: &str) {
    show(title, body, MB_ICONWARNING, MB_OK);
}

pub fn error(title: &str, body: &str) {
    show(title, body, MB_ICONERROR, MB_OK);
}

fn show(title: &str, body: &str, icon: u32, button: u32) -> i32 {
    let title = as_wide_string(title);
    let body = as_wide_string(body);
    unsafe {
        MessageBoxExW(
            ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            icon | button,
            0,
        )
    }
}
