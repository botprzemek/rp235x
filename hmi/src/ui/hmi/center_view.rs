use crate::ui::scoreboard::Scoreboard;

use basketball::{Game, Quarter, Scorable, State as GameState};
use config::DeviceConfig;
use leptos::prelude::*;
use net::{ClientEvent, Device, DeviceType};

#[component]
pub fn MatchControlPanel(
    game: Signal<Game>,
    is_connected: Signal<bool>,
    on_start_drag: Callback<web_sys::MouseEvent>,
    stop_propagation: Callback<web_sys::MouseEvent>,
    on_send_event: Callback<ClientEvent>,
    main_bg_class: Signal<&'static str>,
    devices: Signal<Vec<Device>>,
    #[prop(optional)] on_disconnect_device: Option<Callback<Device>>,
    on_assign_device_game: Callback<(Device, u16)>,
    #[prop(optional)] on_back: Option<Callback<()>>,
) -> impl IntoView {
    let _ = on_disconnect_device;

    let scores = move || game.get().scores();
    let regulation_time = move || game.get().regulation_time();
    let clock_time = move || game.get().clock_time();
    let state = move || game.get().state();

    let is_idle_default = move || {
        let sc = scores();
        sc.home == 0 && sc.away == 0 && state() == GameState::Idle
    };

    let display_home_score = move || {
        if is_idle_default() {
            139
        } else {
            scores().home
        }
    };
    let display_away_score = move || {
        if is_idle_default() {
            127
        } else {
            scores().away
        }
    };

    let display_time = move || {
        if is_idle_default() {
            "02:56".to_string()
        } else {
            let rt = regulation_time();
            format!("{:02}:{:02}", rt.minutes, rt.seconds)
        }
    };

    let display_quarter = move || {
        if is_idle_default() {
            "4Q".to_string()
        } else {
            match game.get().quarter() {
                Quarter::Q2 => "2Q".to_string(),
                Quarter::Q3 => "3Q".to_string(),
                Quarter::Q4 => "4Q".to_string(),
                Quarter::Q1 | Quarter::None | _ => "1Q".to_string(),
            }
        }
    };

    let display_shot_clock = move || {
        if is_idle_default() {
            24
        } else {
            clock_time().seconds
        }
    };

    let send_event = move |evt: ClientEvent| {
        let cb = on_send_event;
        move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            cb.run(evt);
        }
    };

    view! {
        <main class=move || format!("flex-1 flex flex-col justify-between {} transition-colors duration-200 border-r border-white/[0.08] min-w-0 overflow-y-auto select-none", main_bg_class.get())>

            <header
                on:mousedown=move |e| on_start_drag.run(e)
                class="flex items-center justify-between px-5 pt-2.5 pb-2 shrink-0 border-b border-white/[0.04]"
            >
                <div class="flex items-center gap-2.5">
                    <span class="text-[13px] font-medium text-white/90">
                        {move || format!("Gra #{}", game.get().id())}
                    </span>
                    {move || match state() {
                        GameState::Running => view! {
                            <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[4px] text-[11px] font-medium bg-emerald-500/15 text-emerald-300 border border-emerald-500/25">
                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                                {"Trwa"}
                            </span>
                        }.into_any(),
                        GameState::Paused => view! {
                            <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[4px] text-[11px] font-medium bg-amber-500/15 text-amber-300 border border-amber-500/25">
                                <span class="w-1.5 h-1.5 rounded-full bg-amber-400" />
                                {"Pauza"}
                            </span>
                        }.into_any(),
                        GameState::Idle | _ => view! {
                            <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[4px] text-[11px] font-medium bg-white/[0.06] text-white/50 border border-white/[0.08]">
                                <span class="w-1.5 h-1.5 rounded-full bg-white/40" />
                                {"Oczekiwanie"}
                            </span>
                        }.into_any(),
                    }}
                </div>

                <div
                    on:mousedown=move |e| stop_propagation.run(e)
                    class="flex items-center gap-2 text-white/50"
                >
                    <span class="font-mono tabular-nums text-[11px]">
                        {move || {
                            let ct = clock_time();
                            format!("24s: {:02}.{:02}", ct.seconds, ct.centiseconds)
                        }}
                    </span>
                    {on_back.map(|cb| view! {
                        <button
                            type="button"
                            title="Wróć do nowego widoku"
                            on:click=move |e| {
                                e.stop_propagation();
                                cb.run(());
                            }
                            class="ml-2.5 px-2 py-0.5 rounded bg-white/10 hover:bg-white/15 active:scale-95 text-white/80 hover:text-white text-[11px] font-medium transition-all flex items-center gap-1.5 cursor-pointer border border-white/10 shadow-sm"
                        >
                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
                            </svg>
                            <span>"Wróć"</span>
                        </button>
                    })}
                </div>
            </header>

            <div class="flex-1 p-5 flex flex-col items-center justify-between gap-5 overflow-y-auto">

                <div class="w-full flex justify-center scale-95 transition-transform">
                    {move || view! {
                        <Scoreboard
                            home_team="HOME"
                            away_team="AWAY"
                            home_score=display_home_score()
                            away_score=display_away_score()
                            time=display_time()
                            shot_clock=display_shot_clock()
                            quarter=display_quarter()
                            home_timeouts=2
                            away_timeouts=4
                            home_timeout_dashes=5
                            away_timeout_dashes=5
                        />
                    }}
                </div>

                <div class="w-full max-w-[620px] bg-white/[0.04] border border-white/[0.08] backdrop-blur-md rounded-xl p-3.5 flex flex-col gap-3 shadow-lg">
                    <div class="flex items-center justify-between text-[11px] text-white/50 px-0.5">
                        <span class="font-medium text-white/70">{"Sterowanie czasem i punktacją"}</span>
                        <span class="text-[10px] text-white/40 font-mono">{"WebSocket"}</span>
                    </div>

                    <div class="grid grid-cols-4 gap-2">
                        <button
                            disabled=move || !is_connected.get()
                            on:click=send_event(ClientEvent::GameStart)
                            class="flex items-center justify-center px-3 py-1.5 bg-emerald-600/25 hover:bg-emerald-600/40 border border-emerald-400/25 text-emerald-200 rounded-[6px] text-[12px] font-medium cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                        >
                            {"Start"}
                        </button>
                        <button
                            disabled=move || !is_connected.get()
                            on:click=send_event(ClientEvent::GamePause)
                            class="flex items-center justify-center px-3 py-1.5 bg-amber-600/25 hover:bg-amber-600/40 border border-amber-400/25 text-amber-200 rounded-[6px] text-[12px] font-medium cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                        >
                            {"Pauza"}
                        </button>
                        <button
                            disabled=move || !is_connected.get()
                            on:click=send_event(ClientEvent::GameResume)
                            class="flex items-center justify-center px-3 py-1.5 bg-sky-600/25 hover:bg-sky-600/40 border border-sky-400/25 text-sky-200 rounded-[6px] text-[12px] font-medium cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                        >
                            {"Wznów"}
                        </button>
                        <button
                            disabled=move || !is_connected.get()
                            on:click=send_event(ClientEvent::GameReset)
                            class="flex items-center justify-center px-3 py-1.5 bg-rose-600/25 hover:bg-rose-600/40 border border-rose-400/25 text-rose-200 rounded-[6px] text-[12px] font-medium cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                        >
                            {"Reset"}
                        </button>
                    </div>

                    <div class="grid grid-cols-2 gap-3 pt-2 border-t border-white/[0.06]">

                        <div class="flex flex-col gap-1">
                            <span class="text-[11px] font-medium text-white/45 px-0.5">{"Gospodarze"}</span>
                            <div class="grid grid-cols-3 gap-1.5">
                                <button
                                    disabled=move || !is_connected.get()
                                    on:click=send_event(ClientEvent::GameScoreHomeFT)
                                    class="py-1 bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.08] text-white/90 rounded-[5px] text-[12px] font-medium tabular-nums disabled:opacity-30 cursor-pointer transition-colors"
                                >
                                    {"+1"}
                                </button>
                                <button
                                    disabled=move || !is_connected.get()
                                    on:click=send_event(ClientEvent::GameScoreHome2FG)
                                    class="py-1 bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.08] text-white/90 rounded-[5px] text-[12px] font-medium tabular-nums disabled:opacity-30 cursor-pointer transition-colors"
                                >
                                    {"+2"}
                                </button>
                                <button
                                    disabled=move || !is_connected.get()
                                    on:click=send_event(ClientEvent::GameScoreHome3FG)
                                    class="py-1 bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.08] text-white/90 rounded-[5px] text-[12px] font-medium tabular-nums disabled:opacity-30 cursor-pointer transition-colors"
                                >
                                    {"+3"}
                                </button>
                            </div>
                        </div>

                        <div class="flex flex-col gap-1">
                            <span class="text-[11px] font-medium text-white/45 px-0.5">{"Goście"}</span>
                            <div class="grid grid-cols-3 gap-1.5">
                                <button
                                    disabled=move || !is_connected.get()
                                    on:click=send_event(ClientEvent::GameScoreAwayFT)
                                    class="py-1 bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.08] text-white/90 rounded-[5px] text-[12px] font-medium tabular-nums disabled:opacity-30 cursor-pointer transition-colors"
                                >
                                    {"+1"}
                                </button>
                                <button
                                    disabled=move || !is_connected.get()
                                    on:click=send_event(ClientEvent::GameScoreAway2FG)
                                    class="py-1 bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.08] text-white/90 rounded-[5px] text-[12px] font-medium tabular-nums disabled:opacity-30 cursor-pointer transition-colors"
                                >
                                    {"+2"}
                                </button>
                                <button
                                    disabled=move || !is_connected.get()
                                    on:click=send_event(ClientEvent::GameScoreAway3FG)
                                    class="py-1 bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.08] text-white/90 rounded-[5px] text-[12px] font-medium tabular-nums disabled:opacity-30 cursor-pointer transition-colors"
                                >
                                    {"+3"}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                {move || {
                    let current_game_id = game.get().id();
                    let workers: Vec<Device> = devices.get()
                        .into_iter()
                        .filter(|d| d.device_type() == DeviceType::Worker)
                        .collect();

                    if workers.is_empty() {
                        view! { <div/> }.into_any()
                    } else {
                        let assigned_count = workers.iter().filter(|w| w.game_id() == current_game_id).count();
                        let total_workers = workers.len();
                        let title = format!("Workery przypisane do Gry #{} ({})", current_game_id, assigned_count);
                        let visible_workers = workers.into_iter().take(32);
                        view! {
                            <div class="w-full max-w-[620px] bg-white/[0.03] border border-white/[0.06] rounded-xl p-3 flex flex-col gap-2">
                                <span class="text-[11px] font-medium text-white/50">
                                    {title}
                                </span>
                                <div class="flex flex-wrap gap-2">
                                    {visible_workers.map(|worker| {
                                        let sn = worker.serial_number().unwrap_or("-").to_string();
                                        let is_on_this_game = worker.game_id() == current_game_id;
                                        let target_gid = if is_on_this_game { 0 } else { current_game_id };
                                        let on_assign_cb = on_assign_device_game;
                                        let btn_class = format!(
                                            "px-2.5 py-1 rounded-[5px] text-[11px] border font-medium cursor-pointer transition-colors flex items-center gap-2 {}",
                                            if is_on_this_game {
                                                "bg-cyan-500/15 text-cyan-200 border-cyan-400/30 hover:bg-cyan-500/25"
                                            } else {
                                                "bg-white/[0.04] text-white/60 border-white/[0.06] hover:bg-white/[0.08] hover:text-white/80"
                                            }
                                        );

                                        view! {
                                            <button
                                                on:click=move |_| on_assign_cb.run((worker, target_gid))
                                                class=btn_class
                                            >
                                                <span class="font-mono">{sn}</span>
                                                {if is_on_this_game {
                                                    view! {
                                                        <span class="text-[10px] text-cyan-300 font-medium">
                                                            {"Wyświetla"}
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="text-[10px] text-white/40">
                                                            {"Przypisz"}
                                                        </span>
                                                    }.into_any()
                                                }}
                                            </button>
                                        }
                                    }).collect_view()}
                                    {if total_workers > 32 {
                                        view! {
                                            <span class="text-[11px] text-white/35 self-center italic px-1">
                                                {format!("+ {} kolejnych", total_workers - 32)}
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! { <span/> }.into_any()
                                    }}
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </main>
    }
}

pub use MatchControlPanel as HmiCenterView;
