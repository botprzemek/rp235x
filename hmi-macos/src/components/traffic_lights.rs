//! macOS Traffic Light window control button components (`CloseButton`, `MinimizeButton`, `MaximizeButton`)
//! built from the exact SVGs in `lwouis/macos-traffic-light-buttons-as-SVG`.

use yew::{Callback, Html, MouseEvent, Properties, function_component, html, use_state};

/// Visual state of a macOS traffic light button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficLightState {
    /// Unfocused window (`0-all-three-nofocus.svg`).
    NoFocus,
    /// Focused window, cursor not hovering the button group (`*-1-normal.svg`).
    Normal,
    /// Cursor hovering the traffic light group (`*-2-hover.svg`).
    Hover,
    /// Mouse button pressed on this specific button (`*-3-press.svg`).
    Press,
}

/// Properties for an individual macOS traffic light button component.
#[derive(Properties, PartialEq, Clone)]
pub struct TrafficLightButtonProps {
    /// Whether the parent window currently has focus.
    pub focused: bool,
    /// Whether the traffic light button group is currently hovered.
    #[prop_or_default]
    pub hovered: bool,
    /// Click callback invoked when the button is clicked.
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
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

/// Red macOS Close traffic light button (`1-close-1-normal.svg`, `2-close-2-hover.svg`,
/// `2-close-3-press.svg`, `0-all-three-nofocus.svg`).
#[function_component(CloseButton)]
pub fn close_button(props: &TrafficLightButtonProps) -> Html {
    let is_pressed = use_state(|| false);
    let state = resolve_state(props.focused, props.hovered, *is_pressed);

    let onmousedown = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            if event.buttons() == 1 {
                is_pressed.set(true);
            }
        })
    };

    let onmouseup = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            is_pressed.set(false);
        })
    };

    let onmouseleave = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |_| {
            is_pressed.set(false);
        })
    };

    let onclick = {
        let onclick = props.onclick.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            onclick.emit(event);
        })
    };

    let svg = match state {
        TrafficLightState::NoFocus => html! {
            // 0-all-three-nofocus.svg
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#d1d0d2"/>
                    <path d="m42.7 81.7c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#c7c7c7"/>
                </g>
            </svg>
        },
        TrafficLightState::Normal => html! {
            // 1-close-1-normal.svg
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#e24b41"/>
                    <path d="m42.7 81.8c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#ed6a5f"/>
                </g>
            </svg>
        },
        TrafficLightState::Hover => html! {
            // 2-close-2-hover.svg (slightly broader stroke)
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#e24b41"/>
                    <path d="m42.7 81.8c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#ed6a5f"/>
                    <g fill="#9a0000" stroke="#9a0000" stroke-width="0.9" stroke-linejoin="round">
                        <path d="m22.5 57.8 35.3-35.3c1.4-1.4 3.6-1.4 5 0l.1.1c1.4 1.4 1.4 3.6 0 5l-35.3 35.3c-1.4 1.4-3.6 1.4-5 0l-.1-.1c-1.3-1.4-1.3-3.6 0-5z"/>
                        <path d="m27.6 22.5 35.3 35.3c1.4 1.4 1.4 3.6 0 5l-.1.1c-1.4 1.4-3.6 1.4-5 0l-35.3-35.3c-1.4-1.4-1.4-3.6 0-5l.1-.1c1.4-1.3 3.6-1.3 5 0z"/>
                    </g>
                </g>
            </svg>
        },
        TrafficLightState::Press => html! {
            // 2-close-3-press.svg (slightly broader stroke)
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#a14239"/>
                    <path d="m42.7 81.7c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#b15048"/>
                    <g fill="#170101" stroke="#170101" stroke-width="0.9" stroke-linejoin="round">
                        <path d="m22.5 57.8 35.3-35.3c1.4-1.4 3.6-1.4 5 0l.1.1c1.4 1.4 1.4 3.6 0 5l-35.3 35.3c-1.4 1.4-3.6 1.4-5 0l-.1-.1c-1.4-1.4-1.4-3.7 0-5z"/>
                        <path d="m27.5 22.5 35.3 35.3c1.4 1.4 1.4 3.6 0 5l-.1.1c-1.4 1.4-3.6 1.4-5 0l-35.3-35.3c-1.4-1.4-1.4-3.6 0-5l.1-.1c1.4-1.4 3.7-1.4 5 0z"/>
                    </g>
                </g>
            </svg>
        },
    };

    html! {
        <button
            type="button"
            title="Close"
            {onmousedown}
            {onmouseup}
            {onmouseleave}
            {onclick}
            class="w-3 h-3 p-0 m-0 border-0 bg-transparent flex items-center justify-center cursor-pointer"
        >
            {svg}
        </button>
    }
}

/// Yellow macOS Minimize traffic light button (`2-minimize-1-normal.svg`, `2-minimize-2-hover.svg`,
/// `2-minimize-3-press.svg`, `0-all-three-nofocus.svg`).
#[function_component(MinimizeButton)]
pub fn minimize_button(props: &TrafficLightButtonProps) -> Html {
    let is_pressed = use_state(|| false);
    let state = resolve_state(props.focused, props.hovered, *is_pressed);

    let onmousedown = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            if event.buttons() == 1 {
                is_pressed.set(true);
            }
        })
    };

    let onmouseup = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            is_pressed.set(false);
        })
    };

    let onmouseleave = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |_| {
            is_pressed.set(false);
        })
    };

    let onclick = {
        let onclick = props.onclick.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            onclick.emit(event);
        })
    };

    let svg = match state {
        TrafficLightState::NoFocus => html! {
            // 0-all-three-nofocus.svg
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#d1d0d2"/>
                    <path d="m42.7 81.7c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#c7c7c7"/>
                </g>
            </svg>
        },
        TrafficLightState::Normal => html! {
            // 2-minimize-1-normal.svg
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#e1a73e"/>
                    <path d="m42.7 81.8c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#f6be50"/>
                </g>
            </svg>
        },
        TrafficLightState::Hover => html! {
            // 2-minimize-2-hover.svg (minus bar height ~160%)
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#e1a73e"/>
                    <path d="m42.7 81.8c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#f6be50"/>
                    <path d="m17.8 37h49.9c1.9 0 3.5 2.5 3.5 5.6v.2c0 3.1-1.6 5.6-3.5 5.6h-49.9c-1.9 0-3.5-2.5-3.5-5.6v-.2c0-3.1 1.5-5.6 3.5-5.6z" fill="#90591d"/>
                </g>
            </svg>
        },
        TrafficLightState::Press => html! {
            // 2-minimize-3-press.svg (minus bar height ~160%)
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7c0 23.5 19 42.7 42.7 42.7z" fill="#a67f36"/>
                    <path d="m42.7 81.7c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1c-.1 21.6 17.4 39.1 39.1 39.1z" fill="#b8923b"/>
                    <path d="m17.7 37h49.9c1.9 0 3.5 2.5 3.5 5.6v.2c0 3.1-1.6 5.6-3.5 5.6h-49.9c-1.9 0-3.5-2.5-3.5-5.6v-.2c0-3.1 1.6-5.6 3.5-5.6z" fill="#532a0a"/>
                </g>
            </svg>
        },
    };

    html! {
        <button
            type="button"
            title="Minimize"
            {onmousedown}
            {onmouseup}
            {onmouseleave}
            {onclick}
            class="w-3 h-3 p-0 m-0 border-0 bg-transparent flex items-center justify-center cursor-pointer"
        >
            {svg}
        </button>
    }
}

/// Green macOS Zoom / Maximize traffic light button (`3-maximize-1-normal.svg`, `3-maximize-2-hover.svg`,
/// `3-maximize-3-press.svg`, `0-all-three-nofocus.svg`).
#[function_component(MaximizeButton)]
pub fn maximize_button(props: &TrafficLightButtonProps) -> Html {
    let is_pressed = use_state(|| false);
    let state = resolve_state(props.focused, props.hovered, *is_pressed);

    let onmousedown = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            if event.buttons() == 1 {
                is_pressed.set(true);
            }
        })
    };

    let onmouseup = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            is_pressed.set(false);
        })
    };

    let onmouseleave = {
        let is_pressed = is_pressed.clone();
        Callback::from(move |_| {
            is_pressed.set(false);
        })
    };

    let onclick = {
        let onclick = props.onclick.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            onclick.emit(event);
        })
    };

    let svg = match state {
        TrafficLightState::NoFocus => html! {
            // 0-all-three-nofocus.svg
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#d1d0d2"/>
                    <path d="m42.7 81.7c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#c7c7c7"/>
                </g>
            </svg>
        },
        TrafficLightState::Normal => html! {
            // 3-maximize-1-normal.svg
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#2dac2f"/>
                    <path d="m42.7 81.8c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1 17.5 39.1 39.1 39.1z" fill="#61c555"/>
                </g>
            </svg>
        },
        TrafficLightState::Hover => html! {
            // 3-maximize-2-hover.svg (flipped horizontally to top-left & bottom-right)
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#2dac2f"/>
                    <path d="m42.7 81.8c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1c0 21.5 17.5 39.1 39.1 39.1z" fill="#61c555"/>
                    <path d="m54.2 20.8h-26.7c-3.6 0-6.5 2.9-6.5 6.5v26.7zm-23.2 43.7h26.8c3.6 0 6.5-2.9 6.5-6.5v-26.8z" fill="#2a6218"/>
                </g>
            </svg>
        },
        TrafficLightState::Press => html! {
            // 3-maximize-3-press.svg (flipped horizontally to top-left & bottom-right)
            <svg class="w-3 h-3 block" viewBox="0 0 85.4 85.4" xmlns="http://www.w3.org/2000/svg">
                <g clip-rule="evenodd" fill-rule="evenodd">
                    <path d="m42.7 85.4c23.6 0 42.7-19.1 42.7-42.7s-19.1-42.7-42.7-42.7-42.7 19.1-42.7 42.7 19.1 42.7 42.7 42.7z" fill="#428234"/>
                    <path d="m42.7 81.8c21.6 0 39.1-17.5 39.1-39.1s-17.5-39.1-39.1-39.1-39.1 17.5-39.1 39.1c0 21.5 17.5 39.1 39.1 39.1z" fill="#4a9741"/>
                    <path d="m54.2 20.8h-26.7c-3.6 0-6.5 2.9-6.5 6.5v26.7zm-23.2 43.7h26.8c3.6 0 6.5-2.9 6.5-6.5v-26.8z" fill="#113107"/>
                </g>
            </svg>
        },
    };

    html! {
        <button
            type="button"
            title="Zoom"
            {onmousedown}
            {onmouseup}
            {onmouseleave}
            {onclick}
            class="w-3 h-3 p-0 m-0 border-0 bg-transparent flex items-center justify-center cursor-pointer"
        >
            {svg}
        </button>
    }
}
