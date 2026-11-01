use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use rdev::{simulate, EventType};
use tauri::{AppHandle, Emitter};

use crate::macros::recorder::{PLAYING, RECORDING, STOP_PLAYBACK};
use crate::macros::types::{ActionKind, MacroAction};

// Plays the actions in a background thread. F10 or stop_playback ends it early.
pub fn start_playback(app: AppHandle, actions: Vec<MacroAction>, repeat: u32) -> Result<(), String> {
    if RECORDING.load(Ordering::SeqCst) {
        return Err("Recording in progress".to_string());
    }
    if PLAYING.swap(true, Ordering::SeqCst) {
        return Err("Already playing".to_string());
    }
    STOP_PLAYBACK.store(false, Ordering::SeqCst);
    let repeat = repeat.clamp(1, 1000);

    thread::spawn(move || {
        'all: for round in 0..repeat {
            // Short pause between repetitions
            if round > 0 && !sleep_checked(50) {
                break 'all;
            }
            for action in &actions {
                if !sleep_checked(action.delay_ms) {
                    break 'all;
                }
                play_action(&action.kind);
            }
        }

        PLAYING.store(false, Ordering::SeqCst);
        let _ = app.emit("macro-playback-finished", ());
    });

    Ok(())
}

fn play_action(kind: &ActionKind) {
    match kind {
        ActionKind::KeyDown { key } => send(EventType::KeyPress(*key)),
        ActionKind::KeyUp { key } => send(EventType::KeyRelease(*key)),
        ActionKind::MouseDown { button, x, y } => {
            send(EventType::MouseMove { x: *x, y: *y });
            send(EventType::ButtonPress(*button));
        }
        ActionKind::MouseUp { button, x, y } => {
            send(EventType::MouseMove { x: *x, y: *y });
            send(EventType::ButtonRelease(*button));
        }
    }
}

fn send(event_type: EventType) {
    // Errors are ignored: one failed key must not stop the whole macro
    let _ = simulate(&event_type);
    thread::sleep(Duration::from_millis(5));
}

// Sleeps in small slices; returns false as soon as playback must stop
fn sleep_checked(ms: u64) -> bool {
    let mut remaining = ms;
    loop {
        if STOP_PLAYBACK.load(Ordering::SeqCst) {
            return false;
        }
        if remaining == 0 {
            return true;
        }
        let step = remaining.min(20);
        thread::sleep(Duration::from_millis(step));
        remaining -= step;
    }
}
