//! Small JSON files in the app config folder, so remapping (also "button -> macro") and the close
//! setting work from the Rust side alone, with no window open. Written in two steps (a .tmp file,
//! then rename) so a crash never leaves a half file.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct SavedMousePrefs {
    #[serde(default)]
    pub dpi: Option<u32>,
    #[serde(default)]
    pub polling_rate: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct MousePrefsFile {
    #[serde(default)]
    pub active_device_id: Option<String>,
    #[serde(default)]
    pub devices: HashMap<String, SavedMousePrefs>,
}

fn folder(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn write_atomic_path<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temp = path.with_extension("json.tmp");
    let text = serde_json::to_string(value).map_err(|error| error.to_string())?;
    std::fs::write(&temp, text).map_err(|error| error.to_string())?;
    std::fs::rename(&temp, path).map_err(|error| error.to_string())
}

// A missing or broken file gives the default.
fn read_path<T: for<'de> Deserialize<'de> + Default>(path: &Path) -> T {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write_atomic<T: Serialize>(app: &AppHandle, name: &str, value: &T) -> Result<(), String> {
    let path = folder(app)?.join(name);
    write_atomic_path(&path, value)
}

// A missing or broken file gives the default.
fn read<T: for<'de> Deserialize<'de> + Default>(app: &AppHandle, name: &str) -> T {
    folder(app)
        .ok()
        .map(|dir| read_path(&dir.join(name)))
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

pub fn load_mouse_prefs(app: &AppHandle) -> MousePrefsFile {
    read(app, "mouse_prefs.json")
}

pub fn save_mouse_prefs(app: &AppHandle, file: &MousePrefsFile) -> Result<(), String> {
    write_atomic(app, "mouse_prefs.json", file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("mousely_hub_test_{}_{}_{}", label, std::process::id(), unique))
    }

    #[test]
    fn mouse_prefs_atomic_roundtrip_and_fallback() {
        let dir = temp_dir("mouse_prefs");
        let path = dir.join("mouse_prefs.json");

        // Missing file returns default
        let missing: MousePrefsFile = read_path(&path);
        assert_eq!(missing, MousePrefsFile::default());

        // Write and read back
        let mut devices = HashMap::new();
        devices.insert(
            "steelseries-aerox:steelseries-aerox-3".to_string(),
            SavedMousePrefs {
                dpi: Some(1600),
                polling_rate: Some(1000),
            },
        );
        devices.insert(
            "glorious-model-o:glorious-model-o-wireless".to_string(),
            SavedMousePrefs {
                dpi: None,
                polling_rate: Some(500),
            },
        );
        let prefs = MousePrefsFile {
            active_device_id: Some("steelseries-aerox:steelseries-aerox-3".to_string()),
            devices,
        };
        write_atomic_path(&path, &prefs).expect("atomic write succeeds");
        assert!(!path.with_extension("json.tmp").exists(), "temp file renamed away");

        let loaded: MousePrefsFile = read_path(&path);
        assert_eq!(loaded, prefs);

        // Overwrite works cleanly on Windows
        let updated = MousePrefsFile {
            active_device_id: Some("glorious-model-o:glorious-model-o-wireless".to_string()),
            devices: loaded.devices,
        };
        write_atomic_path(&path, &updated).expect("atomic overwrite succeeds");
        let reloaded: MousePrefsFile = read_path(&path);
        assert_eq!(reloaded, updated);

        // Broken JSON falls back to default
        std::fs::write(&path, "{not valid json").expect("write corrupt json");
        let fallback: MousePrefsFile = read_path(&path);
        assert_eq!(fallback, MousePrefsFile::default());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
