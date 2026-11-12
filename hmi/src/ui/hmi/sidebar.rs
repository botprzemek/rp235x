use crate::ui::traffic_lights::{CloseButton, MaximizeButton, MinimizeButton};

use basketball::{Game, Scorable};
use config::DeviceConfig;
use leptos::prelude::*;
use net::Device;

#[component]
pub fn NavigationSidebar(
    is_focused: Signal<bool>,
    is_lights_hovered: RwSignal<bool>,
    on_start_drag: Callback<web_sys::MouseEvent>,
    stop_propagation: Callback<web_sys::MouseEvent>,
    on_close: Callback<web_sys::MouseEvent>,
    on_minimize: Callback<web_sys::MouseEvent>,
    on_maximize: Callback<web_sys::MouseEvent>,
    games: Signal<Vec<Game>>,
    selected_game_id: Signal<u16>,
    on_select_game: Callback<u16>,
    on_create_game: Callback<()>,
    devices: Signal<Vec<Device>>,
    selected_device_sn: Signal<Option<String>>,
    on_select_device: Callback<String>,
    on_create_dummy_device: Callback<()>,
    is_connected: Signal<bool>,
    master_address: Signal<String>,
    on_open_master_modal: Callback<()>,
    sidebar_bg_class: Signal<&'static str>,
    on_open_wifi_setup: Callback<()>,
    #[prop(optional)] on_back: Option<Callback<()>>,
) -> impl IntoView {
    let games_expanded = RwSignal::new(true);
    let devices_expanded = RwSignal::new(true);

    let toggle_games = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        games_expanded.update(|val| *val = !*val);
    };

    let toggle_devices = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        devices_expanded.update(|val| *val = !*val);
    };

    let on_new_game_click = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        on_create_game.run(());
    };

    let on_new_dummy_click = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        on_create_dummy_device.run(());
    };

    let on_master_click = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        on_open_master_modal.run(());
    };

    view! {
        <aside class=move || format!("w-[240px] shrink-0 flex flex-col justify-between border-r border-white/[0.08] {} transition-colors duration-200 select-none overflow-hidden", sidebar_bg_class.get())>
            <div class="flex-1 flex flex-col min-h-0">

                <div
                    on:mousedown=move |e| on_start_drag.run(e)
                    on:mouseenter=move |_| is_lights_hovered.set(true)
                    on:mouseleave=move |_| is_lights_hovered.set(false)
                    class="h-9 w-full flex items-center justify-between px-3.5 shrink-0 cursor-grab active:cursor-grabbing border-b border-white/[0.04]"
                >
                    <div
                        on:mousedown=move |e| stop_propagation.run(e)
                        class="flex items-center gap-2"
                    >
                        <CloseButton
                            focused=is_focused
                            hovered=is_lights_hovered.into()
                            onclick=on_close
                        />
                        <MinimizeButton
                            focused=is_focused
                            hovered=is_lights_hovered.into()
                            onclick=on_minimize
                        />
                        <MaximizeButton
                            focused=is_focused
                            hovered=is_lights_hovered.into()
                            onclick=on_maximize
                        />
                    </div>

                    {on_back.map(|cb| view! {
                        <button
                            type="button"
                            title="Wróć do nowego widoku"
                            on:click=move |e| {
                                e.stop_propagation();
                                cb.run(());
                            }
                            class="px-2 py-0.5 rounded text-[11px] font-medium bg-white/10 hover:bg-white/20 active:scale-95 text-white/80 hover:text-white transition-all flex items-center gap-1 cursor-pointer border border-white/10 shadow-sm"
                        >
                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
                            </svg>
                            <span>"Wróć"</span>
                        </button>
                    })}

                    <div class="flex items-center gap-1.5">
                        <button
                            on:mousedown=move |e| stop_propagation.run(e)
                            on:click=on_new_dummy_click
                            title="Dodaj symulowane urządzenie dummy"
                            class="text-[11px] px-2 py-0.5 rounded-[5px] bg-white/[0.08] hover:bg-white/[0.14] text-white/90 font-medium cursor-pointer transition-colors border border-white/[0.06]"
                        >
                            {"+ Dummy"}
                        </button>
                        <button
                            on:mousedown=move |e| stop_propagation.run(e)
                            on:click=on_new_game_click
                            disabled=move || !is_connected.get()
                            title="Utwórz nową grę"
                            class="text-[11px] px-2 py-0.5 rounded-[5px] bg-white/[0.08] hover:bg-white/[0.14] text-white/90 font-medium cursor-pointer transition-colors disabled:opacity-40 disabled:cursor-not-allowed border border-white/[0.06]"
                        >
                            {"+ Gra"}
                        </button>
                    </div>
                </div>

                <div class="px-2.5 py-1.5 border-b border-white/[0.06] bg-black/15 flex items-center justify-between text-[11px]">
                    <div
                        on:click=on_master_click
                        class="flex items-center gap-1.5 truncate flex-1 cursor-pointer hover:text-white text-white/70 transition-colors mr-1"
                        title="Kliknij, aby zmienić adres serwera Master"
                    >
                        <span class=move || format!(
                            "w-1.5 h-1.5 rounded-full shrink-0 {}",
                            if is_connected.get() { "bg-emerald-400" } else { "bg-rose-400 animate-pulse" }
                        ) />
                        <span class="truncate font-mono text-[10px]">
                            {move || master_address.get()}
                        </span>
                    </div>

                    <button
                        on:click=on_master_click
                        class="px-1.5 py-0.5 rounded-[4px] bg-white/[0.08] hover:bg-white/[0.16] text-white/80 text-[10px] cursor-pointer transition-colors font-medium shrink-0"
                        title="Zmień adres serwera Master"
                    >
                        {"Zmień"}
                    </button>
                </div>

                <div class="px-3 pt-2.5 pb-1 flex items-center justify-between text-white/50 text-[11px] font-medium tracking-wide uppercase">
                    <span>{"Serwer Master"}</span>
                    <span class=move || format!(
                        "text-[10px] px-1.5 py-0.5 rounded-[4px] font-semibold {}",
                        if is_connected.get() { "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30" } else { "bg-rose-500/20 text-rose-300 border border-rose-500/30" }
                    )>
                        {move || if is_connected.get() { "Połączony" } else { "Rozłączony" }}
                    </span>
                </div>

                <div class="flex-1 px-2 py-1 space-y-3 overflow-y-auto min-h-0">

                    <div>
                        <div
                            on:click=toggle_games
                            class="flex items-center justify-between px-2 py-1 text-[11px] font-medium text-white/50 hover:text-white/80 cursor-pointer rounded-[5px] hover:bg-white/[0.04] transition-colors"
                        >
                            <span>{"Gry"}</span>
                            <span class="text-[10px] px-1.5 py-0.2 rounded-[4px] bg-white/[0.06] text-white/50 tabular-nums">
                                {move || games.get().len()}
                            </span>
                        </div>

                        <Show when=move || games_expanded.get()>
                            <div class="mt-0.5 pl-2 space-y-0.5 border-l border-white/[0.06] ml-2.5">
                                {move || {
                                    let list = games.get();
                                    if list.is_empty() {
                                        view! {
                                            <div class="text-[11px] text-white/40 py-1 px-2 italic">
                                                {"Brak aktywnych gier"}
                                            </div>
                                        }.into_any()
                                    } else {
                                        list.into_iter().map(|g| {
                                            let gid = g.id();
                                            let is_sel = Signal::derive(move || selected_game_id.get() == gid);
                                            let scores = g.scores();
                                            view! {
                                                <div
                                                    on:click=move |_| on_select_game.run(gid)
                                                    class=move || format!(
                                                        "group flex items-center justify-between px-2 py-1 rounded-[5px] text-[12px] cursor-pointer transition-colors {}",
                                                        if is_sel.get() {
                                                             "bg-white/[0.14] text-white font-medium border border-white/[0.08]"
                                                        } else {
                                                            "text-white/75 hover:bg-white/[0.06] hover:text-white"
                                                        }
                                                    )
                                                >
                                                    <span class="truncate">{format!("Gra #{}", gid)}</span>
                                                    <span class="text-[11px] tabular-nums text-white/45">
                                                        {format!("{}:{}", scores.home, scores.away)}
                                                    </span>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }}
                            </div>
                        </Show>
                    </div>

                    <div>
                        <div
                            on:click=toggle_devices
                            class="flex items-center justify-between px-2 py-1 text-[11px] font-medium text-white/50 hover:text-white/80 cursor-pointer rounded-[5px] hover:bg-white/[0.04] transition-colors"
                        >
                            <div class="flex items-center gap-1.5">
                                <span>{"Urządzenia"}</span>
                                <span class="text-[10px] px-1.5 py-0.2 rounded-[4px] bg-white/[0.06] text-white/50 tabular-nums">
                                    {move || devices.get().len()}
                                </span>
                            </div>

                            <button
                                on:mousedown=move |e| stop_propagation.run(e)
                                on:click=on_new_dummy_click
                                title="Dodaj symulowane urządzenie dummy"
                                class="text-[10px] px-1.5 py-0.2 rounded-[4px] bg-white/[0.08] hover:bg-white/[0.16] text-white/80 font-medium cursor-pointer transition-colors border border-white/[0.06]"
                            >
                                {"+ Dummy"}
                            </button>
                        </div>

                        <Show when=move || devices_expanded.get()>
                            <div class="mt-0.5 pl-2 space-y-0.5 border-l border-white/[0.06] ml-2.5">
                                {move || {
                                    let list = devices.get();
                                    if list.is_empty() {
                                        view! {
                                            <div class="text-[11px] text-white/40 py-1 px-2 italic">
                                                {"Brak urządzeń w sieci"}
                                            </div>
                                        }.into_any()
                                    } else {
                                        let total = list.len();
                                        let visible = list.into_iter().take(64);
                                        view! {
                                            <div>
                                                {visible.map(|d| {
                                                    let sn = d.serial_number().unwrap_or("-").to_string();
                                                    let sn_clone = sn.clone();
                                                    let sn_for_sel = sn.clone();
                                                    let is_sel = Signal::derive(move || selected_device_sn.get().as_ref() == Some(&sn_for_sel));
                                                    let gid = d.game_id();

                                                    view! {
                                                        <div
                                                            on:click=move |_| on_select_device.run(sn_clone.clone())
                                                            class=move || format!(
                                                                "group flex items-center justify-between px-2 py-1 rounded-[5px] text-[12px] cursor-pointer transition-colors {}",
                                                                if is_sel.get() {
                                                                    "bg-white/[0.14] text-white font-medium border border-white/[0.08]"
                                                                } else {
                                                                    "text-white/75 hover:bg-white/[0.06] hover:text-white"
                                                                }
                                                            )
                                                        >
                                                            <span class="truncate">{sn}</span>
                                                            <span class="text-[10px] tabular-nums text-white/40 bg-white/[0.06] px-1.5 py-0.5 rounded-[3px]">
                                                                {if gid == 0 { "—".to_string() } else { format!("#{}", gid) }}
                                                            </span>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                                {if total > 64 {
                                                    view! {
                                                        <div class="text-[10px] text-white/35 py-1 px-2 italic text-center">
                                                            {format!("+ {} kolejnych urządzeń", total - 64)}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <span/> }.into_any()
                                                }}
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </Show>
                    </div>
                </div>
            </div>

            <div class="p-2.5 border-t border-white/[0.06] flex items-center justify-between text-white/50 gap-2">
                <button
                    on:click=move |_| on_open_wifi_setup.run(())
                    class="w-full px-2 py-1 bg-white/[0.06] hover:bg-white/[0.12] text-white/75 text-[11px] font-medium rounded-[5px] cursor-pointer transition-colors border border-white/[0.06] text-center"
                    title="Konfiguruj Wi-Fi urządzeń przez UDP"
                >
                    {"Wi-Fi Setup (OTA)"}
                </button>
            </div>
        </aside>
    }
}

pub use NavigationSidebar as HmiSidebar;
