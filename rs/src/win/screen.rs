#![cfg(windows)]
use std::io::{Error, ErrorKind};
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
    GetForegroundWindow, GetWindowDC, GetWindowRect, GetWindowThreadProcessId, RegisterClassExA,
    ReleaseDC, SetWindowPos, COLOR_WINDOW, CW_USEDEFAULT, DT_CALCRECT, DT_NOCLIP, DT_SINGLELINE,
    SWP_NOACTIVATE, SWP_NOZORDER, WNDCLASSEXA, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
};

// Structures used for the automatic `Drop` cleanup
struct Window(HWND);
struct WindowDC(HDC);

// TODO Probably should use https://doc.rust-lang.org/std/ffi/index.html to deal with wide strings
// TODO Consider publishing this input lib on crates.io?

#[derive(Clone, Debug)]
pub struct Rect {
    pub left: usize,
    pub top: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct Screenshot {
    pub region: Rect,
    row_size: usize,
    colors: Box<[u8]>,
}

pub struct ScreenshotIter<'s> {
    screenshot: &'s Screenshot,
    y_idx: usize,
    x_cnt: usize,
    i: usize,
}

pub struct Screen {
    dc: HDC,
    dc_mem: HDC,
    bmp: HGDIOBJ,
    bmp_info: BITMAPINFO,
    screenshot: Screenshot,
}

impl Screenshot {
    fn new(region: Rect) -> Self {
        // https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getdibits
        // The scan lines must be aligned on a DWORD except for RLE compressed bitmaps.
        // Else "exit code: 0xc0000374, STATUS_HEAP_CORRUPTION" will occur,
        // because `GetDIBits` will attempt to write outside the designated buffer.
        let row_byte_count = region.width * 3; // RGB, 1 byte per
        let row_size = ((row_byte_count + 3) / 4) * 4; // DWORD, 4 bytes
        let size = region.height * row_size;
        Self {
            region,
            row_size,
            colors: vec![0; size].into_boxed_slice(),
        }
    }

    pub fn color(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let i = (y * self.row_size + x) * 3;
        (self.colors[i + 2], self.colors[i + 1], self.colors[i + 0])
    }

    pub fn colors(&self) -> ScreenshotIter {
        ScreenshotIter {
            screenshot: self,
            y_idx: 0,
            x_cnt: 0,
            i: 0,
        }
    }
}

impl<'s> Iterator for ScreenshotIter<'s> {
    type Item = (u8, u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        if i == self.screenshot.colors.len() {
            return None;
        }

        self.x_cnt += 1;
        if self.x_cnt == self.screenshot.region.width {
            self.x_cnt = 0;
            self.y_idx += self.screenshot.row_size;
            self.i = self.y_idx;
        } else {
            self.i += 3;
        }

        Some((
            self.screenshot.colors[i + 2],
            self.screenshot.colors[i + 1],
            self.screenshot.colors[i + 0],
        ))
    }
}

impl Screen {
    /// Creates a capture of a region in the screen, which can be refreshed to contain data.
    ///
    /// Smaller regions are likely to be faster to refresh (but can still be surprisingly slow, ranging from 2ms to 20ms).
    pub fn capture_region(region: Rect) -> Result<Self, Error> {
        let width = region.width as i32;
        let height = region.height as i32;

        let dc = get_desktop_dc()?;
        let dc_mem = unsafe { CreateCompatibleDC(ptr::null_mut()) };
        if dc_mem.is_null() {
            return Err(Error::new(
                ErrorKind::Other,
                "call to CreateCompatibleDC returned null",
            ));
        }
        let bmp = unsafe { CreateCompatibleBitmap(dc, width, height) as HGDIOBJ };
        if bmp.is_null() {
            return Err(Error::new(
                ErrorKind::Other,
                "call to CreateCompatibleBitmap returned null",
            ));
        }
        let so = unsafe { SelectObject(dc_mem, bmp) };
        if so.is_null() {
            return Err(Error::new(
                ErrorKind::Other,
                "call to SelectObject returned null",
            ));
        }
        if so == HGDI_ERROR {
            return Err(Error::new(
                ErrorKind::Other,
                "call to SelectObject returned HGDI_ERROR",
            ));
        }

        let mut bmp_info: BITMAPINFO = unsafe { mem::zeroed() };
        bmp_info.bmiHeader.biBitCount = 24;
        bmp_info.bmiHeader.biCompression = BI_RGB;
        bmp_info.bmiHeader.biPlanes = 1;
        bmp_info.bmiHeader.biHeight = -height; // a top-down DIB is specified by setting the height to a negative number
        bmp_info.bmiHeader.biWidth = width;
        bmp_info.bmiHeader.biSize = mem::size_of::<BITMAPINFO>() as u32;
        let screenshot = Screenshot::new(region);

        Ok(Self {
            dc,
            dc_mem,
            bmp,
            bmp_info,
            screenshot,
        })
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        let res = unsafe {
            BitBlt(
                self.dc_mem,
                0,
                0,
                self.screenshot.region.width as i32,
                self.screenshot.region.height as i32,
                self.dc,
                self.screenshot.region.left as i32,
                self.screenshot.region.top as i32,
                SRCCOPY,
            )
        };
        if res == 0 {
            return Err(Error::last_os_error());
        };
        let res = unsafe {
            GetDIBits(
                self.dc_mem,
                self.bmp as HBITMAP,
                0,
                self.screenshot.region.height as u32,
                self.screenshot.colors.as_mut_ptr() as LPVOID,
                &mut self.bmp_info,
                DIB_RGB_COLORS,
            )
        };
        if res == 0 {
            return Err(Error::new(
                ErrorKind::Other,
                "failed to get bits from compatible bitmap",
            ));
        }
        if res == ERROR_INVALID_PARAMETER as i32 {
            return Err(Error::new(
                ErrorKind::Other,
                "failed to get bits from compatible bitmap: invalid parameter",
            ));
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

static mut DESKTOP_DC: HDC = ptr::null_mut();

fn get_desktop_dc() -> Result<HDC, Error> {
    // WARNING: NOT THREAD SAFE, BUT THE PROGRAM DOESN'T USE THREADS FOR NOW
    unsafe {
        if DESKTOP_DC.is_null() {
            DESKTOP_DC = GetDC(ptr::null_mut());
        }
        if DESKTOP_DC.is_null() {
            Err(Error::new(ErrorKind::Other, "call to GetDC failed"))
        } else {
            // We never release this, but it's fine because it lives for as long as the program does.
            Ok(DESKTOP_DC)
        }
    }
}

/// Gets the primary screen's size.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getdesktopwindow
pub fn size() -> Result<Rect, Error> {
    unsafe {
        let mut desktop = MaybeUninit::uninit();
        let handle = GetDesktopWindow();
        if GetWindowRect(handle, desktop.as_mut_ptr()) == 0 {
            Err(Error::last_os_error())
        } else {
            let desktop = desktop.assume_init();
            Ok(Rect {
                left: desktop.left as usize,
                top: desktop.top as usize,
                width: (desktop.right - desktop.left) as usize,
                height: (desktop.bottom - desktop.top) as usize,
            })
        }
    }
}

/// Gets an on-screen color as `(r, g, b)`.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/wingdi/nf-wingdi-getpixel
pub fn color(x: usize, y: usize) -> Result<(u8, u8, u8), Error> {
    let dc = get_desktop_dc()?;
    let color = unsafe { GetPixel(dc, x as i32, y as i32) };
    if color != CLR_INVALID {
        Ok((
            ((color >> 0) & 0xff) as u8,
            ((color >> 2) & 0xff) as u8,
            ((color >> 4) & 0xff) as u8,
        ))
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "call to GetPixel returned CLR_INVALID",
        ))
    }
}

/// Get the process ID for the owner of the thread with the window in the foreground.
///
/// # References
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow
/// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
pub fn get_foreground_pid() -> Result<u32, Error> {
    let hwnd = unsafe { GetForegroundWindow() };
    let mut proc_id = 0;
    let thread_id = unsafe { GetWindowThreadProcessId(hwnd, &mut proc_id) };
    if thread_id == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(proc_id)
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
