mod commands;
mod device_watch;
mod macros;
mod mouse;
mod tray;
mod tray_icon;
mod remap;
mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tray::plugin())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Auto-update. If the signing key is not configured yet the plugin simply stays off.
            let _ = app.handle().plugin(tauri_plugin_updater::Builder::new().build());
            let _ = app.handle().plugin(tauri_plugin_process::init());

            macros::recorder::start_listener(app.handle().clone());
            remap::hook::start(app.handle().clone());
            device_watch::start(app.handle().clone());

            // The window starts hidden: it is shown now, unless Windows started the app at login
            // (`--minimized`): then it stays in the tray.
            if !std::env::args().any(|arg| arg == "--minimized") {
                if let Some(window) = tauri::Manager::get_webview_window(app, "main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::macros::start_recording,
            commands::macros::stop_recording,
            commands::macros::play_macro,
            commands::macros::stop_playback,
            tray::tray_text,
            tray::tray_open,
            tray::tray_close,
            tray::tray_quit,
            settings::get_autostart,
            settings::set_autostart,
            settings::set_close_to_tray,
            settings::export_hid_info,
            mouse::commands::get_mouse_info,
            mouse::commands::set_polling_rate,
            mouse::commands::set_mouse_dpi,
            remap::commands::set_mouse_mappings,
            remap::commands::set_remap_enabled,
            remap::commands::start_button_detect,
            remap::commands::cancel_button_detect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}