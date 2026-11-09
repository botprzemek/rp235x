//! Application builder, server startup, and graceful shutdown orchestration.

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::adapters::data::ReDBProvider;
use crate::adapters::net::{WebDriver, WorkerDriver};
use crate::adapters::repositories::Registry;
use crate::services::Services;

use tokio::net::{TcpListener, UdpSocket};
use tokio::signal;

/// Typestate marker indicating that a listener address has not been configured.
pub struct Absent;

/// Typestate marker indicating that a listener address has been configured.
pub struct Present;

/// Configured master server application ready to start HTTP and UDP listeners.
pub struct Application<HttpState, UdpState> {
    http_address: Option<String>,
    udp_address: Option<String>,
    database_path: Option<PathBuf>,
    _http: PhantomData<HttpState>,
    _udp: PhantomData<UdpState>,
}

/// Builder for configuring [`Application`] listener addresses.
pub struct ApplicationBuilder<HttpState = Absent, UdpState = Absent> {
    http_address: Option<String>,
    udp_address: Option<String>,
    database_path: Option<PathBuf>,
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
            database_path: self.database_path,
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
            database_path: self.database_path,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    /// Initializes the database, services, and configured network listeners.
    pub async fn start(self) -> Arc<Services> {
        let database = match &self.database_path {
            Some(path) => {
                println!("INFO: Using ReDB database at {}", path.display());
                ReDBProvider::open(path)
            }
            None => {
                let default_path = ReDBProvider::default_path();
                println!("INFO: Using default ReDB database at {}", default_path.display());
                ReDBProvider::open(&default_path)
            }
        };
        let registry = Registry::new(database);
        let services = Arc::new(Services::new(&registry));

        if let Some(http_address) = &self.http_address {
            let listener = TcpListener::bind(http_address)
                .await
                .expect("failed to bind HTTP TCP listener");
            WebDriver::new(listener).spawn(Arc::clone(&services));
        }

        if let Some(udp_address) = &self.udp_address {
            let addr: std::net::SocketAddr = udp_address
                .parse()
                .expect("failed to parse UDP listen address");

            let socket2_sock = socket2::Socket::new(
                if addr.is_ipv6() {
                    socket2::Domain::IPV6
                } else {
                    socket2::Domain::IPV4
                },
                socket2::Type::DGRAM,
                Some(socket2::Protocol::UDP),
            )
            .expect("failed to create UDP socket2");

            // Increase receive and send kernel buffers to 4MB to prevent drops under high load
            let _ = socket2_sock.set_recv_buffer_size(4 * 1024 * 1024);
            let _ = socket2_sock.set_send_buffer_size(4 * 1024 * 1024);
            let _ = socket2_sock.set_nonblocking(true);

            socket2_sock
                .bind(&addr.into())
                .expect("failed to bind UDP socket with socket2");

            let std_socket: std::net::UdpSocket = socket2_sock.into();
            let socket = UdpSocket::from_std(std_socket)
                .expect("failed to convert to tokio UdpSocket");

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
            database_path: None,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }
}

impl<HttpState, UdpState> ApplicationBuilder<HttpState, UdpState> {
    /// Configures the ReDB database file path.
    #[must_use]
    pub fn database_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.database_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Configures the HTTP server bind `address`.
    #[must_use]
    pub fn http_address(self, address: &str) -> ApplicationBuilder<Present, UdpState> {
        ApplicationBuilder {
            http_address: Some(address.to_string()),
            udp_address: self.udp_address,
            database_path: self.database_path,
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
            database_path: self.database_path,
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
            database_path: self.database_path,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }
}
