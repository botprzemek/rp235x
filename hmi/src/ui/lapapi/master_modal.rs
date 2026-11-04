//! Native macOS modal sheet for configuring Master server address in Leptos,
//! with automatic UDP network discovery, loading state, and manual fallback on timeout.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::tauri::invoke_discover_master;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryState {
    Searching,
    Found,
    TimedOut,
}

#[component]
pub fn MasterServerModal(
    is_open: Signal<bool>,
    current_address: Signal<String>,
    on_close: Callback<web_sys::MouseEvent>,
    on_save: Callback<String>,
) -> impl IntoView {
    let address_input = RwSignal::new(String::new());
    let discovery_state = RwSignal::new(DiscoveryState::Searching);
    let status_msg = RwSignal::new(None::<String>);

    // Function to trigger discovery
    let start_discovery = move || {
        discovery_state.set(DiscoveryState::Searching);
        status_msg.set(Some("Wyszukiwanie serwera Master w sieci lokalnej (UDP)...".to_string()));

        spawn_local(async move {
            match invoke_discover_master(Some(4)).await {
                Ok(discovered_addr) => {
                    discovery_state.set(DiscoveryState::Found);
                    address_input.set(discovered_addr.clone());
                    status_msg.set(Some(format!("Znaleziono serwer Master: {}", discovered_addr)));
                }
                Err(err) => {
                    discovery_state.set(DiscoveryState::TimedOut);
                    status_msg.set(Some(format!("{}. Możesz wpisać adres ręcznie.", err)));
                }
            }
        });
    };

    // When modal opens, initialize address and automatically run discovery
    Effect::new(move |_| {
        if is_open.get() {
            address_input.set(current_address.get());
            start_discovery();
        }
    });

    let stop_prop = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
    };

    let on_save_click = move |e: web_sys::MouseEvent| {
        e.prevent_default();
        let val = address_input.get().trim().to_string();
        if !val.is_empty() {
            on_save.run(val);
        }
    };

    let on_retry_discovery = move |e: web_sys::MouseEvent| {
        e.prevent_default();
        start_discovery();
    };

    view! {
        <Show when=move || is_open.get()>
            <div
                on:click=move |e| on_close.run(e)
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 select-none"
            >
                <div
                    on:click=stop_prop
                    class="bg-[#242832]/95 border border-white/[0.12] rounded-xl w-full max-w-[420px] shadow-2xl p-4 flex flex-col gap-3.5 text-white/90"
                >
                    // Modal Header
                    <div class="flex items-center justify-between border-b border-white/[0.08] pb-2.5">
                        <div class="flex items-center gap-2">
                            <span class=move || format!(
                                "w-2 h-2 rounded-full {}",
                                match discovery_state.get() {
                                    DiscoveryState::Searching => "bg-amber-400 animate-pulse",
                                    DiscoveryState::Found => "bg-emerald-400",
                                    DiscoveryState::TimedOut => "bg-rose-400",
                                }
                            ) />
                            <h3 class="text-[13px] font-medium text-white/95">
                                {"Konfiguracja połączenia (Master)"}
                            </h3>
                        </div>
                        <button
                            on:click=move |e| on_close.run(e)
                            class="px-2 py-0.5 rounded-[4px] text-[11px] text-white/50 hover:text-white hover:bg-white/[0.08] cursor-pointer transition-colors"
                        >
                            {"Zamknij"}
                        </button>
                    </div>

                    // Discovery Banner / Loading / Result
                    {move || {
                        match discovery_state.get() {
                            DiscoveryState::Searching => view! {
                                <div class="p-3 bg-blue-500/10 border border-blue-500/20 rounded-[8px] flex items-center gap-3">
                                    <div class="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin shrink-0" />
                                    <div class="flex-1">
                                        <div class="text-[12px] font-medium text-blue-300">
                                            {"Wyszukiwanie Mastera..."}
                                        </div>
                                        <div class="text-[10px] text-white/50 mt-0.5">
                                            {"Rozsyłanie zapytania UDP broadcast w sieci lokalnej..."}
                                        </div>
                                    </div>
                                </div>
                            }.into_any(),
                            DiscoveryState::Found => view! {
                                <div class="p-2.5 bg-emerald-500/15 border border-emerald-500/25 rounded-[8px] flex items-center justify-between">
                                    <div class="flex items-center gap-2">
                                        <span class="w-2 h-2 rounded-full bg-emerald-400 shrink-0" />
                                        <div class="text-[11px] text-emerald-200">
                                            {"Znaleziono serwer Master w sieci!"}
                                        </div>
                                    </div>
                                    <button
                                        on:click=on_retry_discovery
                                        class="text-[10px] text-white/50 hover:text-white underline cursor-pointer"
                                    >
                                        {"Szukaj ponownie"}
                                    </button>
                                </div>
                            }.into_any(),
                            DiscoveryState::TimedOut => view! {
                                <div class="p-2.5 bg-amber-500/15 border border-amber-500/25 rounded-[8px] flex items-center justify-between">
                                    <div class="text-[11px] text-amber-200">
                                        {"Nie wykryto serwera automatycznie (timeout)."}
                                    </div>
                                    <button
                                        on:click=on_retry_discovery
                                        class="text-[10px] px-2 py-0.5 bg-white/[0.08] hover:bg-white/[0.14] text-white/80 rounded-[4px] cursor-pointer transition-colors"
                                    >
                                        {"Ponów próbę"}
                                    </button>
                                </div>
                            }.into_any(),
                        }
                    }}

                    // Manual Input Field (Always available or unlocked after searching/timeout)
                    <div class="flex flex-col gap-2.5 text-[11px]">
                        <div>
                            <div class="flex items-center justify-between mb-1">
                                <label class="text-white/60 font-medium">
                                    {"Adres Mastera (IP:Port lub Host:Port):"}
                                </label>
                                {move || if discovery_state.get() == DiscoveryState::Searching {
                                    view! {
                                        <span class="text-[10px] text-amber-300/80 italic">{"Wyszukiwanie w toku..."}</span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <span class="text-[10px] text-white/40">{"Wprowadzanie ręczne"}</span>
                                    }.into_any()
                                }}
                            </div>
                            <input
                                type="text"
                                prop:value=move || address_input.get()
                                on:input=move |e| address_input.set(event_target_value(&e))
                                placeholder="127.0.0.1:3000 lub 192.168.1.100:3000"
                                class="w-full bg-black/30 border border-white/[0.1] rounded-[6px] px-2.5 py-1.5 text-white/90 font-mono text-[12px] focus:border-blue-500 focus:outline-none"
                            />
                            <span class="text-[10px] text-white/40 mt-1 block">
                                {"Możesz podać lokalny adres serwera lub pozostawić domyślny "}
                                <code class="text-blue-300">{"127.0.0.1:3000"}</code>
                            </span>
                        </div>
                    </div>

                    // Actions
                    <div class="flex items-center justify-between pt-2 border-t border-white/[0.08]">
                        <button
                            on:click=move |_| address_input.set("127.0.0.1:3000".to_string())
                            class="text-[11px] text-white/50 hover:text-white/80 underline cursor-pointer"
                        >
                            {"Ustaw domyślne (localhost)"}
                        </button>

                        <div class="flex items-center gap-2">
                            <button
                                on:click=move |e| on_close.run(e)
                                class="px-3 py-1 rounded-[5px] text-[11px] text-white/60 hover:text-white hover:bg-white/[0.06] transition-colors cursor-pointer"
                            >
                                {"Anuluj"}
                            </button>
                            <button
                                on:click=on_save_click
                                class="px-3.5 py-1 rounded-[5px] text-[11px] font-medium bg-blue-600 hover:bg-blue-500 text-white shadow-sm transition-colors cursor-pointer"
                            >
                                {"Połącz"}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
