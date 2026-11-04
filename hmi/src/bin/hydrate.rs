//! Browser entry point for mounting the Leptos [`App`].

use hmi::App;
use leptos::mount::mount_to_body;
use web_sys::console;

fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"Initializing Lapapi WASM UI (Leptos)...".into());
    mount_to_body(App);
    console::log_1(&"Lapapi WASM UI mounted to body".into());
}
