//! App settings that need the system: start with Windows, what the close button does, and the
//! export of the HID info of the user's mouse (to add new mice to the support list).

use std::collections::HashSet;
use std::sync::atomic::AtomicBool;

use hidapi::HidApi;

/// True: closing the window keeps the app running in the tray. False: closing quits the app.
pub static CLOSE_TO_TRAY: AtomicBool = AtomicBool::new(true);

// ---------------------------------------------------------------------------------------------
// Start with the system. Windows: registry "Run" key. Linux: a .desktop file in
// ~/.config/autostart. macOS: a LaunchAgent in ~/Library/LaunchAgents.
// In every case the app starts hidden in the tray (`--minimized`).
// ---------------------------------------------------------------------------------------------

#[cfg(windows)]
mod autostart {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "DeviceDeck";
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    fn reg(args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new("reg").args(args).creation_flags(CREATE_NO_WINDOW).output()
    }

    pub fn is_on() -> bool {
        reg(&["query", RUN_KEY, "/v", VALUE_NAME])
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn set(enabled: bool) -> Result<(), String> {
        if enabled {
            let exe = std::env::current_exe().map_err(|error| error.to_string())?;
            let data = format!("\"{}\" --minimized", exe.display());
            let output = reg(&["add", RUN_KEY, "/v", VALUE_NAME, "/t", "REG_SZ", "/d", &data, "/f"])
                .map_err(|error| error.to_string())?;
            if !output.status.success() {
                return Err("Could not turn on start with the system".to_string());
            }
        } else {
            // Fails when the entry is not there: that is fine, it is already off.
            let _ = reg(&["delete", RUN_KEY, "/v", VALUE_NAME, "/f"]);
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod autostart {
    use std::path::PathBuf;

    #[cfg(target_os = "macos")]
    fn file() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join("Library/LaunchAgents/io.github.mouselyhub.app.plist"))
    }

    #[cfg(not(target_os = "macos"))]
    fn file() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(base.join("autostart/mousely-hub.desktop"))
    }

    pub fn is_on() -> bool {
        file().map(|path| path.exists()).unwrap_or(false)
    }

    pub fn set(enabled: bool) -> Result<(), String> {
        let path = file().ok_or_else(|| "Could not find the home folder".to_string())?;

        if !enabled {
            // Fails when the file is not there: that is fine, it is already off.
            let _ = std::fs::remove_file(&path);
            return Ok(());
        }

        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let exe = exe.display();

        #[cfg(target_os = "macos")]
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\"><dict>\n\
<key>Label</key><string>io.github.mouselyhub.app</string>\n\
<key>ProgramArguments</key><array><string>{exe}</string><string>--minimized</string></array>\n\
<key>RunAtLoad</key><true/>\n\
</dict></plist>\n"
        );

        #[cfg(not(target_os = "macos"))]
        let content = format!(
            "[Desktop Entry]\nType=Application\nName=Mousely Hub\nExec=\"{exe}\" --minimized\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
        );

        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
        }
        std::fs::write(&path, content).map_err(|error| error.to_string())
    }
}

#[tauri::command]
pub fn get_autostart() -> bool {
    autostart::is_on()
}

/// Turns "start with the system" on or off. It starts hidden in the tray (`--minimized`).
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<bool, String> {
    autostart::set(enabled)?;
    Ok(autostart::is_on())
}

#[tauri::command]
pub fn set_close_to_tray(enabled: bool) {
    CLOSE_TO_TRAY.store(enabled, std::sync::atomic::Ordering::SeqCst);
}

fn hex4(value: u16) -> String {
    format!("{:04X}", value)
}

fn read_hid_info() -> Result<String, String> {
    let api = HidApi::new().map_err(|error| error.to_string())?;

    // Only the devices that present themselves as a mouse, with all their interfaces.
    let mice: HashSet<(u16, u16)> = api
        .device_list()
        .filter(|device| device.usage_page() == 0x0001 && device.usage() == 0x0002)
        .map(|device| (device.vendor_id(), device.product_id()))
        .collect();

    // No serial number and no device path: nothing that identifies the person.
    let devices: Vec<serde_json::Value> = api
        .device_list()
        .filter(|device| mice.contains(&(device.vendor_id(), device.product_id())))
        .map(|device| {
            serde_json::json!({
                "vid": hex4(device.vendor_id()),
                "pid": hex4(device.product_id()),
                "manufacturer": device.manufacturer_string().unwrap_or(""),
                "product": device.product_string().unwrap_or(""),
                "interface": device.interface_number(),
                "usagePage": format!("0x{}", hex4(device.usage_page())),
                "usage": format!("0x{}", hex4(device.usage())),
                "release": format!("0x{}", hex4(device.release_number())),
            })
        })
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "app": "Mousely Hub",
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "devices": devices,
    }))
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_hid_info() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(read_hid_info)
        .await
        .map_err(|error| error.to_string())?
}
