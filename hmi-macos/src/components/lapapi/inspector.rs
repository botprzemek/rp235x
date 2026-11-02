//! Lapapi macOS right details inspector component showing selected Game or Device metadata & logs.

use basketball::{Game, Scorable};
use config::DeviceConfig;
use net::Device;
use yew::{Callback, Html, MouseEvent, Properties, function_component, html, use_state};

#[derive(Properties, PartialEq, Clone)]
pub struct InspectorProps {
    pub selected_game: Game,
    pub selected_device: Option<Device>,
    pub logs: Vec<String>,
    pub on_start_drag: Callback<MouseEvent>,
    pub inspector_bg_class: &'static str,
    pub on_disconnect_device: Callback<Device>,
    pub on_open_wifi_config: Callback<Device>,
}

#[function_component(LapapiInspector)]
pub fn lapapi_inspector(props: &InspectorProps) -> Html {
    let active_tab = use_state(|| "details"); // "details" or "logs"

    let set_details = {
        let active_tab = active_tab.clone();
        Callback::from(move |_| active_tab.set("details"))
    };
    let set_logs = {
        let active_tab = active_tab.clone();
        Callback::from(move |_| active_tab.set("logs"))
    };

    let game = props.selected_game;
    let scores = game.scores();
    let reg_time = game.regulation_time();

    let device_card = props.selected_device.map(|dev| {
        let dev_copy = dev;
        let on_dc = {
            let dc_cb = props.on_disconnect_device.clone();
            Callback::from(move |_| dc_cb.emit(dev_copy))
        };
        let on_wifi = {
            let wifi_cb = props.on_open_wifi_config.clone();
            Callback::from(move |_| wifi_cb.emit(dev_copy))
        };

        html! {
            <div class="bg-white/[0.05] backdrop-blur-md border border-white/[0.08] rounded-2xl p-3.5 flex flex-col gap-2.5 shadow-md">
                <div class="flex items-center justify-between">
                    <h3 class="text-[12.5px] font-semibold text-white">{"Wybrane Urządzenie"}</h3>
                    <div class="flex items-center gap-1.5">
                        <button
                            onclick={on_wifi}
                            title="Konfiguruj sieć Wi-Fi urządzenia przez UDP"
                            class="text-[10px] px-2 py-0.5 bg-blue-600/80 hover:bg-blue-500 text-white rounded cursor-pointer transition-colors"
                        >
                            {"Wi-Fi 📶"}
                        </button>
                        <button
                            onclick={on_dc}
                            class="text-[10px] px-2 py-0.5 bg-rose-600/80 hover:bg-rose-500 text-white rounded cursor-pointer transition-colors"
                        >
                            {"Rozłącz"}
                        </button>
                    </div>
                </div>
                <div class="space-y-1.5 text-[11.5px] text-gray-300">
                    <div class="flex justify-between">
                        <span class="text-gray-400">{"Typ:"}</span>
                        <span class="font-mono text-white">{ format!("{:?}", dev.device_type()) }</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-gray-400">{"ID:"}</span>
                        <span class="font-mono">{ dev.device_id().unwrap_or("-") }</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-gray-400">{"Numer seryjny:"}</span>
                        <span class="font-mono truncate max-w-[120px]">{ dev.serial_number().unwrap_or("-") }</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-gray-400">{"Przypisana gra:"}</span>
                        <span class="font-mono text-cyan-300 font-semibold">{ format!("#{}", dev.game_id()) }</span>
                    </div>
                </div>
            </div>
        }
    });

    html! {
        <aside class={format!("w-[260px] shrink-0 {} transition-colors duration-200 flex flex-col p-3.5 select-none overflow-hidden", props.inspector_bg_class)}>
            // Virtual Titlebar drag region
            <div onmousedown={props.on_start_drag.clone()} class="h-9 w-full shrink-0 flex items-center justify-between">
                // Segmented tab switch
                <div class="flex items-center p-0.5 bg-black/20 rounded-lg border border-white/[0.06] text-[11px] font-medium">
                    <button
                        onclick={set_details}
                        class={format!(
                            "px-2.5 py-1 rounded-md transition-all cursor-pointer {}",
                            if *active_tab == "details" { "bg-white/[0.15] text-white shadow-sm" } else { "text-gray-400 hover:text-gray-200" }
                        )}
                    >
                        {"Szczegóły"}
                    </button>
                    <button
                        onclick={set_logs}
                        class={format!(
                            "px-2.5 py-1 rounded-md transition-all cursor-pointer {}",
                            if *active_tab == "logs" { "bg-white/[0.15] text-white shadow-sm" } else { "text-gray-400 hover:text-gray-200" }
                        )}
                    >
                        { format!("Logi ({})", props.logs.len()) }
                    </button>
                </div>
            </div>

            if *active_tab == "details" {
                <div class="flex-1 flex flex-col gap-3 overflow-y-auto mt-2">
                    { device_card }

                    // Game Details Card
                    <div class="bg-white/[0.05] backdrop-blur-md border border-white/[0.08] rounded-2xl p-3.5 flex flex-col gap-3 shadow-md">
                        <div>
                            <h3 class="text-[12.5px] font-semibold text-white mb-1">
                                { format!("Gra #{}", game.id()) }
                            </h3>
                            <p class="text-[11px] text-gray-400 leading-relaxed">
                                {"Stan tablicy wyników koszykówki zsynchronizowany w czasie rzeczywistym."}
                            </p>
                        </div>

                        <div class="border-t border-white/[0.06]" />

                        <div class="flex flex-col gap-2 text-[11.5px]">
                            <div class="flex justify-between items-center">
                                <span class="text-gray-400">{"Dyscyplina"}</span>
                                <span class="text-gray-200 font-medium">{"FIBA 5x5"}</span>
                            </div>

                            <div class="flex justify-between items-center">
                                <span class="text-gray-400">{"Status"}</span>
                                <span class="font-mono text-white">{ format!("{:?}", game.state()) }</span>
                            </div>

                            <div class="flex justify-between items-center">
                                <span class="text-gray-400">{"Kwarta"}</span>
                                <span class="font-mono text-white">{ format!("{:?}", game.quarter()) }</span>
                            </div>

                            <div class="flex justify-between items-center">
                                <span class="text-gray-400">{"Czas gry"}</span>
                                <span class="font-mono text-amber-300">{ format!("{:02}:{:02}.{:02}", reg_time.minutes, reg_time.seconds, reg_time.centiseconds) }</span>
                            </div>

                            <div class="flex justify-between items-center">
                                <span class="text-gray-400">{"Wynik (H : A)"}</span>
                                <span class="font-mono text-white font-semibold">{ format!("{} : {}", scores.home, scores.away) }</span>
                            </div>
                        </div>
                    </div>
                </div>
            } else {
                // Logs Tab
                <div class="flex-1 flex flex-col min-h-0 mt-2 bg-black/30 border border-white/[0.06] rounded-xl p-2.5 font-mono text-[10.5px] text-gray-300 overflow-y-auto space-y-1">
                    if props.logs.is_empty() {
                        <div class="text-gray-500 italic">{"Brak wpisów logów"}</div>
                    } else {
                        { for props.logs.iter().rev().map(|log| {
                            html! {
                                <div class="break-words border-b border-white/[0.03] pb-0.5 leading-snug">
                                    { log }
                                </div>
                            }
                        })}
                    }
                </div>
            }
        </aside>
    }
}
