#![allow(non_upper_case_globals)]
use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;

// https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid
const FOLDERID_RoamingAppData: [u8; 16] = [
    0xDB, 0x85, 0xB6, 0x3E, 0xF9, 0x65, 0xF6, 0x4C, 0xA0, 0x3A, 0xE3, 0xEF, 0x65, 0x72, 0x9F, 0x3D,
];

#[link(name = "kernel32")]
unsafe extern "system" {
    fn SHGetKnownFolderPath(
        rfid: *const [u8; 16],
        dwFlags: i32,
        hToken: *mut c_void,
        ppszPath: *mut *mut u16,
    ) -> i32;

    fn CoTaskMemFree(pv: *mut c_void);
}

pub fn get_app_data_path() -> PathBuf {
    let mut buffer_ptr = ptr::null_mut();
    let res = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_RoamingAppData,
            0,
            std::ptr::null_mut(),
            &mut buffer_ptr,
        )
    };
    assert_eq!(res, 0);
    assert!(!buffer_ptr.is_null());

    let result = PathBuf::from_str(
        &char::decode_utf16(
            (0..)
                .map(|o| unsafe { *buffer_ptr.wrapping_add(o) })
                .take_while(|&c| c != 0),
        )
        .filter_map(|c| c.ok())
        .collect::<String>(),
    )
    .unwrap();

    unsafe { CoTaskMemFree(buffer_ptr as _) };

    result
}
