//! Tray icon: battery icon + tooltip. Right click opens a small window of the app (not a native
//! Windows menu: that one kept the whole app waiting while it was open). Closing the main window
//! destroys it (and its WebView2), so the app keeps running in the tray with no web process.
//! Windows are created again on demand.
//!
//! Registered as a plugin: `.plugin(tray::plugin())` in lib.rs.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tauri::image::Image;
use tauri::plugin::{Builder, TauriPlugin};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Manager, PhysicalPosition, RunEvent, Runtime, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};

use crate::mouse::commands::probe;
use crate::mouse::MouseInfoDto;
use crate::tray_icon::{describe, render, BatteryView, TrayState, SIZE};

const TRAY_ID: &str = "mousely-hub-tray";
const MENU_LABEL: &str = "tray-menu";
const MENU_WIDTH: f64 = 250.0;
const MENU_HEIGHT: f64 = 150.0;
const REFRESH_EVERY: Duration = Duration::from_secs(60);

/// The app may only stay alive without windows when the tray really exists, otherwise it could not be reopened.
pub(crate) static TRAY_READY: AtomicBool = AtomicBool::new(false);

/// When the small menu was shown (ms since start of epoch), to ignore the focus change that
/// happens while it opens.
static MENU_SHOWN_AT: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Text shown by the small menu window: (model name, status line).
static MENU_TEXT: Mutex<(String, String)> = Mutex::new((String::new(), String::new()));

/// What the icon and tooltip show now, so they are touched only when something changed.
static LAST_KEY: Mutex<String> = Mutex::new(String::new());

fn focus<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

/// Adapts the initial window size to smaller or high-DPI monitors while keeping 1180x720 whenever it fits.
fn fit_window_to_monitor<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
    target_w: f64,
    target_h: f64,
) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        let _ = window.center();
        return;
    };

    let scale = monitor.scale_factor();
    let scale = if scale > 0.0 { scale } else { 1.0 };
    let work = monitor.work_area();
    let work_w = f64::from(work.size.width) / scale;
    let work_h = f64::from(work.size.height) / scale;
    let max_w = (work_w - 24.0).max(work_w.min(320.0));
    let max_h = (work_h - 24.0).max(work_h.min(240.0));

    if target_w > max_w || target_h > max_h {
        let new_w = target_w.min(max_w).max(860.0_f64.min(max_w)).round();
        let new_h = target_h.min(max_h).max(540.0_f64.min(max_h)).round();
        let min_w = 860.0_f64.min(new_w);
        let min_h = 540.0_f64.min(new_h);
        let _ = window.set_min_size(Some(tauri::LogicalSize::new(min_w, min_h)));
        let _ = window.set_size(tauri::LogicalSize::new(new_w, new_h));
    }

    let _ = window.center();
}

/// Shows the main window. When it does not exist (closed to the tray, or the app started with
/// `--minimized`) it is created from the config first, on its own thread.
pub fn show_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        focus(&window);
        return;
    }

    let app = app.clone();
    std::thread::spawn(move || {
        // Two quick clicks: the other thread may have built it already.
        if let Some(window) = app.get_webview_window("main") {
            focus(&window);
            return;
        }
        let Some(config) = app.config().app.windows.first() else {
            return;
        };
        if let Ok(builder) = WebviewWindowBuilder::from_config(&app, config) {
            if let Ok(window) = builder.build() {
                fit_window_to_monitor(&window, config.width, config.height);
                focus(&window);
            }
        }
    });
}

/// The small menu window is destroyed when it closes (no web process stays), and built again
/// when it is needed.
fn close_menu<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MENU_LABEL) {
        let _ = window.close();
    }
}

fn build_menu_window<R: Runtime>(app: &AppHandle<R>) -> Option<tauri::WebviewWindow<R>> {
    let built = WebviewWindowBuilder::new(app, MENU_LABEL, WebviewUrl::App("index.html".into()))
        .title("Mousely Hub menu")
        .inner_size(MENU_WIDTH, MENU_HEIGHT)
        .decorations(false)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(true)
        .focused(false)
        .visible(false)
        .build();

    match built {
        Ok(window) => Some(window),
        Err(_error) => {
            None
        }
    }
}

fn place_and_show<R: Runtime>(app: &AppHandle<R>, window: &tauri::WebviewWindow<R>, click: PhysicalPosition<f64>) {
    let (width, height) = window
        .outer_size()
        .map(|size| (size.width as i32, size.height as i32))
        .unwrap_or((300, 190));

    let mut x = click.x as i32 - width / 2;
    let mut y = click.y as i32 - height - 8;

    if y < 0 {
        y = click.y as i32 + 8;
    }

    if let Ok(Some(monitor)) = app.primary_monitor() {
        let origin = monitor.position();
        let size = monitor.size();
        x = x.min(origin.x + size.width as i32 - width).max(origin.x);
    } else {
        x = x.max(0);
    }

    let _ = window.set_position(PhysicalPosition::new(x, y));
    MENU_SHOWN_AT.store(now_ms(), Ordering::SeqCst);
    let _ = window.show();
    let _ = window.set_focus();
}

fn icon_for(view: BatteryView) -> Image<'static> {
    Image::new_owned(render(view), SIZE, SIZE)
}

fn state_from(info: &MouseInfoDto) -> TrayState {
    describe(
        info.connected,
        &info.name,
        &info.connection,
        info.battery,
        info.charging,
        info.capabilities.battery,
    )
}

/// Shows a new state: text of the small menu at once, icon and tooltip only when they changed.
/// The update is queued on the main thread without waiting for it.
fn apply_state<R: Runtime>(app: &AppHandle<R>, state: TrayState) {
    if let Ok(mut text) = MENU_TEXT.lock() {
        *text = (state.title.clone(), state.status.clone());
    }

    let key = format!("{}|{}|{}", state.title, state.status, state.tooltip);

    match LAST_KEY.lock() {
        Ok(mut last) => {
            if *last == key {
                return;
            }
            *last = key;
        }
        Err(_) => return,
    }

    let view = state.view;
    let tooltip = state.tooltip.clone();
    let inner = app.clone();

    let _ = app.run_on_main_thread(move || {
        if let Some(tray) = inner.tray_by_id(TRAY_ID) {
            let _ = tray.set_icon(Some(icon_for(view)));
            let _ = tray.set_tooltip(Some(tooltip.as_str()));
        }
    });
}

/// Called when the mouse was read again (cable, receiver...): updates the tray at once.
pub fn apply_info(app: &AppHandle, info: &MouseInfoDto) {
    apply_state(app, state_from(info));
}

/// Opens the small menu window next to the tray icon (or hides it when it is already open).
fn toggle_menu<R: Runtime>(app: &AppHandle<R>, click: PhysicalPosition<f64>) {

    if let Some(window) = app.get_webview_window(MENU_LABEL) {
        if window.is_visible().unwrap_or(false) {
            close_menu(app);
        } else {
            place_and_show(app, &window, click);
        }
        return;
    }

    // The window does not exist: create it on its own thread.
    let app = app.clone();
    std::thread::spawn(move || {
        if let Some(window) = build_menu_window(&app) {
            place_and_show(&app, &window, click);
        }
    });
}

#[tauri::command]
pub fn tray_text() -> (String, String) {
    MENU_TEXT
        .lock()
        .map(|text| text.clone())
        .unwrap_or_else(|error| error.into_inner().clone())
}

#[tauri::command]
pub fn tray_open(app: AppHandle) {
    show_window(&app);
    close_menu(&app);
}

#[tauri::command]
pub fn tray_close(app: AppHandle) {
    close_menu(&app);
}

#[tauri::command]
pub fn tray_quit(app: AppHandle) {
    app.exit(0);
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mousely-hub-tray")
        .setup(|app, _api| {
            if let Ok(mut text) = MENU_TEXT.lock() {
                *text = ("Looking for mouse...".to_string(), String::new());
            }

            TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon_for(BatteryView::Missing))
                .tooltip("Mousely Hub")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button,
                        button_state: MouseButtonState::Up,
                        position,
                        ..
                    } = event
                    {
                        match button {
                            MouseButton::Left => {
                                close_menu(tray.app_handle());
                                show_window(tray.app_handle());
                            }
                            MouseButton::Right => toggle_menu(tray.app_handle(), position),
                            _ => {}
                        }
                    }
                })
                .build(app)?;

            TRAY_READY.store(true, Ordering::SeqCst);

            // One light reading per minute, on its own thread. The icon and tooltip are only
            // touched when something changed, and the update is queued on the main thread
            // without waiting for it.
            let handle = app.clone();

            std::thread::spawn(move || loop {
                let result = probe();
                apply_state(&handle, state_from(&result.info));
                std::thread::sleep(REFRESH_EVERY);
            });

            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::WindowEvent { label, event: window_event, .. } = event {
                match window_event {
                    // Clicking anywhere else closes the small menu.
                    WindowEvent::Focused(false) if label.as_str() == MENU_LABEL => {
                        let age = now_ms().saturating_sub(MENU_SHOWN_AT.load(Ordering::SeqCst));
                        if age > 500 {
                            close_menu(app);
                        }
                    }
                    WindowEvent::Focused(true) if label.as_str() == MENU_LABEL => {
                    }
                    // "Keep running in the tray" is off: closing the window quits the app.
                    WindowEvent::CloseRequested { .. }
                        if label.as_str() == "main" && !crate::settings::CLOSE_TO_TRAY.load(Ordering::SeqCst) =>
                    {
                        app.exit(0);
                    }
                    _ => {}
                }
            }
        })
        .build()
}
