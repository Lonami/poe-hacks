use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread::JoinHandle;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser::{
    CallNextHookEx, PostThreadMessageA, SetWindowsHookExA, UnhookWindowsHookEx,
    GET_WHEEL_DELTA_WPARAM, MSLLHOOKSTRUCT, WHEEL_DELTA, WH_MOUSE_LL, WM_MOUSEWHEEL, WM_QUIT,
};
use winapi::um::winuser::{DispatchMessageA, GetMessageA, TranslateMessage, LPMSG, MSG};

static HOOK_THREAD_HANDLE: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static WHEEL_UP_COUNT: AtomicUsize = AtomicUsize::new(0);
static WHEEL_DOWN_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: usize, lparam: isize) -> isize {
    if code < 0 {
        return CallNextHookEx(ptr::null_mut(), code, wparam, lparam);
    }

    let hook = &*(lparam as *const MSLLHOOKSTRUCT);

    if wparam as u32 == WM_MOUSEWHEEL {
        // https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mousewheel
        let delta = GET_WHEEL_DELTA_WPARAM(hook.mouseData as _) / WHEEL_DELTA;
        if delta > 0 {
            WHEEL_UP_COUNT.fetch_add(delta as _, Ordering::SeqCst);
        } else if delta < 0 {
            WHEEL_DOWN_COUNT.fetch_add(-delta as _, Ordering::SeqCst);
        }
    }
    CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
}

// TODO in theory, a keyboard hook would be even better (= faster, more reliable) than using GetKeyState!
// Currently the mouse hook is used to "emulate" "is wheel being used" behaviour.
pub fn install_mouse_hook() {
    let mut hook_handle = HOOK_THREAD_HANDLE.lock().unwrap();
    if hook_handle.is_some() {
        return;
    }

    *hook_handle = Some(std::thread::spawn(|| {
        unsafe {
            // I've tried several things.
            //
            // Registering a hook without pumping messages (akin to `Application.Run` in .NET)
            // causes extreme input lag and no events are really processed (presumably Windows
            // times out if a hook doesn't respond which it can't if there's no message loop).
            //
            // One thing's for sure: the same thread that installs the hook should pump messages.
            // `GetMessage` seems to never return (some say the application needs a window, which
            // we don't really have). We want it to return so we can eventually uninstall the
            // hook though, so `PeekMessage` can be used for that instead.
            //
            // Unfortunately this peek must be called repeatedly which would burn a fair amount
            // of CPU, even with `yield_now` (a small sleep causes noticeable input lag too).
            //
            // However, it seems `PostQuitMessage` works wonders within the hook callback.
            // Although this does mean there needs to be another event to "wake up" the callback.
            // Thankfully `PostThreadMessage` exists to post messages to other threads.
            // Unfortunately, Rust doesn't really let you access a thread's ID from its handle,
            // so we have to store and use it ourselves.
            let tid = GetCurrentThreadId();
            HOOK_THREAD_ID.store(tid, Ordering::SeqCst);

            // Some use `LoadLibraryA(b"User32".as_ptr() as _);` but it seems to work fine without?
            let user32 = ptr::null_mut();
            let hhook = SetWindowsHookExA(WH_MOUSE_LL, Some(mouse_hook_proc), user32, 0);
            let mut msg: MSG = std::mem::zeroed();
            while HOOK_THREAD_ID.load(Ordering::SeqCst) == tid {
                if GetMessageA(&mut msg as LPMSG, ptr::null_mut(), 0, 0) > 0 {
                    // There never seems to be any message but we should do this anyway
                    TranslateMessage(&mut msg as LPMSG);
                    DispatchMessageA(&mut msg as LPMSG);
                }
            }
            let _result = msg.wParam as i32;
            let _success = UnhookWindowsHookEx(hhook);
        }
    }));

    // Wait until HOOK_THREAD_ID is set before releasing the HOOK_THREAD_HANDLE lock.
    // `uninstall_mouse_hook` needs the ID in order to quit properly.
    while HOOK_THREAD_ID.load(Ordering::SeqCst) == 0 {
        std::thread::yield_now();
    }
}

pub fn uninstall_mouse_hook() {
    let hook_handle = match HOOK_THREAD_HANDLE.lock().unwrap().take() {
        Some(handle) => handle,
        None => return,
    };

    let tid = HOOK_THREAD_ID.swap(0, Ordering::SeqCst);
    unsafe {
        PostThreadMessageA(tid, WM_QUIT, 0, 0);
    }
    hook_handle.join().unwrap();
}

/// Return `true` if the mouse wheel was turned up.
///
/// Should be polled at regular intervals for accurate results.
pub fn poll_mouse_wheel_up() -> bool {
    WHEEL_UP_COUNT
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
            if x > 0 {
                Some(x - 1)
            } else {
                None
            }
        })
        .is_ok()
}

/// Return `false` if the mouse wheel was turned down.
///
/// Should be polled at regular intervals for accurate results.
pub fn poll_mouse_wheel_down() -> bool {
    WHEEL_DOWN_COUNT
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
            if x > 0 {
                Some(x - 1)
            } else {
                None
            }
        })
        .is_ok()
}
