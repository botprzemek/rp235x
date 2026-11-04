//! Native macOS modal sheet for Wi-Fi device configuration in Leptos.

use leptos::prelude::*;

#[component]
pub fn WifiConfigModal(
    is_open: Signal<bool>,
    on_close: Callback<web_sys::MouseEvent>,
    on_submit: Callback<(String, String, String, String, String)>,
    is_loading: Signal<bool>,
    status_message: Signal<Option<String>>,
    initial_target_ip: Signal<String>,
    initial_serial_number: Signal<String>,
    initial_device_id: Signal<String>,
) -> impl IntoView {
    let target_ip = RwSignal::new(String::new());
    let serial_number = RwSignal::new(String::new());
    let device_id = RwSignal::new(String::new());
    let wifi_ssid = RwSignal::new(String::new());
    let wifi_password = RwSignal::new(String::new());

    Effect::new(move |_| {
        if is_open.get() {
            target_ip.set(initial_target_ip.get());
            serial_number.set(initial_serial_number.get());
            device_id.set(initial_device_id.get());
        }
    });

    let stop_prop = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
    };

    let on_submit_click = move |e: web_sys::MouseEvent| {
        e.prevent_default();
        on_submit.run((
            target_ip.get(),
            serial_number.get(),
            device_id.get(),
            wifi_ssid.get(),
            wifi_password.get(),
        ));
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
                            <span class="w-2 h-2 rounded-full bg-blue-400" />
                            <h3 class="text-[13px] font-medium text-white/95">
                                {"Konfiguracja Workera (OTA Config)"}
                            </h3>
                        </div>
                        <button
                            on:click=move |e| on_close.run(e)
                            class="px-2 py-0.5 rounded-[4px] text-[11px] text-white/50 hover:text-white hover:bg-white/[0.08] cursor-pointer transition-colors"
                        >
                            {"Zamknij"}
                        </button>
                    </div>

                    // Fields
                    <div class="flex flex-col gap-2.5 text-[11px]">
                        <div>
                            <label class="block text-white/50 mb-1 font-medium">{"Adres docelowy (IP / Port 8001)"}</label>
                            <input
                                type="text"
                                prop:value=move || target_ip.get()
                                on:input=move |e| target_ip.set(event_target_value(&e))
                                placeholder="192.168.4.1"
                                class="w-full bg-black/30 border border-white/[0.1] rounded-[6px] px-2.5 py-1 text-white/90 font-mono text-[12px] focus:border-blue-500 focus:outline-none"
                            />
                            <span class="text-[10px] text-white/35 mt-0.5 block">{"Domyślnie w trybie AP: 192.168.4.1"}</span>
                        </div>

                        <div class="grid grid-cols-2 gap-2">
                            <div>
                                <label class="block text-white/50 mb-1 font-medium">{"Numer seryjny"}</label>
                                <input
                                    type="text"
                                    maxlength="31"
                                    prop:value=move || serial_number.get()
                                    on:input=move |e| serial_number.set(event_target_value(&e))
                                    placeholder="sn-lap-0000"
                                    class="w-full bg-black/30 border border-white/[0.1] rounded-[6px] px-2.5 py-1 text-white/90 font-mono text-[12px] focus:border-blue-500 focus:outline-none"
                                />
                            </div>
                            <div>
                                <label class="block text-white/50 mb-1 font-medium">{"ID urządzenia"}</label>
                                <input
                                    type="text"
                                    maxlength="31"
                                    prop:value=move || device_id.get()
                                    on:input=move |e| device_id.set(event_target_value(&e))
                                    placeholder="lapapi-app"
                                    class="w-full bg-black/30 border border-white/[0.1] rounded-[6px] px-2.5 py-1 text-white/90 font-mono text-[12px] focus:border-blue-500 focus:outline-none"
                                />
                            </div>
                        </div>

                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <label class="text-white/50 font-medium">{"Nazwa sieci (SSID)"}</label>
                                <span class="text-[10px] font-mono tabular-nums text-white/40">
                                    {move || format!("{}/32 B", wifi_ssid.get().as_bytes().len())}
                                </span>
                            </div>
                            <input
                                type="text"
                                maxlength="32"
                                prop:value=move || wifi_ssid.get()
                                on:input=move |e| wifi_ssid.set(event_target_value(&e))
                                placeholder="Wprowadź SSID sieci Wi-Fi..."
                                class="w-full bg-black/30 border border-white/[0.1] rounded-[6px] px-2.5 py-1 text-white/90 text-[12px] focus:border-blue-500 focus:outline-none"
                            />
                        </div>

                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <label class="text-white/50 font-medium">{"Hasło sieci Wi-Fi"}</label>
                                <span class="text-[10px] font-mono tabular-nums text-white/40">
                                    {move || format!("{}/64 B", wifi_password.get().as_bytes().len())}
                                </span>
                            </div>
                            <input
                                type="password"
                                maxlength="64"
                                prop:value=move || wifi_password.get()
                                on:input=move |e| wifi_password.set(event_target_value(&e))
                                placeholder="Wprowadź hasło Wi-Fi..."
                                class="w-full bg-black/30 border border-white/[0.1] rounded-[6px] px-2.5 py-1 text-white/90 text-[12px] focus:border-blue-500 focus:outline-none"
                            />
                        </div>
                    </div>

                    // Status Message
                    {move || {
                        status_message.get().map(|msg| {
                            let is_err = msg.starts_with("Błąd");
                            view! {
                                <div class=format!(
                                    "p-2 rounded-[5px] text-[11px] font-mono border {}",
                                    if is_err { "bg-rose-500/15 border-rose-500/25 text-rose-300" } else { "bg-emerald-500/15 border-emerald-500/25 text-emerald-300" }
                                )>
                                    {msg}
                                </div>
                            }
                        })
                    }}

                    // Footer Buttons
                    <div class="flex items-center justify-end gap-2 border-t border-white/[0.08] pt-3">
                        <button
                            on:click=move |e| on_close.run(e)
                            class="px-3 py-1 rounded-[6px] text-[12px] font-medium text-white/70 hover:text-white bg-white/[0.06] hover:bg-white/[0.12] border border-white/[0.08] cursor-pointer transition-colors"
                        >
                            {"Anuluj"}
                        </button>
                        <button
                            disabled=move || is_loading.get()
                            on:click=on_submit_click
                            class="px-3 py-1 rounded-[6px] text-[12px] font-medium text-white bg-blue-600 hover:bg-blue-500 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed transition-colors shadow-sm"
                        >
                            {move || if is_loading.get() { "Wysyłanie..." } else { "Zapisz i wyślij" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
