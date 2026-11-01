use std::sync::atomic::Ordering;

use tauri::AppHandle;

use crate::macros::player::start_playback;
use crate::macros::recorder::{begin_recording, finish_recording, STOP_PLAYBACK};
use crate::macros::types::MacroAction;

#[tauri::command]
pub fn start_recording() -> Result<(), String> {
    begin_recording()
}

#[tauri::command]
pub fn stop_recording() -> Vec<MacroAction> {
    finish_recording(true)
}

#[tauri::command]
pub fn play_macro(app: AppHandle, actions: Vec<MacroAction>, repeat: u32) -> Result<(), String> {
    start_playback(app, actions, repeat)
}

#[tauri::command]
pub fn stop_playback() {
    STOP_PLAYBACK.store(true, Ordering::SeqCst);
}
