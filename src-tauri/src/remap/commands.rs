use super::hook;
use super::types::Mapping;

#[tauri::command]
pub fn set_mouse_mappings(mappings: Vec<Mapping>) {
    hook::set_mappings(mappings);
}

#[tauri::command]
pub fn set_remap_enabled(enabled: bool) {
    hook::set_enabled(enabled);
}

#[tauri::command]
pub fn start_button_detect() {
    hook::set_detecting(true);
}

#[tauri::command]
pub fn cancel_button_detect() {
    hook::set_detecting(false);
}