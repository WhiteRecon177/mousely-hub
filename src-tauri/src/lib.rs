mod commands;
mod device_watch;
mod macros;
mod mouse;
mod tray;
mod tray_icon;
mod remap;
mod settings;
mod store;

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
            // What the user saved (remap, macros, close setting) is loaded here, so everything works
            // from Rust alone, with no window.
            settings::CLOSE_TO_TRAY.store(
                store::load_settings(app.handle()).close_to_tray,
                std::sync::atomic::Ordering::SeqCst,
            );
            remap::hook::apply(store::load_remap(app.handle()));
            remap::hook::start(app.handle().clone());
            device_watch::start(app.handle().clone());

            // The window is not created at start: it is created now, unless Windows started the app
            // at login (`--minimized`): then no window exists until the tray asks for one.
            if !std::env::args().any(|arg| arg == "--minimized") {
                tray::show_window(app.handle());
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
            remap::commands::get_remap_state,
            remap::commands::start_button_detect,
            remap::commands::cancel_button_detect,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // With every window closed the app keeps running in the tray (remapping goes on).
            // `app.exit(0)` (tray "Quit") has a code, so it still quits.
            if let tauri::RunEvent::ExitRequested { code: None, api, .. } = event {
                if tray::TRAY_READY.load(std::sync::atomic::Ordering::SeqCst) {
                    api.prevent_exit();
                }
            }
        });
}