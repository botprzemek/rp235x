use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ScoreDisplayProps {
    pub home_score: u16,
    pub away_score: u16,
    pub state: String,
    pub regulation_millis: u64,
    pub quarter: Option<String>,
}

#[function_component]
pub fn ScoreDisplay(props: &ScoreDisplayProps) -> Html {
    let state_mapped = match props.state.as_str() {
        "Idle" => "Oczekuje",
        "Running" => "Trwa",
        "Paused" => "Pauza",
        "Stopped" => "Zatrzymany",
        "Ended" => "Koniec meczu",
        other => other,
    };

    let total_seconds = props.regulation_millis / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let clock_str = format!("{:02}:{:02}", minutes, seconds);
    let quarter_display = props.quarter.as_deref().unwrap_or("Q1");

    html! {
        <div style="margin: 0 32px 20px 32px; background: #ffffff; border: 1px solid #cbd5e1; padding: 16px; border-radius: 6px;">
            <div style="font-size: 14px; margin-bottom: 12px; color: #334155; text-transform: uppercase; font-weight: 500;">
                {"Stan: "}<span id="state">{ state_mapped }</span>
            </div>

            <div style="display: flex; align-items: stretch; height: 54px; background: #ffffff; border: 1px solid #94a3b8; border-radius: 4px; overflow: hidden; font-weight: 700;">
                <div style="background-color: #0b2545; color: #ffffff; display: flex; align-items: center; padding: 0 16px; gap: 16px; flex: 1;">
                    <span style="font-size: 20px; letter-spacing: 1px;">{"HO"}</span>
                    <span style="font-size: 26px; margin-left: auto; font-family: monospace;" id="home">{ props.home_score }</span>
                </div>
                <div style="background-color: #ffffff; color: #0f172a; display: flex; align-items: center; padding: 0 16px; gap: 16px; flex: 1; border-left: 1px solid #cbd5e1; border-right: 1px solid #cbd5e1;">
                    <span style="font-size: 20px; letter-spacing: 1px;">{"AW"}</span>
                    <span style="font-size: 26px; margin-left: auto; font-family: monospace;" id="away">{ props.away_score }</span>
                </div>
                <div style="background-color: #0b132b; color: #ffffff; display: flex; align-items: center; padding: 0 16px; gap: 10px;">
                    <span style="font-size: 14px; color: #cbd5e1;" id="quarter">{ quarter_display }</span>
                    <div style="background-color: #000000; border: 1px solid #334155; border-radius: 20px; padding: 4px 12px; font-size: 20px; font-family: monospace; color: #facc15; letter-spacing: 1px;" id="clock">
                        { clock_str }
                    </div>
                </div>
            </div>
        </div>
    }
}
