//! Entry point for the `master` server binary.

use master::ApplicationBuilder;

#[tokio::main]
async fn main() {
    ApplicationBuilder::new()
        .http_address("0.0.0.0:3000")
        .udp_address("0.0.0.0:9000")
        .build()
        .with_http()
        .await
        .with_udp()
        .await
        .run()
        .await;
}
