use worker_hmi::App;

fn main() {
    let app = yew::Renderer::<App>::new();

    #[cfg(feature = "csr")]
    app.render();
    #[cfg(feature = "ssr")]
    app.hydrate();
}

