//! Native macOS/Linux UI for HMI with Leptos architecture.

use crate::ui::traffic_lights::NavigationBar;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let is_focused = RwSignal::new(true);

    // Listen to focus changes (both DOM window events and Tauri custom events)
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let on_focus = Closure::<dyn FnMut()>::new(move || {
                is_focused.set(true);
            });
            let on_blur = Closure::<dyn FnMut()>::new(move || {
                is_focused.set(false);
            });
            let _ = window.add_event_listener_with_callback("focus", on_focus.as_ref().unchecked_ref());
            let _ = window.add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref());
            on_focus.forget();
            on_blur.forget();

            if let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
                && let Ok(event_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("event"))
                && let Ok(listen_fn) = js_sys::Reflect::get(&event_obj, &JsValue::from_str("listen"))
                && let Ok(func) = listen_fn.dyn_into::<js_sys::Function>()
            {
                let on_tauri_focus = Closure::<dyn FnMut(JsValue)>::new(move |val: JsValue| {
                    if let Ok(payload) = js_sys::Reflect::get(&val, &JsValue::from_str("payload")) {
                        if let Some(focused) = payload.as_bool() {
                            is_focused.set(focused);
                        }
                    }
                });
                let _ = func.call2(
                    &event_obj,
                    &JsValue::from_str("hmi-focus-changed"),
                    on_tauri_focus.as_ref().unchecked_ref(),
                );
                on_tauri_focus.forget();
            }
        }
    });

    view! {
        <div class="w-screen h-screen flex flex-col bg-neutral-100 dark:bg-[#121216] text-neutral-900 dark:text-neutral-100 select-none overflow-hidden font-sans">
            <NavigationBar focused=is_focused title="rp235x-hmi" />

            <div class="flex-1 flex flex-col items-center justify-center gap-8 relative overflow-hidden">
                // Background ambient pulse glow
                <div class="absolute w-72 h-72 rounded-full bg-gradient-to-tr from-[#FF6435]/15 via-amber-500/10 to-indigo-500/10 blur-3xl pointer-events-none animate-pulse-ring"></div>

                // Central orbital animation
                <div class="relative w-36 h-36 flex items-center justify-center animate-float">
                    // Outer static and dashed rotating orbit rings
                    <div class="absolute inset-0 rounded-full border border-neutral-300 dark:border-white/10"></div>
                    <div class="absolute inset-0 rounded-full border border-dashed border-neutral-400/40 dark:border-white/20 animate-orbit"></div>

                    // Middle accented rotating arc
                    <div class="absolute inset-3 rounded-full border-2 border-transparent border-t-[#FF6435] border-r-[#FF6435]/40 animate-orbit-rev"></div>

                    // Inner counter-rotating ring with tracking beacon
                    <div class="absolute inset-6 rounded-full border border-neutral-300 dark:border-white/15 animate-orbit">
                        <span class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 rounded-full bg-[#FF6435] shadow-[0_0_8px_#FF6435]"></span>
                    </div>

                    // Center glowing pulsing core
                    <div class="relative flex items-center justify-center">
                        <div class="absolute w-8 h-8 rounded-full bg-[#FF6435]/25 animate-ping"></div>
                        <div class="w-6 h-6 rounded-full bg-gradient-to-br from-[#FF6435] to-[#e04b1e] shadow-lg shadow-[#FF6435]/50 flex items-center justify-center">
                            <div class="w-2 h-2 rounded-full bg-white/95"></div>
                        </div>
                    </div>
                </div>

                // Animated telemetry bars and indicator label
                <div class="flex flex-col items-center gap-3 relative z-10">
                    <div class="flex items-center gap-1.5 h-7">
                        <span class="w-1 rounded-full bg-[#FF6435]/70 animate-bar-1"></span>
                        <span class="w-1 rounded-full bg-[#FF6435]/80 animate-bar-2"></span>
                        <span class="w-1 rounded-full bg-[#FF6435] animate-bar-3"></span>
                        <span class="w-1 rounded-full bg-[#FF6435]/80 animate-bar-4"></span>
                        <span class="w-1 rounded-full bg-[#FF6435]/70 animate-bar-5"></span>
                    </div>
                    <div class="flex items-center gap-2 text-[11px] font-medium tracking-widest uppercase text-neutral-400 dark:text-neutral-500">
                        <span class="w-1.5 h-1.5 rounded-full bg-[#FF6435] animate-pulse"></span>
                        <span>"System Idle • Telemetry Active"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
