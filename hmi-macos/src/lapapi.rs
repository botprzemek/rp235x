//! Native macOS translucent UI for Lapapi with a functional WebSocket connection,
//! tree-view sidebar (Games and Devices), interactive basketball control center, and inspector.

use core::pin::pin;
use std::cell::RefCell;
use std::rc::Rc;

use basketball::{Discipline, Game};
use config::DeviceConfig;
use futures::StreamExt;
use futures::channel::mpsc::{self, UnboundedSender};
use futures::future::{self, Either};
use net::layout::PACKET_SIZE;
use net::{
    Client, ClientEvent, ClientPacket, Device, DeviceType, NetError, ServerEvent, ServerPacket,
    Transport,
};
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::{BinaryType, MessageEvent, WebSocket};
use yew::platform;
use yew::{
    Callback, Html, MouseEvent, function_component, html, use_effect_with, use_mut_ref, use_state,
};

use crate::components::lapapi::ws::{
    BrowserWebSocketAdapter, LapapiClientHandler, load_or_create_browser_device,
};
use crate::components::lapapi::{
    LapapiCenterView, LapapiInspector, LapapiSidebar, WifiConfigModal,
};

fn invoke_tauri_window_command(_cmd: &'static str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
            && let Ok(core_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core"))
            && let Ok(invoke_fn) = js_sys::Reflect::get(&core_obj, &JsValue::from_str("invoke"))
            && let Ok(func) = invoke_fn.dyn_into::<js_sys::Function>()
        {
            let _ = func.call1(&core_obj, &JsValue::from_str(_cmd));
        }
    }
}

/// Root Lapapi macOS component.
#[function_component(App)]
pub fn app() -> Html {
    let is_focused = use_state(|| true);
    let is_lights_hovered = use_state(|| false);

    // WebSocket state
    let is_connected = use_state(|| false);
    let command_sender = use_state(|| None::<UnboundedSender<ClientPacket>>);
    let sequence_id = use_mut_ref(|| 0u8);
    let logs = use_state(Vec::<String>::new);
    let log_buffer: Rc<RefCell<Vec<String>>> = use_mut_ref(Vec::new);

    // Domain data
    let current_game = use_state(|| Game::new(Discipline::Fiba5vs5));
    let available_games = use_state(Vec::<Game>::new);
    let connected_devices = use_state(Vec::<Device>::new);
    let selected_device_sn = use_state(|| None::<String>);
    let my_device = use_state(|| {
        Device::try_new(DeviceType::WebBrowser, "sn-lap-0000", "lapapi-app")
            .expect("valid browser DeviceConfig")
    });

    // Wi-Fi Config Modal state
    let is_wifi_modal_open = use_state(|| false);
    let wifi_target_ip = use_state(|| "192.168.4.1".to_string());
    let wifi_modal_sn = use_state(|| "sn-lap-0000".to_string());
    let wifi_modal_id = use_state(|| "lapapi-worker".to_string());
    let wifi_loading = use_state(|| false);
    let wifi_status_msg = use_state(|| None::<String>);

    let add_log = {
        let logs = logs.clone();
        let log_buffer = log_buffer.clone();
        Callback::from(move |message: String| {
            let mut buffer = log_buffer.borrow_mut();
            buffer.push(message);
            if buffer.len() > 60 {
                buffer.remove(0);
            }
            logs.set(buffer.clone());
        })
    };

    // Tauri & Browser window focus events
    {
        let is_focused = is_focused.clone();
        use_effect_with((), move |_| {
            #[cfg(target_arch = "wasm32")]
            let mut cleanup: Vec<Box<dyn FnOnce()>> = Vec::new();

            #[cfg(target_arch = "wasm32")]
            if let Some(window) = web_sys::window() {
                let on_focus = {
                    let is_focused = is_focused.clone();
                    Closure::<dyn FnMut()>::new(move || {
                        is_focused.set(true);
                    })
                };
                let on_blur = {
                    let is_focused = is_focused.clone();
                    Closure::<dyn FnMut()>::new(move || {
                        is_focused.set(false);
                    })
                };

                let _ = window
                    .add_event_listener_with_callback("focus", on_focus.as_ref().unchecked_ref());
                let _ = window
                    .add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref());

                if let Ok(tauri_obj) =
                    js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
                    && let Ok(event_obj) =
                        js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("event"))
                    && let Ok(listen_fn) =
                        js_sys::Reflect::get(&event_obj, &JsValue::from_str("listen"))
                    && let Ok(func) = listen_fn.dyn_into::<js_sys::Function>()
                {
                    let on_tauri_focus = {
                        let is_focused = is_focused.clone();
                        Closure::<dyn FnMut(JsValue)>::new(move |evt: JsValue| {
                            if let Ok(payload) =
                                js_sys::Reflect::get(&evt, &JsValue::from_str("payload"))
                                && let Some(focused) = payload.as_bool()
                            {
                                is_focused.set(focused);
                            }
                        })
                    };
                    let _ = func.call2(
                        &event_obj,
                        &JsValue::from_str("lapapi-focus-changed"),
                        on_tauri_focus.as_ref().unchecked_ref(),
                    );
                    on_tauri_focus.forget();
                }

                let win_clone = window.clone();
                cleanup.push(Box::new(move || {
                    let _ = win_clone.remove_event_listener_with_callback(
                        "focus",
                        on_focus.as_ref().unchecked_ref(),
                    );
                    let _ = win_clone.remove_event_listener_with_callback(
                        "blur",
                        on_blur.as_ref().unchecked_ref(),
                    );
                }));
            }

            move || {
                #[cfg(target_arch = "wasm32")]
                for f in cleanup {
                    f();
                }
            }
        });
    }

    // WebSocket connection lifecycle
    {
        let my_device = my_device.clone();
        let is_connected = is_connected.clone();
        let command_sender = command_sender.clone();
        let current_game = current_game.clone();
        let available_games = available_games.clone();
        let connected_devices = connected_devices.clone();
        let add_log = add_log.clone();

        use_effect_with((), move |()| {
            let client_device = load_or_create_browser_device();
            my_device.set(client_device);

            let websocket_url = "ws://192.168.0.106:3000/ws".to_string();

            if let Ok(websocket) = WebSocket::new(&websocket_url) {
                websocket.set_binary_type(BinaryType::Arraybuffer);

                let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded::<[u8; PACKET_SIZE]>();
                let (incoming_tx, incoming_rx) = mpsc::unbounded::<[u8; PACKET_SIZE]>();
                let (packet_tx, mut packet_rx) = mpsc::unbounded::<ServerPacket>();
                let (command_tx, mut command_rx) = mpsc::unbounded::<ClientPacket>();

                // On open
                {
                    let connected_clone = is_connected.clone();
                    let log_clone = add_log.clone();
                    let command_tx_open = command_tx.clone();
                    let onopen = Closure::wrap(Box::new(move |_event: JsValue| {
                        connected_clone.set(true);
                        log_clone.emit("Połączono z serwerem WebSocket.".to_string());

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
                        if let Ok(buffer) = event.data().dyn_into::<web_sys::js_sys::ArrayBuffer>()
                        {
                            let array = web_sys::js_sys::Uint8Array::new(&buffer);
                            if array.length() as usize == PACKET_SIZE {
                                let mut data_array = [0u8; PACKET_SIZE];
                                array.copy_to(&mut data_array);
                                let _ = incoming_tx_message.unbounded_send(data_array);
                            }
                        }
                    })
                        as Box<dyn FnMut(MessageEvent)>);
                    websocket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                    onmessage.forget();
                }

                // On close
                {
                    let connected_clone = is_connected.clone();
                    let devices_clone = connected_devices.clone();
                    let games_clone = available_games.clone();
                    let log_clone = add_log.clone();
                    let incoming_tx_close = incoming_tx;
                    let onclose = Closure::wrap(Box::new(move |_event: JsValue| {
                        connected_clone.set(false);
                        devices_clone.set(Vec::new());
                        games_clone.set(Vec::new());
                        incoming_tx_close.close_channel();
                        log_clone.emit("Połączenie WebSocket zamknięte.".to_string());
                    }) as Box<dyn FnMut(JsValue)>);
                    websocket.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                    onclose.forget();
                }

                // Outgoing WebSocket sender task
                {
                    let websocket_sender = websocket.clone();
                    let log_clone = add_log.clone();
                    platform::spawn_local(async move {
                        while let Some(bytes) = outgoing_rx.next().await {
                            let uint8_array = web_sys::js_sys::Uint8Array::from(&bytes[..]);
                            if let Err(err) =
                                websocket_sender.send_with_array_buffer(&uint8_array.buffer())
                            {
                                log_clone.emit(format!("Błąd nadawania WebSocket: {err:?}"));
                            }
                        }
                    });
                }

                // Incoming server packet handler task
                {
                    let current_game_clone = current_game.clone();
                    let games_state_clone = available_games.clone();
                    let devices_state_clone = connected_devices.clone();
                    let log_clone = add_log.clone();

                    platform::spawn_local(async move {
                        let mut device_list: Vec<Device> = Vec::new();
                        let mut game_list: Vec<Game> = Vec::new();

                        while let Some(first_packet) = packet_rx.next().await {
                            let mut current = Some(first_packet);
                            let mut devices_changed = false;
                            let mut games_changed = false;

                            while let Some(packet) = current {
                                match packet.event() {
                                    ServerEvent::HandshakeAck | ServerEvent::GameUpdate => {
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

                                            // If current selected game updated, sync it
                                            if (*current_game_clone).id() == game.id()
                                                || (*current_game_clone).id() == 0
                                            {
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
                                            if let Some(existing) =
                                                device_list.iter_mut().find(|d| {
                                                    d.serial_number() == updated_dev.serial_number()
                                                })
                                            {
                                                *existing = updated_dev;
                                            } else {
                                                device_list.push(updated_dev);
                                            }
                                            devices_changed = true;
                                            log_clone.emit(format!(
                                                "[URZĄDZENIE] {:?} (sn={}) -> Gra #{}",
                                                updated_dev.device_type(),
                                                updated_dev.serial_number().unwrap_or("-"),
                                                updated_dev.game_id()
                                            ));
                                        }
                                    }
                                    ServerEvent::DeviceDisconnect => {
                                        if let Ok(dc_dev) = Device::try_from(packet) {
                                            device_list.retain(|d| {
                                                d.serial_number() != dc_dev.serial_number()
                                            });
                                            devices_changed = true;
                                            log_clone.emit(format!(
                                                "[ROZŁĄCZONO] {:?}",
                                                dc_dev.serial_number().unwrap_or("-")
                                            ));
                                        }
                                    }
                                    _ => {}
                                }

                                current = packet_rx.try_recv().ok();
                            }

                            if devices_changed {
                                devices_state_clone.set(device_list.clone());
                            }
                            if games_changed {
                                games_state_clone.set(game_list.clone());
                            }
                        }
                    });
                }

                // Client loop
                {
                    let adapter = BrowserWebSocketAdapter::new(outgoing_tx, incoming_rx);
                    let transport = Transport::new(adapter, None);
                    let handler = LapapiClientHandler::new(packet_tx);
                    let mut client = Client::new(transport, handler);
                    let log_clone = add_log.clone();

                    platform::spawn_local(async move {
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
                                    log_clone.emit(format!("[BŁĄD PAKIETU] {err}"));
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
            }

            || ()
        });
    }

    // Packet dispatch helper
    let send_client_packet = {
        let command_sender = command_sender.clone();
        let sequence_id = sequence_id.clone();
        let add_log = add_log.clone();

        move |event: ClientEvent, payload: [u8; 59]| {
            if let Some(sender) = (*command_sender).as_ref() {
                let mut seq = sequence_id.borrow_mut();
                *seq = seq.wrapping_add(1);
                let current_seq = *seq;

                let packet = ClientPacket::new(event, current_seq, payload);
                if sender.unbounded_send(packet).is_ok() {
                    add_log.emit(format!("[WYSŁANO] {event:?} (seq={current_seq})"));
                }
            }
        }
    };

    let on_send_event = {
        let current_game = current_game.clone();
        let send_client_packet = send_client_packet.clone();

        Callback::from(move |event: ClientEvent| {
            send_client_packet(event, (*current_game).to_bytes());
        })
    };

    let on_select_game = {
        let available_games = available_games.clone();
        let current_game = current_game.clone();
        let send_client_packet = send_client_packet.clone();

        Callback::from(move |game_id: u16| {
            if let Some(target) = (*available_games).iter().find(|g| g.id() == game_id) {
                current_game.set(*target);
            }
            let target_game = Game::new_with_id(game_id, Discipline::Fiba5vs5);
            send_client_packet(ClientEvent::GameSelect, target_game.to_bytes());
        })
    };

    let on_create_game = {
        let current_game = current_game.clone();
        let send_client_packet = send_client_packet.clone();

        Callback::from(move |()| {
            send_client_packet(ClientEvent::GameCreate, (*current_game).to_bytes());
        })
    };

    let on_disconnect_device = {
        let send_client_packet = send_client_packet.clone();

        Callback::from(move |target: Device| {
            send_client_packet(ClientEvent::DeviceDisconnect, target.to_bytes());
        })
    };

    let on_assign_device_game = {
        let send_client_packet = send_client_packet.clone();

        Callback::from(move |(mut target, game_id): (Device, u16)| {
            target.set_game_id(game_id);
            send_client_packet(ClientEvent::DeviceAssignGame, target.to_bytes());
        })
    };

    let on_select_device = {
        let selected_device_sn = selected_device_sn.clone();
        Callback::from(move |sn: String| {
            selected_device_sn.set(Some(sn));
        })
    };

    // Window controls
    let on_start_drag = Callback::from(|event: MouseEvent| {
        if event.buttons() == 1 {
            invoke_tauri_window_command("window_start_drag");
        }
    });

    let stop_propagation = Callback::from(|event: MouseEvent| {
        event.stop_propagation();
    });

    let on_close = Callback::from(|event: MouseEvent| {
        event.stop_propagation();
        invoke_tauri_window_command("window_close");
    });

    let on_minimize = Callback::from(|event: MouseEvent| {
        event.stop_propagation();
        invoke_tauri_window_command("window_minimize");
    });

    let on_maximize = Callback::from(|event: MouseEvent| {
        event.stop_propagation();
        invoke_tauri_window_command("window_toggle_maximize");
    });

    let on_lights_enter = {
        let is_lights_hovered = is_lights_hovered.clone();
        Callback::from(move |_| is_lights_hovered.set(true))
    };

    let on_lights_leave = {
        let is_lights_hovered = is_lights_hovered.clone();
        Callback::from(move |_| is_lights_hovered.set(false))
    };

    let on_open_wifi_setup = {
        let is_wifi_modal_open = is_wifi_modal_open.clone();
        let wifi_status_msg = wifi_status_msg.clone();
        let wifi_loading = wifi_loading.clone();
        let wifi_target_ip = wifi_target_ip.clone();
        let add_log = add_log.clone();
        Callback::from(move |()| {
            add_log.emit("[UI] Otwieranie konfiguracji Wi-Fi".to_string());
            wifi_target_ip.set("192.168.4.1".to_string());
            wifi_status_msg.set(None);
            wifi_loading.set(false);
            is_wifi_modal_open.set(true);
        })
    };

    let on_open_wifi_for_device = {
        let is_wifi_modal_open = is_wifi_modal_open.clone();
        let wifi_status_msg = wifi_status_msg.clone();
        let wifi_loading = wifi_loading.clone();
        let wifi_target_ip = wifi_target_ip.clone();
        let wifi_modal_sn = wifi_modal_sn.clone();
        let wifi_modal_id = wifi_modal_id.clone();
        let add_log = add_log.clone();
        Callback::from(move |device: Device| {
            let sn = device.serial_number().unwrap_or("sn-lap-0000").to_string();
            let id = device.device_id().unwrap_or("lapapi-worker").to_string();
            add_log.emit(format!("[UI] Otwieranie konfiguracji Wi-Fi dla urządzenia: {}", sn));
            wifi_modal_sn.set(sn);
            wifi_modal_id.set(id);
            wifi_target_ip.set("192.168.4.1".to_string());
            wifi_status_msg.set(None);
            wifi_loading.set(false);
            is_wifi_modal_open.set(true);
        })
    };

    let on_close_wifi_modal = {
        let is_wifi_modal_open = is_wifi_modal_open.clone();
        Callback::from(move |_: MouseEvent| {
            is_wifi_modal_open.set(false);
        })
    };

    let on_submit_wifi_config = {
        let wifi_loading = wifi_loading.clone();
        let wifi_status_msg = wifi_status_msg.clone();
        let add_log = add_log.clone();

        Callback::from(move |(ip, sn, id, ssid, pass): (String, String, String, String, String)| {
            wifi_loading.set(true);
            wifi_status_msg.set(Some("Wysyłanie pakietu DeviceConfigSet przez UDP...".to_string()));

            #[cfg(target_arch = "wasm32")]
            {
                let wifi_loading = wifi_loading.clone();
                let wifi_status_msg = wifi_status_msg.clone();
                let add_log = add_log.clone();

                platform::spawn_local(async move {
                    if let Some(window) = web_sys::window()
                        && let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
                        && let Ok(core_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core"))
                        && let Ok(invoke_fn) = js_sys::Reflect::get(&core_obj, &JsValue::from_str("invoke"))
                        && let Ok(func) = invoke_fn.dyn_into::<js_sys::Function>()
                    {
                        let args = js_sys::Object::new();
                        let _ = js_sys::Reflect::set(&args, &JsValue::from_str("targetIp"), &JsValue::from_str(&ip));
                        let _ = js_sys::Reflect::set(&args, &JsValue::from_str("serialNumber"), &JsValue::from_str(&sn));
                        let _ = js_sys::Reflect::set(&args, &JsValue::from_str("deviceId"), &JsValue::from_str(&id));
                        let _ = js_sys::Reflect::set(&args, &JsValue::from_str("wifiSsid"), &JsValue::from_str(&ssid));
                        let _ = js_sys::Reflect::set(&args, &JsValue::from_str("wifiPassword"), &JsValue::from_str(&pass));

                        let promise = match func.call2(&core_obj, &JsValue::from_str("send_device_config"), &args) {
                            Ok(val) => val.dyn_into::<js_sys::Promise>().ok(),
                            Err(_) => None,
                        };

                        if let Some(p) = promise {
                            let fut = wasm_bindgen_futures::JsFuture::from(p);
                            match fut.await {
                                Ok(res) => {
                                    let msg = res.as_string().unwrap_or_else(|| "Sukces!".to_string());
                                    wifi_status_msg.set(Some(msg.clone()));
                                    add_log.emit(format!("[UDP SETUP] {}", msg));
                                }
                                Err(err) => {
                                    let err_msg = err.as_string().unwrap_or_else(|| "Błąd połączenia UDP z urządzeniem".to_string());
                                    wifi_status_msg.set(Some(format!("Błąd: {}", err_msg)));
                                    add_log.emit(format!("[UDP ERROR] {}", err_msg));
                                }
                            }
                        }
                    }
                    wifi_loading.set(false);
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                wifi_loading.set(false);
                wifi_status_msg.set(Some("Dostępne w aplikacji desktopowej Tauri".to_string()));
            }
        })
    };

    // Styling based on focus state
    let root_bg_class = if *is_focused {
        "bg-[#0e1118]/50 border-white/[0.12]"
    } else {
        "bg-[#252a38]/60 border-white/[0.08]"
    };

    let sidebar_bg_class = if *is_focused {
        "bg-[#131722]/40"
    } else {
        "bg-[#282e3d]/45"
    };

    let main_bg_class = if *is_focused {
        "bg-[#0b0e15]/25"
    } else {
        "bg-[#222736]/35"
    };

    let inspector_bg_class = if *is_focused {
        "bg-[#131722]/35"
    } else {
        "bg-[#282e3d]/40"
    };

    let selected_device = (*selected_device_sn).as_ref().and_then(|sn| {
        (*connected_devices)
            .iter()
            .find(|d| d.serial_number().ok() == Some(sn.as_str()))
            .copied()
    });

    html! {
        <div class={format!("w-screen h-screen overflow-hidden rounded-xl border {root_bg_class} transition-colors duration-200 flex text-gray-200 select-none font-sans antialiased")}>
            // Tree-based Sidebar (Games and Devices)
            <LapapiSidebar
                is_focused={*is_focused}
                is_lights_hovered={*is_lights_hovered}
                on_start_drag={on_start_drag.clone()}
                stop_propagation={stop_propagation.clone()}
                on_lights_enter={on_lights_enter}
                on_lights_leave={on_lights_leave}
                on_close={on_close}
                on_minimize={on_minimize}
                on_maximize={on_maximize}
                games={(*available_games).clone()}
                selected_game_id={(*current_game).id()}
                on_select_game={on_select_game}
                on_create_game={on_create_game}
                devices={(*connected_devices).clone()}
                selected_device_sn={(*selected_device_sn).clone()}
                on_select_device={on_select_device}
                is_connected={*is_connected}
                sidebar_bg_class={sidebar_bg_class}
                on_open_wifi_setup={on_open_wifi_setup}
            />

            // Center Scoreboard and WebSocket Control View
            <LapapiCenterView
                game={*current_game}
                is_connected={*is_connected}
                on_start_drag={on_start_drag.clone()}
                stop_propagation={stop_propagation}
                on_send_event={on_send_event}
                main_bg_class={main_bg_class}
                devices={(*connected_devices).clone()}
                on_disconnect_device={on_disconnect_device.clone()}
                on_assign_device_game={on_assign_device_game}
            />

            // Right Inspector & Logs Panel
            <LapapiInspector
                selected_game={*current_game}
                selected_device={selected_device}
                logs={(*logs).clone()}
                on_start_drag={on_start_drag}
                inspector_bg_class={inspector_bg_class}
                on_disconnect_device={on_disconnect_device}
                on_open_wifi_config={on_open_wifi_for_device}
            />

            // Wi-Fi Config Modal (OTA UDP)
            <WifiConfigModal
                is_open={*is_wifi_modal_open}
                on_close={on_close_wifi_modal}
                on_submit={on_submit_wifi_config}
                is_loading={*wifi_loading}
                status_message={(*wifi_status_msg).clone()}
                initial_target_ip={(*wifi_target_ip).clone()}
                initial_serial_number={(*wifi_modal_sn).clone()}
                initial_device_id={(*wifi_modal_id).clone()}
            />
        </div>
    }
}
