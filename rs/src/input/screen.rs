#![cfg(windows)]

use crate::globals;

use std::io::Error;
use std::mem::MaybeUninit;

use winapi::um::winuser::{
    CreateWindowExA, DefWindowProcA, GetDesktopWindow, GetWindowRect, RegisterClassExA,
    COLOR_WINDOW, CS_HREDRAW, CS_VREDRAW, WNDCLASSEXA, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use winapi::shared::minwindef::{ATOM, DWORD};
use winapi::shared::windef::{HBRUSH, HWND};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::wingdi::{GetPixel, CLR_INVALID};
use winapi::um::winnt::LPCSTR;

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
pub fn create_window(class_name: &str) -> Result<HWND, DWORD> {
    // TODO Maybe use the atom instead of the class name
    unsafe {
        let window_handle = CreateWindowExA(
            0, // https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
            class_name.as_ptr() as LPCSTR, // lpClassName
            "A GUI window".as_ptr() as LPCSTR, // lpWindowName
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, // dwStyle
            100, // X
            100, // Y
            200, // nWidth
            200, // nHeight
            std::ptr::null_mut(), // hWndParent
            std::ptr::null_mut(), // hMenu
            std::ptr::null_mut(), // hInstance
            std::ptr::null_mut(), // lpParam
        );
        if !window_handle.is_null() {
            Ok(window_handle)
        } else {
            Err(GetLastError())
        }
    }
}

pub fn show_tooltip() {
    // TODO https://stackoverflow.com/a/22729474
    /*
    #pragma comment(linker,"/manifestdependency:\"type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'\"")

    hWndtoolTip = CreateWindowEx(WS_EX_TOPMOST, TOOLTIPS_CLASS, 0, WS_POPUP | TTS_NOPREFIX | TTS_ALWAYSTIP, CW_USEDEFAULT, CW_USEDEFAULT,CW_USEDEFAULT,CW_USEDEFAULT, hWndParent, 0, hInstance, 0);
    SetWindowPos(hWndtoolTip, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);

    TOOLINFO ti = {};
    ti.cbSize = sizeof(TOOLINFO);
    ti.uFlags = TTF_SUBCLASS;
    ti.hwnd   = hWndParent;
    ti.hinst  = hInstance;
    ti.uId    = (UINT)hWndtoolTip;
    ti.lpszText = L"tool-tip";
    GetClientRect(hWndParent, &ti.rect);
    SendMessage(hWndtoolTip, TTM_ADDTOOL, 0, (LPARAM)&ti);

    SetCursorPos(300, 300);
    SendMessage(hWndtoolTip, TTM_POPUP, 0, 0);
        */

    // ..or https://stackoverflow.com/a/19668489
    // https://docs.microsoft.com/en-us/windows/win32/controls/create-a-tooltip-for-a-control
    /*
    HWND CreateToolTip(int toolID, HWND hDlg, HINSTANCE hInst, PTSTR pszText)
    {
        if (!toolID || !hDlg || !pszText)
        {
            return NULL;
        }

        // Get the window of the tool.
        HWND hwndTool = GetDlgItem(hDlg, toolID);
        if (!hwndTool)
        {
            return NULL;
        }

        // Create the tooltip. g_hInst is the global instance handle.
        HWND hwndTip = CreateWindowEx(NULL, TOOLTIPS_CLASS, NULL,
                                  WS_POPUP |TTS_ALWAYSTIP | TTS_BALLOON,
                                  CW_USEDEFAULT, CW_USEDEFAULT,
                                  CW_USEDEFAULT, CW_USEDEFAULT,
                                  hDlg, NULL,
                                  hInst, NULL);

       if (!hwndTip)
       {
           return NULL;
       }

        // Associate the tooltip with the tool.
        TOOLINFO toolInfo = { 0 };
        toolInfo.cbSize = sizeof(toolInfo);
        toolInfo.hwnd = hDlg;
        toolInfo.uFlags = TTF_IDISHWND | TTF_SUBCLASS;
        toolInfo.uId = (UINT_PTR)hwndTool;
        toolInfo.lpszText = pszText;
        if (!SendMessage(hwndTip, TTM_ADDTOOL, 0, (LPARAM)&toolInfo))
        {
            DestroyWindow(hwndTip);
            return NULL;
        }

        return hwndTip;
    }
        */
}
