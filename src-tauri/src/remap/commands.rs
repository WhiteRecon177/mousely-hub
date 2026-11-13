use std::collections::HashMap;

use serde::Serialize;
use tauri::AppHandle;

use super::hook;
use super::types::Mapping;
use crate::macros::types::MacroAction;

// Saves the state to remap.json; a failed save must not break remapping that is already live.
fn persist(app: &AppHandle) {
    let _ = crate::store::save_remap(app, &hook::snapshot());
}

#[tauri::command]
pub fn set_mouse_mappings(app: AppHandle, mappings: Vec<Mapping>, macros: HashMap<String, Vec<MacroAction>>) {
    hook::set_mappings(mappings);
    hook::set_macros(macros);
    persist(&app);
}

#[tauri::command]
pub fn set_remap_enabled(app: AppHandle, enabled: bool) {
    hook::set_enabled(enabled);
    persist(&app);
}

#[derive(Serialize)]
pub struct RemapState {
    enabled: bool,
}

#[tauri::command]
pub fn get_remap_state() -> RemapState {
    RemapState {
        enabled: hook::is_enabled(),
    }
}

#[tauri::command]
pub fn start_button_detect() {
    hook::set_detecting(true);
}

#[tauri::command]
pub fn cancel_button_detect() {
    hook::set_detecting(false);
}
