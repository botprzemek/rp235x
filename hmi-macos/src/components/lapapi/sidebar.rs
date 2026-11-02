//! Lapapi macOS left sidebar with tree view for Games and Devices.

use basketball::{Game, Scorable};
use config::DeviceConfig;
use net::{Device, DeviceType};
use yew::{Callback, Html, MouseEvent, Properties, function_component, html, use_state};

use crate::components::traffic_lights::{CloseButton, MaximizeButton, MinimizeButton};

#[derive(Properties, PartialEq, Clone)]
pub struct SidebarProps {
    pub is_focused: bool,
    pub is_lights_hovered: bool,
    pub on_start_drag: Callback<MouseEvent>,
    pub stop_propagation: Callback<MouseEvent>,
    pub on_lights_enter: Callback<MouseEvent>,
    pub on_lights_leave: Callback<MouseEvent>,
    pub on_close: Callback<MouseEvent>,
    pub on_minimize: Callback<MouseEvent>,
    pub on_maximize: Callback<MouseEvent>,
    pub games: Vec<Game>,
    pub selected_game_id: u16,
    pub on_select_game: Callback<u16>,
    pub on_create_game: Callback<()>,
    pub devices: Vec<Device>,
    pub selected_device_sn: Option<String>,
    pub on_select_device: Callback<String>,
    pub is_connected: bool,
    pub sidebar_bg_class: &'static str,
    pub on_open_wifi_setup: Callback<()>,
}

#[function_component(LapapiSidebar)]
pub fn lapapi_sidebar(props: &SidebarProps) -> Html {
    let games_expanded = use_state(|| true);
    let devices_expanded = use_state(|| true);

    let toggle_games = {
        let games_expanded = games_expanded.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            games_expanded.set(!*games_expanded);
        })
    };

    let toggle_devices = {
        let devices_expanded = devices_expanded.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            devices_expanded.set(!*devices_expanded);
        })
    };

    let on_new_game_click = {
        let on_create_game = props.on_create_game.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            on_create_game.emit(());
        })
    };

    let games_count = props.games.len();
    let devices_count = props.devices.len();

    html! {
        <aside class={format!("w-[260px] shrink-0 flex flex-col justify-between border-r border-white/[0.08] {} transition-colors duration-200 select-none overflow-hidden", props.sidebar_bg_class)}>
            <div class="flex-1 flex flex-col min-h-0">
                // Virtual Titlebar Region
                <div
                    onmousedown={props.on_start_drag.clone()}
                    class="h-10 px-4 flex items-center justify-between shrink-0"
                >
                    <div
                        onmousedown={props.stop_propagation.clone()}
                        onmouseenter={props.on_lights_enter.clone()}
                        onmouseleave={props.on_lights_leave.clone()}
                        class="flex items-center gap-2"
                    >
                        <CloseButton
                            focused={props.is_focused}
                            hovered={props.is_lights_hovered}
                            onclick={props.on_close.clone()}
                        />
                        <MinimizeButton
                            focused={props.is_focused}
                            hovered={props.is_lights_hovered}
                            onclick={props.on_minimize.clone()}
                        />
                        <MaximizeButton
                            focused={props.is_focused}
                            hovered={props.is_lights_hovered}
                            onclick={props.on_maximize.clone()}
                        />
                    </div>

                    // Quick create button in titlebar
                    <button
                        onmousedown={props.stop_propagation.clone()}
                        onclick={on_new_game_click.clone()}
                        disabled={!props.is_connected}
                        title="Utwórz nową grę"
                        class="text-gray-400 hover:text-white disabled:opacity-30 disabled:cursor-not-allowed transition-colors p-1 rounded-md cursor-pointer"
                    >
                        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                            <path d="M12 5v14M5 12h14" />
                        </svg>
                    </button>
                </div>

                // Tree View Content
                <div class="flex-1 overflow-y-auto px-3 py-2 space-y-4">
                    // ================= GAMES BRANCH =================
                    <div>
                        <div
                            onclick={toggle_games}
                            class="flex items-center justify-between px-2 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-gray-400 hover:text-gray-200 cursor-pointer rounded-md hover:bg-white/[0.04] transition-colors"
                        >
                            <div class="flex items-center gap-1.5">
                                <span class={format!("transition-transform duration-150 text-[9px] {}", if *games_expanded { "rotate-90" } else { "" })}>
                                    {"▶"}
                                </span>
                                <span>{"Gry (Games)"}</span>
                            </div>
                            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-white/[0.06] text-gray-400 font-mono">
                                { games_count }
                            </span>
                        </div>

                        if *games_expanded {
                            <div class="mt-1 pl-3 space-y-0.5 border-l border-white/[0.06] ml-2.5">
                                if props.games.is_empty() {
                                    <div class="text-[11.5px] text-gray-500 py-1.5 px-2 italic">
                                        { if props.is_connected { "Brak gier w bazie" } else { "Łączenie z serwerem..." } }
                                    </div>
                                } else {
                                    { for props.games.iter().map(|g| {
                                        let gid = g.id();
                                        let is_sel = gid == props.selected_game_id;
                                        let scores = g.scores();
                                        let on_click = {
                                            let on_select = props.on_select_game.clone();
                                            Callback::from(move |_| on_select.emit(gid))
                                        };

                                        html! {
                                            <div
                                                onclick={on_click}
                                                class={format!(
                                                    "group flex items-center justify-between px-2.5 py-1.5 rounded-lg text-[12.5px] cursor-pointer transition-all {}",
                                                    if is_sel {
                                                        "bg-white/[0.12] text-white font-medium shadow-sm border border-white/[0.08]"
                                                    } else {
                                                        "text-gray-300 hover:bg-white/[0.05] hover:text-white"
                                                    }
                                                )}
                                            >
                                                <div class="flex items-center gap-2 truncate">
                                                    <span class={format!("w-1.5 h-1.5 rounded-full shrink-0 {}", if is_sel { "bg-amber-400 shadow-[0_0_6px_rgba(251,191,36,0.6)]" } else { "bg-gray-500" })} />
                                                    <span class="truncate">{ format!("Gra #{}", gid) }</span>
                                                </div>
                                                <span class="text-[11px] font-mono text-gray-400 group-hover:text-gray-200">
                                                    { format!("{}:{}", scores.home, scores.away) }
                                                </span>
                                            </div>
                                        }
                                    })}
                                }
                            </div>
                        }
                    </div>

                    // ================= DEVICES BRANCH =================
                    <div>
                        <div
                            onclick={toggle_devices}
                            class="flex items-center justify-between px-2 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-gray-400 hover:text-gray-200 cursor-pointer rounded-md hover:bg-white/[0.04] transition-colors"
                        >
                            <div class="flex items-center gap-1.5">
                                <span class={format!("transition-transform duration-150 text-[9px] {}", if *devices_expanded { "rotate-90" } else { "" })}>
                                    {"▶"}
                                </span>
                                <span>{"Urządzenia (Devices)"}</span>
                            </div>
                            <div class="flex items-center gap-1.5">
                                <button
                                    onclick={
                                        let open_wifi = props.on_open_wifi_setup.clone();
                                        Callback::from(move |e: MouseEvent| {
                                            e.stop_propagation();
                                            open_wifi.emit(());
                                        })
                                    }
                                    title="Konfiguruj Wi-Fi urządzenia (OTA UDP)"
                                    class="text-[10px] px-1.5 py-0.5 rounded bg-blue-600/30 hover:bg-blue-600/60 text-blue-300 font-medium cursor-pointer transition-colors"
                                >
                                    {"📶 Setup"}
                                </button>
                                <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-white/[0.06] text-gray-400 font-mono">
                                    { devices_count }
                                </span>
                            </div>
                        </div>

                        if *devices_expanded {
                            <div class="mt-1 pl-3 space-y-0.5 border-l border-white/[0.06] ml-2.5">
                                if props.devices.is_empty() {
                                    <div class="text-[11.5px] text-gray-500 py-1.5 px-2 italic">
                                        { "Brak urządzeń w sieci" }
                                    </div>
                                } else {
                                    { for props.devices.iter().map(|d| {
                                        let sn = d.serial_number().unwrap_or("-").to_string();
                                        let sn_clone = sn.clone();
                                        let is_sel = props.selected_device_sn.as_ref() == Some(&sn);
                                        let dtype = d.device_type();
                                        let gid = d.game_id();
                                        let icon = match dtype {
                                            DeviceType::Worker => "📟",
                                            DeviceType::WebBrowser => "💻",
                                            _ => "📱",
                                        };
                                        let on_click = {
                                            let on_select = props.on_select_device.clone();
                                            Callback::from(move |_| on_select.emit(sn_clone.clone()))
                                        };

                                        html! {
                                            <div
                                                onclick={on_click}
                                                class={format!(
                                                    "group flex items-center justify-between px-2 py-1.5 rounded-lg text-[12px] cursor-pointer transition-all {}",
                                                    if is_sel {
                                                        "bg-white/[0.12] text-white font-medium shadow-sm border border-white/[0.08]"
                                                    } else {
                                                        "text-gray-300 hover:bg-white/[0.05] hover:text-white"
                                                    }
                                                )}
                                            >
                                                <div class="flex items-center gap-1.5 truncate">
                                                    <span class="text-[11px] shrink-0">{ icon }</span>
                                                    <span class="truncate">{ sn }</span>
                                                </div>
                                                <span class="text-[10px] font-mono text-gray-400 bg-white/[0.06] px-1.5 py-0.5 rounded">
                                                    { if gid == 0 { "—".to_string() } else { format!("#{}", gid) } }
                                                </span>
                                            </div>
                                        }
                                    })}
                                }
                            </div>
                        }
                    </div>
                </div>
            </div>

            // Bottom Profile / Connection Indicator
            <div class="p-3 border-t border-white/[0.06] flex items-center justify-between text-xs text-gray-400">
                <div class="flex items-center gap-2">
                    <span class={format!("w-2 h-2 rounded-full {}", if props.is_connected { "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.7)]" } else { "bg-rose-500" })} />
                    <span class="font-medium text-[11.5px] text-gray-300">
                        { if props.is_connected { "WebSocket Połączony" } else { "Rozłączony" } }
                    </span>
                </div>
                <button
                    onclick={on_new_game_click}
                    disabled={!props.is_connected}
                    class="px-2 py-1 bg-white/[0.08] hover:bg-white/[0.14] text-gray-200 text-[11px] font-medium rounded-md cursor-pointer transition-colors"
                >
                    { "+ Gra" }
                </button>
            </div>
        </aside>
    }
}
