#![cfg(windows)]

use crate::globals;

use std::io::Error;
use std::mem::MaybeUninit;

use winapi::um::winuser::{
    CreateWindowExA, DefWindowProcA, DispatchMessageA, GetClientRect, GetDesktopWindow,
    GetMessageA, GetWindowRect, RegisterClassExA, SendMessageA, SetCursorPos, SetWindowPos,
    TranslateMessage, COLOR_WINDOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HWND_TOPMOST,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WNDCLASSEXA, WS_EX_TOPMOST, WS_OVERLAPPEDWINDOW,
    WS_POPUP, WS_VISIBLE,
};

use winapi::shared::minwindef::{ATOM, DWORD};
use winapi::shared::windef::{HBRUSH, HWND, RECT};
use winapi::um::commctrl::{
    TOOLINFOA, TOOLTIPS_CLASS, TTF_SUBCLASS, TTM_ADDTOOLA, TTM_POPUP, TTS_ALWAYSTIP, TTS_NOPREFIX,
};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::wingdi::{GetPixel, CLR_INVALID};
use winapi::um::winnt::{LPCSTR, LPSTR};

/// Gets the primary screen's size as `(width, height)`.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getdesktopwindow
pub fn size() -> Result<(usize, usize), Error> {
    unsafe {
        let mut desktop = MaybeUninit::uninit();
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
pub fn color(x: usize, y: usize) -> Result<(u8, u8, u8), &'static str> {
    unsafe {
        if let Some(dc) = globals::get_desktop_dc() {
            let color = GetPixel(dc, x as i32, y as i32);
            if color != CLR_INVALID {
                Ok((
                    ((color >> 0) & 0xff) as u8,
                    ((color >> 2) & 0xff) as u8,
                    ((color >> 4) & 0xff) as u8,
                ))
            } else {
                Err("failed to get pixel")
            }
        } else {
            Err("failed to get desktop dc")
        }
    }
}

/// Using `CreateWindowExA` without previously having used
/// `RegisterClassExA` will result in the last error to be
/// `57f`: `ERROR_CANNOT_FIND_WND_CLASS`.
///
/// # References
///
/// https://codesteps.com/2014/07/18/win32-programming-how-to-create-a-simple-gui-graphical-user-interface-based-application-part-2/
/// https://codesteps.com/2014/07/22/win32-programming-how-to-create-a-simple-gui-graphical-user-interface-based-application-part-3/
/// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexa
/// https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--1300-1699-
pub fn register_window_class(class_name: &str) -> Result<ATOM, DWORD> {
    unsafe {
        let atom = RegisterClassExA(&WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(DefWindowProcA),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleA(std::ptr::null_mut()),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: COLOR_WINDOW as HBRUSH,
            lpszMenuName: std::ptr::null_mut(),
            lpszClassName: class_name.as_ptr() as LPCSTR,
            hIconSm: std::ptr::null_mut(),
        });

        if atom != 0 {
            Ok(atom)
        } else {
            Err(GetLastError())
        }
    }
}

/// # References
///
/// https://codesteps.com/2014/07/14/win32-programming-how-to-create-a-simple-gui-graphical-user-interface-based-application-part-1/
pub fn create_window() -> Result<HWND, DWORD> {
    // TODO Maybe use the atom instead of the class name
    unsafe {
        register_window_class(TOOLTIPS_CLASS).unwrap();
        let parent = GetDesktopWindow();
        let instance = std::ptr::null_mut();
        let window_handle = CreateWindowExA(
            WS_EX_TOPMOST,                     // dwExStyle
            TOOLTIPS_CLASS.as_ptr() as LPCSTR, // lpClassName
            std::ptr::null_mut(),              // lpWindowName
            //WS_POPUP | TTS_NOPREFIX | TTS_ALWAYSTIP, // dwStyle
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,        // X
            CW_USEDEFAULT,        // Y
            CW_USEDEFAULT,        // nWidth
            CW_USEDEFAULT,        // nHeight
            parent,               // hWndParent
            std::ptr::null_mut(), // hMenu
            instance,             // hInstance
            std::ptr::null_mut(), // lpParam
        );
        if !window_handle.is_null() {
            SetWindowPos(
                window_handle,
                HWND_TOPMOST,
                0,
                0,
                1366,
                768,
                SWP_NOMOVE | SWP_NOSIZE, // | SWP_NOACTIVATE,
            );
            Ok(window_handle)
        } else {
            Err(GetLastError())
        }
    }
}

pub fn show_tooltip() {
    unsafe {
        let tooltip = create_window().unwrap();

        let parent = GetDesktopWindow();
        let instance = std::ptr::null_mut();
        let mut ti = TOOLINFOA {
            cbSize: std::mem::size_of::<TOOLINFOA>() as u32,
            uFlags: TTF_SUBCLASS,
            hwnd: parent,
            hinst: instance,
            uId: tooltip as usize,
            lpszText: "tool-tip".as_ptr() as LPSTR,
            lParam: 0,
            lpReserved: std::ptr::null_mut(),
            rect: RECT {
                left: 200,
                top: 200,
                right: 400,
                bottom: 400,
            },
        };
        //dbg!(GetClientRect(parent, &mut ti.rect));
        dbg!(SendMessageA(
            tooltip,
            TTM_ADDTOOLA,
            0,
            &ti as *const TOOLINFOA as isize
        ));

        dbg!(SetCursorPos(300, 300));
        dbg!(SendMessageA(tooltip, TTM_POPUP, 0, 0));

        let mut msg = std::mem::zeroed();
        while GetMessageA(&mut msg, tooltip, 0, 0) != -1 {
            dbg!(msg.message);
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}
