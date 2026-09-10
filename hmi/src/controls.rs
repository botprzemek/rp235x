use net::data::snapshot::Team;
use yew::platform::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ControlsProps {
    pub is_running: bool,
}

#[function_component]
pub fn ScoreboardControls(props: &ControlsProps) -> Html {
    let send_score = move |team: Team, points: u16| {
        spawn_local(async move {
            let body = serde_json::json!({ "team": team, "points": points });
            let _ = gloo_net::http::Request::post("/api/score")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await;
        });
    };

    let is_running = props.is_running;
    let toggle_match_state = move |_| {
        spawn_local(async move {
            let action = if is_running { "stop" } else { "start" };
            let _ = gloo_net::http::Request::post(&format!("/api/{}", action))
                .send()
                .await;
        });
    };

    html! {
        <div style="display: flex; flex-direction: column; gap: 16px; width: 100%;">
            <div style="background: #ffffff; border: 1px solid #cbd5e1; padding: 20px; border-radius: 8px; display: flex; gap: 16px; justify-content: space-between; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);">
                <div style="display: flex; flex-direction: column; gap: 8px; flex: 1;">
                    <div style="font-size: 12px; color: #64748b; font-weight: bold;">{"DRUŻYNA DOMOWA (HO)"}</div>
                    <button onclick={move |_| send_score(Team::Home, 1)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+1 PKT"}</button>
                    <button onclick={move |_| send_score(Team::Home, 2)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+2 PKT"}</button>
                    <button onclick={move |_| send_score(Team::Home, 3)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+3 PKT"}</button>
                </div>
                <div style="display: flex; flex-direction: column; gap: 8px; flex: 1;">
                    <div style="font-size: 12px; color: #64748b; font-weight: bold;">{"DRUŻYNA GOŚCI (AW)"}</div>
                    <button onclick={move |_| send_score(Team::Away, 1)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+1 PKT"}</button>
                    <button onclick={move |_| send_score(Team::Away, 2)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+2 PKT"}</button>
                    <button onclick={move |_| send_score(Team::Away, 3)} style="padding: 6px 12px; font-size: 14px; font-weight: 500; cursor: pointer; border: 1px solid #cbd5e1; background: #ffffff; color: #334155; border-radius: 4px;">{"+3 PKT"}</button>
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
    }
}
