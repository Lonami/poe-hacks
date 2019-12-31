#![cfg(windows)]

use std::mem::{size_of, MaybeUninit};

use winapi::um::winuser::{
    GetKeyState, INPUT_u, SendInput, VkKeyScanA, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP,
    VK_CONTROL, VK_MENU, VK_SHIFT,
};

/// Type a modifier.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-vkkeyscana#return-value
fn type_mod(input: &mut INPUT, modifer: i16) {
    unsafe {
        if modifer & 1 != 0 {
            input.u.ki_mut().wVk = VK_SHIFT as u16;
            SendInput(1, input, size_of::<INPUT>() as i32);
        }
        if modifer & 2 != 0 {
            input.u.ki_mut().wVk = VK_CONTROL as u16;
            SendInput(1, input, size_of::<INPUT>() as i32);
        }
        if modifer & 4 != 0 {
            input.u.ki_mut().wVk = VK_MENU as u16;
            SendInput(1, input, size_of::<INPUT>() as i32);
        }
    }
}

/// Type a string using the keyboard.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-vkkeyscana#return-value
/// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
pub fn type_string(string: &str) {
    unsafe {
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: MaybeUninit::<INPUT_u>::zeroed().assume_init(),
        };

        input.u.ki_mut().dwFlags = 0;

        for c in string.as_bytes() {
            let res = VkKeyScanA(*c as i8);
            let modifier = res >> 8;
            let key = res & 0xff;

            // Down
            input.u.ki_mut().dwFlags = 0;
            type_mod(&mut input, modifier);
            input.u.ki_mut().wVk = key as u16;
            SendInput(1, &mut input, size_of::<INPUT>() as i32);

            // Up
            input.u.ki_mut().dwFlags = KEYEVENTF_KEYUP;
            SendInput(1, &mut input, size_of::<INPUT>() as i32);
            type_mod(&mut input, modifier);
        }
    }
}

/// Hold down a Virtual Key Code.
///
/// # References
///
/// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
pub fn hold(vk: u16) {
    unsafe {
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: MaybeUninit::<INPUT_u>::zeroed().assume_init(),
        };

        input.u.ki_mut().wVk = vk;
        input.u.ki_mut().dwFlags = 0;
        SendInput(1, &mut input, size_of::<INPUT>() as i32);
    }
}

/// Release a held Virtual Key Code.
///
/// # References
///
/// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
pub fn release(vk: u16) {
    unsafe {
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: MaybeUninit::<INPUT_u>::zeroed().assume_init(),
        };

        input.u.ki_mut().wVk = vk;
        input.u.ki_mut().dwFlags = KEYEVENTF_KEYUP;
        SendInput(1, &mut input, size_of::<INPUT>() as i32);
    }
}

/// Press (hold down and then release) a Virtual Key Code.
///
/// # References
///
/// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
pub fn press(vk: u16) {
    hold(vk);
    release(vk);
}

/// Is the specified Virtual Key Code down?
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeystate
/// https://docs.microsoft.com/en-gb/windows/desktop/inputdev/virtual-key-codes
pub fn is_down(vk: u16) -> bool {
    unsafe { (GetKeyState(vk as i32) & 0x80) != 0 }
}

/// Get the Virtual Key Code corresponding to the specified character.
///
/// # References
///
/// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-vkkeyscana
pub fn get_vk(character: u8) -> u16 {
    unsafe { (VkKeyScanA(character as i8) as u16) & 0xffu16 }
}
