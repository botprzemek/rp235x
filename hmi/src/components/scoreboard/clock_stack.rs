//! Center match clock, shot clock, and quarter stack component.

use yew::{AttrValue, Html, Properties, function_component, html};

/// Properties for [`ClockStack`].
#[derive(Properties, PartialEq, Clone)]
pub struct ClockStackProps {
    pub time: AttrValue,
    pub shot_clock: u8,
    pub quarter: AttrValue,
}

/// Center stack displaying game clock, shot clock, and current quarter.
#[function_component(ClockStack)]
pub fn clock_stack(props: &ClockStackProps) -> Html {
    html! {
        <div class="flex flex-col items-center justify-center font-alt">
            <div class="text-scoreboard-accent font-bold text-3xl leading-none">
                { &props.time }
            </div>
            <div class="flex items-center justify-between w-full space-x-3 text-scoreboard-alt font-bold text-xl leading-none">
                <span>{ props.shot_clock }</span>
                <span>{ &props.quarter }</span>
            </div>
        </div>
    }
}
