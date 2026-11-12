//! macOS Traffic Light window control button components (`CloseButton`, `MinimizeButton`, `MaximizeButton`, `TrafficLights`, and `NavigationBar`).

use crate::adapters::tauri::invoke_tauri_window_command;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// Visual state of a macOS traffic light button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficLightState {
    NoFocus,
    Normal,
    Hover,
    Press,
}

fn resolve_state(focused: bool, hovered: bool, pressed: bool) -> TrafficLightState {
    if pressed {
        TrafficLightState::Press
    } else if hovered {
        TrafficLightState::Hover
    } else if focused {
        TrafficLightState::Normal
    } else {
        TrafficLightState::NoFocus
    }
}

/// Hook that distinguishes a quick click from a click-and-hold (long-press).
#[allow(clippy::type_complexity)]
fn use_click_and_hold(
    onclick: Callback<web_sys::MouseEvent>,
    onhold: Callback<()>,
    hold_delay_ms: i32,
) -> (
    RwSignal<bool>,
    impl Fn(web_sys::MouseEvent) + Copy + 'static,
    impl Fn(web_sys::MouseEvent) + Copy + 'static,
    impl Fn(web_sys::MouseEvent) + Copy + 'static,
    impl Fn(web_sys::MouseEvent) + Copy + 'static,
) {
    let pressed = RwSignal::new(false);
    let timer_id = StoredValue::new(None::<i32>);
    let held = StoredValue::new(false);

    let clear_timer = move || {
        if let Some(id) = timer_id.get_value() {
            if let Some(w) = web_sys::window() {
                w.clear_timeout_with_handle(id);
            }
            timer_id.set_value(None);
        }
    };

    let on_mousedown = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        if e.button() != 0 {
            return;
        }
        pressed.set(true);
        held.set_value(false);
        clear_timer();

        let cb = Closure::wrap(Box::new(move || {
            held.set_value(true);
            onhold.run(());
        }) as Box<dyn FnMut()>);

        if let Some(w) = web_sys::window()
            && let Ok(id) = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                hold_delay_ms,
            )
        {
            timer_id.set_value(Some(id));
        }
        cb.forget();
    };

    let on_mouseup = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        pressed.set(false);
        clear_timer();
        if e.button() == 0 && !held.get_value() {
            onclick.run(e);
        }
        held.set_value(false);
    };

    let on_mouseleave = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        pressed.set(false);
        clear_timer();
        held.set_value(false);
    };

    let on_click = move |e: web_sys::MouseEvent| {
        e.prevent_default();
        e.stop_propagation();
    };

    (pressed, on_mousedown, on_mouseup, on_mouseleave, on_click)
}

#[component]
pub fn CloseButton(
    focused: Signal<bool>,
    hovered: Signal<bool>,
    #[prop(into, default = Callback::new(|_| ()))] onclick: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| ()))] onhold: Callback<()>,
    #[prop(into, default = 500)] hold_delay_ms: i32,
) -> impl IntoView {
    let (pressed, on_mousedown, on_mouseup, on_mouseleave, on_click) =
        use_click_and_hold(onclick, onhold, hold_delay_ms);
    let state = Signal::derive(move || resolve_state(focused.get(), hovered.get(), pressed.get()));

    view! {
        <button
            type="button"
            title="Zamknij"
            on:mousedown=on_mousedown
            on:mouseup=on_mouseup
            on:mouseleave=on_mouseleave
            on:click=on_click
            class="w-3 h-3 rounded-full flex items-center justify-center p-0 border-0 outline-none cursor-pointer bg-transparent pointer-events-auto"
        >
            {move || match state.get() {
                TrafficLightState::NoFocus => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#4a4d52" stroke="#3e4045" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Normal => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#ff5f56" stroke="#e0443e" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Hover => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#ff5f56" stroke="#e0443e" stroke-width="0.5"/>
                        <path d="M3.8 3.8 L8.2 8.2 M8.2 3.8 L3.8 8.2" stroke="#4d0000" stroke-width="1.2" stroke-linecap="round"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Press => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#bf4942" stroke="#993833" stroke-width="0.5"/>
                        <path d="M3.8 3.8 L8.2 8.2 M8.2 3.8 L3.8 8.2" stroke="#260000" stroke-width="1.2" stroke-linecap="round"/>
                    </svg>
                }.into_any(),
            }}
        </button>
    }
}

#[component]
pub fn MinimizeButton(
    focused: Signal<bool>,
    hovered: Signal<bool>,
    #[prop(into, default = Callback::new(|_| ()))] onclick: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| ()))] onhold: Callback<()>,
    #[prop(into, default = 500)] hold_delay_ms: i32,
) -> impl IntoView {
    let (pressed, on_mousedown, on_mouseup, on_mouseleave, on_click) =
        use_click_and_hold(onclick, onhold, hold_delay_ms);
    let state = Signal::derive(move || resolve_state(focused.get(), hovered.get(), pressed.get()));

    view! {
        <button
            type="button"
            title="Minimalizuj"
            on:mousedown=on_mousedown
            on:mouseup=on_mouseup
            on:mouseleave=on_mouseleave
            on:click=on_click
            class="w-3 h-3 rounded-full flex items-center justify-center p-0 border-0 outline-none cursor-pointer bg-transparent pointer-events-auto"
        >
            {move || match state.get() {
                TrafficLightState::NoFocus => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#4a4d52" stroke="#3e4045" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Normal => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#ffbd2e" stroke="#dea123" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Hover => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#ffbd2e" stroke="#dea123" stroke-width="0.5"/>
                        <path d="M2.5 6 L9.5 6" stroke="#5c3c00" stroke-width="1.2" stroke-linecap="round"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Press => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#bf8e22" stroke="#99711b" stroke-width="0.5"/>
                        <path d="M2.5 6 L9.5 6" stroke="#2e1e00" stroke-width="1.2" stroke-linecap="round"/>
                    </svg>
                }.into_any(),
            }}
        </button>
    }
}

#[component]
pub fn MaximizeButton(
    focused: Signal<bool>,
    hovered: Signal<bool>,
    #[prop(into, default = Callback::new(|_| ()))] onclick: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| ()))] onhold: Callback<()>,
    #[prop(into, default = 500)] hold_delay_ms: i32,
) -> impl IntoView {
    let (pressed, on_mousedown, on_mouseup, on_mouseleave, on_click) =
        use_click_and_hold(onclick, onhold, hold_delay_ms);
    let state = Signal::derive(move || resolve_state(focused.get(), hovered.get(), pressed.get()));

    view! {
        <button
            type="button"
            title="Maksymalizuj (przytrzymaj: pełny ekran)"
            on:mousedown=on_mousedown
            on:mouseup=on_mouseup
            on:mouseleave=on_mouseleave
            on:click=on_click
            class="w-3 h-3 rounded-full flex items-center justify-center p-0 border-0 outline-none cursor-pointer bg-transparent pointer-events-auto"
        >
            {move || match state.get() {
                TrafficLightState::NoFocus => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#4a4d52" stroke="#3e4045" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Normal => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#27c93f" stroke="#1aab29" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Hover => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#27c93f" stroke="#1aab29" stroke-width="0.5"/>
                        <path d="M3.5 3.5 L7.5 3.5 L7.5 7.5 Z M8.5 8.5 L4.5 8.5 L4.5 4.5 Z" fill="#003b0c"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Press => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3 pointer-events-none">
                        <circle cx="6" cy="6" r="5.5" fill="#1d972f" stroke="#13761c" stroke-width="0.5"/>
                        <path d="M3.5 3.5 L7.5 3.5 L7.5 7.5 Z M8.5 8.5 L4.5 8.5 L4.5 4.5 Z" fill="#001a05"/>
                    </svg>
                }.into_any(),
            }}
        </button>
    }
}

/// Standalone traffic lights button group (Close, Minimize, Maximize) with hover sync
/// and click-and-hold (long press) support.
#[component]
pub fn TrafficLights(
    #[prop(into, default = Signal::derive(|| true))] focused: Signal<bool>,
    #[prop(into, default = Callback::new(|_| invoke_tauri_window_command("window_close")))]
    on_close: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| invoke_tauri_window_command("window_minimize")))]
    on_minimize: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| invoke_tauri_window_command("window_toggle_maximize")))]
    on_maximize: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| ()))] on_hold_close: Callback<()>,
    #[prop(into, default = Callback::new(|_| ()))] on_hold_minimize: Callback<()>,
    #[prop(into, default = Callback::new(|_| invoke_tauri_window_command("window_toggle_fullscreen")))]
    on_hold_maximize: Callback<()>,
    #[prop(into, default = 500)] hold_delay_ms: i32,
) -> impl IntoView {
    let hovered = RwSignal::new(false);

    view! {
        <div
            on:mouseenter=move |_| hovered.set(true)
            on:mouseleave=move |_| hovered.set(false)
            class="flex items-center gap-2 pointer-events-auto"
        >
            <CloseButton
                focused=focused
                hovered=hovered.into()
                onclick=on_close
                onhold=on_hold_close
                hold_delay_ms=hold_delay_ms
            />
            <MinimizeButton
                focused=focused
                hovered=hovered.into()
                onclick=on_minimize
                onhold=on_hold_minimize
                hold_delay_ms=hold_delay_ms
            />
            <MaximizeButton
                focused=focused
                hovered=hovered.into()
                onclick=on_maximize
                onhold=on_hold_maximize
                hold_delay_ms=hold_delay_ms
            />
        </div>
    }
}

/// Standalone draggable navigation bar containing traffic light window controls.
#[component]
pub fn NavigationBar(
    #[prop(into, default = Signal::derive(|| true))] focused: Signal<bool>,
    #[prop(into, optional)] title: Option<&'static str>,
    #[prop(into, default = Callback::new(|_| invoke_tauri_window_command("window_close")))]
    on_close: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| invoke_tauri_window_command("window_minimize")))]
    on_minimize: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| invoke_tauri_window_command("window_toggle_maximize")))]
    on_maximize: Callback<web_sys::MouseEvent>,
    #[prop(into, default = Callback::new(|_| ()))] on_hold_close: Callback<()>,
    #[prop(into, default = Callback::new(|_| ()))] on_hold_minimize: Callback<()>,
    #[prop(into, default = Callback::new(|_| invoke_tauri_window_command("window_toggle_fullscreen")))]
    on_hold_maximize: Callback<()>,
    #[prop(into, default = 500)] hold_delay_ms: i32,
    #[prop(optional)] right_action: Option<AnyView>,
) -> impl IntoView {
    let on_start_drag = move |e: web_sys::MouseEvent| {
        if e.button() == 0 {
            invoke_tauri_window_command("window_start_drag");
        }
    };

    view! {
        <header
            class="h-9 w-full flex items-center justify-between px-3.5 select-none border-b border-black/10 dark:border-white/[0.08] bg-[#f6f6f6] dark:bg-[#1e1e24] text-neutral-800 dark:text-neutral-200 transition-colors shrink-0 relative"
        >
            // Traffic lights button group - strictly isolated from dragging
            <div class="flex items-center relative z-30 cursor-default">
                <TrafficLights
                    focused=focused
                    on_close=on_close
                    on_minimize=on_minimize
                    on_maximize=on_maximize
                    on_hold_close=on_hold_close
                    on_hold_minimize=on_hold_minimize
                    on_hold_maximize=on_hold_maximize
                    hold_delay_ms=hold_delay_ms
                />
            </div>

            // Draggable center region - ONLY this region handles dragging!
            <div
                data-tauri-drag-region
                on:mousedown=on_start_drag
                class="flex-1 h-full flex items-center justify-center cursor-grab active:cursor-grabbing px-4 relative z-10"
            >
                {title.map(|t| view! {
                    <div class="pointer-events-none text-[12px] font-medium tracking-tight opacity-75">
                        {t}
                    </div>
                })}
            </div>

            <div class="shrink-0 flex items-center justify-end min-w-14">
                {right_action}
            </div>
        </header>
    }
}
