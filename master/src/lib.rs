use std::time::Duration;

use futures::StreamExt;
use gloo_net::websocket::{Message, futures::WebSocket};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::prelude::*;

pub struct ScoreboardComponent {
    home_score: u16,
    away_score: u16,
    state: String,
    regulation_millis: u64,
    quarter: Option<String>,
    controls_hidden: bool,
    connected: bool,
}

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
        if first_render {
            let link = ctx.link().clone();

            spawn_local(async move {
                let window = web_sys::window().unwrap();
                let host = window
                    .location()
                    .host()
                    .unwrap_or_else(|_| "localhost:8080".into());
                let ws_url = format!("ws://{}/ws", host);

                loop {
                    if let Ok(ws) = WebSocket::open(&ws_url) {
                        let (_, mut read) = ws.split();
                        while let Some(Ok(Message::Text(text))) = read.next().await {
                            if let Ok(data) = serde_json::from_str::<Snapshot>(&text) {
                                link.send_message(Msg::UpdateSnapshot(data));
                            }
                        }
                    }

                    link.send_message(Msg::ConnectionFailed);
                    sleep(Duration::from_secs(3)).await;
                }
            });
        }
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

        let state_mapped = match self.state.as_str() {
            "Idle" => "Oczekuje",
            "Running" => "Trwa",
            "Paused" => "Pauza",
            "Stopped" => "Zatrzymany",
            "Ended" => "Koniec meczu",
            other => other,
        };

        let total_seconds = self.regulation_millis / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        let clock_str = format!("{:02}:{:02}", minutes, seconds);

        let send_score = move |team: &'static str, points: u32| {
            spawn_local(async move {
                let body = serde_json::json!({
                    "team": team,
                    "points": points
                });
                let _ = gloo_net::http::Request::post("/api/score")
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .unwrap()
                    .send()
                    .await;
            });
        };

        let toggle_match_state = {
            move |_| {
                spawn_local(async move {
                    let action = if is_running { "stop" } else { "start" };
                    let _ = gloo_net::http::Request::post(&format!("/api/{}", action))
                        .send()
                        .await;
                });
            }
        };

        let on_toggle_controls = ctx.link().callback(|_| Msg::ToggleControls);
        let quarter_display = self.quarter.as_deref().unwrap_or("Q1");

        html! {
            <div style="font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; text-align: center; margin: 0; padding: 30px; background: #f8fafc; color: #0f172a; display: flex; justify-content: center; flex-direction: column; align-items: center; gap: 20px;">
                <main style="display: flex; width: 100%; max-width: 600px; flex-direction: column; border: 1px solid #cbd5e1; background-color: #f8fafc; overflow: hidden; text-align: left; border-radius: 8px; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);">
                    <aside style="background-color: #0f172a; padding: 8px 12px; display: flex; justify-content: space-between; align-items: center;">
                        <p style="font-size: 12px; color: #f8fafc; margin: 0; font-family: monospace;">{"scoreboard@1.0.0"}</p>
                        <button onclick={on_toggle_controls} style="background: transparent; border: 1px solid #475569; color: #f8fafc; font-size: 11px; padding: 2px 8px; border-radius: 4px; cursor: pointer;">
                            {"Przełącz podgląd / kontrolki"}
                        </button>
                    </aside>

                    <header style="margin: 24px 32px 12px 32px; display: flex; flex-direction: column; gap: 8px;">
                        <h1 style="font-size: 24px; color: #0f172a; margin: 0; text-transform: lowercase; font-weight: 600;">{"tablica wyników"}</h1>
                        <p style="font-size: 14px; color: #64748b; margin: 0;">{"zarządzaj stanem oraz punktacją meczu w czasie rzeczywistym"}</p>
                    </header>

                    <div style="margin: 0 32px 20px 32px; background: #ffffff; border: 1px solid #cbd5e1; padding: 16px; border-radius: 6px;">
                        <div style="font-size: 14px; margin-bottom: 12px; color: #334155; text-transform: uppercase; font-weight: 500;">
                            {"Stan: "}<span id="state">{ state_mapped }</span>
                        </div>

                        <div style="display: flex; align-items: stretch; height: 54px; background: #ffffff; border: 1px solid #94a3b8; border-radius: 4px; overflow: hidden; font-weight: 700;">
                            <div style="background-color: #0b2545; color: #ffffff; display: flex; align-items: center; padding: 0 16px; gap: 16px; flex: 1;">
                                <span style="font-size: 20px; letter-spacing: 1px;">{"HO"}</span>
                                <span style="font-size: 26px; margin-left: auto; font-family: monospace;" id="home">{ self.home_score }</span>
                            </div>
                            <div style="background-color: #ffffff; color: #0f172a; display: flex; align-items: center; padding: 0 16px; gap: 16px; flex: 1; border-left: 1px solid #cbd5e1; border-right: 1px solid #cbd5e1;">
                                <span style="font-size: 20px; letter-spacing: 1px;">{"AW"}</span>
                                <span style="font-size: 26px; margin-left: auto; font-family: monospace;" id="away">{ self.away_score }</span>
                            </div>
                            <div style="background-color: #0b132b; color: #ffffff; display: flex; align-items: center; padding: 0 16px; gap: 10px;">
                                <span style="font-size: 14px; color: #cbd5e1;" id="quarter">{ quarter_display }</span>
                                <div style="background-color: #000000; border: 1px solid #334155; border-radius: 20px; padding: 4px 12px; font-size: 20px; font-family: monospace; color: #facc15; letter-spacing: 1px;" id="clock">
                                    { clock_str }
                                </div>
                            </div>
                        </div>
                    </div>
                </main>

                <div id="controls-container" style={format!("width: 100%; max-width: 600px; display: {}; flex-direction: column; gap: 16px; transition: all 0.3s ease;", if self.controls_hidden { "none" } else { "flex" })}>
                    <div style="background: #ffffff; border: 1px solid #cbd5e1; padding: 20px; border-radius: 8px; display: flex; gap: 16px; justify-content: space-between; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);">
                        <div style="display: flex; flex-direction: column; gap: 8px; flex: 1;">
                            <div style="font-size: 12px; color: #64748b; font-weight: bold;">{"DRUŻYNA DOMOWA (HO)"}</div>
                            <button onclick={move |_| send_score("home", 1)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+1 PKT"}</button>
                            <button onclick={move |_| send_score("home", 2)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+2 PKT"}</button>
                            <button onclick={move |_| send_score("home", 3)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+3 PKT"}</button>
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 8px; flex: 1;">
                            <div style="font-size: 12px; color: #64748b; font-weight: bold;">{"DRUŻYNA GOŚCI (AW)"}</div>
                            <button onclick={move |_| send_score("away", 1)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+1 PKT"}</button>
                            <button onclick={move |_| send_score("away", 2)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+2 PKT"}</button>
                            <button onclick={move |_| send_score("away", 3)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+3 PKT"}</button>
                        </div>
                    </div>

                    <div style="background: #ffffff; border: 1px solid #cbd5e1; padding: 16px; border-radius: 8px; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);">
                        <button
                            id="toggle-btn"
                            style={format!("width: 100%; padding: 10px 16px; font-size: 14px; font-weight: 600; border: none; cursor: pointer; color: #ffffff; border-radius: 4px; background-color: {};", if is_running { "#e11d48" } else { "#0f172a" })}
                            onclick={toggle_match_state}
                        >
                            {if is_running { "STOP" } else { "START" }}
                        </button>
                    </div>
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