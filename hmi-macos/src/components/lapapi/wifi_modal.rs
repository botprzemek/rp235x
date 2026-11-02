//! Wi-Fi device configuration modal dialog.

use yew::{
    Callback, Html, InputEvent, MouseEvent, Properties, TargetCast, function_component, html,
    use_state,
};

#[derive(Properties, PartialEq, Clone)]
pub struct WifiConfigModalProps {
    pub is_open: bool,
    pub on_close: Callback<MouseEvent>,
    pub on_submit: Callback<(String, String, String, String, String)>,
    pub is_loading: bool,
    pub status_message: Option<String>,
    pub initial_target_ip: String,
    pub initial_serial_number: String,
    pub initial_device_id: String,
}

#[function_component(WifiConfigModal)]
pub fn wifi_config_modal(props: &WifiConfigModalProps) -> Html {
    let target_ip = use_state(|| props.initial_target_ip.clone());
    let serial_number = use_state(|| props.initial_serial_number.clone());
    let device_id = use_state(|| props.initial_device_id.clone());
    let wifi_ssid = use_state(String::new);
    let wifi_password = use_state(String::new);

    // Sync state when initial props change
    {
        let target_ip = target_ip.clone();
        let serial_number = serial_number.clone();
        let device_id = device_id.clone();
        let init_ip = props.initial_target_ip.clone();
        let init_sn = props.initial_serial_number.clone();
        let init_id = props.initial_device_id.clone();
        yew::use_effect_with(
            (props.is_open, init_ip.clone(), init_sn.clone(), init_id.clone()),
            move |(open, ip, sn, id)| {
                if *open {
                    target_ip.set(ip.clone());
                    serial_number.set(sn.clone());
                    device_id.set(id.clone());
                }
                || ()
            },
        );
    }

    if !props.is_open {
        return html! {};
    }

    let on_ip_input = {
        let target_ip = target_ip.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                target_ip.set(target.value());
            }
        })
    };

    let on_sn_input = {
        let serial_number = serial_number.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                serial_number.set(target.value());
            }
        })
    };

    let on_id_input = {
        let device_id = device_id.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                device_id.set(target.value());
            }
        })
    };

    let on_ssid_input = {
        let wifi_ssid = wifi_ssid.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                wifi_ssid.set(target.value());
            }
        })
    };

    let on_pass_input = {
        let wifi_password = wifi_password.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                wifi_password.set(target.value());
            }
        })
    };

    let on_submit_click = {
        let on_submit = props.on_submit.clone();
        let target_ip = target_ip.clone();
        let serial_number = serial_number.clone();
        let device_id = device_id.clone();
        let wifi_ssid = wifi_ssid.clone();
        let wifi_password = wifi_password.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            on_submit.emit((
                (*target_ip).clone(),
                (*serial_number).clone(),
                (*device_id).clone(),
                (*wifi_ssid).clone(),
                (*wifi_password).clone(),
            ));
        })
    };

    let stop_prop = Callback::from(|e: MouseEvent| {
        e.stop_propagation();
    });

    html! {
        <div onclick={props.on_close.clone()} class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md p-4">
            <div onclick={stop_prop} class="bg-[#1a1e28] border border-white/[0.12] rounded-2xl w-full max-w-md shadow-2xl p-5 flex flex-col gap-4 text-white">
                <div class="flex items-center justify-between border-b border-white/[0.08] pb-3">
                    <h3 class="text-sm font-semibold flex items-center gap-2">
                        <span class="text-blue-400">{"📶"}</span>
                        {"Konfiguracja sieci Wi-Fi (OTA UDP)"}
                    </h3>
                    <button
                        onclick={props.on_close.clone()}
                        class="text-gray-400 hover:text-white text-lg leading-none cursor-pointer"
                    >
                        {"✕"}
                    </button>
                </div>

                <div class="flex flex-col gap-3 text-xs">
                    <div>
                        <label class="block text-gray-400 mb-1 font-medium">{"Adres docelowy (IP / Port 8001):"}</label>
                        <input
                            type="text"
                            value={(*target_ip).clone()}
                            oninput={on_ip_input}
                            placeholder="192.168.4.1 lub 192.168.0.x"
                            class="w-full bg-black/30 border border-white/[0.1] rounded-lg px-3 py-1.5 text-white font-mono focus:border-blue-500 focus:outline-none"
                        />
                        <span class="text-[10px] text-gray-500 mt-0.5 block">{"Domyślnie w trybie AP: 192.168.4.1"}</span>
                    </div>

                    <div class="grid grid-cols-2 gap-2">
                        <div>
                            <label class="block text-gray-400 mb-1 font-medium">{"Numer seryjny:"}</label>
                            <input
                                type="text"
                                maxlength="31"
                                value={(*serial_number).clone()}
                                oninput={on_sn_input}
                                placeholder="sn-lap-0000"
                                class="w-full bg-black/30 border border-white/[0.1] rounded-lg px-3 py-1.5 text-white font-mono focus:border-blue-500 focus:outline-none"
                            />
                        </div>
                        <div>
                            <label class="block text-gray-400 mb-1 font-medium">{"ID Urządzenia:"}</label>
                            <input
                                type="text"
                                maxlength="31"
                                value={(*device_id).clone()}
                                oninput={on_id_input}
                                placeholder="lapapi-app"
                                class="w-full bg-black/30 border border-white/[0.1] rounded-lg px-3 py-1.5 text-white font-mono focus:border-blue-500 focus:outline-none"
                            />
                        </div>
                    </div>

                    <div>
                        <label class="block text-gray-400 mb-1 font-medium">{"SSID Sieci Wi-Fi:"}</label>
                        <input
                            type="text"
                            maxlength="31"
                            value={(*wifi_ssid).clone()}
                            oninput={on_ssid_input}
                            placeholder="Nazwa sieci Wi-Fi (do 31 znaków)"
                            class="w-full bg-black/30 border border-white/[0.1] rounded-lg px-3 py-1.5 text-white focus:border-blue-500 focus:outline-none"
                        />
                    </div>

                    <div>
                        <label class="block text-gray-400 mb-1 font-medium">{"Hasło Wi-Fi:"}</label>
                        <input
                            type="password"
                            maxlength="63"
                            value={(*wifi_password).clone()}
                            oninput={on_pass_input}
                            placeholder="Hasło Wi-Fi (do 63 znaków)"
                            class="w-full bg-black/30 border border-white/[0.1] rounded-lg px-3 py-1.5 text-white focus:border-blue-500 focus:outline-none"
                        />
                    </div>

                    if let Some(ref msg) = props.status_message {
                        <div class="p-2.5 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-300 text-[11px] leading-relaxed">
                            { msg }
                        </div>
                    }
                </div>

                <div class="flex items-center justify-end gap-2 border-t border-white/[0.08] pt-3">
                    <button
                        onclick={props.on_close.clone()}
                        class="px-3 py-1.5 rounded-lg text-xs font-medium text-gray-400 hover:text-white hover:bg-white/[0.05] transition-colors cursor-pointer"
                    >
                        {"Anuluj"}
                    </button>
                    <button
                        onclick={on_submit_click}
                        disabled={props.is_loading || (*wifi_ssid).is_empty()}
                        class="px-4 py-1.5 rounded-lg text-xs font-medium bg-blue-600 hover:bg-blue-500 text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors shadow-sm cursor-pointer"
                    >
                        if props.is_loading {
                            {"Wysyłanie UDP..."}
                        } else {
                            {"Zapisz do Flash (UDP)"}
                        }
                    </button>
                </div>
            </div>
        </div>
    }
}
