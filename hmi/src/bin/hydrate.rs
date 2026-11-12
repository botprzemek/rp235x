use hmi::App;
use leptos::mount::mount_to_body;
use web_sys::console;

fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"Initializing HMI...".into());
    mount_to_body(App);
    console::log_1(&"HMI mounted to body".into());
}
