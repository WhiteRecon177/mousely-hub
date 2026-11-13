//! Small JSON files in the app config folder, so remapping (also "button -> macro") and the close
//! setting work from the Rust side alone, with no window open. Written in two steps (a .tmp file,
//! then rename) so a crash never leaves a half file.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::macros::types::MacroAction;
use crate::remap::types::Mapping;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RemapFile {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub mappings: Vec<Mapping>,
    // Only the macros that are assigned to a button.
    #[serde(default)]
    pub macros: HashMap<String, Vec<MacroAction>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingsFile {
    #[serde(default = "yes")]
    pub close_to_tray: bool,
}

fn yes() -> bool {
    true
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self { close_to_tray: true }
    }
}

fn folder(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn write_atomic<T: Serialize>(app: &AppHandle, name: &str, value: &T) -> Result<(), String> {
    let path = folder(app)?.join(name);
    let temp = path.with_extension("json.tmp");
    let text = serde_json::to_string(value).map_err(|error| error.to_string())?;
    std::fs::write(&temp, text).map_err(|error| error.to_string())?;
    std::fs::rename(&temp, &path).map_err(|error| error.to_string())
}

// A missing or broken file gives the default.
fn read<T: for<'de> Deserialize<'de> + Default>(app: &AppHandle, name: &str) -> T {
    folder(app)
        .ok()
        .and_then(|dir| std::fs::read_to_string(dir.join(name)).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn load_remap(app: &AppHandle) -> RemapFile {
    read(app, "remap.json")
}

pub fn save_remap(app: &AppHandle, file: &RemapFile) -> Result<(), String> {
    write_atomic(app, "remap.json", file)
}

pub fn load_settings(app: &AppHandle) -> SettingsFile {
    read(app, "settings.json")
}

pub fn save_settings(app: &AppHandle, file: &SettingsFile) -> Result<(), String> {
    write_atomic(app, "settings.json", file)
}
