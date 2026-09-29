mod adapters;
mod services;

mod application;

use application::ApplicationBuilder;

#[tokio::main]
async fn main() {
    let builder = ApplicationBuilder::new();
    let application = builder
        .http_address("0.0.0.0:3000")
        .udp_address("0.0.0.0:9000")
        .build();

    application.with_http().await.with_udp().await.run().await;
}
