#![allow(unsafe_code)]

use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

use windows_sys::Win32::Foundation::LPARAM;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MapVirtualKeyW, SendInput,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_INJECTED, PM_NOREMOVE,
    PeekMessageW, PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use super::state::SnapKeyState;
use super::types::SnapKeyPreset;

const SNAPKEY_EXTRA_INFO: usize = 0x534E_4150; // "SNAP"

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::borrow_as_ptr
)]
pub fn send_key_win32(vk: u16, key_down: bool) {
    let scan_code = unsafe { MapVirtualKeyW(u32::from(vk), 0) } as u16;
    let mut flags = KEYEVENTF_SCANCODE;
    if !key_down {
        flags |= KEYEVENTF_KEYUP;
    }
    // Extended keys: Arrow keys (Left 0x25, Up 0x26, Right 0x27, Down 0x28),
    // Insert 0x2D, Delete 0x2E, Home 0x24, End 0x23, PageUp 0x21, PageDown 0x22
    if matches!(vk, 0x21..=0x28 | 0x2D..=0x2E) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan_code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: SNAPKEY_EXTRA_INFO,
            },
        },
    };

    unsafe {
        SendInput(1, &raw const input, std::mem::size_of::<INPUT>() as i32);
    }
}

static STATE: OnceLock<Arc<Mutex<SnapKeyState>>> = OnceLock::new();

struct HookThread {
    thread_id: u32,
    handle: JoinHandle<()>,
}

static HOOK_THREAD: Mutex<Option<HookThread>> = Mutex::new(None);

pub fn get_state() -> &'static Arc<Mutex<SnapKeyState>> {
    STATE.get_or_init(|| Arc::new(Mutex::new(SnapKeyState::new())))
}

#[allow(clippy::cast_possible_truncation, clippy::borrow_as_ptr)]
unsafe extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: usize,
    l_param: LPARAM,
) -> isize {
    if n_code >= 0 && l_param != 0 {
        // SAFETY: for a non-negative low-level keyboard hook code Windows documents
        // `l_param` as a valid pointer to a `KBDLLHOOKSTRUCT` for this callback.
        let kbd = unsafe { &*(l_param as *const KBDLLHOOKSTRUCT) };
        // Ignore events injected by SendInput (flags & LLKHF_INJECTED) or carrying our magic tag
        let is_injected =
            (kbd.flags & LLKHF_INJECTED) != 0 || kbd.dwExtraInfo == SNAPKEY_EXTRA_INFO;

        if !is_injected {
            let vk = kbd.vkCode as u16;
            if vk < 256 {
                let msg = w_param as u32;
                let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
                let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

                if is_down || is_up {
                    let state_arc = get_state();
                    if let Ok(mut state) = state_arc.lock() {
                        // Resynchronize any ghost/stuck keys before handling new input
                        state.sync_ghost_keys(
                            vk,
                            |k| unsafe { GetAsyncKeyState(i32::from(k)) } < 0,
                            send_key_win32,
                        );

                        if state.is_registered(vk) {
                            if is_down {
                                state.handle_key_down(vk, send_key_win32);
                            } else {
                                state.handle_key_up(vk, send_key_win32);
                            }
                            return 1; // Intercept & block the original hardware key event
                        }
                    }
                }
            }
        }
    }
    // SAFETY: forwarding the unchanged callback parameters is required by the hook contract.
    unsafe { CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param) }
}

#[allow(clippy::borrow_as_ptr)]
fn ensure_hook_thread_running() -> Result<(), String> {
    let mut hook_thread = HOOK_THREAD
        .lock()
        .map_err(|_| "SnapKey hook state is poisoned".to_string())?;
    if hook_thread.is_some() {
        return Ok(());
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::Builder::new()
        .name("winsentials-snapkey".into())
        .spawn(move || {
            // SAFETY: this call only queries the identifier of the current OS thread.
            let thread_id = unsafe { GetCurrentThreadId() };
            // SAFETY: the callback has the required ABI and remains valid for the process lifetime;
            // a null module handle and zero thread id install the documented global low-level hook.
            let hook = unsafe {
                SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(keyboard_hook_proc),
                    std::ptr::null_mut(),
                    0,
                )
            };
            if hook.is_null() {
                let _ = tx.send(Err(format!(
                    "Failed to install SnapKey keyboard hook: {}",
                    std::io::Error::last_os_error()
                )));
                return;
            }

            let mut msg = unsafe { std::mem::zeroed() };
            // SAFETY: peeking with a valid `MSG` pointer creates this thread's message queue
            // before its id is published to callers that may post `WM_QUIT` immediately.
            unsafe {
                PeekMessageW(&raw mut msg, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
            }
            let _ = tx.send(Ok(thread_id));

            // SAFETY: `msg` remains initialized and writable for the entire Win32 message loop;
            // `hook` is unhooked exactly once after the loop terminates.
            unsafe {
                while GetMessageW(&raw mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&raw const msg);
                    DispatchMessageW(&raw const msg);
                }
                UnhookWindowsHookEx(hook);
            }
        })
        .map_err(|error| format!("Failed to start SnapKey hook thread: {error}"))?;

    match rx.recv() {
        Ok(Ok(thread_id)) => {
            *hook_thread = Some(HookThread { thread_id, handle });
            Ok(())
        }
        Ok(Err(error)) => {
            let _ = handle.join();
            Err(error)
        }
        Err(error) => {
            let _ = handle.join();
            Err(format!(
                "SnapKey hook thread stopped during startup: {error}"
            ))
        }
    }
}

fn stop_hook_thread() -> Result<(), String> {
    let hook_thread = HOOK_THREAD
        .lock()
        .map_err(|_| "SnapKey hook state is poisoned".to_string())?
        .take();
    let Some(hook_thread) = hook_thread else {
        return Ok(());
    };

    // SAFETY: `thread_id` belongs to the live SnapKey thread and its message queue is
    // created before the handle is stored in `HOOK_THREAD`.
    if unsafe { PostThreadMessageW(hook_thread.thread_id, WM_QUIT, 0, 0) } == 0 {
        return Err(format!(
            "Failed to stop SnapKey hook thread: {}",
            std::io::Error::last_os_error()
        ));
    }
    hook_thread
        .handle
        .join()
        .map_err(|_| "SnapKey hook thread panicked while stopping".to_string())
}

pub fn set_snapkey_preset(preset: SnapKeyPreset) -> Result<(), String> {
    if preset == SnapKeyPreset::Off {
        stop_hook_thread()?;
    } else {
        ensure_hook_thread_running()?;
    }

    let state_arc = get_state();
    let mut state = state_arc
        .lock()
        .map_err(|_| "SnapKey key state is poisoned".to_string())?;
    state.set_preset(preset, send_key_win32);
    Ok(())
}

#[must_use]
pub fn current_snapkey_preset() -> SnapKeyPreset {
    let state_arc = get_state();
    state_arc.lock().map_or(SnapKeyPreset::Off, |s| s.preset)
}

pub fn shutdown_snapkey() {
    let _ = set_snapkey_preset(SnapKeyPreset::Off);
}
