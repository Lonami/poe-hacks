#![cfg(windows)]

use crate::globals;

use std::io::Error;
use std::mem::{self, MaybeUninit};
use std::ptr;
use winapi::shared::minwindef::{ATOM, DWORD, LPVOID};
use winapi::shared::windef::{HBITMAP, HBRUSH, HDC, HGDIOBJ, HWND, RECT};
use winapi::shared::winerror::ERROR_INVALID_PARAMETER;
use winapi::um::commctrl::TOOLTIPS_CLASS;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::wingdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
    GetPixel, SelectObject, SetBkMode, SetTextColor, BITMAPINFO, BI_RGB, CLR_INVALID,
    DIB_RGB_COLORS, HGDI_ERROR, SRCCOPY, TRANSPARENT,
};
use winapi::um::winnt::LPCSTR;
use winapi::um::winuser::{
    CreateWindowExA, DefWindowProcA, DestroyWindow, DrawTextA, GetDC, GetDesktopWindow,
    GetWindowDC, GetWindowRect, RegisterClassExA, ReleaseDC, SetWindowPos, COLOR_WINDOW,
    CW_USEDEFAULT, DT_CALCRECT, DT_NOCLIP, DT_SINGLELINE, SWP_NOACTIVATE, SWP_NOZORDER,
    WNDCLASSEXA, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
};

// Structures used for the automatic `Drop` cleanup
struct Window(HWND);
struct WindowDC(HDC);

// TODO Probably should use https://doc.rust-lang.org/std/ffi/index.html to deal with wide strings
// TODO Consider publishing this input lib on crates.io?

const ORB_START_Y: f64 = 0.8;

#[derive(Clone)]
pub struct Screenshot {
    pub width: usize,
    pub height: usize,
    colors: Box<[u8]>,
    offset_y: usize,
}

pub struct Screen {
    dc: HDC,
    dc_mem: HDC,
    bmp: HGDIOBJ,
    bmp_info: BITMAPINFO,
    screenshot: Screenshot,
}

impl Screenshot {
    fn new(width: usize, height: usize, offset_y: usize) -> Self {
        Self {
            width,
            height,
            colors: vec![0; height * width * 3].into_boxed_slice(),
            offset_y,
        }
    }

    pub fn color(&self, x: f64, y: f64) -> (u8, u8, u8) {
        let x = (self.width as f64 * x) as usize;
        let y = ((self.height + self.offset_y) as f64 * y) as usize;

        // Will be out of bounds when selecting colors above the orbs or out of screen
        let i = ((y - self.offset_y) * self.width + x) * 3;
        (self.colors[i + 2], self.colors[i + 1], self.colors[i + 0])
    }

    pub fn update_from(&mut self, other: &Screenshot) {
        self.colors.copy_from_slice(&other.colors);
    }
}

impl Screen {
    pub fn new() -> Result<Self, &'static str> {
        let (width, height) = size().unwrap();
        let offset_y = (ORB_START_Y * height as f64) as usize;

        // On average, taking just 20% of the screen seems to take 5ms less (from 25ms otherwise).
        // Still quite expensive to get pixels from the screen, but slightly better.
        // This makes the code highly specialised for that one use case, and won't work beyond that.
        let height = height - offset_y;

        let dc = unsafe { GetDC(ptr::null_mut()) };
        if dc.is_null() {
            return Err("failed to get root dc");
        }
        let dc_mem = unsafe { CreateCompatibleDC(ptr::null_mut()) };
        if dc_mem.is_null() {
            return Err("failed to create compatible root dc");
        }
        let bmp = unsafe { CreateCompatibleBitmap(dc, width as i32, height as i32) as HGDIOBJ };
        if bmp.is_null() {
            return Err("failed to create compatible bitmap");
        }
        let so = unsafe { SelectObject(dc_mem, bmp) };
        if so.is_null() {
            return Err("failed to select object: invalid region");
        }
        if so == HGDI_ERROR {
            return Err("failed to select object: gdi error");
        }

        let mut bmp_info: BITMAPINFO = unsafe { mem::zeroed() };
        bmp_info.bmiHeader.biBitCount = 24;
        bmp_info.bmiHeader.biCompression = BI_RGB;
        bmp_info.bmiHeader.biPlanes = 1;
        bmp_info.bmiHeader.biHeight = -(height as i32); // a top-down DIB is specified by setting the height to a negative number
        bmp_info.bmiHeader.biWidth = width as i32;
        bmp_info.bmiHeader.biSize = mem::size_of::<BITMAPINFO>() as u32;
        let screenshot = Screenshot::new(width, height, offset_y);

        Ok(Self {
            dc,
            dc_mem,
            bmp,
            bmp_info,
            screenshot,
        })
    }

    pub fn refresh(&mut self) -> Result<(), &'static str> {
        let res = unsafe {
            BitBlt(
                self.dc_mem,
                0,
                0,
                self.screenshot.width as i32,
                self.screenshot.height as i32,
                self.dc,
                0,
                self.screenshot.offset_y as i32,
                SRCCOPY,
            )
        };
        if res == 0 {
            return Err("failed to bit-block transfer screen data");
        };
        let res = unsafe {
            GetDIBits(
                self.dc_mem,
                self.bmp as HBITMAP,
                0,
                self.screenshot.height as u32,
                self.screenshot.colors.as_mut_ptr() as LPVOID,
                &mut self.bmp_info,
                DIB_RGB_COLORS,
            )
        };
        if res == 0 {
            return Err("failed to get bits from compatible bitmap");
        }
        if res == ERROR_INVALID_PARAMETER as i32 {
            return Err("failed to get bits from compatible bitmap: invalid parameter");
        }
        Ok(())
    }

    pub fn screenshot(&self) -> &Screenshot {
        &self.screenshot
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.bmp);
            DeleteDC(self.dc_mem);
            ReleaseDC(ptr::null_mut(), self.dc);
        }
    }
}

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
        let mouse = crate::win::mouse::get().map_err(|_| GetLastError())?;
        let x = if (mouse.0 as i32) < rect.right {
            mouse.0 as i32 + 15
        } else {
            mouse.0 as i32 - rect.right
        };
        let y = mouse.1 as i32;

        if SetWindowPos(
            window.0,
            std::ptr::null_mut(),
            x,
            y,
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
