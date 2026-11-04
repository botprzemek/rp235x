// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::net::UdpSocket;
use std::time::{Duration, Instant};

use net::event::{ClientEvent, ServerEvent};
use net::layout::PACKET_SIZE;
use net::{ClientPacket, Device, DeviceConfigPacket, DeviceType, ServerPacket};
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::window::{Color, Effect, EffectState, EffectsBuilder};
use tauri::{AppHandle, Emitter, Manager, PhysicalSize, WebviewWindow, WindowEvent};

fn apply_macos_vibrancy(window: &WebviewWindow) {
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
    let effects = EffectsBuilder::new()
        .effect(Effect::HudWindow)
        .state(EffectState::Active)
        .radius(24.0)
        .build();
    let _ = window.set_effects(effects);
}

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

#[tauri::command]
async fn discover_master(timeout_secs: Option<u64>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Błąd bind socketu: {}", e))?;
        let _ = socket.set_broadcast(true);
        let _ = socket.set_read_timeout(Some(Duration::from_millis(400)));

        let device = Device::try_new(DeviceType::WebBrowser, "sn-lap-disc", "lapapi-hmi")
            .map_err(|e| format!("Błąd inicjalizacji urządzenia: {:?}", e))?;
        let packet = ClientPacket::new(ClientEvent::HandshakeRequest, 0, device.to_bytes());
        let packet_bytes = packet.to_bytes();

        let timeout = Duration::from_secs(timeout_secs.unwrap_or(4));
        let start = Instant::now();
        let mut buf = [0u8; PACKET_SIZE];

        let targets = ["255.255.255.255:9000", "127.0.0.1:9000"];

        while start.elapsed() < timeout {
            for target in &targets {
                let _ = socket.send_to(&packet_bytes, target);
            }

            let step_start = Instant::now();
            while step_start.elapsed() < Duration::from_millis(400) {
                match socket.recv_from(&mut buf) {
                    Ok((size, remote_addr)) => {
                        if size == PACKET_SIZE {
                            if let Ok(server_packet) = ServerPacket::from_bytes(buf) {
                                if server_packet.event() == ServerEvent::HandshakeAck {
                                    let ip = remote_addr.ip().to_string();
                                    return Ok(format!("{}:3000", ip));
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        }

        Err("Nie wykryto serwera Master w sieci lokalnej.".to_string())
    })
    .await
    .map_err(|e| format!("Błąd zadania: {}", e))?
}

#[tauri::command]
fn send_device_config(
    target_ip: String,
    serial_number: String,
    device_id: String,
    wifi_ssid: String,
    wifi_password: String,
) -> Result<String, String> {
    let packet = DeviceConfigPacket::try_new(
        &serial_number,
        &device_id,
        &wifi_ssid,
        &wifi_password,
    )
    .map_err(|e| format!("Błąd parametrów: {:?}", e))?;

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket
        .set_read_timeout(Some(Duration::from_millis(3000)))
        .map_err(|e| e.to_string())?;

    let target_addr = if target_ip.contains(':') {
        target_ip
    } else {
        format!("{}:8001", target_ip)
    };

    socket
        .send_to(&packet.to_bytes(), &target_addr)
        .map_err(|e| format!("Błąd wysyłania pakietu UDP: {}", e))?;

    let mut buf = [0u8; PACKET_SIZE];
    // Wait for Ack / Success
    let (n, _) = socket
        .recv_from(&mut buf)
        .map_err(|e| format!("Brak odpowiedzi od urządzenia (timeout): {}", e))?;

    if n != PACKET_SIZE {
        return Err("Niepoprawny rozmiar odpowiedzi".to_string());
    }

    let server_packet = ServerPacket::from_bytes(buf).map_err(|e| format!("Błąd pakietu: {}", e))?;

    match server_packet.event() {
        ServerEvent::DeviceConfigAck => {
            // Also try to read the next packet which should be Success
            let _ = socket.set_read_timeout(Some(Duration::from_millis(2000)));
            if let Ok((n2, _)) = socket.recv_from(&mut buf) {
                if n2 == PACKET_SIZE {
                    if let Ok(p2) = ServerPacket::from_bytes(buf) {
                        if p2.event() == ServerEvent::DeviceConfigSuccess {
                            return Ok("Sukces! Urządzenie zaktualizowało pamięć flash i restartuje się.".to_string());
                        }
                    }
                }
            }
            Ok("Konfiguracja zaakceptowana przez urządzenie (Ack).".to_string())
        }
        ServerEvent::DeviceConfigSuccess => {
            Ok("Sukces! Urządzenie zaktualizowało pamięć flash i restartuje się.".to_string())
        }
        other => Err(format!("Nieoczekiwana odpowiedź od urządzenia: {:?}", other)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                apply_macos_vibrancy(&window);
                if let Some(icon) = app.default_window_icon() {
                    let _ = window.set_icon(icon.clone());
                } else if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png")) {
                    let _ = window.set_icon(icon);
                }
            }

            // Native macOS Application Menu
            let app_submenu = SubmenuBuilder::new(app, "Lapapi")
                .about(None)
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()?;

            let file_submenu = SubmenuBuilder::new(app, "Plik")
                .item(&MenuItemBuilder::with_id("new_game", "Nowa gra").accelerator("CmdOrCtrl+N").build(app)?)
                .separator()
                .close_window()
                .build()?;

            let edit_submenu = SubmenuBuilder::new(app, "Edycja")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;

            let view_submenu = SubmenuBuilder::new(app, "Widok")
                .item(&MenuItemBuilder::with_id("zoom_in", "Przybliż (Zoom In)").accelerator("CmdOrCtrl+=").build(app)?)
                .item(&MenuItemBuilder::with_id("zoom_out", "Oddal (Zoom Out)").accelerator("CmdOrCtrl+-").build(app)?)
                .item(&MenuItemBuilder::with_id("zoom_reset", "Rzeczywisty rozmiar (100%)").accelerator("CmdOrCtrl+0").build(app)?)
                .separator()
                .fullscreen()
                .build()?;

            let window_submenu = SubmenuBuilder::new(app, "Okno")
                .minimize()
                .separator()
                .close_window()
                .build()?;

            let menu = MenuBuilder::new(app)
                .items(&[&app_submenu, &file_submenu, &edit_submenu, &view_submenu, &window_submenu])
                .build()?;

            app.set_menu(menu)?;

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                "zoom_in" => {
                    let _ = app.emit("lapapi-zoom-in", ());
                }
                "zoom_out" => {
                    let _ = app.emit("lapapi-zoom-out", ());
                }
                "zoom_reset" => {
                    let _ = app.emit("lapapi-zoom-reset", ());
                }
                "new_game" => {
                    let _ = app.emit("lapapi-new-game", ());
                }
                _ => {}
            }
        })
        .on_window_event(|window, event| match event {
            WindowEvent::Focused(focused) => {
                let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
                let _ = window.emit("lapapi-focus-changed", *focused);
            }
            WindowEvent::Resized(_) | WindowEvent::ThemeChanged(_) => {
                let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            window_start_drag,
            window_minimize,
            window_toggle_maximize,
            window_toggle_fullscreen,
            window_close,
            set_transparent_mode,
            set_island_mode,
            send_device_config,
            discover_master
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
