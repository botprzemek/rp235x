use basketball::Discipline;
use basketball::Game;
use net::ClientEvent;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::js_sys;
use web_sys::wasm_bindgen;
use web_sys::{MessageEvent, WebSocket};
use yew::prelude::*;

// Importujemy stałe i pakiety z Waszej współdzielonej biblioteki sieciowej
use net::layout::PACKET_SIZE;
use net::{ClientPacket, ServerPacket};

#[function_component(App)]
pub fn app() -> Html {
    let status = use_state(|| "Rozłączony".to_string());
    let status_color = use_state(|| "red".to_string());
    let is_connected = use_state(|| false);
    let ws_ref = use_state(|| None::<WebSocket>);
    let logs = use_state(|| Vec::<String>::new());

    // Przechowujemy ostatnio odebrany surowy pakiet (zgodny z PACKET_SIZE)
    let last_packet = use_state(|| None::<[u8; PACKET_SIZE]>);

    let add_log = {
        let logs = logs.clone();
        move |msg: &str| {
            let mut current = (*logs).clone();
            current.push(msg.to_string());
            if current.len() > 50 {
                current.remove(0); // Ograniczamy liczbę logów
            }
            logs.set(current);
        }
    };

    // Obsługa przycisku "Połącz"
    let connect = {
        let status = status.clone();
        let status_color = status_color.clone();
        let is_connected = is_connected.clone();
        let ws_ref = ws_ref.clone();
        let last_packet = last_packet.clone();
        let add_log = add_log.clone();

        Callback::from(move |_| {
            let ws = match WebSocket::new("ws://127.0.0.1:3000") {
                Ok(w) => w,
                Err(e) => {
                    add_log(&format!("Błąd tworzenia WebSocket: {:?}", e));
                    return;
                }
            };

            // WAŻNE: Wymuszamy odbieranie danych jako ArrayBuffer (surowe bajty)
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            // Callback onopen
            let status_c = status.clone();
            let color_c = status_color.clone();
            let conn_c = is_connected.clone();
            let log_c = add_log.clone();
            let onopen = Closure::wrap(Box::new(move |_e: JsValue| {
                status_c.set("Połączono!".to_string());
                color_c.set("green".to_string());
                conn_c.set(true);
                log_c("Nawiązano połączenie WebSocket.");
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();

            // Callback onmessage (Odbieranie surowych bajtów i mapowanie na ServerPacket)
            let log_c = add_log.clone();
            let packet_c = last_packet.clone();
            let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let array = js_sys::Uint8Array::new(&buf);
                    if array.length() as usize == PACKET_SIZE {
                        let mut data_arr = [0u8; PACKET_SIZE];
                        array.copy_to(&mut data_arr);

                        // Tutaj możemy bezpośrednio przepuścić bajty przez Waszą logikę paczek!
                        // let server_packet = ServerPacket::from_bytes(&data_arr);

                        packet_c.set(Some(data_arr));
                        log_c(&format!("[ODEBRANO] Pakiet ${} bajtów", PACKET_SIZE));
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            // Callback onclose
            let status_c = status.clone();
            let color_c = status_color.clone();
            let conn_c = is_connected.clone();
            let log_c = add_log.clone();
            let onclose = Closure::wrap(Box::new(move |_e: JsValue| {
                status_c.set("Rozłączony".to_string());
                color_c.set("red".to_string());
                conn_c.set(false);
                log_c("Połączenie zostało zamknięte.");
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
            onclose.forget();

            ws_ref.set(Some(ws));
        })
    };

    // Obsługa wysyłania testowego pakietu (z uśpionym ostanim pakietem lub nowym ClientPacket)
    let send_packet = {
        let ws_ref = ws_ref.clone();
        let add_log = add_log.clone();

        Callback::from(move |_| {
            if let Some(ws) = (*ws_ref).as_ref() {
                let game = Game::new(Discipline::Fiba5vs5);
                let packet = ClientPacket::new(ClientEvent::HandshakeRequest, 0, game.to_bytes());

                let uint8_array = js_sys::Uint8Array::from(&packet.to_bytes()[..]);

                if let Err(e) = ws.send_with_array_buffer(&uint8_array.buffer()) {
                    add_log(&format!("Błąd wysyłania: {:?}", e));
                } else {
                    add_log("[WYSŁANO] Odesłano pakiet binarny do serwera.");
                }
            }
        })
    };

    html! {
        <div style="font-family: sans-serif; max-width: 600px; margin: 40px auto; padding: 20px;">
            <h2>{ "Panel HMI (Yew / Rust) - WebSocket" }</h2>

            <div style="border: 1px solid #ccc; padding: 15px; margin-bottom: 10px; border-radius: 5px;">
                <h3>{ "Status połączenia: " }<span style={format!("color: {};", *status_color)}>{ &*status }</span></h3>
                <button disabled={*is_connected} onclick={connect} style="padding: 10px 15px; cursor: pointer; margin-right: 10px;">{ "Połącz" }</button>
                <button disabled={!*is_connected} onclick={send_packet} style="padding: 10px 15px; cursor: pointer;">{ "Wyślij pakiet" }</button>
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
