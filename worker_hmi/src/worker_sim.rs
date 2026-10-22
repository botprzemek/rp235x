use core::f64::consts::PI;
use core::net::SocketAddr;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use basketball::{Discipline, Game, Quarter, Scorable};
use config::DeviceConfig;
use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures::future::{Either, select};
use hub75::{DisplayBuffer, hue_to_rgb};
use net::layout::PACKET_SIZE;
use net::{
    Client, ClientEvent, ClientHandler, ClientPacket, Device, DeviceType, NetError, RawConnection,
    ServerEvent, ServerPacket, Transport,
};
use web_sys::js_sys;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::prelude::*;
use web_sys::{BinaryType, CanvasRenderingContext2d, HtmlCanvasElement, MessageEvent, WebSocket};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::prelude::*;

const MATRIX_WIDTH: usize = 64;
const MATRIX_HEIGHT: usize = 32;
const MATRIX_HALF_HEIGHT: usize = 16;
const PIXEL_PITCH: f64 = 12.0;
const CANVAS_WIDTH: u32 = (MATRIX_WIDTH as u32) * (PIXEL_PITCH as u32);
const CANVAS_HEIGHT: u32 = (MATRIX_HEIGHT as u32) * (PIXEL_PITCH as u32);

const WORKER_SERIAL_STORAGE_KEY: &str = "worker_hmi_serial_number";

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

pub struct WorkerClientHandler {
    packet_tx: UnboundedSender<ServerPacket>,
}

impl WorkerClientHandler {
    pub fn new(packet_tx: UnboundedSender<ServerPacket>) -> Self {
        Self { packet_tx }
    }
}

impl ClientHandler for WorkerClientHandler {
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

fn load_or_create_worker_device() -> Device {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());

        if let Some(ref store) = storage
            && let Ok(Some(saved_sn)) = store.get_item(WORKER_SERIAL_STORAGE_KEY)
            && let Ok(device) = Device::new(DeviceType::Worker, &saved_sn, "rp235x-sim")
        {
            return device;
        }

        let suffix = (js_sys::Math::random() * 10000.0) as u16 % 10000;
        let sn = format!("sn-wrk-{:04}", suffix);

        if let Some(store) = storage {
            let _ = store.set_item(WORKER_SERIAL_STORAGE_KEY, &sn);
        }

        Device::new(DeviceType::Worker, &sn, "rp235x-sim").expect("valid worker DeviceConfig")
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Device::new(DeviceType::Worker, "sn-wrk-0000", "rp235x-sim")
            .expect("valid worker DeviceConfig")
    }
}

fn render_hub75_to_canvas(
    canvas: &HtmlCanvasElement,
    display: &mut DisplayBuffer<MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_HALF_HEIGHT>,
    game: &Game,
    phase_hue: u16,
) -> [u8; 3] {
    let color = if game.id() == 0 {
        display.clear();
        [0, 0, 0]
    } else {
        let (home_score, away_score) = game.get_scores();
        let total_seconds = game.get_regulation_millis() / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;

        let home_str = format!("{:03}", home_score);
        let guest_str = format!("{:03}", away_score);
        let quarter_str = match game.get_quarter() {
            Quarter::None => "Q0".to_string(),
            q => format!("{:?}", q),
        };
        let clock_str = format!("{:02}:{:02}", minutes, seconds);

        let color = hue_to_rgb(phase_hue);
        display.draw_animated_text(color, &home_str, &guest_str, &quarter_str, &clock_str);
        color
    };

    let Ok(Some(ctx_obj)) = canvas.get_context("2d") else {
        return color;
    };
    let Ok(ctx) = ctx_obj.dyn_into::<CanvasRenderingContext2d>() else {
        return color;
    };

    // Obudowa / tło matrycy HUB75
    ctx.set_fill_style_str("#05070a");
    ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH as f64, CANVAS_HEIGHT as f64);

    let led_radius = PIXEL_PITCH * 0.36;

    for y in 0..MATRIX_HEIGHT {
        for x in 0..MATRIX_WIDTH {
            let [r, g, b] = display.frame_buffer[y][x];
            let cx = (x as f64) * PIXEL_PITCH + (PIXEL_PITCH / 2.0);
            let cy = (y as f64) * PIXEL_PITCH + (PIXEL_PITCH / 2.0);

            ctx.begin_path();
            let _ = ctx.arc(cx, cy, led_radius, 0.0, 2.0 * PI);

            if r == 0 && g == 0 && b == 0 {
                // Wygaszona dioda SMD RGB
                ctx.set_fill_style_str("#161b24");
                ctx.fill();
            } else {
                // Świecąca dioda SMD RGB
                let fill = format!("rgb({}, {}, {})", r, g, b);
                ctx.set_fill_style_str(&fill);
                ctx.fill();

                // Delikatny środek soczewki LED
                ctx.begin_path();
                let _ = ctx.arc(cx, cy, led_radius * 0.45, 0.0, 2.0 * PI);
                let core_r = (r as u16 + 80).min(255) as u8;
                let core_g = (g as u16 + 80).min(255) as u8;
                let core_b = (b as u16 + 80).min(255) as u8;
                ctx.set_fill_style_str(&format!("rgb({}, {}, {})", core_r, core_g, core_b));
                ctx.fill();
            }
        }
    }

    color
}

#[function_component(App)]
pub fn app() -> Html {
    let canvas_ref = use_node_ref();
    let display_buffer = use_mut_ref(
        || DisplayBuffer::<MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_HALF_HEIGHT>::default(),
    );

    let status = use_state(|| "Rozłączony".to_string());
    let status_color = use_state(|| "#ef4444".to_string());
    let ws_endpoint = use_state(|| "ws://127.0.0.1:3000/ws".to_string());
    let last_game = use_state(|| Game::with_id(0, Discipline::Fiba5vs5));
    let last_seq = use_state(|| 0u8);
    let packets_received = use_state(|| 0u64);
    let heartbeats_sent = use_state(|| 0u64);
    let current_rgb = use_state(|| [0u8, 0, 0]);
    let logs = use_state(Vec::<String>::new);
    let device = use_state(|| {
        Device::new(DeviceType::Worker, "sn-wrk-0000", "rp235x-sim")
            .expect("valid worker DeviceConfig")
    });

    let log_buffer: Rc<RefCell<Vec<String>>> = use_mut_ref(Vec::new);

    let add_log = {
        let logs = logs.clone();
        let log_buffer = log_buffer.clone();
        Callback::from(move |msg: String| {
            let mut buf = log_buffer.borrow_mut();
            buf.push(msg);
            if buf.len() > 80 {
                buf.remove(0);
            }
            logs.set(buf.clone());
        })
    };

    let clear_logs = {
        let logs = logs.clone();
        let log_buffer = log_buffer.clone();
        Callback::from(move |_| {
            log_buffer.borrow_mut().clear();
            logs.set(Vec::new());
        })
    };

    // Przerysowanie matrycy HUB75 na <canvas> przy każdej zmianie stanu gry
    {
        let canvas_ref = canvas_ref.clone();
        let display_buffer = display_buffer.clone();
        let current_rgb = current_rgb.clone();
        let game = *last_game;
        let pkt_count = *packets_received;

        use_effect_with((game, pkt_count), move |(game, _)| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                #[cfg(target_arch = "wasm32")]
                let now_ms = js_sys::Date::now() as u64;
                #[cfg(not(target_arch = "wasm32"))]
                let now_ms = 0u64;

                let phase_hue = ((now_ms % 10_000) * 360 / 10_000) as u16;
                let mut display = display_buffer.borrow_mut();
                let rgb = render_hub75_to_canvas(&canvas, &mut display, game, phase_hue);
                current_rgb.set(rgb);
            }
            || ()
        });
    }

    // Automatyczne połączenie WebSocket i pętla wirtualnego Workera
    {
        let device = device.clone();
        let status = status.clone();
        let status_color = status_color.clone();
        let ws_endpoint = ws_endpoint.clone();
        let last_game = last_game.clone();
        let last_seq = last_seq.clone();
        let packets_received = packets_received.clone();
        let heartbeats_sent = heartbeats_sent.clone();
        let add_log = add_log.clone();

        use_effect_with((), move |_| {
            let worker_dev = load_or_create_worker_device();
            device.set(worker_dev);

            let hostname = web_sys::window()
                .and_then(|w| w.location().hostname().ok())
                .filter(|h| !h.is_empty())
                .unwrap_or_else(|| "127.0.0.1".to_string());
            let ws_url = format!("ws://{}:3000/ws", hostname);
            ws_endpoint.set(ws_url.clone());

            add_log.emit(format!(
                "[INIT] Wirtualny Worker uruchomiony (typ={:?}, id={}, sn={})",
                worker_dev.device_type(),
                worker_dev.get_device_id().unwrap_or("-"),
                worker_dev.get_serial_number().unwrap_or("-")
            ));
            add_log.emit(format!("[SIEĆ] Łączenie z Masterem: {} ...", ws_url));

            if let Ok(ws) = WebSocket::new(&ws_url) {
                ws.set_binary_type(BinaryType::Arraybuffer);

                let (outgoing_tx, mut outgoing_rx) = unbounded::<[u8; PACKET_SIZE]>();
                let (incoming_tx, incoming_rx) = unbounded::<[u8; PACKET_SIZE]>();
                let (packet_tx, mut packet_rx) = unbounded::<ServerPacket>();
                let (cmd_tx, mut cmd_rx) = unbounded::<ClientPacket>();

                // Callback onopen -> wysyła HandshakeRequest z DeviceType::Worker
                {
                    let status_c = status.clone();
                    let color_c = status_color.clone();
                    let log_c = add_log.clone();
                    let cmd_tx_open = cmd_tx.clone();
                    let onopen = Closure::wrap(Box::new(move |_e: JsValue| {
                        status_c.set("Oczekuje na wybór gry w HMI".to_string());
                        color_c.set("#f59e0b".to_string());
                        log_c.emit(
                            "[WS] Nawiązano połączenie. Wysyłanie HandshakeRequest (oczekiwanie na wybór gry w HMI)...".to_string(),
                        );

                        let handshake = ClientPacket::new(
                            ClientEvent::HandshakeRequest,
                            0,
                            worker_dev.to_bytes(),
                        );
                        let _ = cmd_tx_open.unbounded_send(handshake);
                    }) as Box<dyn FnMut(JsValue)>);
                    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                    onopen.forget();
                }

                // Callback onmessage -> przekazuje 24-bajtowe ramki do Transportu
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

                // Callback onclose -> zamknięcie połączenia
                {
                    let status_c = status.clone();
                    let color_c = status_color.clone();
                    let log_c = add_log.clone();
                    let incoming_tx_close = incoming_tx;
                    let onclose = Closure::wrap(Box::new(move |_e: JsValue| {
                        status_c.set("Rozłączony".to_string());
                        color_c.set("#ef4444".to_string());
                        incoming_tx_close.close_channel();
                        log_c.emit("[WS] Połączenie z Masterem zostało zamknięte.".to_string());
                    }) as Box<dyn FnMut(JsValue)>);
                    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                    onclose.forget();
                }

                // Task fizycznego wysyłania pakietów przez WebSocket
                {
                    let ws_sender = ws.clone();
                    let log_c = add_log.clone();
                    spawn_local(async move {
                        while let Some(bytes) = outgoing_rx.next().await {
                            let uint8_array = js_sys::Uint8Array::from(&bytes[..]);
                            if let Err(e) = ws_sender.send_with_array_buffer(&uint8_array.buffer())
                            {
                                log_c.emit(format!("[BŁĄD WS TX] {:?}", e));
                            }
                        }
                    });
                }

                // Task okresowego wysyłania HandshakeHeartbeat co 3 sekundy (jak w fizycznym Workerze)
                {
                    let cmd_tx_hb = cmd_tx.clone();
                    let hb_counter = heartbeats_sent.clone();
                    let log_c = add_log.clone();
                    spawn_local(async move {
                        let mut seq: u8 = 0;
                        let mut count: u64 = 0;
                        loop {
                            sleep(Duration::from_secs(3)).await;
                            seq = seq.wrapping_add(1);
                            let packet = ClientPacket::new(
                                ClientEvent::HandshakeHeartbeat,
                                seq,
                                worker_dev.to_bytes(),
                            );
                            if cmd_tx_hb.unbounded_send(packet).is_err() {
                                break;
                            }
                            count += 1;
                            hb_counter.set(count);
                            log_c.emit(format!(
                                "[HEARTBEAT] Wysłano HandshakeHeartbeat #{} (seq={})",
                                count, seq
                            ));
                        }
                    });
                }

                // Task odbioru pakietów z WorkerClientHandler i aktualizacji stanu gry oraz matrycy HUB75
                {
                    let status_c = status.clone();
                    let color_c = status_color.clone();
                    let last_game_c = last_game.clone();
                    let last_seq_c = last_seq.clone();
                    let packets_received_c = packets_received.clone();
                    let log_c = add_log.clone();
                    spawn_local(async move {
                        let mut prev_game: Option<Game> = None;
                        let mut total_pkts: u64 = 0;

                        while let Some(first_packet) = packet_rx.next().await {
                            let mut latest_game_packet: Option<(u8, Game)> = None;

                            let mut current = Some(first_packet);
                            while let Some(packet) = current {
                                total_pkts += 1;

                                match packet.event() {
                                    ServerEvent::HandshakeAck => {
                                        if let Ok(game) = Game::try_from(packet) {
                                            latest_game_packet = Some((packet.sequence_id(), game));
                                        }
                                        log_c.emit(format!(
                                            "[ODEBRANO] HandshakeAck (seq={})",
                                            packet.sequence_id()
                                        ));
                                    }
                                    ServerEvent::GameUpdate => {
                                        if let Ok(game) = Game::try_from(packet) {
                                            latest_game_packet = Some((packet.sequence_id(), game));
                                        }
                                    }
                                    ServerEvent::GameListUpdate
                                    | ServerEvent::DeviceUpdate
                                    | ServerEvent::DeviceDisconnect => {}
                                }

                                current = match packet_rx.try_next() {
                                    Ok(Some(next_pkt)) => Some(next_pkt),
                                    _ => None,
                                };
                            }

                            packets_received_c.set(total_pkts);

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
                                    if game.id() == 0 {
                                        status_c.set("Oczekuje na wybór gry w HMI".to_string());
                                        color_c.set("#f59e0b".to_string());
                                        log_c.emit(format!(
                                            "[STAN] Worker nieprzypisany do żadnej gry (seq={}) — wybierz grę w panelu HMI",
                                            seq
                                        ));
                                    } else {
                                        status_c.set(format!("Aktywny — Gra #{}", game.id()));
                                        color_c.set("#22c55e".to_string());
                                        let (h, a) = game.get_scores();
                                        let (min, sec, cs) = game.get_regulation_time();
                                        log_c.emit(format!(
                                            "[HUB75 UPDATE] Gra #{} (seq={}) -> Stan: {:?} | Kwarta: {:?} | Czas: {:02}:{:02}.{:02} | Wynik: {:03} : {:03}",
                                            game.id(),
                                            seq,
                                            game.get_state(),
                                            game.get_quarter(),
                                            min,
                                            sec,
                                            cs,
                                            h,
                                            a
                                        ));
                                    }
                                }

                                prev_game = Some(game);
                                last_seq_c.set(seq);
                                last_game_c.set(game);
                            }
                        }
                    });
                }

                // Główna pętla `net::Client`
                {
                    let adapter = BrowserWebSocketAdapter::new(outgoing_tx, incoming_rx);
                    let transport = Transport::new(adapter, None);
                    let handler = WorkerClientHandler::new(packet_tx);
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
            }

            || ()
        });
    }

    let game = *last_game;
    let (home_score, away_score) = game.get_scores();
    let (reg_min, reg_sec, reg_cs) = game.get_regulation_time();
    let (shot_sec, shot_cs) = game.get_clock_time();
    let dev = *device;
    let [r, g, b] = *current_rgb;

    html! {
        <div style="max-width: 900px; margin: 28px auto; padding: 0 20px; display: flex; flex-direction: column; gap: 20px;">
            <header style="display: flex; justify-content: space-between; align-items: center; background: #1e293b; border: 1px solid #334155; padding: 16px 20px; border-radius: 8px;">
                <div>
                    <h1 style="margin: 0; font-size: 20px; font-weight: 700; color: #f8fafc;">
                        { "Wirtualizowany Worker — Matryca LED HUB75 (64×32)" }
                    </h1>
                    <p style="margin: 4px 0 0 0; font-size: 13px; color: #94a3b8;">
                        { format!(" Symulator sprzętowego węzła RP235x | Endpoint: {}", &*ws_endpoint) }
                    </p>
                </div>
                <div style="display: flex; align-items: center; gap: 8px; background: #0f172a; padding: 8px 14px; border-radius: 999px; border: 1px solid #334155;">
                    <span style={format!("width: 10px; height: 10px; border-radius: 50%; background: {}; display: inline-block;", *status_color)}></span>
                    <span style={format!("font-size: 13px; font-weight: 600; color: {};", *status_color)}>
                        { &*status }
                    </span>
                </div>
            </header>

            // Sekcja 1: Canvas z matrycą LED HUB75 64x32
            <section style="background: #1e293b; border: 1px solid #334155; border-radius: 8px; padding: 20px; display: flex; flex-direction: column; align-items: center; gap: 12px;">
                <div style="width: 100%; display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 13px; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.5px;">
                        { "Panel HUB75 RGB (hub75::DisplayBuffer<64, 32, 16>)" }
                    </span>
                    <span style="font-size: 12px; font-family: monospace; color: #cbd5e1;">
                        { format!("Kolor tekstu RGB: ({}, {}, {})", r, g, b) }
                    </span>
                </div>

                <div style="background: #020408; padding: 14px; border-radius: 10px; border: 3px solid #334155; box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.7);">
                    <canvas
                        ref={canvas_ref}
                        width={CANVAS_WIDTH.to_string()}
                        height={CANVAS_HEIGHT.to_string()}
                        style="width: 100%; max-width: 768px; height: auto; display: block; border-radius: 4px;"
                    />
                </div>
            </section>

            // Sekcja 2: Dane aktualne (urządzenie, stan gry, telemetria)
            <section style="background: #1e293b; border: 1px solid #334155; border-radius: 8px; padding: 18px;">
                <h2 style="margin: 0 0 14px 0; font-size: 15px; font-weight: 600; color: #e2e8f0; text-transform: uppercase; letter-spacing: 0.5px;">
                    { "Dane aktualne" }
                </h2>

                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; font-size: 13px;">
                    <div style="background: #0f172a; border: 1px solid #334155; border-radius: 6px; padding: 12px;">
                        <div style="color: #64748b; font-size: 11px; text-transform: uppercase; margin-bottom: 4px;">{ "Tożsamość Workera" }</div>
                        <div style="font-family: monospace; color: #f8fafc; font-weight: 600;">
                            { format!("Typ: {:?}", dev.device_type()) }
                        </div>
                        <div style="font-family: monospace; color: #cbd5e1; margin-top: 2px;">
                            { format!("ID: {}", dev.get_device_id().unwrap_or("-")) }
                        </div>
                        <div style="font-family: monospace; color: #38bdf8; margin-top: 2px;">
                            { format!("SN: {}", dev.get_serial_number().unwrap_or("-")) }
                        </div>
                    </div>

                    <div style="background: #0f172a; border: 1px solid #334155; border-radius: 6px; padding: 12px;">
                        <div style="color: #64748b; font-size: 11px; text-transform: uppercase; margin-bottom: 4px;">{ "Kontekst Gry" }</div>
                        if game.id() == 0 {
                            <div style="font-family: monospace; color: #f59e0b; font-weight: 600;">
                                { "Brak przypisanej gry" }
                            </div>
                            <div style="font-family: monospace; color: #94a3b8; margin-top: 2px;">
                                { "Wybierz grę w panelu HMI" }
                            </div>
                        } else {
                            <div style="font-family: monospace; color: #f8fafc; font-weight: 600;">
                                { format!("Gra #{} ({:?})", game.id(), game.discipline()) }
                            </div>
                            <div style="font-family: monospace; color: #cbd5e1; margin-top: 2px;">
                                { format!("Stan: {:?}", game.get_state()) }
                            </div>
                            <div style="font-family: monospace; color: #facc15; margin-top: 2px;">
                                { format!("Kwarta: {:?}", game.get_quarter()) }
                            </div>
                        }
                    </div>

                    <div style="background: #0f172a; border: 1px solid #334155; border-radius: 6px; padding: 12px;">
                        <div style="color: #64748b; font-size: 11px; text-transform: uppercase; margin-bottom: 4px;">{ "Zegary i Wynik" }</div>
                        <div style="font-family: monospace; color: #f8fafc; font-weight: 600;">
                            { format!("Wynik: {:03} : {:03}", home_score, away_score) }
                        </div>
                        <div style="font-family: monospace; color: #cbd5e1; margin-top: 2px;">
                            { format!("Czas gry: {:02}:{:02}.{:02}", reg_min, reg_sec, reg_cs) }
                        </div>
                        <div style="font-family: monospace; color: #cbd5e1; margin-top: 2px;">
                            { format!("Zegar 24s: {:02}.{:02}s ({} ms)", shot_sec, shot_cs, game.clock_millis()) }
                        </div>
                    </div>

                    <div style="background: #0f172a; border: 1px solid #334155; border-radius: 6px; padding: 12px;">
                        <div style="color: #64748b; font-size: 11px; text-transform: uppercase; margin-bottom: 4px;">{ "Telemetria Sieci" }</div>
                        <div style="font-family: monospace; color: #f8fafc; font-weight: 600;">
                            { format!("Odebrane pakiety: {}", *packets_received) }
                        </div>
                        <div style="font-family: monospace; color: #cbd5e1; margin-top: 2px;">
                            { format!("Ostatni seq: {}", *last_seq) }
                        </div>
                        <div style="font-family: monospace; color: #cbd5e1; margin-top: 2px;">
                            { format!("Heartbeat TX: {}", *heartbeats_sent) }
                        </div>
                    </div>
                </div>
            </section>

            // Sekcja 3: Logi debugujące
            <section style="background: #1e293b; border: 1px solid #334155; border-radius: 8px; padding: 18px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                    <h2 style="margin: 0; font-size: 15px; font-weight: 600; color: #e2e8f0; text-transform: uppercase; letter-spacing: 0.5px;">
                        { format!("Logi debugujące ({})", (*logs).len()) }
                    </h2>
                    <button
                        onclick={clear_logs}
                        style="padding: 4px 10px; font-size: 12px; cursor: pointer; background: #334155; color: #f8fafc; border: 1px solid #475569; border-radius: 4px;"
                    >
                        { "Wyczyść logi" }
                    </button>
                </div>
                <pre style="margin: 0; background: #090d16; color: #94a3b8; border: 1px solid #334155; padding: 12px; border-radius: 6px; max-height: 220px; overflow-y: auto; font-family: monospace; font-size: 12px; line-height: 1.5;">
                    if (*logs).is_empty() {
                        <div style="color: #475569;">{ "Brak logów..." }</div>
                    } else {
                        { for (*logs).iter().map(|log| html! { <div>{ log }</div> }) }
                    }
                </pre>
            </section>
        </div>
    }
}

