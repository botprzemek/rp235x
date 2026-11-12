use leptos::prelude::*;

#[component]
pub fn MatrixView() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center p-4">
            <span class="text-xs text-zinc-500 font-mono">{"HUB75 Simulator"}</span>
        </div>
    }
}

pub use MatrixView as MatrixSimulator;
