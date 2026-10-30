//! Interactive Yew control panel and WebSocket client for managing basketball games and devices.

use core::net::SocketAddr;
use core::pin::pin;
use std::cell::RefCell;
use std::rc::Rc;

use basketball::{Discipline, Game, Quarter, Scorable, State as GameState};
use config::DeviceConfig;
use futures::StreamExt;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::future::{self, Either};
use net::layout::PACKET_SIZE;
use net::{
    Client, ClientEvent, ClientHandler, ClientPacket, Device, DeviceType, NetError, RawConnection,
    ServerEvent, ServerPacket, Transport,
};
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::JsValue;
use web_sys::wasm_bindgen::closure::Closure;
use web_sys::{BinaryType, MessageEvent, WebSocket};
use yew::platform;
use yew::{Callback, Html, function_component, html, use_effect_with, use_mut_ref, use_state};

use crate::scoreboard::Scoreboard;

fn invoke_command(_cmd: &'static str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")) {
                if let Ok(core_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core")) {
                    if let Ok(invoke_fn) =
                        js_sys::Reflect::get(&core_obj, &JsValue::from_str("invoke"))
                    {
                        if let Ok(func) = invoke_fn.dyn_into::<js_sys::Function>() {
                            let _ = func.call1(&core_obj, &JsValue::from_str(_cmd));
                        }
                    }
                }
            }
        }
    }
}

fn invoke_set_island(island: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    let style = if island {
                        "background: none !important; background-color: transparent !important;"
                    } else {
                        "background-color: #ffffff !important;"
                    };
                    body.set_attribute("style", style).ok();
                }
                if let Some(element) = document.document_element() {
                    let style = if island {
                        "background: none !important; background-color: transparent !important;"
                    } else {
                        "background-color: #ffffff !important;"
                    };
                    element.set_attribute("style", style).ok();
                }
            }

            if let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")) {
                if let Ok(core_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core")) {
                    if let Ok(invoke_fn) =
                        js_sys::Reflect::get(&core_obj, &JsValue::from_str("invoke"))
                    {
                        if let Ok(func) = invoke_fn.dyn_into::<js_sys::Function>() {
                            let args = js_sys::Object::new();
                            let _ = js_sys::Reflect::set(
                                &args,
                                &JsValue::from_str("island"),
                                &JsValue::from_bool(island),
                            );
                            let _ =
                                func.call2(&core_obj, &JsValue::from_str("set_island_mode"), &args);
                        }
                    }
                }
            }
        }
    }
}

fn invoke_set_transparent(transparent: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    let style = if transparent {
                        "background: none !important; background-color: transparent !important;"
                    } else {
                        "background-color: #ffffff !important;"
                    };
                    body.set_attribute("style", style).ok();
                }
                if let Some(element) = document.document_element() {
                    let style = if transparent {
                        "background: none !important; background-color: transparent !important;"
                    } else {
                        "background-color: #ffffff !important;"
                    };
                    element.set_attribute("style", style).ok();
                }
            }

            if let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")) {
                if let Ok(core_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core")) {
                    if let Ok(invoke_fn) =
                        js_sys::Reflect::get(&core_obj, &JsValue::from_str("invoke"))
                    {
                        if let Ok(func) = invoke_fn.dyn_into::<js_sys::Function>() {
                            let args = js_sys::Object::new();
                            let _ = js_sys::Reflect::set(
                                &args,
                                &JsValue::from_str("transparent"),
                                &JsValue::from_bool(transparent),
                            );
                            let _ = func.call2(
                                &core_obj,
                                &JsValue::from_str("set_transparent_mode"),
                                &args,
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Adapter bridging browser [`WebSocket`] channels to [`RawConnection`].
pub struct BrowserWebSocketAdapter {
    outgoing_tx: UnboundedSender<[u8; PACKET_SIZE]>,
    incoming_rx: UnboundedReceiver<[u8; PACKET_SIZE]>,
}

impl BrowserWebSocketAdapter {
    /// Creates a new [`BrowserWebSocketAdapter`] from outgoing and incoming packet channels.
    #[must_use]
    pub fn new(
        outgoing_tx: UnboundedSender<[u8; PACKET_SIZE]>,
        incoming_rx: UnboundedReceiver<[u8; PACKET_SIZE]>,
    ) -> Self {
        Self {
            outgoing_tx,
            incoming_rx,
        }
    }
}

impl RawConnection for BrowserWebSocketAdapter {
    async fn send(&mut self, data: &[u8; PACKET_SIZE]) -> Result<(), ()> {
        self.outgoing_tx.unbounded_send(*data).map_err(|_| ())
    }

    async fn receive(&mut self, buffer: &mut [u8; PACKET_SIZE]) -> Result<usize, ()> {
        match self.incoming_rx.next().await {
            Some(packet) => {
                *buffer = packet;
                Ok(PACKET_SIZE)
            }
            None => Err(()),
        }
    }
}

/// [`ClientHandler`] implementation that forwards received [`ServerPacket`]s to the UI task.
pub struct HmiClientHandler {
    packet_tx: UnboundedSender<ServerPacket>,
}

impl HmiClientHandler {
    /// Creates a new [`HmiClientHandler`].
    #[must_use]
    pub fn new(packet_tx: UnboundedSender<ServerPacket>) -> Self {
        Self { packet_tx }
    }
}

impl ClientHandler for HmiClientHandler {
    async fn handle_packet(
        &mut self,
        packet: ServerPacket,
        _address: Option<SocketAddr>,
    ) -> Option<ClientPacket> {
        let _ = self.packet_tx.unbounded_send(packet);

        match packet.event() {
            ServerEvent::HandshakeAck
            | ServerEvent::GameUpdate
            | ServerEvent::GameListUpdate
            | ServerEvent::DeviceUpdate
            | ServerEvent::DeviceDisconnect
            | _ => None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
const SERIAL_NUMBER_STORAGE_KEY: &str = "hmi_serial_number";

#[derive(Clone, Copy)]
struct SequencedGame {
    sequence_id: u8,
    game: Game,
}

fn load_or_create_browser_device() -> Device {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = web_sys::window().and_then(|window| window.local_storage().ok().flatten());

        if let Some(ref store) = storage
            && let Ok(Some(saved_serial_number)) = store.get_item(SERIAL_NUMBER_STORAGE_KEY)
            && let Ok(device) =
                Device::try_new(DeviceType::WebBrowser, &saved_serial_number, "hmi-browser")
        {
            return device;
        }

        let suffix = (js_sys::Math::random() * 10000.0) as u16 % 10000;
        let serial_number = format!("sn-web-{suffix:04}");

        if let Some(store) = storage {
            let _ = store.set_item(SERIAL_NUMBER_STORAGE_KEY, &serial_number);
        }

        Device::try_new(DeviceType::WebBrowser, &serial_number, "hmi-browser")
            .expect("valid browser DeviceConfig")
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Device::try_new(DeviceType::WebBrowser, "sn-web-0000", "hmi-browser")
            .expect("valid browser DeviceConfig")
    }
}

/// Root Yew application component for the HMI control panel.
#[function_component(App)]
pub fn app() -> Html {
    let status = use_state(|| "Rozłączony".to_string());
    let status_color = use_state(|| "red".to_string());
    let is_connected = use_state(|| false);
    let command_sender = use_state(|| None::<UnboundedSender<ClientPacket>>);
    let sequence_id = use_mut_ref(|| 0u8);
    let logs = use_state(Vec::<String>::new);
    let last_game = use_state(|| Game::new(Discipline::Fiba5vs5));
    let available_games = use_state(Vec::<Game>::new);
    let connected_devices = use_state(Vec::<Device>::new);
    let transparent_mode = use_state(|| false);
    let island_mode = use_state(|| false);
    let device = use_state(|| {
        Device::try_new(DeviceType::WebBrowser, "sn-web-0000", "hmi-browser")
            .expect("valid browser DeviceConfig")
    });

    let log_buffer: Rc<RefCell<Vec<String>>> = use_mut_ref(Vec::new);

    let add_log = {
        let logs = logs.clone();
        let log_buffer = log_buffer.clone();
        Callback::from(move |message: String| {
            let mut buffer = log_buffer.borrow_mut();
            buffer.push(message);
            if buffer.len() > 50 {
                buffer.remove(0);
            }
            logs.set(buffer.clone());
        })
    };

    {
        let device = device.clone();
        let status = status.clone();
        let status_color = status_color.clone();
        let is_connected = is_connected.clone();
        let command_sender = command_sender.clone();
        let last_game = last_game.clone();
        let available_games = available_games.clone();
        let connected_devices = connected_devices.clone();
        let add_log = add_log.clone();

        use_effect_with((), move |()| {
            let client_device = load_or_create_browser_device();
            device.set(client_device);

            let websocket_url = "ws://127.0.0.1:3000/ws".to_string();

            if let Ok(websocket) = WebSocket::new(&websocket_url) {
                websocket.set_binary_type(BinaryType::Arraybuffer);

                let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded::<[u8; PACKET_SIZE]>();
                let (incoming_tx, incoming_rx) = mpsc::unbounded::<[u8; PACKET_SIZE]>();
                let (packet_tx, mut packet_rx) = mpsc::unbounded::<ServerPacket>();
                let (command_tx, mut command_rx) = mpsc::unbounded::<ClientPacket>();

                // Callback onopen -> sends HandshakeRequest with DeviceConfig
                {
                    let status_clone = status.clone();
                    let color_clone = status_color.clone();
                    let connected_clone = is_connected.clone();
                    let log_clone = add_log.clone();
                    let command_tx_open = command_tx.clone();
                    let onopen = Closure::wrap(Box::new(move |_event: JsValue| {
                        status_clone.set("Połączono!".to_string());
                        color_clone.set("green".to_string());
                        connected_clone.set(true);
                        log_clone.emit("Nawiązano połączenie WebSocket.".to_string());

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

                // Callback onmessage -> forwards raw bytes to Transport
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

                // Callback onclose -> closes incoming channel, terminating Client loop
                {
                    let status_clone = status.clone();
                    let color_clone = status_color.clone();
                    let connected_clone = is_connected.clone();
                    let devices_clone = connected_devices.clone();
                    let games_clone = available_games.clone();
                    let log_clone = add_log.clone();
                    let incoming_tx_close = incoming_tx;
                    let onclose = Closure::wrap(Box::new(move |_event: JsValue| {
                        status_clone.set("Rozłączony".to_string());
                        color_clone.set("red".to_string());
                        connected_clone.set(false);
                        devices_clone.set(Vec::new());
                        games_clone.set(Vec::new());
                        incoming_tx_close.close_channel();
                        log_clone.emit("Połączenie zostało zamknięte.".to_string());
                    }) as Box<dyn FnMut(JsValue)>);
                    websocket.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                    onclose.forget();
                }

                // Task sending raw bytes over WebSocket
                {
                    let websocket_sender = websocket.clone();
                    let log_clone = add_log.clone();
                    platform::spawn_local(async move {
                        while let Some(bytes) = outgoing_rx.next().await {
                            let uint8_array = web_sys::js_sys::Uint8Array::from(&bytes[..]);
                            if let Err(error) =
                                websocket_sender.send_with_array_buffer(&uint8_array.buffer())
                            {
                                log_clone.emit(format!("Błąd wysyłania: {error:?}"));
                            }
                        }
                    });
                }

                // Task updating UI from decoded packets emitted by HmiClientHandler
                {
                    let last_game_clone = last_game.clone();
                    let games_state_clone = available_games.clone();
                    let devices_state_clone = connected_devices.clone();
                    let log_clone = add_log.clone();
                    platform::spawn_local(async move {
                        let mut previous_game: Option<Game> = None;
                        let mut device_list: Vec<Device> = Vec::new();
                        let mut game_list: Vec<Game> = Vec::new();

                        while let Some(first_packet) = packet_rx.next().await {
                            let mut latest_game_packet: Option<SequencedGame> = None;
                            let mut devices_changed = false;
                            let mut games_changed = false;

                            let mut current = Some(first_packet);
                            while let Some(packet) = current {
                                match packet.event() {
                                    ServerEvent::HandshakeAck => {
                                        if let Ok(game) = Game::try_from(packet) {
                                            latest_game_packet = Some(SequencedGame {
                                                sequence_id: packet.sequence_id(),
                                                game,
                                            });
                                            if let Some(existing) = game_list
                                                .iter_mut()
                                                .find(|entry| entry.id() == game.id())
                                            {
                                                *existing = game;
                                            } else {
                                                game_list.push(game);
                                                game_list.sort_by_key(Game::id);
                                            }
                                            games_changed = true;
                                        }
                                        log_clone.emit(format!(
                                            "[ODEBRANO] HandshakeAck (seq={})",
                                            packet.sequence_id()
                                        ));
                                    }
                                    ServerEvent::GameUpdate => {
                                        if let Ok(game) = Game::try_from(packet) {
                                            latest_game_packet = Some(SequencedGame {
                                                sequence_id: packet.sequence_id(),
                                                game,
                                            });
                                            if let Some(existing) = game_list
                                                .iter_mut()
                                                .find(|entry| entry.id() == game.id())
                                            {
                                                if existing.scores() != game.scores()
                                                    || existing.state() != game.state()
                                                    || existing.quarter() != game.quarter()
                                                {
                                                    games_changed = true;
                                                }
                                                *existing = game;
                                            } else {
                                                game_list.push(game);
                                                game_list.sort_by_key(Game::id);
                                                games_changed = true;
                                            }
                                        }
                                    }
                                    ServerEvent::GameListUpdate => {
                                        if let Ok(game) = Game::try_from(packet) {
                                            if let Some(existing) = game_list
                                                .iter_mut()
                                                .find(|entry| entry.id() == game.id())
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
                                        if let Ok(updated_device) = Device::try_from(packet) {
                                            let serial_number = updated_device
                                                .serial_number()
                                                .unwrap_or("unknown")
                                                .to_owned();
                                            let device_id = updated_device
                                                .device_id()
                                                .unwrap_or("unknown")
                                                .to_owned();
                                            if let Some(existing) =
                                                device_list.iter_mut().find(|entry| {
                                                    entry.serial_number()
                                                        == updated_device.serial_number()
                                                })
                                            {
                                                if existing.game_id() != updated_device.game_id() {
                                                    log_clone.emit(format!(
                                                        "[URZĄDZENIE~] {:?} (sn={serial_number}) -> przypisano do Gry #{}",
                                                        updated_device.device_type(),
                                                        updated_device.game_id()
                                                    ));
                                                }
                                                *existing = updated_device;
                                            } else {
                                                device_list.push(updated_device);
                                                log_clone.emit(format!(
                                                    "[URZĄDZENIE+] {:?} (id={device_id}, sn={serial_number})",
                                                    updated_device.device_type(),
                                                ));
                                            }
                                            devices_changed = true;
                                        }
                                    }
                                    ServerEvent::DeviceDisconnect => {
                                        if let Ok(disconnected_device) = Device::try_from(packet) {
                                            let serial_number = disconnected_device
                                                .serial_number()
                                                .unwrap_or("unknown")
                                                .to_owned();
                                            let device_id = disconnected_device
                                                .device_id()
                                                .unwrap_or("unknown")
                                                .to_owned();
                                            let before = device_list.len();
                                            device_list.retain(|entry| {
                                                entry.serial_number()
                                                    != disconnected_device.serial_number()
                                            });
                                            if device_list.len() != before {
                                                log_clone.emit(format!(
                                                    "[URZĄDZENIE-] {:?} (id={device_id}, sn={serial_number})",
                                                    disconnected_device.device_type(),
                                                ));
                                                devices_changed = true;
                                            }
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

                            if let Some(sequenced) = latest_game_packet {
                                let game = sequenced.game;
                                let changed = match previous_game {
                                    Some(previous) => {
                                        previous.id() != game.id()
                                            || previous.scores() != game.scores()
                                            || previous.state() != game.state()
                                            || previous.quarter() != game.quarter()
                                    }
                                    None => true,
                                };

                                if changed {
                                    let scores = game.scores();
                                    log_clone.emit(format!(
                                        "[ODEBRANO] Gra #{} (seq={}) -> Stan: {:?}, Kwarta: {:?}, Wynik: {} : {}",
                                        game.id(),
                                        sequenced.sequence_id,
                                        game.state(),
                                        game.quarter(),
                                        scores.home,
                                        scores.away
                                    ));
                                }

                                previous_game = Some(game);
                                last_game_clone.set(game);
                            }
                        }
                    });
                }

                // Main loop of the unified `net::Client`
                {
                    let adapter = BrowserWebSocketAdapter::new(outgoing_tx, incoming_rx);
                    let transport = Transport::new(adapter, None);
                    let handler = HmiClientHandler::new(packet_tx);
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
                                    Either::Left((result, _)) => Step::Polled(result),
                                    Either::Right((command, _)) => Step::Command(command),
                                }
                            };

                            match step {
                                Step::Polled(Ok(_)) => {}
                                Step::Polled(Err(NetError::InvalidPacket(error))) => {
                                    log_clone.emit(format!("[BŁĄD PAKIETU] {error}"));
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

    let send_event = {
        let command_sender = command_sender.clone();
        let sequence_id = sequence_id.clone();
        let last_game = last_game.clone();
        let add_log = add_log.clone();
        let client_device = *device;

        move |event: ClientEvent| {
            let command_sender = command_sender.clone();
            let sequence_id = sequence_id.clone();
            let last_game = last_game.clone();
            let add_log = add_log.clone();

            Callback::from(move |_| {
                if let Some(sender) = (*command_sender).as_ref() {
                    let mut seq = sequence_id.borrow_mut();
                    *seq = seq.wrapping_add(1);
                    let current_sequence_id = *seq;

                    let payload = match event {
                        ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat => {
                            client_device.to_bytes()
                        }
                        ClientEvent::GameStart
                        | ClientEvent::GameResume
                        | ClientEvent::GamePause
                        | ClientEvent::GameEnd
                        | ClientEvent::GameReset
                        | ClientEvent::GameSelect
                        | ClientEvent::GameCreate
                        | ClientEvent::GameScoreHomeFT
                        | ClientEvent::GameScoreHome2FG
                        | ClientEvent::GameScoreHome3FG
                        | ClientEvent::GameScoreAwayFT
                        | ClientEvent::GameScoreAway2FG
                        | ClientEvent::GameScoreAway3FG
                        | ClientEvent::DeviceDisconnect
                        | ClientEvent::DeviceAssignGame
                        | _ => (*last_game).to_bytes(),
                    };

                    let packet = ClientPacket::new(event, current_sequence_id, payload);
                    if sender.unbounded_send(packet).is_ok() {
                        add_log.emit(format!("[WYSŁANO] {event:?} (seq={current_sequence_id})"));
                    }
                }
            })
        }
    };

    let select_game = {
        let command_sender = command_sender.clone();
        let sequence_id = sequence_id.clone();
        let add_log = add_log.clone();

        move |game_id: u16| {
            let command_sender = command_sender.clone();
            let sequence_id = sequence_id.clone();
            let add_log = add_log.clone();

            Callback::from(move |_| {
                if let Some(sender) = (*command_sender).as_ref() {
                    let mut seq = sequence_id.borrow_mut();
                    *seq = seq.wrapping_add(1);
                    let current_sequence_id = *seq;

                    let target = Game::new_with_id(game_id, Discipline::Fiba5vs5);
                    let packet = ClientPacket::new(
                        ClientEvent::GameSelect,
                        current_sequence_id,
                        target.to_bytes(),
                    );
                    if sender.unbounded_send(packet).is_ok() {
                        add_log.emit(format!(
                            "[WYSŁANO] Wybór gry #{game_id} (seq={current_sequence_id})"
                        ));
                    }
                }
            })
        }
    };

    let disconnect_device = {
        let command_sender = command_sender.clone();
        let sequence_id = sequence_id.clone();
        let add_log = add_log.clone();

        move |target_device: Device| {
            let command_sender = command_sender.clone();
            let sequence_id = sequence_id.clone();
            let add_log = add_log.clone();

            Callback::from(move |_| {
                if let Some(sender) = (*command_sender).as_ref() {
                    let mut seq = sequence_id.borrow_mut();
                    *seq = seq.wrapping_add(1);
                    let current_sequence_id = *seq;

                    let serial_number = target_device.serial_number().unwrap_or("-").to_owned();
                    let packet = ClientPacket::new(
                        ClientEvent::DeviceDisconnect,
                        current_sequence_id,
                        target_device.to_bytes(),
                    );
                    if sender.unbounded_send(packet).is_ok() {
                        add_log.emit(format!(
                            "[WYSŁANO] Rozłącz urządzenie {serial_number} (seq={current_sequence_id})"
                        ));
                    }
                }
            })
        }
    };

    let assign_device_game = {
        let command_sender = command_sender.clone();
        let sequence_id = sequence_id.clone();
        let add_log = add_log.clone();

        move |target_device: Device, target_game_id: u16| {
            let command_sender = command_sender.clone();
            let sequence_id = sequence_id.clone();
            let add_log = add_log.clone();

            Callback::from(move |_| {
                if let Some(sender) = (*command_sender).as_ref() {
                    let mut seq = sequence_id.borrow_mut();
                    *seq = seq.wrapping_add(1);
                    let current_sequence_id = *seq;

                    let mut updated_device = target_device;
                    updated_device.set_game_id(target_game_id);
                    let serial_number = updated_device.serial_number().unwrap_or("-").to_owned();
                    let packet = ClientPacket::new(
                        ClientEvent::DeviceAssignGame,
                        current_sequence_id,
                        updated_device.to_bytes(),
                    );
                    if sender.unbounded_send(packet).is_ok() {
                        add_log.emit(format!(
                            "[WYSŁANO] Przypisz Worker {serial_number} do Gry #{target_game_id} (seq={current_sequence_id})"
                        ));
                    }
                }
            })
        }
    };

    let game = *last_game;
    let scores = game.scores();
    let regulation_time = game.regulation_time();
    let clock_time = game.clock_time();
    let quarter_label = match game.quarter() {
        Quarter::Q1 => "Q1",
        Quarter::Q2 => "Q2",
        Quarter::Q3 => "Q3",
        Quarter::Q4 => "Q4",
        Quarter::None | _ => "Brak",
    };
    let is_idle_default = scores.home == 0 && scores.away == 0 && game.state() == GameState::Idle;
    let display_home_score = if is_idle_default { 139 } else { scores.home };
    let display_away_score = if is_idle_default { 127 } else { scores.away };
    let display_time = if is_idle_default {
        "02:56".to_string()
    } else {
        format!(
            "{:02}:{:02}",
            regulation_time.minutes, regulation_time.seconds
        )
    };
    let display_quarter = if is_idle_default {
        "4Q".to_string()
    } else {
        match game.quarter() {
            Quarter::Q2 => "2Q".to_string(),
            Quarter::Q3 => "3Q".to_string(),
            Quarter::Q4 => "4Q".to_string(),
            Quarter::Q1 | Quarter::None | _ => "1Q".to_string(),
        }
    };
    let display_shot_clock = if is_idle_default {
        24
    } else {
        clock_time.seconds
    };
    let current_device = *device;
    let my_serial_number = current_device.serial_number().unwrap_or("-");
    let watchers: Vec<Device> = (*connected_devices)
        .iter()
        .copied()
        .filter(|entry| entry.game_id() == game.id())
        .collect();
    let workers: Vec<Device> = (*connected_devices)
        .iter()
        .copied()
        .filter(|entry| entry.device_type() == DeviceType::Worker)
        .collect();

    let game_buttons = (*available_games).iter().map(|entry| {
        let game_id = entry.id();
        let entry_scores = entry.scores();
        let is_active = game_id == game.id();
        let count = (*connected_devices)
            .iter()
            .filter(|connected| connected.game_id() == game_id)
            .count();
        html! {
            <button
                onclick={select_game(game_id)}
                style={format!(
                    "padding: 8px 12px; cursor: pointer; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-weight: {};",
                    if is_active { "#0f172a" } else { "#cbd5e1" },
                    if is_active { "#0f172a" } else { "#f8fafc" },
                    if is_active { "#ffffff" } else { "#0f172a" },
                    if is_active { "bold" } else { "normal" }
                )}
            >
                { format!("Gra #{game_id} ({}:{} | {:?} | urz.: {count})", entry_scores.home, entry_scores.away, entry.state()) }
            </button>
        }
    });

    let worker_items = workers.iter().map(|worker| {
        let worker_copy = *worker;
        let serial_number = worker.serial_number().unwrap_or("-");
        let device_id = worker.device_id().unwrap_or("-");
        let current_worker_game_id = worker.game_id();
        let assigned_label = if current_worker_game_id == 0 {
            "Brak (oczekuje na wybór)".to_string()
        } else {
            format!("Gra #{current_worker_game_id}")
        };
        let worker_game_buttons = (*available_games).iter().map(|entry| {
            let game_id = entry.id();
            let entry_scores = entry.scores();
            let is_assigned = game_id == current_worker_game_id;
            html! {
                <button
                    onclick={assign_device_game(worker_copy, game_id)}
                    style={format!(
                        "padding: 4px 10px; font-size: 12px; cursor: pointer; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-weight: {};",
                        if is_assigned { "#0284c7" } else { "#cbd5e1" },
                        if is_assigned { "#0284c7" } else { "#ffffff" },
                        if is_assigned { "#ffffff" } else { "#0f172a" },
                        if is_assigned { "bold" } else { "normal" }
                    )}
                >
                    { format!("Gra #{game_id} ({}:{})", entry_scores.home, entry_scores.away) }
                </button>
            }
        });

        html! {
            <li style="display: flex; flex-direction: column; gap: 8px; padding: 10px 12px; background: #f8fafc; border: 1px solid #cbd5e1; border-radius: 6px;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-weight: 600; color: #0f172a;">
                        { format!("Worker — ID: {device_id} | SN: {serial_number} | Wyświetla: {assigned_label}") }
                    </span>
                    <button
                        onclick={disconnect_device(worker_copy)}
                        style="padding: 4px 10px; font-size: 12px; cursor: pointer; background: #e11d48; color: white; border: none; border-radius: 4px;"
                    >
                        { "Rozłącz" }
                    </button>
                </div>
                <div style="display: flex; align-items: center; gap: 6px; flex-wrap: wrap;">
                    <span style="font-size: 12px; color: #475569;">{ "Wybierz grę:" }</span>
                    <button
                        onclick={assign_device_game(worker_copy, 0)}
                        style={format!(
                            "padding: 4px 10px; font-size: 12px; cursor: pointer; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-weight: {};",
                            if current_worker_game_id == 0 { "#64748b" } else { "#cbd5e1" },
                            if current_worker_game_id == 0 { "#64748b" } else { "#ffffff" },
                            if current_worker_game_id == 0 { "#ffffff" } else { "#475569" },
                            if current_worker_game_id == 0 { "bold" } else { "normal" }
                        )}
                    >
                        { "Brak (wygaszony)" }
                    </button>
                    { for worker_game_buttons }
                </div>
            </li>
        }
    });

    let watcher_items = watchers.iter().map(|watcher| {
        let device_copy = *watcher;
        let serial_number = watcher.serial_number().unwrap_or("-");
        let device_id = watcher.device_id().unwrap_or("-");
        let is_self = serial_number == my_serial_number;
        html! {
            <li style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 4px;">
                <span style={if is_self { "font-weight: bold; color: #0f172a;" } else { "color: #334155;" }}>
                    { format!("{:?} — ID: {device_id} | SN: {serial_number} | Gra #{}{}", watcher.device_type(), watcher.game_id(), if is_self { " (Ty)" } else { "" }) }
                </span>
                <button
                    onclick={disconnect_device(device_copy)}
                    style="padding: 4px 10px; font-size: 12px; cursor: pointer; background: #e11d48; color: white; border: none; border-radius: 4px;"
                >
                    { "Rozłącz" }
                </button>
            </li>
        }
    });

    let log_items = (*logs).iter().map(|log| html! { <div>{ log }</div> });

    let toggle_transparent = {
        let transparent_mode = transparent_mode.clone();
        Callback::from(move |_| {
            let next = !*transparent_mode;
            transparent_mode.set(next);
            invoke_set_transparent(next);
        })
    };

    let toggle_island = {
        let island_mode = island_mode.clone();
        Callback::from(move |_| {
            let next = !*island_mode;
            island_mode.set(next);
            invoke_set_island(next);
        })
    };

    if *island_mode {
        return html! {
            <div
                oncontextmenu={Callback::from(|e: web_sys::MouseEvent| e.prevent_default())}
                data-tauri-drag-region=""
                class="w-screen h-screen bg-transparent flex flex-col justify-center items-center p-2 box-border overflow-hidden select-none cursor-grab active:cursor-grabbing relative group"
            >
                <div class="absolute top-1 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-50">
                    <button
                        onclick={toggle_island.clone()}
                        class="px-2 py-0.5 text-[11px] font-bold bg-slate-900/80 hover:bg-slate-800 text-slate-300 rounded cursor-pointer border border-white/10"
                        title="Przywróć pełny panel"
                    >
                        { "✕" }
                    </button>
                </div>
                <div data-tauri-drag-region="" class="pointer-events-auto">
                    <Scoreboard
                        home_team={"HOME"}
                        away_team={"AWAY"}
                        home_score={display_home_score}
                        away_score={display_away_score}
                        time={display_time}
                        shot_clock={display_shot_clock}
                        quarter={display_quarter}
                        home_timeouts={2}
                        away_timeouts={4}
                        home_timeout_dashes={5}
                        away_timeout_dashes={5}
                    />
                </div>
            </div>
        };
    }

    if *transparent_mode {
        return html! {
            <div
                oncontextmenu={Callback::from(|e: web_sys::MouseEvent| e.prevent_default())}
                class="w-screen h-screen bg-transparent flex flex-col justify-end items-center pb-6 box-border overflow-hidden select-none"
            >
                <div class="absolute top-3 right-3 z-[9999]">
                    <button
                        onclick={toggle_transparent.clone()}
                        class="px-3 py-1.5 text-xs font-bold bg-slate-800/85 hover:bg-slate-700 text-slate-200 border border-white/20 rounded-md cursor-pointer backdrop-blur-md shadow-lg"
                    >
                        { "✕ Zamknij nakładkę" }
                    </button>
                </div>
                <div data-tauri-drag-region="" class="cursor-grab active:cursor-grabbing">
                    <Scoreboard
                        home_team={"HOME"}
                        away_team={"AWAY"}
                        home_score={display_home_score}
                        away_score={display_away_score}
                        time={display_time}
                        shot_clock={display_shot_clock}
                        quarter={display_quarter}
                        home_timeouts={2}
                        away_timeouts={4}
                        home_timeout_dashes={5}
                        away_timeout_dashes={5}
                    />
                </div>
            </div>
        };
    }

    html! {
        <div
            oncontextmenu={Callback::from(|e: web_sys::MouseEvent| e.prevent_default())}
            class="font-display bg-white min-h-screen p-4 text-slate-800"
        >
            <div class="flex justify-end items-center gap-2 pb-2">
                <button
                    onclick={Callback::from(|_| invoke_command("window_toggle_fullscreen"))}
                    class="px-3.5 py-1.5 text-xs font-semibold cursor-pointer bg-slate-700 hover:bg-slate-600 text-white rounded-md shadow-sm flex items-center gap-1.5 transition-colors"
                    title="Pełny ekran (F11)"
                >
                    <span>{ "⛶" }</span>
                    { "Pełny ekran" }
                </button>
                <button
                    onclick={toggle_island.clone()}
                    class="px-3.5 py-1.5 text-xs font-semibold cursor-pointer bg-emerald-600 hover:bg-emerald-500 text-white rounded-md shadow-sm flex items-center gap-1.5 transition-colors"
                    title="Pływająca wyspa ze scorebaordem"
                >
                    <span>{ "🏝️" }</span>
                    { "Tryb wyspy (Scoreboard)" }
                </button>
                <button
                    onclick={toggle_transparent.clone()}
                    class="px-3.5 py-1.5 text-xs font-semibold cursor-pointer bg-sky-600 hover:bg-sky-500 text-white rounded-md shadow-sm flex items-center gap-1.5 transition-colors"
                >
                    <span>{ "📺" }</span>
                    { "Tryb przezroczysty (Scoreboard na dole)" }
                </button>
            </div>
            <Scoreboard
                home_team={"HOME"}
                away_team={"AWAY"}
                home_score={display_home_score}
                away_score={display_away_score}
                time={display_time}
                shot_clock={display_shot_clock}
                quarter={display_quarter}
                home_timeouts={2}
                away_timeouts={4}
                home_timeout_dashes={5}
                away_timeout_dashes={5}
            />
            <div style="border: 1px solid #ccc; padding: 15px; margin-bottom: 10px; border-radius: 5px;">
                <h3>{ "Status połączenia: " }<span style={format!("color: {};", *status_color)}>{ &*status }</span></h3>
                <p style="font-size: 13px; color: #475569; margin: 4px 0;">
                    { format!(
                        "To urządzenie: {:?} | ID: {} | SN: {my_serial_number}",
                        current_device.device_type(),
                        current_device.device_id().unwrap_or("-"),
                    ) }
                </p>

                <p style="font-size: 12px; color: #64748b; margin: 6px 0 12px 0;">
                    { format!(
                        "Gra #{} | Stan: {:?} | Kwarta: {quarter_label} | Czas: {:02}:{:02}.{:02} (24s: {:02}.{:02}) | Wynik: {} : {}",
                        game.id(),
                        game.state(),
                        regulation_time.minutes,
                        regulation_time.seconds,
                        regulation_time.centiseconds,
                        clock_time.seconds,
                        clock_time.centiseconds,
                        scores.home,
                        scores.away
                    ) }
                </p>
                <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                    <button disabled={!*is_connected} onclick={send_event(ClientEvent::HandshakeRequest)} style="padding: 10px 15px; cursor: pointer;">{ "Wyślij pakiet (Handshake)" }</button>
                    <button disabled={!*is_connected} onclick={send_event(ClientEvent::GameStart)} style="padding: 10px 15px; cursor: pointer;">{ "Start" }</button>
                    <button disabled={!*is_connected} onclick={send_event(ClientEvent::GamePause)} style="padding: 10px 15px; cursor: pointer;">{ "Pauza" }</button>
                    <button disabled={!*is_connected} onclick={send_event(ClientEvent::GameReset)} style="padding: 10px 15px; cursor: pointer;">{ "Reset" }</button>
                    <button disabled={!*is_connected} onclick={send_event(ClientEvent::GameScoreHome2FG)} style="padding: 10px 15px; cursor: pointer;">{ "Home +2" }</button>
                    <button disabled={!*is_connected} onclick={send_event(ClientEvent::GameScoreAway2FG)} style="padding: 10px 15px; cursor: pointer;">{ "Away +2" }</button>
                </div>
            </div>

            <div style="border: 1px solid #ccc; padding: 15px; margin-bottom: 10px; border-radius: 5px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                    <h3 style="margin: 0;">{ format!("Dostępne gry w bazie ({}):", (*available_games).len()) }</h3>
                    <button
                        disabled={!*is_connected}
                        onclick={send_event(ClientEvent::GameCreate)}
                        style="padding: 6px 12px; cursor: pointer; background: #0f172a; color: white; border: none; border-radius: 4px;"
                    >
                        { "+ Nowa gra" }
                    </button>
                </div>
                if (*available_games).is_empty() {
                    <p style="font-size: 13px; color: #64748b; margin: 0;">{ "Połącz się z serwerem, aby pobrać listę gier" }</p>
                } else {
                    <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                        { for game_buttons }
                    </div>
                }
            </div>

            <div style="border: 1px solid #ccc; padding: 15px; margin-bottom: 10px; border-radius: 5px;">
                <h3>{ format!("Workery w sieci — wybór wyświetlanej gry ({}):", workers.len()) }</h3>
                if workers.is_empty() {
                    <p style="font-size: 13px; color: #64748b; margin: 0;">{ "Brak podłączonych Workerów (sprzętowych ani symulowanych)" }</p>
                } else {
                    <ul style="margin: 0; padding-left: 0; list-style: none; font-size: 14px; display: flex; flex-direction: column; gap: 8px;">
                        { for worker_items }
                    </ul>
                }
            </div>

            <div style="border: 1px solid #ccc; padding: 15px; margin-bottom: 10px; border-radius: 5px;">
                <h3>{ format!("Urządzenia podglądające Grę #{} ({}):", game.id(), watchers.len()) }</h3>
                if watchers.is_empty() {
                    <p style="font-size: 13px; color: #64748b; margin: 0;">{ "Brak urządzeń podglądających tę grę" }</p>
                } else {
                    <ul style="margin: 0; padding-left: 0; list-style: none; font-size: 14px; display: flex; flex-direction: column; gap: 6px;">
                        { for watcher_items }
                    </ul>
                }
            </div>

            <div style="border: 1px solid #ccc; padding: 15px; border-radius: 5px;">
                <h3>{ "Logi konsoli:" }</h3>
                <pre style="background: #f4f4f4; padding: 10px; border-radius: 4px; max-height: 200px; overflow-y: auto;">
                    { for log_items }
                </pre>
            </div>
        </div>
    }
}
