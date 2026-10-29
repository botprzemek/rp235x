//! Team banner panel with name and timeouts.

use yew::{AttrValue, Html, Properties, function_component, html};

use crate::components::scoreboard::TeamSide;
use crate::components::scoreboard::Timeouts;

/// Properties for [`TeamPanel`].
#[derive(Properties, PartialEq, Clone)]
pub struct TeamPanelProps {
    pub side: TeamSide,
    pub name: AttrValue,
    pub timeouts: u8,
    pub timeout_dashes: u8,
}

/// Left or right team panel on the scoreboard.
#[function_component(TeamPanel)]
pub fn team_panel(props: &TeamPanelProps) -> Html {
    let (rounded_class, align_class, text_align_class) = match props.side {
        TeamSide::Home => ("rounded-l-[16px] rounded-r-none", "", ""),
        TeamSide::Away => ("rounded-r-[16px] rounded-l-none", "items-end", "text-right"),
    };

    html! {
        <div class={format!("bg-scoreboard-alt {rounded_class} px-4 py-2 sm:px-5 sm:py-3 flex flex-col justify-between {align_class}")}>
            <div class={format!("text-scoreboard-accent font-bold text-xl font-alt uppercase {text_align_class}")}>
                { &props.name }
            </div>
            <Timeouts
                    side={TeamSide::Home}
                    count={props.timeouts} total={props.timeout_dashes} />
        </div>
    }
}
