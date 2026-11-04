//! Native macOS right details inspector component showing selected Game or Device metadata & logs in Leptos.

use basketball::{Game, Scorable};
use config::DeviceConfig;
use leptos::prelude::*;
use net::Device;

#[component]
pub fn LapapiInspector(
    selected_game: Signal<Game>,
    selected_device: Signal<Option<Device>>,
    logs: Signal<Vec<String>>,
    on_start_drag: Callback<web_sys::MouseEvent>,
    inspector_bg_class: Signal<&'static str>,
    on_disconnect_device: Callback<Device>,
    on_open_wifi_config: Callback<Device>,
) -> impl IntoView {
    let active_tab = RwSignal::new("details"); // "details" or "logs"

    view! {
        <aside class=move || format!("w-[240px] shrink-0 {} transition-colors duration-200 flex flex-col p-3 select-none overflow-hidden", inspector_bg_class.get())>
            // Titlebar Header Region with macOS Segmented Control
            <div on:mousedown=move |e| on_start_drag.run(e) class="h-9 w-full shrink-0 flex items-center justify-between">
                <div class="flex items-center p-0.5 bg-black/25 rounded-[6px] border border-white/[0.06] text-[11px] font-medium w-full">
                    <button
                        on:click=move |_| active_tab.set("details")
                        class=move || format!(
                            "flex-1 text-center py-0.5 rounded-[4px] transition-colors cursor-pointer {}",
                            if active_tab.get() == "details" { "bg-white/[0.14] text-white font-medium" } else { "text-white/45 hover:text-white/75" }
                        )
                    >
                        {"Szczegóły"}
                    </button>
                    <button
                        on:click=move |_| active_tab.set("logs")
                        class=move || format!(
                            "flex-1 text-center py-0.5 rounded-[4px] transition-colors cursor-pointer {}",
                            if active_tab.get() == "logs" { "bg-white/[0.14] text-white font-medium" } else { "text-white/45 hover:text-white/75" }
                        )
                    >
                        {move || format!("Logi ({})", logs.get().len())}
                    </button>
                </div>
            </div>

            {move || {
                if active_tab.get() == "details" {
                    let game = selected_game.get();
                    let scores = game.scores();
                    let reg_time = game.regulation_time();
                    let dev_opt = selected_device.get();

                    view! {
                        <div class="flex-1 flex flex-col gap-2.5 overflow-y-auto mt-2">
                            // Device Card if selected
                            {dev_opt.map(|dev| {
                                let dev_copy = dev;
                                let dev_for_wifi = dev;
                                let dev_type_str = format!("{:?}", dev.device_type());
                                let dev_id_str = dev.device_id().unwrap_or("-").to_string();
                                let dev_sn_str = dev.serial_number().unwrap_or("-").to_string();
                                let dev_game_id_str = format!("#{}", dev.game_id());

                                view! {
                                    <div class="bg-white/[0.04] backdrop-blur-md border border-white/[0.08] rounded-xl p-3 flex flex-col gap-2">
                                        <div class="flex items-center justify-between">
                                            <h3 class="text-[12px] font-medium text-white/90">{"Wybrane urządzenie"}</h3>
                                            <div class="flex items-center gap-1">
                                                <button
                                                    on:click=move |_| on_open_wifi_config.run(dev_for_wifi)
                                                    title="Konfiguracja sieci Wi-Fi urządzenia przez UDP"
                                                    class="text-[10px] px-2 py-0.5 bg-blue-600/25 hover:bg-blue-600/40 text-blue-300 rounded-[4px] cursor-pointer transition-colors border border-blue-400/20 font-medium"
                                                >
                                                    {"Wi-Fi"}
                                                </button>
                                                <button
                                                    on:click=move |_| on_disconnect_device.run(dev_copy)
                                                    class="text-[10px] px-1.5 py-0.5 bg-rose-600/20 hover:bg-rose-600/35 text-rose-300 rounded-[4px] cursor-pointer transition-colors border border-rose-400/20 font-medium"
                                                >
                                                    {"Rozłącz"}
                                                </button>
                                            </div>
                                        </div>
                                        <div class="space-y-1 text-[11px] text-white/70">
                                            <div class="flex justify-between">
                                                <span class="text-white/40">{"Typ:"}</span>
                                                <span class="font-mono text-white/90">{dev_type_str}</span>
                                            </div>
                                            <div class="flex justify-between">
                                                <span class="text-white/40">{"ID:"}</span>
                                                <span class="font-mono text-white/80">{dev_id_str}</span>
                                            </div>
                                            <div class="flex justify-between">
                                                <span class="text-white/40">{"Numer seryjny:"}</span>
                                                <span class="font-mono truncate max-w-[110px] text-white/80">{dev_sn_str}</span>
                                            </div>
                                            <div class="flex justify-between">
                                                <span class="text-white/40">{"Przypisana gra:"}</span>
                                                <span class="font-mono text-cyan-300 font-medium">{dev_game_id_str}</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}

                            // Game Details Card
                            <div class="bg-white/[0.04] backdrop-blur-md border border-white/[0.08] rounded-xl p-3 flex flex-col gap-2">
                                <div>
                                    <h3 class="text-[12px] font-medium text-white/90">
                                        {format!("Gra #{}", game.id())}
                                    </h3>
                                    <p class="text-[11px] text-white/40 mt-0.5">
                                        {"Stan tablicy wyników zsynchronizowany w czasie rzeczywistym."}
                                    </p>
                                </div>

                                <div class="border-t border-white/[0.06]" />

                                <div class="flex flex-col gap-1.5 text-[11px]">
                                    <div class="flex items-center justify-between">
                                        <span class="text-white/40">{"Wynik:"}</span>
                                        <span class="font-mono tabular-nums text-white/90 font-medium">
                                            {format!("{}:{}", scores.home, scores.away)}
                                        </span>
                                    </div>
                                    <div class="flex items-center justify-between">
                                        <span class="text-white/40">{"Czas gry:"}</span>
                                        <span class="font-mono tabular-nums text-white/80">
                                            {format!("{:02}:{:02}", reg_time.minutes, reg_time.seconds)}
                                        </span>
                                    </div>
                                    <div class="flex items-center justify-between">
                                        <span class="text-white/40">{"Stan:"}</span>
                                        <span class="text-white/80">{format!("{:?}", game.state())}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex-1 flex flex-col bg-black/35 rounded-xl border border-white/[0.06] p-2 overflow-y-auto mt-2 font-mono text-[10px] text-white/65 space-y-1">
                            {move || {
                                let log_items = logs.get();
                                if log_items.is_empty() {
                                    view! {
                                        <div class="text-white/30 italic">{"Brak zarejestrowanych logów..."}</div>
                                    }.into_any()
                                } else {
                                    log_items.into_iter().rev().map(|log| {
                                        view! {
                                            <div class="truncate select-text hover:text-white/90 transition-colors">
                                                {log}
                                            </div>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </div>
                    }.into_any()
                }
            }}
        </aside>
    }
}
