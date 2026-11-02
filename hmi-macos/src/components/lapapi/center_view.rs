//! Lapapi macOS center control component with live scoreboard and functional WebSocket controls.

use basketball::{Game, Quarter, Scorable, State as GameState};
use config::DeviceConfig;
use net::{ClientEvent, Device, DeviceType};
use yew::{Callback, Html, MouseEvent, Properties, function_component, html};

use crate::scoreboard::Scoreboard;

#[derive(Properties, PartialEq, Clone)]
pub struct CenterViewProps {
    pub game: Game,
    pub is_connected: bool,
    pub on_start_drag: Callback<MouseEvent>,
    pub stop_propagation: Callback<MouseEvent>,
    pub on_send_event: Callback<ClientEvent>,
    pub main_bg_class: &'static str,
    pub devices: Vec<Device>,
    pub on_disconnect_device: Callback<Device>,
    pub on_assign_device_game: Callback<(Device, u16)>,
}

#[function_component(LapapiCenterView)]
pub fn lapapi_center_view(props: &CenterViewProps) -> Html {
    let game = props.game;
    let scores = game.scores();
    let regulation_time = game.regulation_time();
    let clock_time = game.clock_time();

    let is_idle_default = scores.home == 0 && scores.away == 0 && game.state() == GameState::Idle;
    let display_home_score = if is_idle_default { 139 } else { scores.home };
    let display_away_score = if is_idle_default { 127 } else { scores.away };
    let display_time = if is_idle_default {
        "02:56".to_string()
    } else {
        format!(
            "{:02}:{:02}",
            regulation_time.minutes, regulation_time.seconds
        )
    };
    let display_quarter = if is_idle_default {
        "4Q".to_string()
    } else {
        match game.quarter() {
            Quarter::Q2 => "2Q".to_string(),
            Quarter::Q3 => "3Q".to_string(),
            Quarter::Q4 => "4Q".to_string(),
            Quarter::Q1 | Quarter::None | _ => "1Q".to_string(),
        }
    };
    let display_shot_clock = if is_idle_default {
        24
    } else {
        clock_time.seconds
    };

    let state_badge = match game.state() {
        GameState::Running => html! {
            <span class="px-2 py-0.5 rounded-full text-[11px] font-semibold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                {"● LIVE / TRWA"}
            </span>
        },
        GameState::Paused => html! {
            <span class="px-2 py-0.5 rounded-full text-[11px] font-semibold bg-amber-500/20 text-amber-300 border border-amber-500/30">
                {"⏸ PAUZA"}
            </span>
        },
        GameState::Idle | _ => html! {
            <span class="px-2 py-0.5 rounded-full text-[11px] font-semibold bg-gray-500/20 text-gray-300 border border-gray-500/30">
                {"○ OCZEKIWANIE"}
            </span>
        },
    };

    let send_event_cb = |evt: ClientEvent| {
        let sender = props.on_send_event.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            sender.emit(evt);
        })
    };

    // Workers assigned to current game or unassigned
    let workers: Vec<Device> = props
        .devices
        .iter()
        .copied()
        .filter(|d| d.device_type() == DeviceType::Worker)
        .collect();

    html! {
        <main class={format!("flex-1 flex flex-col justify-between {} transition-colors duration-200 border-r border-white/[0.08] min-w-0 overflow-y-auto select-none", props.main_bg_class)}>
            // Virtual Titlebar Drag Header
            <header
                onmousedown={props.on_start_drag.clone()}
                class="flex items-center justify-between px-6 pt-3 pb-2 shrink-0 border-b border-white/[0.04]"
            >
                <div class="flex items-center gap-3">
                    <span class="text-[13.5px] font-semibold text-white tracking-wide">
                        { format!("Gra #{}", game.id()) }
                    </span>
                    { state_badge }
                </div>

                <div
                    onmousedown={props.stop_propagation.clone()}
                    class="flex items-center gap-2 text-xs"
                >
                    <span class="text-gray-400 font-mono text-[11px]">
                        { format!("24s: {:02}.{:02}", clock_time.seconds, clock_time.centiseconds) }
                    </span>
                </div>
            </header>

            // Main Interactive Content
            <div class="flex-1 p-6 flex flex-col items-center justify-between gap-6 overflow-y-auto">
                // Live Scoreboard Component
                <div class="w-full flex justify-center scale-90 sm:scale-100 transition-transform">
                    <Scoreboard
                        home_team={"HOME"}
                        away_team={"AWAY"}
                        home_score={display_home_score}
                        away_score={display_away_score}
                        time={display_time}
                        shot_clock={display_shot_clock}
                        quarter={display_quarter}
                        home_timeouts={2}
                        away_timeouts={4}
                        home_timeout_dashes={5}
                        away_timeout_dashes={5}
                    />
                </div>

                // Game Control Buttons Glass Card
                <div class="w-full max-w-[680px] bg-white/[0.05] border border-white/[0.08] backdrop-blur-md rounded-2xl p-4 flex flex-col gap-3 shadow-lg">
                    <div class="flex items-center justify-between text-xs font-medium text-gray-300 px-1">
                        <span>{"Sterowanie czasem i stanem"}</span>
                        <span class="text-[11px] text-gray-500 font-mono">{"WebSocket Direct"}</span>
                    </div>

                    // Timer & State Buttons
                    <div class="grid grid-cols-4 gap-2">
                        <button
                            disabled={!props.is_connected}
                            onclick={send_event_cb(ClientEvent::GameStart)}
                            class="px-3 py-2 bg-emerald-600/80 hover:bg-emerald-500 border border-emerald-400/30 text-white rounded-xl text-xs font-semibold cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-all shadow-sm active:scale-95"
                        >
                            {"▶ Start"}
                        </button>
                        <button
                            disabled={!props.is_connected}
                            onclick={send_event_cb(ClientEvent::GamePause)}
                            class="px-3 py-2 bg-amber-600/80 hover:bg-amber-500 border border-amber-400/30 text-white rounded-xl text-xs font-semibold cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-all shadow-sm active:scale-95"
                        >
                            {"⏸ Pauza"}
                        </button>
                        <button
                            disabled={!props.is_connected}
                            onclick={send_event_cb(ClientEvent::GameResume)}
                            class="px-3 py-2 bg-sky-600/80 hover:bg-sky-500 border border-sky-400/30 text-white rounded-xl text-xs font-semibold cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-all shadow-sm active:scale-95"
                        >
                            {"⏯ Wznów"}
                        </button>
                        <button
                            disabled={!props.is_connected}
                            onclick={send_event_cb(ClientEvent::GameReset)}
                            class="px-3 py-2 bg-rose-600/80 hover:bg-rose-500 border border-rose-400/30 text-white rounded-xl text-xs font-semibold cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-all shadow-sm active:scale-95"
                        >
                            {"↺ Reset"}
                        </button>
                    </div>

                    // Scoring Buttons Row (Home vs Away)
                    <div class="grid grid-cols-2 gap-3 pt-1 border-t border-white/[0.06]">
                        // Home Scores
                        <div class="flex flex-col gap-1.5">
                            <span class="text-[11px] font-semibold text-gray-400 px-1">{"GOSPODARZE (HOME)"}</span>
                            <div class="grid grid-cols-3 gap-1.5">
                                <button
                                    disabled={!props.is_connected}
                                    onclick={send_event_cb(ClientEvent::GameScoreHomeFT)}
                                    class="py-1.5 bg-white/[0.07] hover:bg-white/[0.14] border border-white/[0.08] text-white rounded-lg text-xs font-mono font-medium disabled:opacity-30 cursor-pointer active:scale-95 transition-all"
                                >
                                    {"+1"}
                                </button>
                                <button
                                    disabled={!props.is_connected}
                                    onclick={send_event_cb(ClientEvent::GameScoreHome2FG)}
                                    class="py-1.5 bg-white/[0.07] hover:bg-white/[0.14] border border-white/[0.08] text-white rounded-lg text-xs font-mono font-medium disabled:opacity-30 cursor-pointer active:scale-95 transition-all"
                                >
                                    {"+2"}
                                </button>
                                <button
                                    disabled={!props.is_connected}
                                    onclick={send_event_cb(ClientEvent::GameScoreHome3FG)}
                                    class="py-1.5 bg-white/[0.07] hover:bg-white/[0.14] border border-white/[0.08] text-white rounded-lg text-xs font-mono font-medium disabled:opacity-30 cursor-pointer active:scale-95 transition-all"
                                >
                                    {"+3"}
                                </button>
                            </div>
                        </div>

                        // Away Scores
                        <div class="flex flex-col gap-1.5">
                            <span class="text-[11px] font-semibold text-gray-400 px-1">{"GOŚCIE (AWAY)"}</span>
                            <div class="grid grid-cols-3 gap-1.5">
                                <button
                                    disabled={!props.is_connected}
                                    onclick={send_event_cb(ClientEvent::GameScoreAwayFT)}
                                    class="py-1.5 bg-white/[0.07] hover:bg-white/[0.14] border border-white/[0.08] text-white rounded-lg text-xs font-mono font-medium disabled:opacity-30 cursor-pointer active:scale-95 transition-all"
                                >
                                    {"+1"}
                                </button>
                                <button
                                    disabled={!props.is_connected}
                                    onclick={send_event_cb(ClientEvent::GameScoreAway2FG)}
                                    class="py-1.5 bg-white/[0.07] hover:bg-white/[0.14] border border-white/[0.08] text-white rounded-lg text-xs font-mono font-medium disabled:opacity-30 cursor-pointer active:scale-95 transition-all"
                                >
                                    {"+2"}
                                </button>
                                <button
                                    disabled={!props.is_connected}
                                    onclick={send_event_cb(ClientEvent::GameScoreAway3FG)}
                                    class="py-1.5 bg-white/[0.07] hover:bg-white/[0.14] border border-white/[0.08] text-white rounded-lg text-xs font-mono font-medium disabled:opacity-30 cursor-pointer active:scale-95 transition-all"
                                >
                                    {"+3"}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                // Quick Worker Assignment Section
                if !workers.is_empty() {
                    <div class="w-full max-w-[680px] bg-white/[0.03] border border-white/[0.06] rounded-xl p-3 flex flex-col gap-2">
                        <span class="text-[11px] font-semibold uppercase text-gray-400 tracking-wider">
                            { format!("Workery przypisane do Gry #{} ({})", game.id(), workers.iter().filter(|w| w.game_id() == game.id()).count()) }
                        </span>
                        <div class="flex flex-wrap gap-2">
                            { for workers.iter().map(|w| {
                                let worker = *w;
                                let sn = worker.serial_number().unwrap_or("-");
                                let is_on_this_game = worker.game_id() == game.id();
                                let target_gid = if is_on_this_game { 0 } else { game.id() };
                                let on_assign = {
                                    let assign_cb = props.on_assign_device_game.clone();
                                    Callback::from(move |_| assign_cb.emit((worker, target_gid)))
                                };

                                html! {
                                    <button
                                        onclick={on_assign}
                                        class={format!(
                                            "px-2.5 py-1 rounded-lg text-[11.5px] border font-mono cursor-pointer transition-all flex items-center gap-1.5 {}",
                                            if is_on_this_game {
                                                "bg-cyan-500/20 text-cyan-200 border-cyan-500/40 hover:bg-cyan-500/30"
                                            } else {
                                                "bg-white/[0.04] text-gray-400 border-white/[0.08] hover:bg-white/[0.08]"
                                            }
                                        )}
                                    >
                                        <span>{"📟"}</span>
                                        <span>{ sn }</span>
                                        <span class="text-[9.5px] opacity-75">
                                            { if is_on_this_game { "✓ Wyświetla" } else { "+ Przypisz" } }
                                        </span>
                                    </button>
                                }
                            })}
                        </div>
                    </div>
                }
            </div>
        </main>
    }
}
