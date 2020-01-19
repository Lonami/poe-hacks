#![cfg(windows)]

use crate::globals;

use std::io::Error;
use std::mem::MaybeUninit;

use winapi::um::winuser::{
    CreateWindowExA, DefWindowProcA, DestroyWindow, DrawTextA, GetDesktopWindow, GetWindowDC,
    GetWindowRect, RegisterClassExA, SetWindowPos, COLOR_WINDOW, CW_USEDEFAULT, DT_CALCRECT,
    DT_NOCLIP, DT_SINGLELINE, SWP_NOACTIVATE, SWP_NOZORDER, WNDCLASSEXA, WS_EX_TOPMOST, WS_POPUP,
    WS_VISIBLE,
};

use winapi::shared::minwindef::{ATOM, DWORD};
use winapi::shared::windef::{HBRUSH, HDC, HWND, RECT};
use winapi::um::commctrl::TOOLTIPS_CLASS;
use winapi::um::errhandlingapi::GetLastError;

use winapi::um::wingdi::{DeleteDC, GetPixel, SetBkMode, SetTextColor, CLR_INVALID, TRANSPARENT};
use winapi::um::winnt::LPCSTR;

// Structures used for the automatic `Drop` cleanup
struct Window(HWND);
struct WindowDC(HDC);

pub struct Tooltip {
    _window: Window,
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if DestroyWindow(self.0) == 0 {
                eprintln!(
                    "failed to destroy window {:?}: error code {}",
                    self.0,
                    GetLastError()
                );
            }
        }
    }
}

impl Drop for WindowDC {
    fn drop(&mut self) {
        unsafe {
            if DeleteDC(self.0) == 0 {
                eprintln!("failed to destroy dc {:?}", self.0);
            }
        }
    }
}

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
/// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexa
/// https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--1300-1699-
pub fn register_window_class() -> Result<ATOM, DWORD> {
    unsafe {
        // TODO: Why can `hInstance` be both null and `GetModuleHandleA(std::ptr::null())`?
        let atom = RegisterClassExA(&WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: 0,
            lpfnWndProc: Some(DefWindowProcA),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: COLOR_WINDOW as HBRUSH,
            lpszMenuName: std::ptr::null_mut(),
            lpszClassName: TOOLTIPS_CLASS.as_ptr() as LPCSTR,
            hIconSm: std::ptr::null_mut(),
        });

        if atom != 0 {
            Ok(atom)
        } else {
            Err(GetLastError())
        }
    }
}

fn create_window() -> Result<Window, DWORD> {
    // TODO Maybe use the atom instead of the class name
    unsafe {
        // TODO: Why can the parent be both null and `GetDesktopWindow()`?
        let parent = std::ptr::null_mut();

        // TODO: Why can the app handle be both null and `GetModuleHandleA(std::ptr::null())`?
        let app_handle = std::ptr::null_mut();

        let handle = CreateWindowExA(
            WS_EX_TOPMOST,
            TOOLTIPS_CLASS.as_ptr() as LPCSTR,
            std::ptr::null_mut(),
            WS_POPUP | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            parent,
            std::ptr::null_mut(),
            app_handle,
            std::ptr::null_mut(),
        );

        if !handle.is_null() {
            Ok(Window(handle))
        } else {
            Err(GetLastError())
        }
    }
}

fn get_window_dc(window: &Window) -> Result<WindowDC, DWORD> {
    unsafe {
        let window_dc = GetWindowDC(window.0);
        if window_dc.is_null() {
            Err(1)
        } else {
            Ok(WindowDC(window_dc))
        }
    }
}

pub fn create_tooltip(text: &str) -> Result<Tooltip, DWORD> {
    unsafe {
        let window = create_window()?;
        let window_dc = get_window_dc(&window)?;

        // Calculate required width
        let mut rect: RECT = std::mem::zeroed();
        if DrawTextA(
            window_dc.0,
            text.as_ptr() as LPCSTR,
            text.len() as i32,
            &mut rect,
            DT_CALCRECT,
        ) == 0
        {
            return Err(2);
        }

        // Update position to be below mouse and of the right size
        let mouse = crate::input::mouse::get().map_err(|_| GetLastError())?;
        if SetWindowPos(
            window.0,
            std::ptr::null_mut(),
            mouse.0 as i32,
            mouse.1 as i32 + 20,
            rect.right,
            rect.bottom,
            SWP_NOZORDER | SWP_NOACTIVATE,
        ) == 0
        {
            return Err(GetLastError());
        }

        // Prepare fill and background color
        if SetTextColor(window_dc.0, 0x00000000) == CLR_INVALID {
            return Err(3);
        };
        if SetBkMode(window_dc.0, TRANSPARENT as i32) == 0 {
            return Err(4);
        };

        // Draw the text
        if DrawTextA(
            window_dc.0,
            text.as_ptr() as LPCSTR,
            text.len() as i32,
            &mut rect,
            DT_SINGLELINE | DT_NOCLIP,
        ) == 0
        {
            return Err(5);
        };

        Ok(Tooltip { _window: window })
    }
}
