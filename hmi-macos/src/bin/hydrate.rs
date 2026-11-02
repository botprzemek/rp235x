//! Browser entry point for rendering or hydrating the Yew [`App`].

use tauri_app_ui::App;
use web_sys::console;
use yew::Renderer;

fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"Initializing Lapapi WASM UI...".into());
    Renderer::<App>::new().render();
    console::log_1(&"Lapapi WASM UI rendered into body".into());
}

