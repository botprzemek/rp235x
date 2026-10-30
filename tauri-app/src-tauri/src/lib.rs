// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn window_start_drag(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.start_dragging();
    }
}

#[tauri::command]
fn window_minimize(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

#[tauri::command]
fn window_toggle_maximize(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_maximized().unwrap_or(false) {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
fn window_toggle_fullscreen(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_fs = window.is_fullscreen().unwrap_or(false);
        let _ = window.set_fullscreen(!is_fs);
    }
}

#[tauri::command]
fn window_close(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
}

#[tauri::command]
fn set_transparent_mode(app: AppHandle, transparent: bool) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_decorations(!transparent);
        let _ = window.set_shadow(!transparent);
        if transparent {
            let _ = window.set_fullscreen(true);
        } else {
            let _ = window.set_fullscreen(false);
            let _ = window.set_size(PhysicalSize::new(1280, 800));
        }
    }
}

#[tauri::command]
fn set_island_mode(app: AppHandle, island: bool) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_fullscreen(false);
        let _ = window.set_decorations(!island);
        let _ = window.set_shadow(island);
        let _ = window.set_always_on_top(island);
        if island {
            // Compact island window sized precisely for the scoreboard
            let _ = window.set_size(PhysicalSize::new(760, 240));
        } else {
            let _ = window.set_size(PhysicalSize::new(1280, 800));
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            window_start_drag,
            window_minimize,
            window_toggle_maximize,
            window_toggle_fullscreen,
            window_close,
            set_transparent_mode,
            set_island_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
