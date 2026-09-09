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
        <div style="margin: 0 16px 16px 16px; background: #ffffff; border: 1px solid #cbd5e1; padding: 12px; border-radius: 6px;">
            <div style="font-size: 13px; margin-bottom: 8px; color: #334155; text-transform: uppercase; font-weight: 500; text-align: left;">
                {"Stan: "}<span id="state">{ state_mapped }</span>
            </div>

            <div style="display: flex; flex-direction: column; gap: 8px; background: #ffffff; border: 1px solid #94a3b8; border-radius: 4px; padding: 8px; font-weight: 700;">
                <div style="display: flex; justify-content: space-between; align-items: center; background-color: #0b2545; color: #ffffff; padding: 10px 14px; border-radius: 4px;">
                    <span style="font-size: 16px; letter-spacing: 1px;">{"HO"}</span>
                    <span style="font-size: 24px; font-family: monospace;" id="home">{ props.home_score }</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; background-color: #f1f5f9; color: #0f172a; padding: 10px 14px; border-radius: 4px; border: 1px solid #cbd5e1;">
                    <span style="font-size: 16px; letter-spacing: 1px;">{"AW"}</span>
                    <span style="font-size: 24px; font-family: monospace;" id="away">{ props.away_score }</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; background-color: #0b132b; color: #ffffff; padding: 8px 14px; border-radius: 4px;">
                    <span style="font-size: 13px; color: #cbd5e1;" id="quarter">{ quarter_display }</span>
                    <div style="background-color: #000000; border: 1px solid #334155; border-radius: 16px; padding: 4px 12px; font-size: 18px; font-family: monospace; color: #facc15; letter-spacing: 1px;" id="clock">
                        { clock_str }
                    </div>
                </div>
            </div>
        </div>
    }
}
