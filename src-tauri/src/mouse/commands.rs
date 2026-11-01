use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use hidapi::HidApi;

use super::drivers;
use super::{MouseCapabilitiesDto, MouseInfoDto, ProbeResult, SupportLevel};

/// One HID conversation at a time: the tray refresh and the window must not talk to the mouse together.
static HID_LOCK: Mutex<()> = Mutex::new(());

fn hid_lock() -> MutexGuard<'static, ()> {
    HID_LOCK.lock().unwrap_or_else(|error| error.into_inner())
}

const POLLING_OPTIONS: [u32; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];

fn not_found(diagnostics: Vec<String>) -> ProbeResult {
    ProbeResult {
        info: MouseInfoDto {
            connected: false,
            name: String::new(),
            connection: "unknown".to_string(),
            battery: None,
            charging: false,
            dpi: 0,
            dpi_min: 0,
            dpi_max: 0,
            dpi_step: 50,
            dpi_levels: Vec::new(),
            polling_rate: 0,
            polling_options: POLLING_OPTIONS.to_vec(),
            polling_supported: Vec::new(),
            brand: "Unknown".to_string(),
            capabilities: MouseCapabilitiesDto::unsupported(),
            support: SupportLevel::Unknown,
        },
        diagnostics,
    }
}

/// The last reading of a connected mouse, used for the instant update when a cable is plugged.
static LAST_INFO: Mutex<Option<MouseInfoDto>> = Mutex::new(None);

pub fn last_info() -> Option<MouseInfoDto> {
    LAST_INFO
        .lock()
        .map(|info| info.clone())
        .unwrap_or_else(|error| error.into_inner().clone())
}

/// True while the receiver is there but the mouse does not answer (it went to sleep to save
/// battery). It wakes up by itself when moved, so the watcher looks again every few seconds.
static ASLEEP: AtomicBool = AtomicBool::new(false);

pub fn is_asleep() -> bool {
    ASLEEP.load(Ordering::Relaxed)
}

/// A Logitech receiver that is there while the mouse sleeps answers "mouse detected" but nothing
/// readable. Only Logitech behaves like this: other brands (read only, battery only...) are complete.
pub fn is_half(info: &MouseInfoDto) -> bool {
    info.connected && !info.capabilities.dpi_read && info.brand == "Logitech"
}

pub fn probe() -> ProbeResult {
    let result = probe_inner();

    let half = is_half(&result.info);
    ASLEEP.store(half, Ordering::Relaxed);

    if result.info.connected && result.info.capabilities.dpi_read {
        if let Ok(mut last) = LAST_INFO.lock() {
            *last = Some(result.info.clone());
        }
    }

    result
}

fn probe_inner() -> ProbeResult {
    let _guard = hid_lock();
    let mut log = Vec::new();
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(error) => {
            log.push(format!("Cannot start HID access: {}", error));
            return not_found(log);
        }
    };

    let found = drivers::probe(&api, &mut log);
    match found {
        Some(info) => ProbeResult {
            info,
            diagnostics: log,
        },
        None => not_found(log),
    }
}

fn set_polling(hz: u32) -> Result<u32, String> {
    let _guard = hid_lock();
    let api = HidApi::new().map_err(|error| error.to_string())?;

    // Quick check of which brand is plugged in: no full reading before writing.
    for driver in drivers::registered() {
        if driver.present(&api) {
            return driver.set_polling(hz);
        }
    }

    Err("No mouse with a configurable polling rate found".to_string())
}

fn set_dpi(value: u32) -> Result<u32, String> {
    let _guard = hid_lock();
    let api = HidApi::new().map_err(|error| error.to_string())?;

    for driver in drivers::registered() {
        if driver.present(&api) {
            return driver.set_dpi(value);
        }
    }

    Err("No mouse with adjustable DPI found".to_string())
}

// Read-only: looks for a supported mouse and reports what it says.
#[tauri::command]
pub async fn get_mouse_info() -> Result<ProbeResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut result = probe();

        // A sleeping mouse cannot be read: keep showing the last good reading instead of "Unknown".
        if is_half(&result.info) {
            if let Some(last) = last_info() {
                result.info = last;
            }
        }

        result
    })
    .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn set_polling_rate(hz: u32) -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(move || set_polling(hz))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_mouse_dpi(value: u32) -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(move || set_dpi(value))
        .await
        .map_err(|error| error.to_string())?
}