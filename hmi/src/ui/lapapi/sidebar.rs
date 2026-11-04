//! Native macOS sidebar with tree-view for Games and Devices in Leptos (pure typography, no icons/emojis).

use basketball::{Game, Scorable};
use config::DeviceConfig;
use leptos::prelude::*;
use net::Device;

use crate::ui::traffic_lights::{CloseButton, MaximizeButton, MinimizeButton};

#[component]
pub fn LapapiSidebar(
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
    is_connected: Signal<bool>,
    sidebar_bg_class: Signal<&'static str>,
    on_open_wifi_setup: Callback<()>,
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

    view! {
        <aside class=move || format!("w-[240px] shrink-0 flex flex-col justify-between border-r border-white/[0.08] {} transition-colors duration-200 select-none overflow-hidden", sidebar_bg_class.get())>
            <div class="flex-1 flex flex-col min-h-0">
                // Titlebar Header Region with Traffic Lights
                <div
                    on:mousedown=move |e| on_start_drag.run(e)
                    class="h-10 px-3.5 flex items-center justify-between shrink-0"
                >
                    <div
                        on:mousedown=move |e| stop_propagation.run(e)
                        on:mouseenter=move |_| is_lights_hovered.set(true)
                        on:mouseleave=move |_| is_lights_hovered.set(false)
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

                    // Quick create text button
                    <button
                        on:mousedown=move |e| stop_propagation.run(e)
                        on:click=on_new_game_click
                        disabled=move || !is_connected.get()
                        title="Utwórz nową grę"
                        class="text-[11px] px-2 py-0.5 rounded-[5px] bg-white/[0.08] hover:bg-white/[0.14] text-white/90 font-medium cursor-pointer transition-colors disabled:opacity-40 disabled:cursor-not-allowed border border-white/[0.06]"
                    >
                        {"Nowa gra"}
                    </button>
                </div>

                // Sidebar Section Header
                <div class="px-3.5 pt-1 pb-1.5 flex items-center justify-between">
                    <span class="text-[11px] font-medium text-white/45">
                        {"Lapapi Control"}
                    </span>
                    <span class=move || format!(
                        "text-[10px] px-1.5 py-0.5 rounded-[4px] font-medium {}",
                        if is_connected.get() { "bg-emerald-500/15 text-emerald-300 border border-emerald-500/20" } else { "bg-rose-500/15 text-rose-300 border border-rose-500/20" }
                    )>
                        {move || if is_connected.get() { "Live" } else { "Offline" }}
                    </span>
                </div>

                // Tree View Content
                <div class="flex-1 px-2 py-1 space-y-3 overflow-y-auto min-h-0">
                    // ================= GAMES SECTION =================
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

                    // ================= DEVICES SECTION =================
                    <div>
                        <div
                            on:click=toggle_devices
                            class="flex items-center justify-between px-2 py-1 text-[11px] font-medium text-white/50 hover:text-white/80 cursor-pointer rounded-[5px] hover:bg-white/[0.04] transition-colors"
                        >
                            <span>{"Urządzenia"}</span>
                            <div class="flex items-center gap-1.5">
                                <button
                                    on:click=move |e| {
                                        e.stop_propagation();
                                        on_open_wifi_setup.run(());
                                    }
                                    title="Konfiguracja sieci Wi-Fi przez UDP"
                                    class="text-[10px] px-1.5 py-0.5 rounded-[4px] bg-blue-600/25 hover:bg-blue-600/40 text-blue-300 font-medium cursor-pointer transition-colors border border-blue-400/20"
                                >
                                    {"Wi-Fi"}
                                </button>
                                <span class="text-[10px] px-1.5 py-0.2 rounded-[4px] bg-white/[0.06] text-white/50 tabular-nums">
                                    {move || devices.get().len()}
                                </span>
                            </div>
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
                                        list.into_iter().map(|d| {
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
                                        }).collect_view().into_any()
                                    }
                                }}
                            </div>
                        </Show>
                    </div>
                </div>
            </div>

            // Bottom Profile / Connection Indicator
            <div class="p-2.5 border-t border-white/[0.06] flex items-center justify-between text-white/50">
                <span class="text-[11px] text-white/70">
                    {move || if is_connected.get() { "Połączono" } else { "Rozłączono" }}
                </span>
                <button
                    on:click=on_new_game_click
                    disabled=move || !is_connected.get()
                    class="px-2 py-0.5 bg-white/[0.06] hover:bg-white/[0.12] text-white/80 text-[11px] font-medium rounded-[5px] cursor-pointer transition-colors border border-white/[0.06] disabled:opacity-40"
                >
                    {"+ Gra"}
                </button>
            </div>
        </aside>
    }
}
