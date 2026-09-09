mod controls;
mod display;
mod navigation;

use futures::StreamExt;
use gloo_net::websocket::{Message, futures::WebSocket};
use std::time::Duration;
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::prelude::*;

use controls::ScoreboardControls;
use display::ScoreDisplay;
use navigation::Navbar;

pub enum Msg {
    UpdateSnapshot(Snapshot),
    ConnectionFailed,
    ToggleControls,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Snapshot {
    pub home_score: u16,
    pub away_score: u16,
    pub state: String,
    #[serde(default)]
    pub regulation_millis: u64,
    #[serde(default)]
    pub quarter: Option<String>,
}

pub struct ScoreboardComponent {
    home_score: u16,
    away_score: u16,
    state: String,
    regulation_millis: u64,
    quarter: Option<String>,
    controls_hidden: bool,
    connected: bool,
}

impl Component for ScoreboardComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            home_score: 0,
            away_score: 0,
            state: "Oczekuje".to_string(),
            regulation_millis: 0,
            quarter: Some("Q1".to_string()),
            controls_hidden: false,
            connected: false,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }
        let link = ctx.link().clone();

        spawn_local(async move {
            let window = web_sys::window().unwrap();
            let host = window
                .location()
                .host()
                .unwrap_or_else(|_| "localhost:8080".into());
            let ws_url = format!("ws://{}/ws", host);

            loop {
                let ws = match WebSocket::open(&ws_url) {
                    Ok(ws) => ws,
                    Err(e) => {
                        println!("{}", e);
                        return;
                    }
                };

                let (_, mut read) = ws.split();
                while let Some(Ok(Message::Text(text))) = read.next().await {
                    if let Ok(data) = serde_json::from_str::<Snapshot>(&text) {
                        link.send_message(Msg::UpdateSnapshot(data));
                    }
                }

                link.send_message(Msg::ConnectionFailed);
                sleep(Duration::from_secs(3)).await;
            }
        });
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateSnapshot(snapshot) => {
                self.home_score = snapshot.home_score;
                self.away_score = snapshot.away_score;
                self.state = snapshot.state;
                self.regulation_millis = snapshot.regulation_millis;
                if let Some(q) = snapshot.quarter {
                    self.quarter = Some(q);
                }
                self.connected = true;
                true
            }
            Msg::ConnectionFailed => {
                self.state = "Rozłączono".to_string();
                self.connected = false;
                true
            }
            Msg::ToggleControls => {
                self.controls_hidden = !self.controls_hidden;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_running = self.state == "Running";
        let on_toggle_controls = ctx.link().callback(|_| Msg::ToggleControls);

        html! {
            <div style="font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; text-align: center; margin: 0; padding: 30px; background: #f8fafc; color: #0f172a; display: flex; justify-content: center; flex-direction: column; align-items: center; gap: 20px;">
                <main style="display: flex; width: 100%; max-width: 600px; flex-direction: column; border: 1px solid #cbd5e1; background-color: #f8fafc; overflow: hidden; text-align: left; border-radius: 8px; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);">
                    <Navbar on_toggle_controls={on_toggle_controls} />

                    <header style="margin: 24px 32px 12px 32px; display: flex; flex-direction: column; gap: 8px;">
                        <h1 style="font-size: 24px; color: #0f172a; margin: 0; text-transform: lowercase; font-weight: 600;">{"tablica wyników"}</h1>
                        <p style="font-size: 14px; color: #64748b; margin: 0;">{"zarządzaj stanem oraz punktacją meczu w czasie rzeczywistym"}</p>
                    </header>

                    <ScoreDisplay
                        home_score={self.home_score}
                        away_score={self.away_score}
                        state={self.state.clone()}
                        regulation_millis={self.regulation_millis}
                        quarter={self.quarter.clone()}
                    />
                </main>

                <div id="controls-container" style={format!("width: 100%; max-width: 600px; display: {}; flex-direction: column; gap: 16px; transition: all 0.3s ease;", if self.controls_hidden { "none" } else { "flex" })}>
                    <ScoreboardControls is_running={is_running} />
                </div>
            </div>
        }
    }
}

#[function_component]
pub fn App() -> Html {
    html! {
        <ScoreboardComponent />
    }
}
