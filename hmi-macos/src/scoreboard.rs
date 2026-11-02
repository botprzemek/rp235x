//! Scoreboard banner Yew component.

use yew::{AttrValue, Html, Properties, function_component, html};

use crate::components::scoreboard::{CenterDisplay, TeamPanel, TeamSide};

/// Properties for the [`Scoreboard`] component.
#[derive(Properties, PartialEq, Clone)]
pub struct ScoreboardProps {
    #[prop_or_else(|| AttrValue::from("HOME"))]
    pub home_team: AttrValue,
    #[prop_or_else(|| AttrValue::from("AWAY"))]
    pub away_team: AttrValue,
    #[prop_or(139)]
    pub home_score: u16,
    #[prop_or(127)]
    pub away_score: u16,
    #[prop_or_else(|| AttrValue::from("02:56"))]
    pub time: AttrValue,
    #[prop_or(24)]
    pub shot_clock: u8,
    #[prop_or_else(|| AttrValue::from("4Q"))]
    pub quarter: AttrValue,
    #[prop_or(2)]
    pub home_timeouts: u8,
    #[prop_or(4)]
    pub away_timeouts: u8,
    #[prop_or(5)]
    pub home_timeout_dashes: u8,
    #[prop_or(5)]
    pub away_timeout_dashes: u8,
}

impl Default for ScoreboardProps {
    fn default() -> Self {
        Self {
            home_team: AttrValue::from("LA LAKERS"),
            away_team: AttrValue::from("GS WARRIORS"),
            home_score: 139,
            away_score: 127,
            time: AttrValue::from("02:56"),
            shot_clock: 24,
            quarter: AttrValue::from("4Q"),
            home_timeouts: 2,
            away_timeouts: 4,
            home_timeout_dashes: 5,
            away_timeout_dashes: 5,
        }
    }
}

/// Broadcast-style basketball scoreboard component.
#[function_component(Scoreboard)]
pub fn scoreboard(props: &ScoreboardProps) -> Html {
    html! {
        // <div class="w-fit grid grid-rows-2 gap-4">
            // <img src="/scoreboard.png"/>
            <div class="bg-scoreboard-bg rounded-[20px] p-2 grid grid-cols-3 items-stretch select-none gap-3 text-scoreboard-accent">
                <TeamPanel
                    side={TeamSide::Home}
                    name={props.home_team.clone()}
                    timeouts={props.home_timeouts}
                    timeout_dashes={props.home_timeout_dashes}
                />

                <CenterDisplay
                    home_score={props.home_score}
                    away_score={props.away_score}
                    time={props.time.clone()}
                    shot_clock={props.shot_clock}
                    quarter={props.quarter.clone()}
                />

                <TeamPanel
                    side={TeamSide::Away}
                    name={props.away_team.clone()}
                    timeouts={props.away_timeouts}
                    timeout_dashes={props.away_timeout_dashes}
                />
            </div>
        // </div>
    }
}
