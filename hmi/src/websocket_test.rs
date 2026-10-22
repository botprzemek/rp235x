use core::net::SocketAddr;
use std::cell::RefCell;
use std::rc::Rc;

use basketball::{Discipline, Game, Quarter, Scorable};
use config::DeviceConfig;
use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures::future::{Either, select};
use net::layout::PACKET_SIZE;
use net::{
    Client, ClientEvent, ClientHandler, ClientPacket, Device, DeviceType, NetError, RawConnection,
    ServerEvent, ServerPacket, Transport,
};
use web_sys::js_sys;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::prelude::*;
use web_sys::{BinaryType, MessageEvent, WebSocket};
use yew::platform::spawn_local;
use yew::prelude::*;

pub struct BrowserWebSocketAdapter {
    outgoing_tx: UnboundedSender<[u8; PACKET_SIZE]>,
    incoming_rx: UnboundedReceiver<[u8; PACKET_SIZE]>,
}

impl BrowserWebSocketAdapter {
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

pub struct HmiClientHandler {
    packet_tx: UnboundedSender<ServerPacket>,
}

impl HmiClientHandler {
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
            | ServerEvent::DeviceDisconnect => None,
        }
    }
}

const SERIAL_NUMBER_STORAGE_KEY: &str = "hmi_serial_number";

fn load_or_create_browser_device() -> Device {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());

        if let Some(ref store) = storage
            && let Ok(Some(saved_sn)) = store.get_item(SERIAL_NUMBER_STORAGE_KEY)
            && let Ok(device) = Device::new(DeviceType::WebBrowser, &saved_sn, "hmi-browser")
        {
            return device;
        }

        let suffix = (js_sys::Math::random() * 10000.0) as u16 % 10000;
        let sn = format!("sn-web-{:04}", suffix);

        if let Some(store) = storage {
            let _ = store.set_item(SERIAL_NUMBER_STORAGE_KEY, &sn);
        }

        Device::new(DeviceType::WebBrowser, &sn, "hmi-browser").expect("valid browser DeviceConfig")
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Device::new(DeviceType::WebBrowser, "sn-web-0000", "hmi-browser")
            .expect("valid browser DeviceConfig")
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let status = use_state(|| "Rozłączony".to_string());
    let status_color = use_state(|| "red".to_string());
    let is_connected = use_state(|| false);
    let cmd_sender = use_state(|| None::<UnboundedSender<ClientPacket>>);
    let sequence_id = use_mut_ref(|| 0u8);
    let logs = use_state(Vec::<String>::new);
    let last_game = use_state(|| Game::new(Discipline::Fiba5vs5));
    let available_games = use_state(Vec::<Game>::new);
    let connected_devices = use_state(Vec::<Device>::new);
    let device = use_state(|| {
        Device::new(DeviceType::WebBrowser, "sn-web-0000", "hmi-browser")
            .expect("valid browser DeviceConfig")
    });

    let log_buffer: Rc<RefCell<Vec<String>>> = use_mut_ref(Vec::new);

    let add_log = {
        let logs = logs.clone();
        let log_buffer = log_buffer.clone();
        Callback::from(move |msg: String| {
            let mut buf = log_buffer.borrow_mut();
            buf.push(msg);
            if buf.len() > 50 {
                buf.remove(0);
            }
            logs.set(buf.clone());
        })
    };

    {
        let device = device.clone();
        let status = status.clone();
        let status_color = status_color.clone();
        let is_connected = is_connected.clone();
        let cmd_sender = cmd_sender.clone();
        let last_game = last_game.clone();
        let available_games = available_games.clone();
        let connected_devices = connected_devices.clone();
        let add_log = add_log.clone();

        use_effect_with((), move |_| {
            let client_dev = load_or_create_browser_device();
            device.set(client_dev);

            let host = web_sys::window()
                .and_then(|w| w.location().host().ok())
                .filter(|h| !h.is_empty())
                .unwrap_or_else(|| "127.0.0.1:3000".to_string());
            let ws_url = format!("ws://{}/ws", host);

            if let Ok(ws) = WebSocket::new(&ws_url) {
                ws.set_binary_type(BinaryType::Arraybuffer);

                let (outgoing_tx, mut outgoing_rx) = unbounded::<[u8; PACKET_SIZE]>();
                let (incoming_tx, incoming_rx) = unbounded::<[u8; PACKET_SIZE]>();
                let (packet_tx, mut packet_rx) = unbounded::<ServerPacket>();
                let (cmd_tx, mut cmd_rx) = unbounded::<ClientPacket>();

                // Callback onopen -> wysyła HandshakeRequest z DeviceConfig
                {
                    let status_c = status.clone();
                    let color_c = status_color.clone();
                    let conn_c = is_connected.clone();
                    let log_c = add_log.clone();
                    let cmd_tx_open = cmd_tx.clone();
                    let onopen = Closure::wrap(Box::new(move |_e: JsValue| {
                        status_c.set("Połączono!".to_string());
                        color_c.set("green".to_string());
                        conn_c.set(true);
                        log_c.emit("Nawiązano połączenie WebSocket.".to_string());

                        let handshake = ClientPacket::new(
                            ClientEvent::HandshakeRequest,
                            0,
                            client_dev.to_bytes(),
                        );
                        let _ = cmd_tx_open.unbounded_send(handshake);
                    }) as Box<dyn FnMut(JsValue)>);
                    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                    onopen.forget();
                }

                // Callback onmessage -> przekazuje surowe bajty do Transportu
                {
                    let incoming_tx_msg = incoming_tx.clone();
                    let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
                        if let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                            let array = js_sys::Uint8Array::new(&buf);
                            if array.length() as usize == PACKET_SIZE {
                                let mut data_arr = [0u8; PACKET_SIZE];
                                array.copy_to(&mut data_arr);
                                let _ = incoming_tx_msg.unbounded_send(data_arr);
                            }
                        }
                    })
                        as Box<dyn FnMut(MessageEvent)>);
                    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                    onmessage.forget();
                }

                // Callback onclose -> zamyka kanał wejściowy, co kończy pętlę Client
                {
                    let status_c = status.clone();
                    let color_c = status_color.clone();
                    let conn_c = is_connected.clone();
                    let devices_c = connected_devices.clone();
                    let games_c = available_games.clone();
                    let log_c = add_log.clone();
                    let incoming_tx_close = incoming_tx;
                    let onclose = Closure::wrap(Box::new(move |_e: JsValue| {
                        status_c.set("Rozłączony".to_string());
                        color_c.set("red".to_string());
                        conn_c.set(false);
                        devices_c.set(Vec::new());
                        games_c.set(Vec::new());
                        incoming_tx_close.close_channel();
                        log_c.emit("Połączenie zostało zamknięte.".to_string());
                    }) as Box<dyn FnMut(JsValue)>);
                    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                    onclose.forget();
                }

                // Task fizycznego wysyłania bajtów przez WebSocket
                {
                    let ws_sender = ws.clone();
                    let log_c = add_log.clone();
                    spawn_local(async move {
                        while let Some(bytes) = outgoing_rx.next().await {
                            let uint8_array = js_sys::Uint8Array::from(&bytes[..]);
                            if let Err(e) = ws_sender.send_with_array_buffer(&uint8_array.buffer())
                            {
                                log_c.emit(format!("Błąd wysyłania: {:?}", e));
                            }
                        }
                    });
                }

                // Task aktualizacji UI na podstawie zdekodowanych pakietów z HmiClientHandler
                {
                    let last_game_c = last_game.clone();
                    let games_state_c = available_games.clone();
                    let devices_state_c = connected_devices.clone();
                    let log_c = add_log.clone();
                    spawn_local(async move {
                        let mut prev_game: Option<Game> = None;
                        let mut device_list: Vec<Device> = Vec::new();
                        let mut game_list: Vec<Game> = Vec::new();

                        while let Some(first_packet) = packet_rx.next().await {
                            let mut latest_game_packet: Option<(u8, Game)> = None;
                            let mut devices_changed = false;
                            let mut games_changed = false;

                            let mut current = Some(first_packet);
                            while let Some(packet) = current {
                                match packet.event() {
                                    ServerEvent::HandshakeAck => {
                                        if let Ok(game) = Game::try_from(packet) {
                                            latest_game_packet = Some((packet.sequence_id(), game));
                                            if let Some(existing) =
                                                game_list.iter_mut().find(|g| g.id() == game.id())
                                            {
                                                *existing = game;
                                            } else {
                                                game_list.push(game);
                                                game_list.sort_by_key(|g| g.id());
                                            }
                                            games_changed = true;
                                        }
                                        log_c.emit(format!(
                                            "[ODEBRANO] HandshakeAck (seq={})",
                                            packet.sequence_id()
                                        ));
                                    }
                                    ServerEvent::GameUpdate => {
                                        if let Ok(game) = Game::try_from(packet) {
                                            latest_game_packet = Some((packet.sequence_id(), game));
                                            if let Some(existing) =
                                                game_list.iter_mut().find(|g| g.id() == game.id())
                                            {
                                                if existing.get_scores() != game.get_scores()
                                                    || existing.get_state() != game.get_state()
                                                    || existing.get_quarter() != game.get_quarter()
                                                {
                                                    games_changed = true;
                                                }
                                                *existing = game;
                                            } else {
                                                game_list.push(game);
                                                game_list.sort_by_key(|g| g.id());
                                                games_changed = true;
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
                                                game_list.sort_by_key(|g| g.id());
                                            }
                                            games_changed = true;
                                        }
                                    }
                                    ServerEvent::DeviceUpdate => {
                                        if let Ok(dev) = Device::try_from(packet) {
                                            let sn = dev
                                                .get_serial_number()
                                                .unwrap_or("unknown")
                                                .to_owned();
                                            let id =
                                                dev.get_device_id().unwrap_or("unknown").to_owned();
                                            if let Some(existing) =
                                                device_list.iter_mut().find(|d| {
                                                    d.get_serial_number() == dev.get_serial_number()
                                                })
                                            {
                                                if existing.game_id() != dev.game_id() {
                                                    log_c.emit(format!(
                                                        "[URZĄDZENIE~] {:?} (sn={}) -> przypisano do Gry #{}",
                                                        dev.device_type(),
                                                        sn,
                                                        dev.game_id()
                                                    ));
                                                }
                                                *existing = dev;
                                            } else {
                                                device_list.push(dev);
                                                log_c.emit(format!(
                                                    "[URZĄDZENIE+] {:?} (id={}, sn={})",
                                                    dev.device_type(),
                                                    id,
                                                    sn
                                                ));
                                            }
                                            devices_changed = true;
                                        }
                                    }
                                    ServerEvent::DeviceDisconnect => {
                                        if let Ok(dev) = Device::try_from(packet) {
                                            let sn = dev
                                                .get_serial_number()
                                                .unwrap_or("unknown")
                                                .to_owned();
                                            let id =
                                                dev.get_device_id().unwrap_or("unknown").to_owned();
                                            let before = device_list.len();
                                            device_list.retain(|d| {
                                                d.get_serial_number() != dev.get_serial_number()
                                            });
                                            if device_list.len() != before {
                                                log_c.emit(format!(
                                                    "[URZĄDZENIE-] {:?} (id={}, sn={})",
                                                    dev.device_type(),
                                                    id,
                                                    sn
                                                ));
                                                devices_changed = true;
                                            }
                                        }
                                    }
                                }

                                current = match packet_rx.try_recv() {
                                    Ok(next_pkt) => Some(next_pkt),
                                    _ => None,
                                };
                            }

                            if devices_changed {
                                devices_state_c.set(device_list.clone());
                            }

                            if games_changed {
                                games_state_c.set(game_list.clone());
                            }

                            if let Some((seq, game)) = latest_game_packet {
                                let changed = match prev_game {
                                    Some(prev) => {
                                        prev.id() != game.id()
                                            || prev.get_scores() != game.get_scores()
                                            || prev.get_state() != game.get_state()
                                            || prev.get_quarter() != game.get_quarter()
                                    }
                                    None => true,
                                };

                                if changed {
                                    let (h, a) = game.get_scores();
                                    log_c.emit(format!(
                                        "[ODEBRANO] Gra #{} (seq={}) -> Stan: {:?}, Kwarta: {:?}, Wynik: {} : {}",
                                        game.id(),
                                        seq,
                                        game.get_state(),
                                        game.get_quarter(),
                                        h,
                                        a
                                    ));
                                }

                                prev_game = Some(game);
                                last_game_c.set(game);
                            }
                        }
                    });
                }

                // Główna pętla zunifikowanego klienta `net::Client`
                {
                    let adapter = BrowserWebSocketAdapter::new(outgoing_tx, incoming_rx);
                    let transport = Transport::new(adapter, None);
                    let handler = HmiClientHandler::new(packet_tx);
                    let mut client = Client::new(transport, handler);
                    let log_c = add_log.clone();

                    spawn_local(async move {
                        enum Step {
                            Polled(Result<ServerPacket, NetError>),
                            Command(Option<ClientPacket>),
                        }

                        loop {
                            let step = {
                                let poll_fut = core::pin::pin!(client.poll());
                                let send_fut = core::pin::pin!(cmd_rx.next());

                                match select(poll_fut, send_fut).await {
                                    Either::Left((res, _)) => Step::Polled(res),
                                    Either::Right((cmd, _)) => Step::Command(cmd),
                                }
                            };

                            match step {
                                Step::Polled(Ok(_)) => {}
                                Step::Polled(Err(NetError::InvalidPacket(err))) => {
                                    log_c.emit(format!("[BŁĄD PAKIETU] {}", err));
                                }
                                Step::Polled(Err(NetError::Disconnected)) => break,
                                Step::Command(Some(packet)) => {
                                    client.send(&packet).await;
                                }
                                Step::Command(None) => break,
                            }
                        }
                    });
                }

                cmd_sender.set(Some(cmd_tx));
            }

            || ()
        });
    }

    let send_event = {
        let cmd_sender = cmd_sender.clone();
        let sequence_id = sequence_id.clone();
        let last_game = last_game.clone();
        let add_log = add_log.clone();
        let client_dev = *device;

        move |event: ClientEvent| {
            let cmd_sender = cmd_sender.clone();
            let sequence_id = sequence_id.clone();
            let last_game = last_game.clone();
            let add_log = add_log.clone();

            Callback::from(move |_| {
                if let Some(tx) = (*cmd_sender).as_ref() {
                    let mut seq = sequence_id.borrow_mut();
                    *seq = seq.wrapping_add(1);
                    let current_seq = *seq;

                    let payload = match event {
                        ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat => {
                            client_dev.to_bytes()
                        }
                        _ => (*last_game).to_bytes(),
                    };

                    let packet = ClientPacket::new(event, current_seq, payload);
                    if tx.unbounded_send(packet).is_ok() {
                        add_log.emit(format!("[WYSŁANO] {:?} (seq={})", event, current_seq));
                    }
                }
            })
        }
    };

    let select_game = {
        let cmd_sender = cmd_sender.clone();
        let sequence_id = sequence_id.clone();
        let add_log = add_log.clone();

        move |game_id: u16| {
            let cmd_sender = cmd_sender.clone();
            let sequence_id = sequence_id.clone();
            let add_log = add_log.clone();

            Callback::from(move |_| {
                if let Some(tx) = (*cmd_sender).as_ref() {
                    let mut seq = sequence_id.borrow_mut();
                    *seq = seq.wrapping_add(1);
                    let current_seq = *seq;

                    let target = Game::with_id(game_id, Discipline::Fiba5vs5);
                    let packet =
                        ClientPacket::new(ClientEvent::GameSelect, current_seq, target.to_bytes());
                    if tx.unbounded_send(packet).is_ok() {
                        add_log.emit(format!(
                            "[WYSŁANO] Wybór gry #{} (seq={})",
                            game_id, current_seq
                        ));
                    }
                }
            })
        }
    };

    let disconnect_device = {
        let cmd_sender = cmd_sender.clone();
        let sequence_id = sequence_id.clone();
        let add_log = add_log.clone();

        move |target_dev: Device| {
            let cmd_sender = cmd_sender.clone();
            let sequence_id = sequence_id.clone();
            let add_log = add_log.clone();

            Callback::from(move |_| {
                if let Some(tx) = (*cmd_sender).as_ref() {
                    let mut seq = sequence_id.borrow_mut();
                    *seq = seq.wrapping_add(1);
                    let current_seq = *seq;

                    let sn = target_dev.get_serial_number().unwrap_or("-").to_owned();
                    let packet = ClientPacket::new(
                        ClientEvent::DeviceDisconnect,
                        current_seq,
                        target_dev.to_bytes(),
                    );
                    if tx.unbounded_send(packet).is_ok() {
                        add_log.emit(format!(
                            "[WYSŁANO] Rozłącz urządzenie {} (seq={})",
                            sn, current_seq
                        ));
                    }
                }
            })
        }
    };

    let assign_device_game = {
        let cmd_sender = cmd_sender.clone();
        let sequence_id = sequence_id.clone();
        let add_log = add_log.clone();

        move |target_dev: Device, target_game_id: u16| {
            let cmd_sender = cmd_sender.clone();
            let sequence_id = sequence_id.clone();
            let add_log = add_log.clone();

            Callback::from(move |_| {
                if let Some(tx) = (*cmd_sender).as_ref() {
                    let mut seq = sequence_id.borrow_mut();
                    *seq = seq.wrapping_add(1);
                    let current_seq = *seq;

                    let mut updated_dev = target_dev;
                    updated_dev.set_game_id(target_game_id);
                    let sn = updated_dev.get_serial_number().unwrap_or("-").to_owned();
                    let packet = ClientPacket::new(
                        ClientEvent::DeviceAssignGame,
                        current_seq,
                        updated_dev.to_bytes(),
                    );
                    if tx.unbounded_send(packet).is_ok() {
                        add_log.emit(format!(
                            "[WYSŁANO] Przypisz Worker {} do Gry #{} (seq={})",
                            sn, target_game_id, current_seq
                        ));
                    }
                }
            })
        }
    };

    let game = *last_game;
    let (home_score, away_score) = game.get_scores();
    let (reg_min, reg_sec, reg_cs) = game.get_regulation_time();
    let (shot_sec, shot_cs) = game.get_clock_time();
    let quarter_label = match game.get_quarter() {
        Quarter::None => "Brak",
        Quarter::Q1 => "Q1",
        Quarter::Q2 => "Q2",
        Quarter::Q3 => "Q3",
        Quarter::Q4 => "Q4",
    };
    let dev = *device;
    let my_sn = dev.get_serial_number().unwrap_or("-");
    let watchers: Vec<Device> = (*connected_devices)
        .iter()
        .copied()
        .filter(|d| d.game_id() == game.id())
        .collect();
    let workers: Vec<Device> = (*connected_devices)
        .iter()
        .copied()
        .filter(|d| d.device_type() == DeviceType::Worker)
        .collect();

    html! {
        <div style="font-family: sans-serif; max-width: 650px; margin: 40px auto; padding: 20px;">
            <h2>{ "Panel HMI (Yew / Rust) - WebSocket" }</h2>

            <div style="border: 1px solid #ccc; padding: 15px; margin-bottom: 10px; border-radius: 5px;">
                <h3>{ "Status połączenia: " }<span style={format!("color: {};", *status_color)}>{ &*status }</span></h3>
                <p style="font-size: 13px; color: #475569; margin: 4px 0;">
                    { format!(
                        "To urządzenie: {:?} | ID: {} | SN: {}",
                        dev.device_type(),
                        dev.get_device_id().unwrap_or("-"),
                        my_sn
                    ) }
                </p>
                <p style="font-size: 16px; font-weight: bold; margin: 8px 0;">
                    { format!(
                        "Gra #{} | Stan: {:?} | Kwarta: {} | Czas: {:02}:{:02}.{:02} (24s: {:02}.{:02}) | Wynik: {} : {}",
                        game.id(),
                        game.get_state(),
                        quarter_label,
                        reg_min,
                        reg_sec,
                        reg_cs,
                        shot_sec,
                        shot_cs,
                        home_score,
                        away_score
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
                        { for (*available_games).iter().map(|g| {
                            let gid = g.id();
                            let (h, a) = g.get_scores();
                            let is_active = gid == game.id();
                            let count = (*connected_devices)
                                .iter()
                                .filter(|d| d.game_id() == gid)
                                .count();
                            html! {
                                <button
                                    onclick={select_game(gid)}
                                    style={format!(
                                        "padding: 8px 12px; cursor: pointer; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-weight: {};",
                                        if is_active { "#0f172a" } else { "#cbd5e1" },
                                        if is_active { "#0f172a" } else { "#f8fafc" },
                                        if is_active { "#ffffff" } else { "#0f172a" },
                                        if is_active { "bold" } else { "normal" }
                                    )}
                                >
                                    { format!("Gra #{} ({}:{} | {:?} | urz.: {})", gid, h, a, g.get_state(), count) }
                                </button>
                            }
                        }) }
                    </div>
                }
            </div>

            <div style="border: 1px solid #ccc; padding: 15px; margin-bottom: 10px; border-radius: 5px;">
                <h3>{ format!("Workery w sieci — wybór wyświetlanej gry ({}):", workers.len()) }</h3>
                if workers.is_empty() {
                    <p style="font-size: 13px; color: #64748b; margin: 0;">{ "Brak podłączonych Workerów (sprzętowych ani symulowanych)" }</p>
                } else {
                    <ul style="margin: 0; padding-left: 0; list-style: none; font-size: 14px; display: flex; flex-direction: column; gap: 8px;">
                        { for workers.iter().map(|w| {
                            let worker_copy = *w;
                            let sn = w.get_serial_number().unwrap_or("-");
                            let id = w.get_device_id().unwrap_or("-");
                            let current_worker_gid = w.game_id();
                            let assigned_label = if current_worker_gid == 0 {
                                "Brak (oczekuje na wybór)".to_string()
                            } else {
                                format!("Gra #{}", current_worker_gid)
                            };
                            html! {
                                <li style="display: flex; flex-direction: column; gap: 8px; padding: 10px 12px; background: #f8fafc; border: 1px solid #cbd5e1; border-radius: 6px;">
                                    <div style="display: flex; justify-content: space-between; align-items: center;">
                                        <span style="font-weight: 600; color: #0f172a;">
                                            { format!("Worker — ID: {} | SN: {} | Wyświetla: {}", id, sn, assigned_label) }
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
                                                if current_worker_gid == 0 { "#64748b" } else { "#cbd5e1" },
                                                if current_worker_gid == 0 { "#64748b" } else { "#ffffff" },
                                                if current_worker_gid == 0 { "#ffffff" } else { "#475569" },
                                                if current_worker_gid == 0 { "bold" } else { "normal" }
                                            )}
                                        >
                                            { "Brak (wygaszony)" }
                                        </button>
                                        { for (*available_games).iter().map(|g| {
                                            let gid = g.id();
                                            let (h, a) = g.get_scores();
                                            let is_assigned = gid == current_worker_gid;
                                            html! {
                                                <button
                                                    onclick={assign_device_game(worker_copy, gid)}
                                                    style={format!(
                                                        "padding: 4px 10px; font-size: 12px; cursor: pointer; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-weight: {};",
                                                        if is_assigned { "#0284c7" } else { "#cbd5e1" },
                                                        if is_assigned { "#0284c7" } else { "#ffffff" },
                                                        if is_assigned { "#ffffff" } else { "#0f172a" },
                                                        if is_assigned { "bold" } else { "normal" }
                                                    )}
                                                >
                                                    { format!("Gra #{} ({}:{})", gid, h, a) }
                                                </button>
                                            }
                                        }) }
                                    </div>
                                </li>
                            }
                        }) }
                    </ul>
                }
            </div>

            <div style="border: 1px solid #ccc; padding: 15px; margin-bottom: 10px; border-radius: 5px;">
                <h3>{ format!("Urządzenia podglądające Grę #{} ({}):", game.id(), watchers.len()) }</h3>
                if watchers.is_empty() {
                    <p style="font-size: 13px; color: #64748b; margin: 0;">{ "Brak urządzeń podglądających tę grę" }</p>
                } else {
                    <ul style="margin: 0; padding-left: 0; list-style: none; font-size: 14px; display: flex; flex-direction: column; gap: 6px;">
                        { for watchers.iter().map(|d| {
                            let dev_copy = *d;
                            let sn = d.get_serial_number().unwrap_or("-");
                            let id = d.get_device_id().unwrap_or("-");
                            let is_self = sn == my_sn;
                            html! {
                                <li style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 4px;">
                                    <span style={if is_self { "font-weight: bold; color: #0f172a;" } else { "color: #334155;" }}>
                                        { format!("{:?} — ID: {} | SN: {} | Gra #{}{}", d.device_type(), id, sn, d.game_id(), if is_self { " (Ty)" } else { "" }) }
                                    </span>
                                    <button
                                        onclick={disconnect_device(dev_copy)}
                                        style="padding: 4px 10px; font-size: 12px; cursor: pointer; background: #e11d48; color: white; border: none; border-radius: 4px;"
                                    >
                                        { "Rozłącz" }
                                    </button>
                                </li>
                            }
                        }) }
                    </ul>
                }
            </div>

            <div style="border: 1px solid #ccc; padding: 15px; border-radius: 5px;">
                <h3>{ "Logi konsoli:" }</h3>
                <pre style="background: #f4f4f4; padding: 10px; border-radius: 4px; max-height: 200px; overflow-y: auto;">
                    { for (*logs).iter().map(|log| html! { <div>{ log }</div> }) }
                </pre>
            </div>
        </div>
    }
}
