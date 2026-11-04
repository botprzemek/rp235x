//! Clean Leptos Scoreboard banner components with native macOS aesthetics.

pub mod center_display;
pub mod clock_stack;
pub mod team_panel;
pub mod team_size;
pub mod timeouts;

use leptos::prelude::*;

#[component]
pub fn Scoreboard(
    #[prop(into, default = "HOME".into())] home_team: String,
    #[prop(into, default = "AWAY".into())] away_team: String,
    #[prop(default = 139)] home_score: u16,
    #[prop(default = 127)] away_score: u16,
    #[prop(into, default = "02:56".into())] time: String,
    #[prop(default = 24)] shot_clock: u8,
    #[prop(into, default = "4Q".into())] quarter: String,
    #[prop(default = 2)] home_timeouts: u8,
    #[prop(default = 4)] away_timeouts: u8,
    #[prop(default = 5)] home_timeout_dashes: u8,
    #[prop(default = 5)] away_timeout_dashes: u8,
) -> impl IntoView {
    view! {
        <div class="h-full flex items-center justify-center p-3 select-none">
            <div class="relative bg-zinc-900/65 text-white rounded-2xl p-5 border border-white/10 flex items-center gap-7 backdrop-blur-xl shadow-xl">
                // Home Team
                <div class="flex flex-col items-center min-w-[70px]">
                    <span class="text-[11px] uppercase text-zinc-400 font-medium mb-1">{home_team}</span>
                    <span class="text-4xl font-semibold tabular-nums text-amber-400">{home_score}</span>
                    <div class="flex gap-1 mt-2">
                        {(0..home_timeout_dashes).map(|i| {
                            let active = i < home_timeouts;
                            view! {
                                <div class=format!("w-2.5 h-1 rounded-full {}", if active { "bg-amber-400" } else { "bg-zinc-700" })/>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Center Clock & Quarter
                <div class="flex flex-col items-center border-x border-white/10 px-6 min-w-[140px]">
                    <span class="text-[11px] font-mono font-medium text-zinc-400 mb-0.5">{quarter}</span>
                    <span class="text-3xl font-mono font-semibold tabular-nums text-white mb-2">{time}</span>
                    <div class="flex items-center gap-1.5">
                        <span class="text-[10px] uppercase font-medium text-zinc-500">{"Shot Clock"}</span>
                        <span class="text-[11px] font-mono tabular-nums font-medium text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded-[4px] border border-amber-500/20">{shot_clock}</span>
                    </div>
                </div>

                // Away Team
                <div class="flex flex-col items-center min-w-[70px]">
                    <span class="text-[11px] uppercase text-zinc-400 font-medium mb-1">{away_team}</span>
                    <span class="text-4xl font-semibold tabular-nums text-cyan-400">{away_score}</span>
                    <div class="flex gap-1 mt-2">
                        {(0..away_timeout_dashes).map(|i| {
                            let active = i < away_timeouts;
                            view! {
                                <div class=format!("w-2.5 h-1 rounded-full {}", if active { "bg-cyan-400" } else { "bg-zinc-700" })/>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </div>
    }
}
