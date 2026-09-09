use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct NavbarProps {
    pub on_toggle_controls: Callback<MouseEvent>,
}

#[function_component]
pub fn Navbar(props: &NavbarProps) -> Html {
    html! {
        <aside style="background-color: #0f172a; padding: 10px 14px; display: flex; justify-content: space-between; align-items: center; box-sizing: border-box;">
            <p style="font-size: 11px; color: #f8fafc; margin: 0; font-family: monospace;">{"scoreboard@1.0.0"}</p>
            <button onclick={&props.on_toggle_controls} style="background: transparent; border: 1px solid #475569; color: #f8fafc; font-size: 11px; padding: 4px 8px; border-radius: 4px; cursor: pointer; -webkit-tap-highlight-color: transparent;">
                {"Przełącz podgląd / kontrolki"}
            </button>
        </aside>
    }
}
