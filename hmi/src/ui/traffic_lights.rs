//! macOS Traffic Light window control button components (`CloseButton`, `MinimizeButton`, `MaximizeButton`).

use leptos::prelude::*;

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

#[component]
pub fn CloseButton(
    focused: Signal<bool>,
    hovered: Signal<bool>,
    #[prop(into, default = Callback::new(|_| ()))] onclick: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    let pressed = RwSignal::new(false);
    let state = Signal::derive(move || resolve_state(focused.get(), hovered.get(), pressed.get()));

    view! {
        <button
            on:mousedown=move |_| pressed.set(true)
            on:mouseup=move |_| pressed.set(false)
            on:mouseleave=move |_| pressed.set(false)
            on:click=move |e| onclick.run(e)
            class="w-3 h-3 rounded-full flex items-center justify-center p-0 border-0 outline-none cursor-pointer bg-transparent"
        >
            {move || match state.get() {
                TrafficLightState::NoFocus => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#4a4d52" stroke="#3e4045" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Normal => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#ff5f56" stroke="#e0443e" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Hover => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#ff5f56" stroke="#e0443e" stroke-width="0.5"/>
                        <path d="M3.8 3.8 L8.2 8.2 M8.2 3.8 L3.8 8.2" stroke="#4d0000" stroke-width="1.2" stroke-linecap="round"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Press => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
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
) -> impl IntoView {
    let pressed = RwSignal::new(false);
    let state = Signal::derive(move || resolve_state(focused.get(), hovered.get(), pressed.get()));

    view! {
        <button
            on:mousedown=move |_| pressed.set(true)
            on:mouseup=move |_| pressed.set(false)
            on:mouseleave=move |_| pressed.set(false)
            on:click=move |e| onclick.run(e)
            class="w-3 h-3 rounded-full flex items-center justify-center p-0 border-0 outline-none cursor-pointer bg-transparent"
        >
            {move || match state.get() {
                TrafficLightState::NoFocus => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#4a4d52" stroke="#3e4045" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Normal => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#ffbd2e" stroke="#dea123" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Hover => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#ffbd2e" stroke="#dea123" stroke-width="0.5"/>
                        <path d="M2.5 6 L9.5 6" stroke="#5c3c00" stroke-width="1.2" stroke-linecap="round"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Press => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
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
) -> impl IntoView {
    let pressed = RwSignal::new(false);
    let state = Signal::derive(move || resolve_state(focused.get(), hovered.get(), pressed.get()));

    view! {
        <button
            on:mousedown=move |_| pressed.set(true)
            on:mouseup=move |_| pressed.set(false)
            on:mouseleave=move |_| pressed.set(false)
            on:click=move |e| onclick.run(e)
            class="w-3 h-3 rounded-full flex items-center justify-center p-0 border-0 outline-none cursor-pointer bg-transparent"
        >
            {move || match state.get() {
                TrafficLightState::NoFocus => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#4a4d52" stroke="#3e4045" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Normal => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#27c93f" stroke="#1aab29" stroke-width="0.5"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Hover => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#27c93f" stroke="#1aab29" stroke-width="0.5"/>
                        <path d="M3.5 3.5 L7.5 3.5 L7.5 7.5 Z M8.5 8.5 L4.5 8.5 L4.5 4.5 Z" fill="#003b0c"/>
                    </svg>
                }.into_any(),
                TrafficLightState::Press => view! {
                    <svg viewBox="0 0 12 12" class="w-3 h-3">
                        <circle cx="6" cy="6" r="5.5" fill="#1d972f" stroke="#13761c" stroke-width="0.5"/>
                        <path d="M3.5 3.5 L7.5 3.5 L7.5 7.5 Z M8.5 8.5 L4.5 8.5 L4.5 4.5 Z" fill="#001a05"/>
                    </svg>
                }.into_any(),
            }}
        </button>
    }
}

