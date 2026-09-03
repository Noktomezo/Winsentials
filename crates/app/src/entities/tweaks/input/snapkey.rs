#![allow(unsafe_code)]

use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::LPARAM;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    KEYEVENTF_SCANCODE, MapVirtualKeyW, SendInput,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_INJECTED, PM_NOREMOVE,
    PeekMessageW, PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

const SNAPKEY_EXTRA_INFO: usize = 0x534E_4150; // "SNAP"

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapKeyPreset {
    #[default]
    Off,
    Wasd,
    ArrowKeys,
    Esdf,
    Azerty,
}

impl SnapKeyPreset {
    pub const ALL: [Self; 5] = [
        Self::Off,
        Self::Wasd,
        Self::ArrowKeys,
        Self::Esdf,
        Self::Azerty,
    ];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Wasd => "wasd",
            Self::ArrowKeys => "arrow_keys",
            Self::Esdf => "esdf",
            Self::Azerty => "azerty",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "off" => Some(Self::Off),
            "wasd" => Some(Self::Wasd),
            "arrow_keys" => Some(Self::ArrowKeys),
            "esdf" => Some(Self::Esdf),
            "azerty" => Some(Self::Azerty),
            _ => None,
        }
    }
}

#[must_use]
pub fn snapkey_preset_label(preset: SnapKeyPreset) -> String {
    match preset {
        SnapKeyPreset::Off => rust_i18n::t!("tweaks.snapkey_off").to_string(),
        SnapKeyPreset::Wasd => rust_i18n::t!("tweaks.snapkey_wasd").to_string(),
        SnapKeyPreset::ArrowKeys => rust_i18n::t!("tweaks.snapkey_arrow_keys").to_string(),
        SnapKeyPreset::Esdf => rust_i18n::t!("tweaks.snapkey_esdf").to_string(),
        SnapKeyPreset::Azerty => rust_i18n::t!("tweaks.snapkey_azerty").to_string(),
    }
}

#[must_use]
pub const fn snapkey_preset_icon(preset: SnapKeyPreset) -> &'static str {
    match preset {
        SnapKeyPreset::Off => "icons/ban.svg",
        SnapKeyPreset::Wasd => "icons/gamepad-2.svg",
        SnapKeyPreset::ArrowKeys => "icons/gamepad-directional.svg",
        SnapKeyPreset::Esdf | SnapKeyPreset::Azerty => "icons/keyboard.svg",
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct KeyState {
    pub registered: bool,
    pub key_down: bool,
    pub group: u8,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct GroupState {
    pub previous_key: u16,
    pub active_key: u16,
}

#[derive(Clone, Debug)]
pub struct SnapKeyState {
    pub preset: SnapKeyPreset,
    pub keys: [KeyState; 256],
    pub groups: [GroupState; 4],
}

impl Default for SnapKeyState {
    fn default() -> Self {
        Self {
            preset: SnapKeyPreset::Off,
            keys: [KeyState::default(); 256],
            groups: [GroupState::default(); 4],
        }
    }
}

impl SnapKeyState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_preset<F: FnMut(u16, bool)>(&mut self, preset: SnapKeyPreset, mut send_key: F) {
        // First, release any active simulated keys from previous preset
        for group in &mut self.groups {
            if group.active_key != 0 {
                send_key(group.active_key, false);
            }
            group.previous_key = 0;
            group.active_key = 0;
        }

        self.preset = preset;
        self.keys = [KeyState::default(); 256];
        self.groups = [GroupState::default(); 4];

        match preset {
            SnapKeyPreset::Off => {}
            SnapKeyPreset::Wasd => {
                // Group 0: Strafe (A = 65, D = 68)
                self.register_pair(0, 65, 68);
                // Group 1: Movement (S = 83, W = 87)
                self.register_pair(1, 83, 87);
            }
            SnapKeyPreset::ArrowKeys => {
                // Group 0: Horizontal (Left = 37, Right = 39)
                self.register_pair(0, 37, 39);
                // Group 1: Vertical (Up = 38, Down = 40)
                self.register_pair(1, 38, 40);
            }
            SnapKeyPreset::Esdf => {
                // Group 0: Strafe (S = 83, F = 70)
                self.register_pair(0, 83, 70);
                // Group 1: Movement (D = 68, E = 69)
                self.register_pair(1, 68, 69);
            }
            SnapKeyPreset::Azerty => {
                // Group 0: Strafe (Q = 81, D = 68)
                self.register_pair(0, 81, 68);
                // Group 1: Movement (S = 83, Z = 90)
                self.register_pair(1, 83, 90);
            }
        }
    }

    fn register_pair(&mut self, group: u8, key1: u16, key2: u16) {
        if let Some(k1) = self.keys.get_mut(key1 as usize) {
            k1.registered = true;
            k1.group = group;
            k1.key_down = false;
        }
        if let Some(k2) = self.keys.get_mut(key2 as usize) {
            k2.registered = true;
            k2.group = group;
            k2.key_down = false;
        }
    }

    #[must_use]
    pub fn is_registered(&self, vk: u16) -> bool {
        self.keys.get(vk as usize).is_some_and(|k| k.registered)
    }

    pub fn handle_key_down<F: FnMut(u16, bool)>(&mut self, vk: u16, mut send_key: F) {
        let Some(key_info) = self.keys.get_mut(vk as usize) else {
            return;
        };
        if !key_info.registered {
            return;
        }

        let group_idx = key_info.group as usize;
        if !key_info.key_down {
            key_info.key_down = true;
            send_key(vk, true);

            if let Some(group) = self.groups.get_mut(group_idx) {
                if group.active_key == 0 || group.active_key == vk {
                    group.active_key = vk;
                } else {
                    group.previous_key = group.active_key;
                    group.active_key = vk;
                    send_key(group.previous_key, false);
                }
            }
        }
    }

    pub fn handle_key_up<F: FnMut(u16, bool)>(&mut self, vk: u16, mut send_key: F) {
        let Some(key_info) = self.keys.get_mut(vk as usize) else {
            return;
        };
        if !key_info.registered {
            return;
        }

        let group_idx = key_info.group as usize;
        let Some(group) = self.groups.get_mut(group_idx) else {
            return;
        };

        if group.previous_key == vk && !key_info.key_down {
            group.previous_key = 0;
        }

        if key_info.key_down {
            key_info.key_down = false;
            if group.active_key == vk && group.previous_key != 0 {
                send_key(vk, false);
                group.active_key = group.previous_key;
                let to_press = group.active_key;
                group.previous_key = 0;
                send_key(to_press, true);
            } else {
                group.previous_key = 0;
                if group.active_key == vk {
                    group.active_key = 0;
                }
                send_key(vk, false);
            }
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::borrow_as_ptr
)]
fn send_key_win32(vk: u16, key_down: bool) {
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

fn get_state() -> &'static Arc<Mutex<SnapKeyState>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapkey_presets_enum() {
        assert_eq!(SnapKeyPreset::ALL.len(), 5);
        for preset in SnapKeyPreset::ALL {
            assert_eq!(SnapKeyPreset::from_id(preset.id()), Some(preset));
        }
        assert_eq!(SnapKeyPreset::from_id("invalid"), None);
    }

    #[test]
    fn test_snapkey_wasd_socd_counter_strafe() {
        let mut state = SnapKeyState::new();
        let mut sent = Vec::new();
        state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
        sent.clear();

        // 1. Press A (65)
        state.handle_key_down(65, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(65, true)]);
        sent.clear();

        // 2. While holding A, press D (68) -> Opposing cardinal direction
        // SnappyTappy behavior: D down is sent, A is immediately released!
        state.handle_key_down(68, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(68, true), (65, false)]);
        sent.clear();

        // 3. Release D while still holding A
        // SnappyTappy behavior: D up is sent, A is immediately re-pressed!
        state.handle_key_up(68, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(68, false), (65, true)]);
        sent.clear();

        // 4. Release A
        state.handle_key_up(65, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(65, false)]);
    }

    #[test]
    fn test_snapkey_press_both_release_first() {
        let mut state = SnapKeyState::new();
        let mut sent = Vec::new();
        state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
        sent.clear();

        // 1. Press A (65)
        state.handle_key_down(65, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(65, true)]);
        sent.clear();

        // 2. Press D (68) -> D down sent, A released
        state.handle_key_down(68, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(68, true), (65, false)]);
        sent.clear();

        // 3. Release A while D is still held
        state.handle_key_up(65, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(65, false)]);
        sent.clear();

        // 4. Release D
        state.handle_key_up(68, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(68, false)]);
    }

    #[test]
    fn test_snapkey_preset_switching_releases_held_keys() {
        let mut state = SnapKeyState::new();
        let mut sent = Vec::new();
        state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
        sent.clear();

        // Press A
        state.handle_key_down(65, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(65, true)]);
        sent.clear();

        // Switch to ArrowKeys while A is held
        state.set_preset(SnapKeyPreset::ArrowKeys, |vk, down| sent.push((vk, down)));
        // A must be released during preset transition
        assert_eq!(sent, vec![(65, false)]);
        assert!(!state.is_registered(65));
        assert!(state.is_registered(37)); // Left arrow is now registered
    }

    #[test]
    fn test_snapkey_independent_groups_movement_and_strafe() {
        let mut state = SnapKeyState::new();
        let mut sent = Vec::new();
        state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
        sent.clear();

        // Press W (87) [Group 1 - Forward]
        state.handle_key_down(87, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(87, true)]);
        sent.clear();

        // Press A (65) [Group 0 - Strafe] - Should NOT affect W!
        state.handle_key_down(65, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(65, true)]);
        sent.clear();

        // Press S (83) [Group 1 - Backward] - Should interrupt W, but NOT A!
        state.handle_key_down(83, |vk, down| sent.push((vk, down)));
        assert_eq!(sent, vec![(83, true), (87, false)]);
        sent.clear();

        // Clean release
        state.handle_key_up(83, |vk, down| sent.push((vk, down)));
        state.handle_key_up(87, |vk, down| sent.push((vk, down)));
        state.handle_key_up(65, |vk, down| sent.push((vk, down)));
    }
}
