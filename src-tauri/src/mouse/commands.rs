use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use hidapi::HidApi;
use tauri::{AppHandle, Emitter};

use super::drivers;
use super::{MouseCapabilitiesDto, MouseInfoDto, ProbeResult, SupportLevel, ValueSource};
use crate::store::{load_mouse_prefs, save_mouse_prefs, MousePrefsFile, SavedMousePrefs};

/// One HID conversation at a time: the tray refresh, device watcher, and window must not talk to the mouse together.
static HID_LOCK: Mutex<()> = Mutex::new(());

fn hid_lock() -> MutexGuard<'static, ()> {
    HID_LOCK.lock().unwrap_or_else(|error| error.into_inner())
}

const POLLING_OPTIONS: [u32; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];

fn not_found_info() -> MouseInfoDto {
    MouseInfoDto {
        id: String::new(),
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
        dpi_source: ValueSource::Unset,
        polling_rate: 0,
        polling_options: POLLING_OPTIONS.to_vec(),
        polling_supported: Vec::new(),
        polling_source: ValueSource::Unset,
        brand: "Unknown".to_string(),
        capabilities: MouseCapabilitiesDto::unsupported(),
        support: SupportLevel::Unknown,
    }
}

fn not_found(diagnostics: Vec<String>) -> ProbeResult {
    ProbeResult {
        info: not_found_info(),
        diagnostics,
        devices: Vec::new(),
        active_id: None,
    }
}

/// Shared probe cache populated by tray startup, device watcher, and window commands.
static LAST_PROBE: Mutex<Option<ProbeResult>> = Mutex::new(None);
/// Last complete Logitech reading, kept so a sleeping mouse does not flash as "Unknown".
static LAST_GOOD_LOGITECH: Mutex<Option<MouseInfoDto>> = Mutex::new(None);

pub fn last_info() -> Option<MouseInfoDto> {
    LAST_PROBE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        .map(|probe| probe.info.clone())
        .or_else(last_good_logitech)
}

pub fn last_good_logitech() -> Option<MouseInfoDto> {
    LAST_GOOD_LOGITECH
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
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

pub fn apply_saved_prefs(devices: &mut [MouseInfoDto], prefs: &MousePrefsFile) {
    for dev in devices.iter_mut() {
        let saved: Option<&SavedMousePrefs> = prefs.devices.get(&dev.id);

        if dev.capabilities.dpi_read {
            dev.dpi_source = ValueSource::Hardware;
        } else if dev.capabilities.dpi_write {
            if let Some(val) = saved.and_then(|s| s.dpi).filter(|&v| v > 0) {
                let clamped = if dev.dpi_max > 0 && dev.dpi_min > 0 {
                    val.clamp(dev.dpi_min, dev.dpi_max)
                } else {
                    val
                };
                dev.dpi = clamped;
                dev.dpi_source = ValueSource::Saved;
            } else {
                dev.dpi = 0;
                dev.dpi_source = ValueSource::Unset;
            }
        } else {
            dev.dpi = 0;
            dev.dpi_source = ValueSource::Unset;
        }

        if dev.capabilities.polling_read {
            dev.polling_source = ValueSource::Hardware;
        } else if dev.capabilities.polling_write {
            if let Some(hz) = saved
                .and_then(|s| s.polling_rate)
                .filter(|&r| r > 0 && (dev.polling_supported.is_empty() || dev.polling_supported.contains(&r)))
            {
                dev.polling_rate = hz;
                dev.polling_source = ValueSource::Saved;
            } else {
                dev.polling_rate = 0;
                dev.polling_source = ValueSource::Unset;
            }
        } else {
            dev.polling_rate = 0;
            dev.polling_source = ValueSource::Unset;
        }
    }
}

pub fn resolve_active(
    devices: &[MouseInfoDto],
    preferred_id: Option<&str>,
) -> (Option<String>, MouseInfoDto) {
    if devices.is_empty() {
        return (None, not_found_info());
    }
    if let Some(wanted) = preferred_id {
        if let Some(found) = devices.iter().find(|d| d.id == wanted) {
            return (Some(found.id.clone()), found.clone());
        }
    }
    let first = &devices[0];
    (Some(first.id.clone()), first.clone())
}

pub fn probe_state_changed(prev: &ProbeResult, next: &ProbeResult) -> bool {
    prev.active_id != next.active_id || prev.info != next.info || prev.devices != next.devices
}

fn probe_hardware_locked(app: &AppHandle) -> ProbeResult {
    let mut log = Vec::new();
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(error) => {
            log.push(format!("Cannot start HID access: {}", error));
            return not_found(log);
        }
    };

    let mut devices = drivers::probe_all(&api, &mut log);

    let mut any_half = false;
    let mut any_logitech_present = false;
    for dev in devices.iter_mut() {
        if dev.brand == "Logitech" {
            any_logitech_present = true;
            if is_half(dev) {
                any_half = true;
                if let Some(last) = last_good_logitech() {
                    *dev = last;
                }
            } else if dev.connected && dev.capabilities.dpi_read {
                if let Ok(mut slot) = LAST_GOOD_LOGITECH.lock() {
                    *slot = Some(dev.clone());
                }
            }
        }
    }

    if !any_logitech_present {
        if let Ok(mut slot) = LAST_GOOD_LOGITECH.lock() {
            *slot = None;
        }
    }

    ASLEEP.store(any_half, Ordering::Relaxed);

    let prefs = load_mouse_prefs(app);
    apply_saved_prefs(&mut devices, &prefs);
    let (active_id, info) = resolve_active(&devices, prefs.active_device_id.as_deref());

    ProbeResult {
        info,
        diagnostics: log,
        devices,
        active_id,
    }
}

fn commit_probe_result(app: &AppHandle, result: &ProbeResult) -> bool {
    let changed = {
        let mut guard = LAST_PROBE.lock().unwrap_or_else(|error| error.into_inner());
        let changed = match guard.as_ref() {
            Some(prev) => probe_state_changed(prev, result),
            None => true,
        };
        *guard = Some(result.clone());
        changed
    };
    crate::tray::apply_info(app, &result.info);
    changed
}

/// Shared probe helper for tray startup, 60s refresh, and device watcher.
/// Returns `(result, state_changed)`.
pub fn probe_and_sync(app: &AppHandle) -> (ProbeResult, bool) {
    let _guard = hid_lock();
    let result = probe_hardware_locked(app);
    let changed = commit_probe_result(app, &result);
    (result, changed)
}

fn get_or_probe_sync(app: &AppHandle, force: bool) -> (ProbeResult, bool) {
    if !force {
        if let Some(cached) = LAST_PROBE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
        {
            return (cached, false);
        }
    }

    let _guard = hid_lock();
    if !force {
        if let Some(cached) = LAST_PROBE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
        {
            return (cached, false);
        }
    }

    let result = probe_hardware_locked(app);
    let changed = commit_probe_result(app, &result);
    (result, changed)
}

fn resolve_target_device_id_locked(app: &AppHandle) -> Option<String> {
    if let Some(cached) = LAST_PROBE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
    {
        if cached.info.connected && !cached.info.id.is_empty() {
            return cached.active_id.or(Some(cached.info.id));
        }
    }

    let result = probe_hardware_locked(app);
    let _ = commit_probe_result(app, &result);
    if result.info.connected && !result.info.id.is_empty() {
        result.active_id.or(Some(result.info.id))
    } else {
        None
    }
}

fn driver_for_id(device_id: &str) -> Option<Box<dyn drivers::MouseDriver>> {
    let prefix = device_id.split(':').next()?;
    drivers::registered().into_iter().find(|d| d.id() == prefix)
}

fn persist_and_update_cache(
    app: &AppHandle,
    device_id: &str,
    dpi: Option<u32>,
    polling_rate: Option<u32>,
) {
    let mut prefs = load_mouse_prefs(app);
    let entry = prefs.devices.entry(device_id.to_string()).or_default();
    if let Some(val) = dpi {
        entry.dpi = Some(val);
    }
    if let Some(hz) = polling_rate {
        entry.polling_rate = Some(hz);
    }
    let _ = save_mouse_prefs(app, &prefs);

    let updated_active = {
        let mut guard = LAST_PROBE.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(probe) = guard.as_mut() {
            for dev in probe.devices.iter_mut() {
                if dev.id == device_id {
                    if let Some(val) = dpi {
                        dev.dpi = val;
                        dev.dpi_source = if dev.capabilities.dpi_read {
                            ValueSource::Hardware
                        } else {
                            ValueSource::Saved
                        };
                    }
                    if let Some(hz) = polling_rate {
                        dev.polling_rate = hz;
                        dev.polling_source = if dev.capabilities.polling_read {
                            ValueSource::Hardware
                        } else {
                            ValueSource::Saved
                        };
                    }
                    if dev.brand == "Logitech" && dev.capabilities.dpi_read {
                        if let Ok(mut slot) = LAST_GOOD_LOGITECH.lock() {
                            *slot = Some(dev.clone());
                        }
                    }
                }
            }
            if probe.info.id == device_id || probe.active_id.as_deref() == Some(device_id) {
                if let Some(val) = dpi {
                    probe.info.dpi = val;
                    probe.info.dpi_source = if probe.info.capabilities.dpi_read {
                        ValueSource::Hardware
                    } else {
                        ValueSource::Saved
                    };
                }
                if let Some(hz) = polling_rate {
                    probe.info.polling_rate = hz;
                    probe.info.polling_source = if probe.info.capabilities.polling_read {
                        ValueSource::Hardware
                    } else {
                        ValueSource::Saved
                    };
                }
            }
            Some(probe.info.clone())
        } else {
            None
        }
    };

    if let Some(info) = updated_active {
        crate::tray::apply_info(app, &info);
    }
}

fn set_polling(app: &AppHandle, hz: u32) -> Result<u32, String> {
    let _guard = hid_lock();
    let api = HidApi::new().map_err(|error| error.to_string())?;
    let active_id = resolve_target_device_id_locked(app)
        .ok_or_else(|| "No mouse with a configurable polling rate found".to_string())?;

    let driver = driver_for_id(&active_id)
        .ok_or_else(|| "This mouse does not support changing the polling rate".to_string())?;
    if !driver.present(&api) {
        return Err("Selected mouse is not connected".to_string());
    }

    let applied = driver.set_polling_for(&active_id, hz)?;
    persist_and_update_cache(app, &active_id, None, Some(applied));
    Ok(applied)
}

fn set_dpi(app: &AppHandle, value: u32) -> Result<u32, String> {
    let _guard = hid_lock();
    let api = HidApi::new().map_err(|error| error.to_string())?;
    let active_id = resolve_target_device_id_locked(app)
        .ok_or_else(|| "No mouse with adjustable DPI found".to_string())?;

    let driver = driver_for_id(&active_id)
        .ok_or_else(|| "This mouse does not support changing DPI".to_string())?;
    if !driver.present(&api) {
        return Err("Selected mouse is not connected".to_string());
    }

    let applied = driver.set_dpi_for(&active_id, value)?;
    persist_and_update_cache(app, &active_id, Some(applied), None);
    Ok(applied)
}

// Read-only: looks for connected mice and reports cached or fresh state.
#[tauri::command]
pub async fn get_mouse_info(app: AppHandle, force: Option<bool>) -> Result<ProbeResult, String> {
    let force_probe = force.unwrap_or(false);
    if !force_probe {
        if let Some(cached) = LAST_PROBE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
        {
            return Ok(cached);
        }
    }

    tauri::async_runtime::spawn_blocking(move || {
        let (result, changed) = get_or_probe_sync(&app, force_probe);
        if changed {
            let _ = app.emit("mouse-changed", &result);
        }
        result
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn select_active_mouse(app: AppHandle, id: String) -> Result<ProbeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = hid_lock();
        let mut current = LAST_PROBE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
            .unwrap_or_else(|| probe_hardware_locked(&app));

        let selected = current
            .devices
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .ok_or_else(|| format!("Mouse '{}' is not connected", id))?;

        let mut prefs = load_mouse_prefs(&app);
        prefs.active_device_id = Some(id.clone());
        save_mouse_prefs(&app, &prefs)?;

        current.active_id = Some(id);
        current.info = selected;

        let _ = commit_probe_result(&app, &current);
        let _ = app.emit("mouse-changed", &current);
        Ok(current)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_polling_rate(app: AppHandle, hz: u32) -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(move || set_polling(&app, hz))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_mouse_dpi(app: AppHandle, value: u32) -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(move || set_dpi(&app, value))
        .await
        .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_device(
        id: &str,
        dpi_read: bool,
        dpi_write: bool,
        polling_read: bool,
        polling_write: bool,
    ) -> MouseInfoDto {
        MouseInfoDto {
            id: id.to_string(),
            connected: true,
            name: id.to_string(),
            connection: "wired".to_string(),
            battery: None,
            charging: false,
            dpi: if dpi_read { 1600 } else { 0 },
            dpi_min: 100,
            dpi_max: 18000,
            dpi_step: 100,
            dpi_levels: Vec::new(),
            dpi_source: if dpi_read {
                ValueSource::Hardware
            } else {
                ValueSource::Unset
            },
            polling_rate: if polling_read { 1000 } else { 0 },
            polling_options: vec![125, 250, 500, 1000],
            polling_supported: vec![125, 250, 500, 1000],
            polling_source: if polling_read {
                ValueSource::Hardware
            } else {
                ValueSource::Unset
            },
            brand: "Test".to_string(),
            capabilities: MouseCapabilitiesDto {
                battery: false,
                dpi_read,
                dpi_write,
                polling_read,
                polling_write,
            },
            support: SupportLevel::Expected,
        }
    }

    #[test]
    fn value_source_hardware_saved_and_unset_behavior() {
        let mut devices = vec![
            // Write-only DPI + Polling (e.g. SteelSeries Aerox 3) with saved prefs
            make_device("steelseries-aerox:steelseries-aerox-3", false, true, false, true),
            // Write-only DPI + Polling without saved prefs
            make_device("steelseries-sensei-ten:steelseries-sensei-ten", false, true, false, true),
            // Hardware-readable DPI, write-only polling (e.g. HyperX Pulsefire Dart)
            make_device("hyperx-pulsefire-dart:hyperx-pulsefire-dart", true, true, false, true),
            // Hardware-readable DPI + Polling (e.g. Logitech)
            make_device("logitech:pro-x-superlight-2", true, true, true, true),
            // Read-only with nothing readable (e.g. Lamzu Maya)
            make_device("lamzu-maya:lamzu-maya-x-8k", false, false, false, false),
        ];

        let mut map = HashMap::new();
        map.insert(
            "steelseries-aerox:steelseries-aerox-3".to_string(),
            SavedMousePrefs {
                dpi: Some(1200),
                polling_rate: Some(500),
            },
        );
        map.insert(
            "hyperx-pulsefire-dart:hyperx-pulsefire-dart".to_string(),
            SavedMousePrefs {
                dpi: Some(400), // ignored because dpi_read == true
                polling_rate: Some(1000),
            },
        );
        map.insert(
            "logitech:pro-x-superlight-2".to_string(),
            SavedMousePrefs {
                dpi: Some(400),
                polling_rate: Some(250),
            },
        );
        map.insert(
            "lamzu-maya:lamzu-maya-x-8k".to_string(),
            SavedMousePrefs {
                dpi: Some(800),
                polling_rate: Some(1000),
            },
        );

        let prefs = MousePrefsFile {
            active_device_id: Some("hyperx-pulsefire-dart:hyperx-pulsefire-dart".to_string()),
            devices: map,
        };

        apply_saved_prefs(&mut devices, &prefs);

        // SteelSeries Aerox 3: Saved from prefs
        assert_eq!(devices[0].dpi, 1200);
        assert_eq!(devices[0].dpi_source, ValueSource::Saved);
        assert_eq!(devices[0].polling_rate, 500);
        assert_eq!(devices[0].polling_source, ValueSource::Saved);

        // Sensei Ten: Unset (0)
        assert_eq!(devices[1].dpi, 0);
        assert_eq!(devices[1].dpi_source, ValueSource::Unset);
        assert_eq!(devices[1].polling_rate, 0);
        assert_eq!(devices[1].polling_source, ValueSource::Unset);

        // HyperX: Hardware DPI kept, polling Saved
        assert_eq!(devices[2].dpi, 1600);
        assert_eq!(devices[2].dpi_source, ValueSource::Hardware);
        assert_eq!(devices[2].polling_rate, 1000);
        assert_eq!(devices[2].polling_source, ValueSource::Saved);

        // Logitech: Hardware always wins
        assert_eq!(devices[3].dpi, 1600);
        assert_eq!(devices[3].dpi_source, ValueSource::Hardware);
        assert_eq!(devices[3].polling_rate, 1000);
        assert_eq!(devices[3].polling_source, ValueSource::Hardware);

        // Lamzu Maya (not writable): stays Unset (0)
        assert_eq!(devices[4].dpi, 0);
        assert_eq!(devices[4].dpi_source, ValueSource::Unset);
        assert_eq!(devices[4].polling_rate, 0);
        assert_eq!(devices[4].polling_source, ValueSource::Unset);

        // Active selection picks HyperX; if missing, falls back to first device
        let (active_id, active_info) =
            resolve_active(&devices, prefs.active_device_id.as_deref());
        assert_eq!(active_id.as_deref(), Some("hyperx-pulsefire-dart:hyperx-pulsefire-dart"));
        assert_eq!(active_info.id, "hyperx-pulsefire-dart:hyperx-pulsefire-dart");

        let (fallback_id, fallback_info) = resolve_active(&devices, Some("missing:mouse"));
        assert_eq!(fallback_id.as_deref(), Some("steelseries-aerox:steelseries-aerox-3"));
        assert_eq!(fallback_info.id, "steelseries-aerox:steelseries-aerox-3");
    }

    #[test]
    fn saved_prefs_clamp_dpi_and_reject_unsupported_polling() {
        let mut devices = vec![make_device(
            "steelseries-aerox:steelseries-aerox-3",
            false,
            true,
            false,
            true,
        )];
        let mut map = HashMap::new();
        map.insert(
            "steelseries-aerox:steelseries-aerox-3".to_string(),
            SavedMousePrefs {
                dpi: Some(99999),
                polling_rate: Some(8000), // not in polling_supported [125,250,500,1000]
            },
        );
        let prefs = MousePrefsFile {
            active_device_id: None,
            devices: map,
        };

        apply_saved_prefs(&mut devices, &prefs);
        assert_eq!(devices[0].dpi, 18000);
        assert_eq!(devices[0].dpi_source, ValueSource::Saved);
        assert_eq!(devices[0].polling_rate, 0);
        assert_eq!(devices[0].polling_source, ValueSource::Unset);
    }
}