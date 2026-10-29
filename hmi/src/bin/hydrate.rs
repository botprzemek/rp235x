//! Browser entry point for rendering or hydrating the Yew [`App`].

use hmi::App;
use yew::Renderer;

fn main() {
    let app = Renderer::<App>::new();

    #[cfg(feature = "csr")]
    app.render();
    #[cfg(feature = "ssr")]
    app.hydrate();
}
