use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ScoreboardProps {
    #[prop_or_else(|| AttrValue::from("LA LAKERS"))]
    pub home_team: AttrValue,
    #[prop_or_else(|| AttrValue::from("GS WARRIORS"))]
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

#[function_component(Scoreboard)]
pub fn scoreboard(props: &ScoreboardProps) -> Html {
    html! {
        <div
            class="w-full bg-[#ff5522] rounded-[24px] p-2 sm:p-2.5 flex items-stretch justify-between shadow-xl select-none my-3 text-white"
            style="background-color: #ff5522; border-radius: 24px; padding: 10px; display: flex; align-items: stretch; justify-content: space-between; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.2), 0 4px 6px -4px rgba(0,0,0,0.2); user-select: none; margin: 16px 0; width: 100%; box-sizing: border-box;"
        >
            // LEFT TEAM BOX: LA LAKERS
            <div
                class="bg-[#1d2125] rounded-[16px] px-4 py-2 sm:px-5 sm:py-3 flex flex-col justify-between shrink-0"
                style="background-color: #1d2125; border-radius: 16px; padding: 10px 18px; display: flex; flex-direction: column; justify-content: space-between; min-width: 155px;"
            >
                <div
                    class="text-white font-black text-base sm:text-lg tracking-wider uppercase leading-tight"
                    style="color: #ffffff; font-weight: 900; font-size: 18px; letter-spacing: 0.05em; text-transform: uppercase; line-height: 1.1;"
                >
                    { &props.home_team }
                </div>
                <div
                    class="flex items-center space-x-2.5 pt-1.5"
                    style="display: flex; align-items: center; gap: 8px; padding-top: 6px;"
                >
                    <span
                        class="text-[#ff5522] font-black text-xs sm:text-sm tracking-wider"
                        style="color: #ff5522; font-weight: 900; font-size: 13px; letter-spacing: 0.05em;"
                    >
                        { format!("{} TO", props.home_timeouts) }
                    </span>
                    <div
                        class="flex items-center space-x-1"
                        style="display: flex; align-items: center; gap: 4px;"
                    >
                        { for (0..props.home_timeout_dashes).map(|_| html! {
                            <span
                                class="w-3.5 sm:w-4 h-[3px] bg-[#ff5522] rounded-full inline-block"
                                style="width: 14px; height: 3px; background-color: #ff5522; border-radius: 9999px; display: inline-block;"
                            ></span>
                        }) }
                    </div>
                </div>
            </div>

            // CENTER: 139  02:56 / 24 4Q  127
            <div
                class="flex-1 flex items-center justify-center px-2 sm:px-4 space-x-3 sm:space-x-5"
                style="flex: 1; display: flex; align-items: center; justify-content: center; padding: 0 16px; gap: 18px;"
            >
                // Home Score (139)
                <div
                    class="text-white font-black text-4xl sm:text-5xl tracking-tight leading-none"
                    style="color: #ffffff; font-weight: 900; font-size: 50px; line-height: 1; letter-spacing: -0.03em;"
                >
                    { props.home_score }
                </div>

                // Clock & Shot/Quarter Stack
                <div
                    class="flex flex-col items-center justify-center space-y-0.5 px-1"
                    style="display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 1px; padding: 0 4px;"
                >
                    // Game Clock (02:56)
                    <div
                        class="text-white font-black text-xl sm:text-2xl leading-none tracking-tight"
                        style="color: #ffffff; font-weight: 900; font-size: 26px; line-height: 1; letter-spacing: -0.01em;"
                    >
                        { &props.time }
                    </div>
                    // Shot Clock (24) & Quarter (4Q) in dark black/charcoal #1d2125
                    <div
                        class="flex items-center justify-between w-full space-x-3 text-[#1d2125] font-black text-sm sm:text-base leading-none tracking-wider"
                        style="display: flex; align-items: center; justify-content: space-between; width: 100%; gap: 12px; color: #1d2125; font-weight: 900; font-size: 15px; line-height: 1; letter-spacing: 0.05em;"
                    >
                        <span>{ props.shot_clock }</span>
                        <span>{ &props.quarter }</span>
                    </div>
                </div>

                // Away Score (127)
                <div
                    class="text-white font-black text-4xl sm:text-5xl tracking-tight leading-none"
                    style="color: #ffffff; font-weight: 900; font-size: 50px; line-height: 1; letter-spacing: -0.03em;"
                >
                    { props.away_score }
                </div>
            </div>

            // RIGHT TEAM BOX: GS WARRIORS
            <div
                class="bg-[#1d2125] rounded-[16px] px-4 py-2 sm:px-5 sm:py-3 flex flex-col justify-between items-end shrink-0"
                style="background-color: #1d2125; border-radius: 16px; padding: 10px 18px; display: flex; flex-direction: column; justify-content: space-between; align-items: flex-end; min-width: 155px;"
            >
                <div
                    class="text-white font-black text-base sm:text-lg tracking-wider uppercase leading-tight text-right"
                    style="color: #ffffff; font-weight: 900; font-size: 18px; letter-spacing: 0.05em; text-transform: uppercase; line-height: 1.1; text-align: right;"
                >
                    { &props.away_team }
                </div>
                <div
                    class="flex items-center space-x-2.5 pt-1.5"
                    style="display: flex; align-items: center; gap: 8px; padding-top: 6px;"
                >
                    <div
                        class="flex items-center space-x-1"
                        style="display: flex; align-items: center; gap: 4px;"
                    >
                        { for (0..props.away_timeout_dashes).map(|_| html! {
                            <span
                                class="w-3.5 sm:w-4 h-[3px] bg-[#ff5522] rounded-full inline-block"
                                style="width: 14px; height: 3px; background-color: #ff5522; border-radius: 9999px; display: inline-block;"
                            ></span>
                        }) }
                    </div>
                    <span
                        class="text-[#ff5522] font-black text-xs sm:text-sm tracking-wider"
                        style="color: #ff5522; font-weight: 900; font-size: 13px; letter-spacing: 0.05em;"
                    >
                        { format!("{} TO", props.away_timeouts) }
                    </span>
                </div>
            </div>
        </div>
    }
}
