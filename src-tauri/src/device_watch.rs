//! Watches for changes of the connected input devices (cable plugged or unplugged, receiver,
//! other HID devices) and tells the tray and the window at once, without polling the mouse.
//!
//! The check is only a list of raw input devices from Windows (a few microseconds); the mouse is
//! read again only when that list really changed.

use std::collections::BTreeSet;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
#[cfg(windows)]
use windows::Win32::UI::Input::{
    GetRawInputDeviceInfoW, GetRawInputDeviceList, RAWINPUTDEVICELIST, RIDI_DEVICENAME,
};

// Windows answers in microseconds. On macOS and Linux the check lists the HID devices, which
// costs more, so it runs once a second.
#[cfg(windows)]
const CHECK_EVERY: Duration = Duration::from_millis(100);
#[cfg(not(windows))]
const CHECK_EVERY: Duration = Duration::from_millis(1000);

/// A number that changes whenever a raw input device (mouse, keyboard, HID) appears or goes away,
/// and how many devices there are.
#[cfg(windows)]
fn signature() -> (u64, BTreeSet<String>) {
    // The set holds the names of the Logitech interfaces: comparing two sets tells which one
    // appeared or went away (a cable) even when the receiver stays and the count does not change.
    unsafe {
        let size = std::mem::size_of::<RAWINPUTDEVICELIST>() as u32;
        let mut count: u32 = 0;

        if GetRawInputDeviceList(None, &mut count, size) == u32::MAX || count == 0 {
            return (0, BTreeSet::new());
        }

        let mut list = vec![RAWINPUTDEVICELIST::default(); count as usize];
        let got = GetRawInputDeviceList(Some(list.as_mut_ptr()), &mut count, size);

        if got == u32::MAX {
            return (0, BTreeSet::new());
        }

        let mut hash: u64 = 0xcbf29ce484222325;
        let mut logitech: BTreeSet<String> = BTreeSet::new();

        for item in list.iter().take(got as usize) {
            hash ^= item.hDevice.0 as usize as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            hash ^= u64::from(item.dwType.0);
            hash = hash.wrapping_mul(0x100000001b3);

            let mut length: u32 = 0;
            GetRawInputDeviceInfoW(item.hDevice, RIDI_DEVICENAME, None, &mut length);
            if length == 0 || length > 1024 {
                continue;
            }

            let mut name = vec![0u16; length as usize];
            let read = GetRawInputDeviceInfoW(
                item.hDevice,
                RIDI_DEVICENAME,
                Some(name.as_mut_ptr().cast()),
                &mut length,
            );
            if read == u32::MAX {
                continue;
            }

            let text = String::from_utf16_lossy(&name[..(read as usize).min(name.len())]);
            if text.to_ascii_uppercase().contains("VID_046D") {
                logitech.insert(text.to_ascii_uppercase());
            }
        }

        (hash ^ u64::from(got), logitech)
    }
}

/// Same idea on macOS and Linux: a list of the HID devices (hidapi), hashed. The set holds the
/// Logitech interfaces (vendor 046D) so a cable or a receiver is noticed.
#[cfg(not(windows))]
fn signature() -> (u64, BTreeSet<String>) {
    use std::sync::{Mutex, OnceLock};

    static API: OnceLock<Mutex<Option<hidapi::HidApi>>> = OnceLock::new();
    let cell = API.get_or_init(|| Mutex::new(hidapi::HidApi::new().ok()));
    let mut guard = cell.lock().unwrap_or_else(|e| e.into_inner());

    let Some(api) = guard.as_mut() else {
        return (0, BTreeSet::new());
    };
    if api.refresh_devices().is_err() {
        return (0, BTreeSet::new());
    }

    let mut hash: u64 = 0xcbf29ce484222325;
    let mut logitech: BTreeSet<String> = BTreeSet::new();
    let mut count: u64 = 0;

    for device in api.device_list() {
        count += 1;
        let text = format!(
            "{:04X}:{:04X}:{}:{}",
            device.vendor_id(),
            device.product_id(),
            device.interface_number(),
            device.path().to_string_lossy()
        );
        for byte in text.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        if device.vendor_id() == 0x046D {
            logitech.insert(text);
        }
    }

    (hash ^ count, logitech)
}

/// Instant guess when a cable is plugged or unplugged: the real reading comes a moment later and
/// corrects it if needed. Only the connection and the charging state are guessed.
fn quick_update(app: &AppHandle, plugged: bool) {
    let Some(mut info) = crate::mouse::commands::last_info() else {
        return;
    };

    if !info.connected {
        return;
    }

    // Nothing to guess when the screen already shows that state.
    if (plugged && info.connection == "wired") || (!plugged && info.connection != "wired") {
        return;
    }


    if plugged {
        info.connection = "wired".to_string();
        info.charging = info.battery.map(|level| level < 100).unwrap_or(true);
    } else {
        info.connection = "wireless".to_string();
        info.charging = false;
    }

    crate::tray::apply_info(app, &info);
    let _ = app.emit(
        "mouse-quick",
        serde_json::json!({ "connection": info.connection, "charging": info.charging }),
    );
}

fn publish(app: &AppHandle) {
    let mut result = crate::mouse::commands::probe();

    // Right after a cable change the receiver can answer half-way ("mouse detected" but nothing
    // readable yet). Do not show that: read again a few times until the full reading is there.
    for _ in 0..6 {
        let half = crate::mouse::commands::is_half(&result.info);
        if !half || crate::mouse::commands::last_info().is_none() {
            break;
        }
        std::thread::sleep(Duration::from_millis(350));
        result = crate::mouse::commands::probe();
    }

    if crate::mouse::commands::is_half(&result.info) && crate::mouse::commands::last_info().is_some() {
        // Still half-way: keep what is on screen; the next reading will fix it.
        return;
    }

    crate::tray::apply_info(app, &result.info);
    let _ = app.emit("mouse-changed", &result);
}

pub fn start(app: AppHandle) {
    let _ = std::thread::Builder::new()
        .name("device-watch".into())
        .spawn(move || {
            let (mut last, mut last_set) = signature();
            let mut asleep_ticks: u32 = 0;

            loop {
                std::thread::sleep(CHECK_EVERY);

                let (now, set) = signature();
                if now == last {
                    // Nothing was plugged or unplugged. If the mouse is asleep, look again every
                    // few seconds: it wakes up when moved and nothing else tells us.
                    if crate::mouse::commands::is_asleep() {
                        asleep_ticks += 1;
                        if asleep_ticks >= 40 {
                            asleep_ticks = 0;
                            publish(&app);
                        }
                    } else {
                        asleep_ticks = 0;
                    }
                    continue;
                }

                // Which Logitech interfaces appeared or went away.
                let added = set.difference(&last_set).count();
                let removed = last_set.difference(&set).count();

                // Tell the window and the tray at once, before the (slower) real reading.
                if added > 0 && removed == 0 {
                    quick_update(&app, true);
                } else if removed > 0 && added == 0 {
                    quick_update(&app, false);
                }

                // A replug can change the list several times: wait until it settles.
                let mut stable = now;
                for _ in 0..4 {
                    std::thread::sleep(Duration::from_millis(150));
                    let (again, _) = signature();
                    if again == stable {
                        break;
                    }
                    stable = again;
                }

                publish(&app);

                // Second look: a wireless link needs a moment before it reports everything.
                std::thread::sleep(Duration::from_millis(1500));
                publish(&app);

                let (after, after_set) = signature();
                last = after;
                last_set = after_set;
            }
        });
}
