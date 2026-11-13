// Button remapping uses Windows low-level hooks, so it only works on Windows for now.
// On macOS and Linux the same functions exist but do nothing (see `start` at the bottom).
#![cfg_attr(not(windows), allow(dead_code, unused_imports))]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;

use rdev::{simulate, EventType, Key};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
#[cfg(windows)]
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HHOOK,
    KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL,
};

use super::types::{Mapping, RemapAction};
use crate::macros::types::MacroAction;
use crate::store::RemapFile;

// Remapping is OFF until the UI turns it on (safe default).
static ENABLED: AtomicBool = AtomicBool::new(false);
static DETECTING: AtomicBool = AtomicBool::new(false);
static STARTED: AtomicBool = AtomicBool::new(false);
static APP: OnceLock<AppHandle> = OnceLock::new();
static MAPPINGS: LazyLock<Mutex<HashMap<String, RemapAction>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
// The actions of the macros assigned to a button, so playing needs no window.
static MACROS: LazyLock<Mutex<HashMap<String, Vec<MacroAction>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
// Buttons whose "down" we swallowed: their "up" must be swallowed too.
static SWALLOWED: LazyLock<Mutex<HashSet<&'static str>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// Work that must not run inside a hook callback: the worker thread does it.
enum Job {
    SendKeys(Vec<String>),
    Emit(&'static str, serde_json::Value),
    PlayMacro(String),
    // F10 turned remapping off: save that to remap.json.
    PersistDisabled,
}

static JOBS: OnceLock<SyncSender<Job>> = OnceLock::new();

// Never blocks: when the queue is full the job is dropped.
fn queue(job: Job) {
    if let Some(sender) = JOBS.get() {
        let _ = sender.try_send(job);
    }
}

fn emit<S: Serialize>(event: &'static str, payload: S) {
    if let Ok(value) = serde_json::to_value(payload) {
        queue(Job::Emit(event, value));
    }
}

// Starts the worker thread that runs the queued jobs (only once).
fn start_worker() {
    let (sender, receiver) = sync_channel::<Job>(64);
    if JOBS.set(sender).is_err() {
        return;
    }
    thread::spawn(move || {
        for job in receiver {
            match job {
                Job::SendKeys(keys) => send_keys(&keys),
                Job::Emit(event, payload) => {
                    if let Some(app) = APP.get() {
                        let _ = app.emit(event, payload);
                    }
                }
                Job::PlayMacro(id) => {
                    let actions = lock(&MACROS).get(&id).cloned();
                    if let (Some(app), Some(actions)) = (APP.get(), actions) {
                        let _ = crate::macros::player::start_playback(app.clone(), actions, 1);
                    }
                }
                Job::PersistDisabled => {
                    if let Some(app) = APP.get() {
                        let _ = crate::store::save_remap(app, &snapshot());
                    }
                }
            }
        }
    });
}

pub fn set_mappings(list: Vec<Mapping>) {
    let mut map = lock(&MAPPINGS);
    map.clear();
    for m in list {
        // left / right click can never be remapped
        if m.button == "left" || m.button == "right" {
            continue;
        }
        map.insert(m.button, m.action);
    }
}

pub fn set_macros(macros: HashMap<String, Vec<MacroAction>>) {
    *lock(&MACROS) = macros;
}

// What is saved to remap.json: the current state of the hook.
pub fn snapshot() -> RemapFile {
    let mappings = lock(&MAPPINGS)
        .iter()
        .map(|(button, action)| Mapping {
            button: button.clone(),
            action: action.clone(),
        })
        .collect();
    RemapFile {
        enabled: ENABLED.load(Ordering::SeqCst),
        mappings,
        macros: lock(&MACROS).clone(),
    }
}

// Loads what was saved on disk (called once at start, before the hooks).
pub fn apply(file: RemapFile) {
    set_mappings(file.mappings);
    set_macros(file.macros);
    set_enabled(file.enabled);
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::SeqCst);
    if !enabled {
        lock(&SWALLOWED).clear();
    }
}

pub fn set_detecting(on: bool) {
    DETECTING.store(on, Ordering::SeqCst);
}

fn classify(msg: u32, data: u32) -> Option<(&'static str, bool)> {
    match msg {
        0x0201 => Some(("left", true)),
        0x0202 => Some(("left", false)),
        0x0204 => Some(("right", true)),
        0x0205 => Some(("right", false)),
        0x0207 => Some(("middle", true)),
        0x0208 => Some(("middle", false)),
        0x020B | 0x020C => {
            let down = msg == 0x020B;
            match (data >> 16) & 0xFFFF {
                1 => Some(("back", down)),
                2 => Some(("forward", down)),
                _ => None,
            }
        }
        _ => None,
    }
}

// Returns true when the event must be swallowed.
fn handle(msg: u32, data: u32) -> bool {
    let Some((name, down)) = classify(msg, data) else {
        return false;
    };

    if !down {
        return lock(&SWALLOWED).remove(name);
    }

    let protected = name == "left" || name == "right";

    if DETECTING.swap(false, Ordering::SeqCst) {
        emit("mouse-button-detected", name);
        if protected {
            return false;
        }
        lock(&SWALLOWED).insert(name);
        return true;
    }

    if protected || !ENABLED.load(Ordering::SeqCst) {
        return false;
    }

    let action = lock(&MAPPINGS).get(name).cloned();
    match action {
        None | Some(RemapAction::Normal) => false,
        Some(RemapAction::Disabled) => {
            lock(&SWALLOWED).insert(name);
            true
        }
        Some(RemapAction::Keys { keys }) => {
            lock(&SWALLOWED).insert(name);
            queue(Job::SendKeys(keys));
            true
        }
        Some(RemapAction::Macro { id }) => {
            lock(&SWALLOWED).insert(name);
            queue(Job::PlayMacro(id));
            true
        }
    }
}

fn send_keys(names: &[String]) {
    let keys: Vec<Key> = names
        .iter()
        .filter_map(|n| serde_json::from_value::<Key>(serde_json::Value::String(n.clone())).ok())
        .filter(|k| *k != Key::F10) // F10 is the kill switch, never simulate it
        .collect();

    for k in &keys {
        let _ = simulate(&EventType::KeyPress(*k));
        thread::sleep(Duration::from_millis(8));
    }
    for k in keys.iter().rev() {
        let _ = simulate(&EventType::KeyRelease(*k));
        thread::sleep(Duration::from_millis(8));
    }
}

#[cfg(windows)]
unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == 0 {
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        // 0x01 = LLMHF_INJECTED: ignore events generated by software (loop prevention)
        if info.flags & 0x01 == 0 && handle(wparam.0 as u32, info.mouseData) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
}

#[cfg(windows)]
unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == 0 {
        let msg = wparam.0 as u32;
        if msg == 0x0100 || msg == 0x0104 {
            let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
            // 0x10 = LLKHF_INJECTED, 0x79 = VK_F10
            if info.flags.0 & 0x10 == 0 && info.vkCode == 0x79 && ENABLED.swap(false, Ordering::SeqCst) {
                lock(&SWALLOWED).clear();
                emit("mouse-remap-disabled", true);
                queue(Job::PersistDisabled);
            }
        }
    }
    unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
}

#[cfg(windows)]
pub fn start(app: AppHandle) {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = APP.set(app);
    start_worker();

    thread::spawn(|| unsafe {
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), HINSTANCE::default(), 0);
        let keyboard = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), HINSTANCE::default(), 0);
        if mouse.is_err() || keyboard.is_err() {
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

/// macOS and Linux: remapping is not available yet, so nothing is hooked and no click is touched.
#[cfg(not(windows))]
pub fn start(app: AppHandle) {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = APP.set(app);
    start_worker();
}
