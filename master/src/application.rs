//! Application builder, server startup, and graceful shutdown orchestration.

use std::marker::PhantomData;
use std::sync::Arc;

use tokio::net::{TcpListener, UdpSocket};
use tokio::signal;

use crate::adapters::data::ReDBProvider;
use crate::adapters::net::{WebDriver, WorkerDriver};
use crate::adapters::repositories::Registry;
use crate::services::Services;

/// Typestate marker indicating that a listener address has not been configured.
pub struct Absent;

/// Typestate marker indicating that a listener address has been configured.
pub struct Present;

/// Configured master server application ready to start HTTP and UDP listeners.
pub struct Application<HttpState, UdpState> {
    http_address: Option<String>,
    udp_address: Option<String>,
    _http: PhantomData<HttpState>,
    _udp: PhantomData<UdpState>,
}

/// Builder for configuring [`Application`] listener addresses.
pub struct ApplicationBuilder<HttpState = Absent, UdpState = Absent> {
    http_address: Option<String>,
    udp_address: Option<String>,
    _http: PhantomData<HttpState>,
    _udp: PhantomData<UdpState>,
}

impl Default for ApplicationBuilder<Absent, Absent> {
    fn default() -> Self {
        Self::new()
    }
}

impl<HttpState, UdpState> Application<HttpState, UdpState> {
    async fn wait_for_ctrl_c() {
        signal::ctrl_c()
            .await
            .expect("ERROR: Failed to install Ctrl+C handler");
    }

    async fn wait_for_terminate() {
        #[cfg(not(unix))]
        std::future::pending::<()>().await;

        #[cfg(unix)]
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("ERROR: Failed to install SIGNAL handler")
            .recv()
            .await;
    }

    async fn wait_for_shutdown() {
        tokio::select! {
            () = Self::wait_for_ctrl_c() => {
                println!("\nINFO: Gracefully exiting (CTRL+C)");
            }
            () = Self::wait_for_terminate() => {
                println!("\nINFO: Gracefully exiting (SIGTERM)");
            }
        }
    }

    /// Logs the HTTP listener configuration and marks the HTTP typestate as [`Present`].
    pub async fn with_http(self) -> Application<Present, UdpState> {
        match &self.http_address {
            Some(address) => println!("INFO: Listening on http://{address}"),
            None => println!("INFO: HTTP listener is not enabled... omitting"),
        }

        Application {
            http_address: self.http_address,
            udp_address: self.udp_address,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    /// Logs the UDP listener configuration and marks the UDP typestate as [`Present`].
    pub async fn with_udp(self) -> Application<HttpState, Present> {
        match &self.udp_address {
            Some(address) => println!("INFO: Listening on udp://{address}"),
            None => println!("INFO: UDP listener is not enabled... omitting"),
        }

        Application {
            http_address: self.http_address,
            udp_address: self.udp_address,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    /// Initializes the database, services, and configured network listeners.
    pub async fn start(self) -> Arc<Services> {
        let database = ReDBProvider::new();
        let registry = Registry::new(database);
        let services = Arc::new(Services::new(&registry));

        if let Some(http_address) = &self.http_address {
            let listener = TcpListener::bind(http_address)
                .await
                .expect("failed to bind HTTP TCP listener");
            WebDriver::new(listener).spawn(Arc::clone(&services));
        }

        if let Some(udp_address) = &self.udp_address {
            let socket = UdpSocket::bind(udp_address)
                .await
                .expect("failed to bind UDP socket");
            WorkerDriver::new(socket).spawn(Arc::clone(&services));
        }

        services
    }

    /// Starts the application and waits until a termination signal is received.
    pub async fn run(self) {
        let _services = self.start().await;
        Self::wait_for_shutdown().await;
    }
}

impl ApplicationBuilder<Absent, Absent> {
    /// Creates a new [`ApplicationBuilder`] with no listeners configured.
    #[must_use]
    pub fn new() -> Self {
        Self {
            http_address: None,
            udp_address: None,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }
}

impl<HttpState, UdpState> ApplicationBuilder<HttpState, UdpState> {
    /// Configures the HTTP server bind `address`.
    #[must_use]
    pub fn http_address(self, address: &str) -> ApplicationBuilder<Present, UdpState> {
        ApplicationBuilder {
            http_address: Some(address.to_string()),
            udp_address: self.udp_address,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    /// Configures the UDP worker bind `address`.
    #[must_use]
    pub fn udp_address(self, address: &str) -> ApplicationBuilder<HttpState, Present> {
        ApplicationBuilder {
            http_address: self.http_address,
            udp_address: Some(address.to_string()),
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    /// Builds the [`Application`].
    #[must_use]
    pub fn build(self) -> Application<HttpState, UdpState> {
        Application {
            http_address: self.http_address,
            udp_address: self.udp_address,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }
}
