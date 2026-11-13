use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Instant;

use rdev::{Event, EventType, Key};
use tauri::{AppHandle, Emitter};

use crate::macros::types::{ActionKind, MacroAction};

pub static RECORDING: AtomicBool = AtomicBool::new(false);
pub static PLAYING: AtomicBool = AtomicBool::new(false);
pub static STOP_PLAYBACK: AtomicBool = AtomicBool::new(false);
static LISTENER_STARTED: AtomicBool = AtomicBool::new(false);

static STOPPED_TX: OnceLock<SyncSender<Vec<MacroAction>>> = OnceLock::new();
static ACTIONS: Mutex<Vec<MacroAction>> = Mutex::new(Vec::new());
static LAST_POS: Mutex<(f64, f64)> = Mutex::new((0.0, 0.0));
static LAST_TIME: Mutex<Option<Instant>> = Mutex::new(None);

// Locks a mutex even if another thread panicked while holding it
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

// Starts the global keyboard and mouse listener (only once)
pub fn start_listener(app: AppHandle) {
    if LISTENER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    // Worker that tells the UI the recording stopped (the listener callback must stay fast)
    let (sender, receiver) = sync_channel::<Vec<MacroAction>>(4);
    let _ = STOPPED_TX.set(sender);
    thread::spawn(move || {
        for actions in receiver {
            let _ = app.emit("macro-recording-stopped", actions);
        }
    });

    thread::spawn(|| {
        // listen() blocks this thread for the whole life of the app
        let _result = rdev::listen(handle_event);
    });
}

fn handle_event(event: Event) {
    let event_type = event.event_type;

    // Remember the mouse position: clicks are saved together with it
    if let EventType::MouseMove { x, y } = event_type {
        *lock(&LAST_POS) = (x, y);
        return;
    }

    // F10 is the emergency stop and is never recorded
    match event_type {
        EventType::KeyPress(Key::F10) => {
            if RECORDING.load(Ordering::SeqCst) {
                // Do not emit from the listener callback: hand it to the worker thread
                let actions = finish_recording(false);
                if let Some(sender) = STOPPED_TX.get() {
                    let _ = sender.try_send(actions);
                }
            } else if PLAYING.load(Ordering::SeqCst) {
                STOP_PLAYBACK.store(true, Ordering::SeqCst);
            }
            return;
        }
        EventType::KeyRelease(Key::F10) => return,
        _ => {}
    }

    if !RECORDING.load(Ordering::SeqCst) {
        return;
    }

    let kind = match event_type {
        EventType::KeyPress(key) => ActionKind::KeyDown { key },
        EventType::KeyRelease(key) => ActionKind::KeyUp { key },
        EventType::ButtonPress(button) => {
            let (x, y) = *lock(&LAST_POS);
            ActionKind::MouseDown { button, x, y }
        }
        EventType::ButtonRelease(button) => {
            let (x, y) = *lock(&LAST_POS);
            ActionKind::MouseUp { button, x, y }
        }
        _ => return,
    };

    // Wait time since the previous action (0 for the first one)
    let now = Instant::now();
    let delay_ms = {
        let mut last = lock(&LAST_TIME);
        let delay = match *last {
            Some(previous) => now.duration_since(previous).as_millis() as u64,
            None => 0,
        };
        *last = Some(now);
        delay
    };

    lock(&ACTIONS).push(MacroAction { delay_ms, kind });
}

pub fn begin_recording() -> Result<(), String> {
    if PLAYING.load(Ordering::SeqCst) {
        return Err("Playback in progress".to_string());
    }
    if RECORDING.load(Ordering::SeqCst) {
        return Err("Already recording".to_string());
    }
    lock(&ACTIONS).clear();
    *lock(&LAST_TIME) = None;
    RECORDING.store(true, Ordering::SeqCst);
    Ok(())
}

// Stops recording and returns the actions. With trim_stop_click, the click on
// the Stop button (a trailing mouse up and mouse down) is removed.
pub fn finish_recording(trim_stop_click: bool) -> Vec<MacroAction> {
    RECORDING.store(false, Ordering::SeqCst);
    let mut result = std::mem::take(&mut *lock(&ACTIONS));

    if trim_stop_click {
        if matches!(result.last(), Some(a) if matches!(a.kind, ActionKind::MouseUp { .. })) {
            result.pop();
        }
        if matches!(result.last(), Some(a) if matches!(a.kind, ActionKind::MouseDown { .. })) {
            result.pop();
        }
    }
    result
}
