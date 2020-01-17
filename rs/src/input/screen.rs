#![cfg(windows)]

use crate::globals;

use std::io::Error;
use std::mem::MaybeUninit;

use winapi::um::winuser::{
    CreateWindowExA, CreateWindowExW, DefWindowProcA, DefWindowProcW, DispatchMessageA,
    GetClientRect, GetDesktopWindow, GetMessageA, GetWindowRect, RegisterClassExA,
    RegisterClassExW, SendMessageA, SendMessageW, SetCursorPos, SetWindowPos, TranslateMessage,
    COLOR_WINDOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, WNDCLASSEXA, WNDCLASSEXW, WS_EX_TOPMOST, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE,
};

use winapi::shared::minwindef::{ATOM, DWORD, MAKELONG};
use winapi::shared::windef::{HBRUSH, HWND, RECT};
use winapi::um::commctrl::{
    TOOLINFOA, TOOLINFOW, TOOLTIPS_CLASS, TTF_ABSOLUTE, TTF_IDISHWND, TTF_SUBCLASS, TTM_ADDTOOLA,
    TTM_ADDTOOLW, TTM_POPUP, TTM_SETMAXTIPWIDTH, TTM_TRACKACTIVATE, TTM_TRACKPOSITION,
    TTM_UPDATETIPTEXTW, TTS_ALWAYSTIP, TTS_NOPREFIX,
};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::{GetModuleHandleA, GetModuleHandleW};
use winapi::um::wingdi::{GetPixel, CLR_INVALID};
use winapi::um::winnt::{LPCSTR, LPCWSTR, LPSTR};

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
        // https://stackoverflow.com/q/5896030
        let parent = GetDesktopWindow();
        let appHandle = GetModuleHandleW(std::ptr::null());

        let atom = RegisterClassExW(&WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(DefWindowProcW),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(std::ptr::null_mut()),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: COLOR_WINDOW as HBRUSH,
            lpszMenuName: std::ptr::null_mut(),
            lpszClassName: TOOLTIPS_CLASS.as_ptr() as LPCWSTR,
            hIconSm: std::ptr::null_mut(),
        });

        let toolTipWnd = CreateWindowExW(
            WS_EX_TOPMOST,
            TOOLTIPS_CLASS.as_ptr() as LPCWSTR,
            std::ptr::null_mut(),
            WS_POPUP | TTS_NOPREFIX | TTS_ALWAYSTIP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            parent,
            std::ptr::null_mut(),
            appHandle,
            std::ptr::null_mut(),
        );
        dbg!(toolTipWnd);

        let ti = TOOLINFOW {
            cbSize: std::mem::size_of::<TOOLINFOW>() as u32,
            uFlags: TTF_ABSOLUTE | TTF_IDISHWND, /* | TTF_TRACK */
            // Don't specify TTF_TRACK here. Otherwise the tooltip won't show up.
            hwnd: toolTipWnd,
            hinst: std::ptr::null_mut(),
            uId: toolTipWnd as usize,
            lpszText: "\0\0".as_ptr() as LPSTR,
            lParam: 0,
            lpReserved: std::ptr::null_mut(),
            rect: RECT {
                top: 0,
                left: 0,
                bottom: 0,
                right: 0,
            },
        };

        SendMessageW(
            toolTipWnd,
            TTM_ADDTOOLW,
            0,
            &ti as *const TOOLINFOW as isize,
        );
        SendMessageW(toolTipWnd, TTM_SETMAXTIPWIDTH, 0, 350);

        let ti = TOOLINFOW {
            cbSize: std::mem::size_of::<TOOLINFOW>() as u32,
            hwnd: toolTipWnd,
            uId: toolTipWnd as usize,
            lpszText: "Sample Tip Text\0\0".as_ptr() as LPSTR,
            hinst: std::ptr::null_mut(),
            lParam: 0,
            lpReserved: std::ptr::null_mut(),
            rect: RECT {
                top: 0,
                left: 0,
                bottom: 0,
                right: 0,
            },
            uFlags: 0,
        };

        SendMessageW(
            toolTipWnd,
            TTM_UPDATETIPTEXTW,
            0,
            &ti as *const TOOLINFOW as isize,
        ); // This will update the tooltip content.
        SendMessageW(
            toolTipWnd,
            TTM_TRACKACTIVATE,
            1,
            &ti as *const TOOLINFOW as isize,
        );
        SendMessageW(
            toolTipWnd,
            TTM_TRACKPOSITION,
            0,
            MAKELONG(300, 300) as isize,
        ); // Update the position of your tooltip. Screen coordinate.
        SendMessageW(toolTipWnd, TTM_POPUP, 0, 0); // TTM_POPUP not working.. Don't know why.

        let mut msg = std::mem::zeroed();
        while GetMessageA(&mut msg, toolTipWnd, 0, 0) != -1 {
            dbg!(msg.message);
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}
