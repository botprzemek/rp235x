//! Legacy HMI interface with 3-column macOS layout: Sidebar, MatchControlPanel, and DeviceInspector.

use core::pin::pin;
use core::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use crate::adapters::tauri::{invoke_send_device_config, invoke_tauri_window_command};
use crate::adapters::ws::{
    BrowserWebSocketAdapter, HmiClientHandler, load_master_address, load_or_create_browser_device,
    save_master_address,
};
use crate::ui::hmi::{
    DeviceInspector, MasterServerModal, MatchControlPanel, NavigationSidebar, WifiConfigModal,
};

use basketball::{Discipline, Game};
use config::DeviceConfig;
use futures::StreamExt;
use futures::channel::mpsc::{self, UnboundedSender};
use futures::future::{self, Either};
use leptos::prelude::*;
use net::layout::{DATA_SIZE, PACKET_SIZE};
use net::{
    Client, ClientEvent, ClientPacket, Device, DeviceType, NetError, ServerEvent, ServerPacket,
    Transport,
};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, MessageEvent, WebSocket};

/// Full Legacy HMI component with match controls, tree navigation, and inspector.
#[component]
pub fn LegacyHmiView(
    on_back: Callback<()>,
    #[prop(into)] is_focused: Signal<bool>,
) -> impl IntoView {
    let is_lights_hovered = RwSignal::new(false);
    let zoom_level = RwSignal::new(1.0f64);

    // Master server address & modal state
    let master_address = RwSignal::new(load_master_address());
    let is_master_modal_open = RwSignal::new(false);
    let reconnect_trigger = RwSignal::new(0u32);

    // WebSocket state
    let is_connected = RwSignal::new(false);
    let command_sender = RwSignal::new(None::<UnboundedSender<ClientPacket>>);
    let sequence_id = Arc::new(AtomicU16::new(0));
    let logs = RwSignal::new(Vec::<String>::new());

    // Domain data
    let current_game = RwSignal::new(Game::new(Discipline::Fiba5v5));
    let available_games = RwSignal::new(Vec::<Game>::new());
    let connected_devices = RwSignal::new(Vec::<Device>::new());
    let selected_device_sn = RwSignal::new(None::<String>);
    let my_device = RwSignal::new(
        Device::try_new(DeviceType::WebBrowser, "sn-hmi-0000", "hmi-app")
            .expect("valid browser DeviceConfig"),
    );

    // Wi-Fi Config Modal state
    let is_wifi_modal_open = RwSignal::new(false);
    let wifi_target_ip = RwSignal::new("192.168.4.1".to_string());
    let wifi_modal_sn = RwSignal::new("sn-hmi-0000".to_string());
    let wifi_modal_id = RwSignal::new("hmi-worker".to_string());
    let wifi_loading = RwSignal::new(false);
    let wifi_status_msg = RwSignal::new(None::<String>);

    let add_log = move |message: String| {
        logs.update(|buf| {
            buf.push(message);
            if buf.len() > 60 {
                buf.remove(0);
            }
        });
    };

    // Zooming logic: driven by macOS System Menu (Properties) and ⌘+/⌘-/⌘0
    let on_zoom_in = Callback::new(move |()| {
        zoom_level.update(|z| {
            *z = (*z + 0.1).min(1.75);
        });
    });

    let on_zoom_out = Callback::new(move |()| {
        zoom_level.update(|z| {
            *z = (*z - 0.1).max(0.6);
        });
    });

    let on_zoom_reset = Callback::new(move |()| {
        zoom_level.set(1.0);
    });

    // Window events, OS menu events and keyboard shortcuts (⌘+, ⌘-, ⌘0)
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let on_keydown = {
                let on_zoom_in = on_zoom_in;
                let on_zoom_out = on_zoom_out;
                let on_zoom_reset = on_zoom_reset;

                Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
                    move |e: web_sys::KeyboardEvent| {
                        if e.meta_key() || e.ctrl_key() {
                            match e.key().as_str() {
                                "=" | "+" => {
                                    e.prevent_default();
                                    on_zoom_in.run(());
                                }
                                "-" => {
                                    e.prevent_default();
                                    on_zoom_out.run(());
                                }
                                "0" => {
                                    e.prevent_default();
                                    on_zoom_reset.run(());
                                }
                                _ => {}
                            }
                        }
                    },
                )
            };
            let _ = window
                .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref());
            on_keydown.forget();

            // Native Tauri Event Listeners (macOS System Menu)
            if let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
                && let Ok(event_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("event"))
                && let Ok(listen_fn) =
                    js_sys::Reflect::get(&event_obj, &JsValue::from_str("listen"))
                && let Ok(func) = listen_fn.dyn_into::<js_sys::Function>()
            {
                // OS Menu: Zoom In
                let on_zin = {
                    let on_zoom_in = on_zoom_in;
                    Closure::<dyn FnMut(JsValue)>::new(move |_| {
                        on_zoom_in.run(());
                    })
                };
                let _ = func.call2(
                    &event_obj,
                    &JsValue::from_str("hmi-zoom-in"),
                    on_zin.as_ref().unchecked_ref(),
                );
                on_zin.forget();

                // OS Menu: Zoom Out
                let on_zout = {
                    let on_zoom_out = on_zoom_out;
                    Closure::<dyn FnMut(JsValue)>::new(move |_| {
                        on_zoom_out.run(());
                    })
                };
                let _ = func.call2(
                    &event_obj,
                    &JsValue::from_str("hmi-zoom-out"),
                    on_zout.as_ref().unchecked_ref(),
                );
                on_zout.forget();

                // OS Menu: Zoom Reset
                let on_zreset = {
                    let on_zoom_reset = on_zoom_reset;
                    Closure::<dyn FnMut(JsValue)>::new(move |_| {
                        on_zoom_reset.run(());
                    })
                };
                let _ = func.call2(
                    &event_obj,
                    &JsValue::from_str("hmi-zoom-reset"),
                    on_zreset.as_ref().unchecked_ref(),
                );
                on_zreset.forget();
            }
        }
    });

    // Background auto-reconnect timer and auto-discovery
    Effect::new(move |_| {
        async fn sleep_ms(millis: i32) {
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                if let Some(window) = web_sys::window() {
                    let _ = window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis);
                }
            });
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        }

        wasm_bindgen_futures::spawn_local(async move {
            let mut check_discovery = true;

            loop {
                sleep_ms(2500).await;

                if !is_connected.get_untracked() {
                    // If not connected yet in Tauri desktop environment, try auto-discovery if address is default
                    if check_discovery {
                        let current = master_address.get_untracked();
                        if (current == "127.0.0.1:3000" || current.is_empty())
                            && let Ok(discovered) =
                                crate::adapters::tauri::invoke_discover_master(Some(2)).await
                            && !discovered.is_empty()
                            && discovered != current
                        {
                            add_log(format!(
                                "[AUTO-DISCOVERY] Wykryto Mastera w sieci: {discovered}"
                            ));
                            save_master_address(&discovered);
                            master_address.set(discovered);
                            check_discovery = false;
                            continue;
                        }
                    }

                    // Trigger reconnection attempt
                    reconnect_trigger.update(|n| *n = n.wrapping_add(1));
                } else {
                    check_discovery = false;
                }
            }
        });
    });

    // Reactive WebSocket connection lifecycle (reconnects whenever master_address or reconnect_trigger changes)
    Effect::new(move |_| {
        let _ = reconnect_trigger.get();
        let client_device = load_or_create_browser_device();
        my_device.set(client_device);

        let target_addr = master_address.get();
        let target_addr = target_addr.trim();

        // Support formats like "127.0.0.1:3000", "ws://127.0.0.1:3000", "http://127.0.0.1:3000"
        let websocket_url = if target_addr.starts_with("ws://") || target_addr.starts_with("wss://")
        {
            if target_addr.ends_with("/ws") {
                target_addr.to_string()
            } else {
                format!("{}/ws", target_addr.trim_end_matches('/'))
            }
        } else if let Some(stripped) = target_addr.strip_prefix("http://") {
            format!("ws://{}/ws", stripped.trim_end_matches('/'))
        } else if let Some(stripped) = target_addr.strip_prefix("https://") {
            format!("wss://{}/ws", stripped.trim_end_matches('/'))
        } else {
            format!("ws://{}/ws", target_addr.trim_end_matches('/'))
        };

        add_log(format!("[WS] Łączenie z Masterem: {}", websocket_url));

        if let Ok(websocket) = WebSocket::new(&websocket_url) {
            websocket.set_binary_type(BinaryType::Arraybuffer);

            let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded::<[u8; PACKET_SIZE]>();
            let (incoming_tx, incoming_rx) = mpsc::unbounded::<[u8; PACKET_SIZE]>();
            let (packet_tx, mut packet_rx) = mpsc::unbounded::<ServerPacket>();
            let (command_tx, mut command_rx) = mpsc::unbounded::<ClientPacket>();

            // On open
            {
                let connected_clone = is_connected;
                let command_tx_open = command_tx.clone();
                let onopen = Closure::wrap(Box::new(move |_event: JsValue| {
                    connected_clone.set(true);
                    add_log("Połączono z serwerem Master.".to_string());

                    let handshake = ClientPacket::new(
                        ClientEvent::HandshakeRequest,
                        0,
                        client_device.to_bytes(),
                    );
                    let _ = command_tx_open.unbounded_send(handshake);
                }) as Box<dyn FnMut(JsValue)>);
                websocket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                onopen.forget();
            }

            // On message
            {
                let incoming_tx_message = incoming_tx.clone();
                let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
                    if let Ok(buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                        let array = js_sys::Uint8Array::new(&buffer);
                        if array.length() as usize == PACKET_SIZE {
                            let mut data_array = [0u8; PACKET_SIZE];
                            array.copy_to(&mut data_array);
                            let _ = incoming_tx_message.unbounded_send(data_array);
                        }
                    }
                }) as Box<dyn FnMut(MessageEvent)>);
                websocket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();
            }

            // On close
            {
                let connected_clone = is_connected;
                let devices_clone = connected_devices;
                let games_clone = available_games;
                let incoming_tx_close = incoming_tx;
                let onclose = Closure::wrap(Box::new(move |_event: JsValue| {
                    connected_clone.set(false);
                    devices_clone.set(Vec::new());
                    games_clone.set(Vec::new());
                    incoming_tx_close.close_channel();
                    add_log("Rozłączono z serwerem Master.".to_string());
                }) as Box<dyn FnMut(JsValue)>);
                websocket.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                onclose.forget();
            }

            // Outgoing WebSocket sender task
            {
                let websocket_sender = websocket.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    while let Some(bytes) = outgoing_rx.next().await {
                        let uint8_array = js_sys::Uint8Array::from(&bytes[..]);
                        if let Err(err) =
                            websocket_sender.send_with_array_buffer(&uint8_array.buffer())
                        {
                            add_log(format!("Błąd nadawania WebSocket: {err:?}"));
                        }
                    }
                });
            }

            // Incoming server packet handler task
            {
                let current_game_clone = current_game;
                let games_state_clone = available_games;
                let devices_state_clone = connected_devices;

                wasm_bindgen_futures::spawn_local(async move {
                    let mut device_list = Vec::<Device>::new();
                    let mut game_list = Vec::<Game>::new();
                    let mut last_ui_flush = 0.0f64;

                    while let Some(packet) = packet_rx.next().await {
                        let mut devices_changed = false;
                        let mut games_changed = false;

                        let mut current = Some(packet);
                        while let Some(packet) = current {
                            match packet.event() {
                                ServerEvent::HandshakeAck => {
                                    if let Ok(game) = Game::try_from(packet) {
                                        current_game_clone.set(game);
                                        if !game_list.iter().any(|g| g.id() == game.id()) {
                                            game_list.push(game);
                                            game_list.sort_by_key(Game::id);
                                            games_changed = true;
                                        }
                                        add_log(format!("[SESJA] Gra #{}", game.id()));
                                    }
                                }
                                ServerEvent::GameUpdate => {
                                    if let Ok(game) = Game::try_from(packet) {
                                        if let Some(existing) =
                                            game_list.iter_mut().find(|g| g.id() == game.id())
                                        {
                                            *existing = game;
                                        } else {
                                            game_list.push(game);
                                            game_list.sort_by_key(Game::id);
                                        }
                                        games_changed = true;

                                        let current_id =
                                            current_game_clone.with_untracked(Game::id);
                                        if current_id == game.id() || current_id == 0 {
                                            current_game_clone.set(game);
                                        }
                                    }
                                }
                                ServerEvent::GameListUpdate => {
                                    if let Ok(game) = Game::try_from(packet) {
                                        if let Some(existing) =
                                            game_list.iter_mut().find(|g| g.id() == game.id())
                                        {
                                            *existing = game;
                                        } else {
                                            game_list.push(game);
                                            game_list.sort_by_key(Game::id);
                                        }
                                        games_changed = true;
                                    }
                                }
                                ServerEvent::DeviceUpdate => {
                                    if let Ok(updated_dev) = Device::try_from(packet) {
                                        let is_new = if let Some(existing) =
                                            device_list.iter_mut().find(|d| {
                                                d.serial_number() == updated_dev.serial_number()
                                            }) {
                                            let changed = *existing != updated_dev;
                                            *existing = updated_dev;
                                            changed
                                        } else {
                                            device_list.push(updated_dev);
                                            true
                                        };

                                        if is_new {
                                            devices_changed = true;
                                            if device_list.len() <= 64 {
                                                add_log(format!(
                                                    "[URZĄDZENIE] {:?} (sn={}) -> Gra #{}",
                                                    updated_dev.device_type(),
                                                    updated_dev.serial_number().unwrap_or("-"),
                                                    updated_dev.game_id()
                                                ));
                                            }
                                        }
                                    }
                                }
                                ServerEvent::DeviceDisconnect => {
                                    if let Ok(dc_dev) = Device::try_from(packet) {
                                        device_list.retain(|d| {
                                            d.serial_number() != dc_dev.serial_number()
                                        });
                                        devices_changed = true;
                                        add_log(format!(
                                            "[ROZŁĄCZONO] {:?}",
                                            dc_dev.serial_number().unwrap_or("-")
                                        ));
                                    }
                                }
                                _ => {}
                            }

                            current = packet_rx.try_recv().ok();
                        }

                        let now = js_sys::Date::now();
                        let should_flush = (now - last_ui_flush) >= 100.0 || current.is_none();

                        if should_flush {
                            if devices_changed {
                                devices_state_clone.set(device_list.clone());
                            }
                            if games_changed {
                                games_state_clone.set(game_list.clone());
                            }
                            last_ui_flush = now;
                        }
                    }
                });
            }

            // Client loop
            {
                let adapter = BrowserWebSocketAdapter::new(outgoing_tx, incoming_rx);
                let transport = Transport::new(adapter, None);
                let handler = HmiClientHandler::new(packet_tx);
                let mut client = Client::new(transport, handler);

                wasm_bindgen_futures::spawn_local(async move {
                    enum Step {
                        Polled(Result<ServerPacket, NetError>),
                        Command(Option<ClientPacket>),
                    }

                    loop {
                        let step = {
                            let poll_future = pin!(client.poll());
                            let send_future = pin!(command_rx.next());

                            match future::select(poll_future, send_future).await {
                                Either::Left((res, _)) => Step::Polled(res),
                                Either::Right((cmd, _)) => Step::Command(cmd),
                            }
                        };

                        match step {
                            Step::Polled(Ok(_)) => {}
                            Step::Polled(Err(NetError::InvalidPacket(err))) => {
                                add_log(format!("[BŁĄD PAKIETU] {err}"));
                            }
                            Step::Polled(Err(NetError::Disconnected | _)) => break,
                            Step::Command(Some(packet)) => {
                                client.send(&packet).await;
                            }
                            Step::Command(None) => break,
                        }
                    }
                });
            }

            command_sender.set(Some(command_tx));
        } else {
            add_log(format!(
                "[WS] Nieprawidłowy adres WebSocket: {}",
                websocket_url
            ));
        }
    });

    // Helper for dispatching ClientPackets
    let send_client_packet = {
        let seq_arc = sequence_id.clone();
        Arc::new(move |event: ClientEvent, payload: [u8; DATA_SIZE]| {
            if let Some(sender) = command_sender.get().as_ref() {
                let current_seq = seq_arc.fetch_add(1, Ordering::Relaxed).wrapping_add(1);

                let packet = ClientPacket::new(event, current_seq, payload);
                if sender.unbounded_send(packet).is_ok() {
                    add_log(format!("[WYSŁANO] {event:?} (seq={current_seq})"));
                }
            }
        })
    };

    let on_send_event = {
        let send = send_client_packet.clone();
        Callback::new(move |event: ClientEvent| {
            send(event, current_game.get().to_bytes());
        })
    };

    let on_select_game = {
        let send = send_client_packet.clone();
        Callback::new(move |game_id: u16| {
            if let Some(target) = available_games.get().iter().find(|g| g.id() == game_id) {
                current_game.set(*target);
            }
            let target_game = Game::new_with_id(game_id, Discipline::Fiba5v5);
            send(ClientEvent::GameSelect, target_game.to_bytes());
        })
    };

    let on_create_game = {
        let send = send_client_packet.clone();
        Callback::new(move |()| {
            let next_id = available_games
                .get()
                .iter()
                .map(Game::id)
                .max()
                .unwrap_or(0)
                + 1;
            let new_game = Game::new_with_id(next_id, Discipline::Fiba5v5);
            current_game.set(new_game);
            send(ClientEvent::GameCreate, new_game.to_bytes());
        })
    };

    let on_disconnect_device = {
        let send = send_client_packet.clone();
        Callback::new(move |target: Device| {
            send(ClientEvent::DeviceDisconnect, target.to_bytes());
        })
    };

    let on_create_dummy_device = {
        let send = send_client_packet.clone();
        Callback::new(move |()| {
            let next_num = connected_devices
                .get()
                .iter()
                .filter_map(|d| {
                    let sn = d.serial_number().ok()?;
                    sn.strip_prefix("dummy-")?.parse::<u32>().ok()
                })
                .max()
                .unwrap_or(0)
                + 1;

            let sn = format!("dummy-{next_num:04}");
            let id = format!("dummy-worker-{next_num:04}");

            if let Ok(dummy) = Device::try_new(DeviceType::Worker, &sn, &id) {
                // Register via master websocket
                send(ClientEvent::DeviceAssignGame, dummy.to_bytes());
                // Immediately register locally so UI updates even if offline
                connected_devices.update(|list| {
                    if !list.iter().any(|d| d.serial_number() == Ok(&sn)) {
                        list.push(dummy);
                    }
                });
                add_log(format!("[DUMMY] Utworzono urządzenie testowe: {sn} ({id})"));
            }
        })
    };

    let on_assign_device_game = {
        let send = send_client_packet.clone();
        Callback::new(move |(mut target, game_id): (Device, u16)| {
            target.set_game_id(game_id);
            send(ClientEvent::DeviceAssignGame, target.to_bytes());
        })
    };

    let on_select_device = Callback::new(move |sn: String| {
        selected_device_sn.update(|current| {
            if current.as_ref() == Some(&sn) {
                *current = None;
            } else {
                *current = Some(sn);
            }
        });
    });

    // Window drag and controls
    let on_start_drag = Callback::new(|event: web_sys::MouseEvent| {
        if event.buttons() == 1 {
            invoke_tauri_window_command("window_start_drag");
        }
    });

    let stop_propagation = Callback::new(|event: web_sys::MouseEvent| {
        event.stop_propagation();
    });

    let on_close = Callback::new(|event: web_sys::MouseEvent| {
        event.stop_propagation();
        invoke_tauri_window_command("window_close");
    });

    let on_minimize = Callback::new(|event: web_sys::MouseEvent| {
        event.stop_propagation();
        invoke_tauri_window_command("window_minimize");
    });

    let on_maximize = Callback::new(|event: web_sys::MouseEvent| {
        event.stop_propagation();
        invoke_tauri_window_command("window_toggle_maximize");
    });

    // Master server modal callbacks
    let on_open_master_modal = Callback::new(move |()| {
        is_master_modal_open.set(true);
    });

    let on_close_master_modal = Callback::new(move |_: web_sys::MouseEvent| {
        is_master_modal_open.set(false);
    });

    let on_save_master_address = Callback::new(move |new_address: String| {
        save_master_address(&new_address);
        master_address.set(new_address);
        is_master_modal_open.set(false);
    });

    // Wi-Fi dialog callbacks
    let on_open_wifi_setup = Callback::new(move |()| {
        add_log("[UI] Otwieranie konfiguracji Wi-Fi".to_string());
        wifi_target_ip.set("192.168.4.1".to_string());
        wifi_status_msg.set(None);
        wifi_loading.set(false);
        is_wifi_modal_open.set(true);
    });

    let on_open_wifi_for_device = Callback::new(move |device: Device| {
        let sn = device.serial_number().unwrap_or("sn-hmi-0000").to_string();
        let id = device.device_id().unwrap_or("hmi-worker").to_string();
        add_log(format!(
            "[UI] Otwieranie konfiguracji Wi-Fi dla urządzenia: {}",
            sn
        ));
        wifi_modal_sn.set(sn);
        wifi_modal_id.set(id);
        wifi_target_ip.set("192.168.4.1".to_string());
        wifi_status_msg.set(None);
        wifi_loading.set(false);
        is_wifi_modal_open.set(true);
    });

    let on_close_wifi_modal = Callback::new(move |_: web_sys::MouseEvent| {
        is_wifi_modal_open.set(false);
    });

    let on_submit_wifi_config = Callback::new(
        move |(ip, sn, id, ssid, pass): (String, String, String, String, String)| {
            wifi_loading.set(true);
            wifi_status_msg.set(Some(
                "Wysyłanie pakietu DeviceConfigSet przez UDP...".to_string(),
            ));

            wasm_bindgen_futures::spawn_local(async move {
                match invoke_send_device_config(ip, sn, id, ssid, pass).await {
                    Ok(msg) => {
                        wifi_status_msg.set(Some(msg.clone()));
                        add_log(format!("[UDP SETUP] {}", msg));
                    }
                    Err(err_msg) => {
                        wifi_status_msg.set(Some(format!("Błąd: {}", err_msg)));
                        add_log(format!("[UDP ERROR] {}", err_msg));
                    }
                }
                wifi_loading.set(false);
            });
        },
    );

    // Dynamic background styling based on focus
    let root_bg_class = move || {
        if is_focused.get() {
            "bg-[#101318]/70 border-white/[0.12]"
        } else {
            "bg-[#1c202a]/75 border-white/[0.08]"
        }
    };

    let sidebar_bg_class = Signal::derive(move || {
        if is_focused.get() {
            "bg-[#13161f]/45"
        } else {
            "bg-[#202533]/50"
        }
    });

    let main_bg_class = Signal::derive(move || {
        if is_focused.get() {
            "bg-[#0b0e14]/30"
        } else {
            "bg-[#1e2230]/40"
        }
    });

    let inspector_bg_class = Signal::derive(move || {
        if is_focused.get() {
            "bg-[#13161f]/40"
        } else {
            "bg-[#202533]/45"
        }
    });

    let selected_device = Signal::derive(move || {
        selected_device_sn.get().and_then(|sn| {
            connected_devices
                .get()
                .iter()
                .find(|d| d.serial_number().ok() == Some(sn.as_str()))
                .copied()
        })
    });

    view! {
        <div
            class=move || format!("w-screen h-screen overflow-hidden rounded-xl border {} transition-colors duration-200 flex text-white/90 select-none font-sans antialiased text-[13px]", root_bg_class())
            style=move || format!("zoom: {};", zoom_level.get())
        >
            // Tree-based Sidebar (Games and Devices)
            <NavigationSidebar
                is_focused=is_focused
                is_lights_hovered=is_lights_hovered
                on_start_drag=on_start_drag
                stop_propagation=stop_propagation
                on_close=on_close
                on_minimize=on_minimize
                on_maximize=on_maximize
                games=available_games.into()
                selected_game_id=Signal::derive(move || current_game.get().id())
                on_select_game=on_select_game
                on_create_game=on_create_game
                devices=connected_devices.into()
                selected_device_sn=selected_device_sn.into()
                on_select_device=on_select_device
                on_create_dummy_device=on_create_dummy_device
                is_connected=is_connected.into()
                master_address=master_address.into()
                on_open_master_modal=on_open_master_modal
                sidebar_bg_class=sidebar_bg_class
                on_open_wifi_setup=on_open_wifi_setup
                on_back=on_back
            />

            // Center Scoreboard and WebSocket Control View
            <MatchControlPanel
                game=current_game.into()
                is_connected=is_connected.into()
                on_start_drag=on_start_drag
                stop_propagation=stop_propagation
                on_send_event=on_send_event
                main_bg_class=main_bg_class
                devices=connected_devices.into()
                on_disconnect_device=on_disconnect_device
                on_assign_device_game=on_assign_device_game
                on_back=on_back
            />

            // Right Inspector & Logs Panel
            <DeviceInspector
                selected_game=current_game.into()
                selected_device=selected_device
                logs=logs.into()
                on_start_drag=on_start_drag
                inspector_bg_class=inspector_bg_class
                on_disconnect_device=on_disconnect_device
                on_open_wifi_config=on_open_wifi_for_device
            />

            // Master Server Address Config Modal
            <MasterServerModal
                is_open=is_master_modal_open.into()
                current_address=master_address.into()
                on_close=on_close_master_modal
                on_save=on_save_master_address
            />

            // Wi-Fi Config Modal (OTA UDP)
            <WifiConfigModal
                is_open=is_wifi_modal_open.into()
                on_close=on_close_wifi_modal
                on_submit=on_submit_wifi_config
                is_loading=wifi_loading.into()
                status_message=wifi_status_msg.into()
                initial_target_ip=wifi_target_ip.into()
                initial_serial_number=wifi_modal_sn.into()
                initial_device_id=wifi_modal_id.into()
            />
        </div>
    }
}
