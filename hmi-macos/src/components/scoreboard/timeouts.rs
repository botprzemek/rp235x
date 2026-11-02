//! Timeout indicator dots Yew component.

use yew::{Html, Properties, function_component, html};

use crate::components::scoreboard::TeamSide;

/// Properties for [`Timeouts`].
#[derive(Properties, PartialEq, Clone)]
pub struct TimeoutsProps {
    pub side: TeamSide,
    pub count: u8,
    pub total: u8,
}

/// Displays the timeouts left count and indicator dashes.
#[function_component(Timeouts)]
pub fn timeouts(props: &TimeoutsProps) -> Html {
    let indicators = (0..props.total).map(|_| {
        html! {
            <span class="w-3.5 sm:w-4 h-[2px] bg-scoreboard-bg inline-block"></span>
        }
    });

    let justify_class = match props.side {
        TeamSide::Home => "justify-start",
        TeamSide::Away => "justify-end",
    };

    html! {
        <div class={format!("flex items-center {justify_class} space-x-2.5 pt-1.5 font-alt")}>
            <span class="text-scoreboard-bg font-regular text-base">
                { format!("{} TO", props.count) }
            </span>
            <div class="flex space-x-1">
                { for indicators }
            </div>
        </div>
    }
}
