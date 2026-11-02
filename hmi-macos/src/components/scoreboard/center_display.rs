//! Center score and clock area component.

use yew::{AttrValue, Html, Properties, function_component, html};

use crate::components::scoreboard::ClockStack;

/// Properties for [`CenterDisplay`].
#[derive(Properties, PartialEq, Clone)]
pub struct CenterDisplayProps {
    pub home_score: u16,
    pub away_score: u16,
    pub time: AttrValue,
    pub shot_clock: u8,
    pub quarter: AttrValue,
}

/// Center column of the scoreboard displaying scores and game time.
#[function_component(CenterDisplay)]
pub fn center_display(props: &CenterDisplayProps) -> Html {
    html! {
        <div class="grid grid-cols-[6fr_5fr_6fr] items-center font-alt">
            <div class="text-scoreboard-accent font-black text-4xl sm:text-5xl text-left">
                { props.home_score }
            </div>

            <div class="flex justify-center">
                <ClockStack
                    time={props.time.clone()}
                    shot_clock={props.shot_clock}
                    quarter={props.quarter.clone()}
                />
            </div>

            <div class="text-scoreboard-accent font-black text-4xl sm:text-5xl text-right">
                { props.away_score }
            </div>
        </div>
    }
}
