use std::marker::PhantomData;
use std::sync::Arc;

use tokio::net::{TcpListener, UdpSocket};

use crate::adapters::data::ReDBProvider;
use crate::adapters::net::{WebDriver, WorkerDriver};
use crate::adapters::repositories::Registry;
use crate::services::Services;

pub struct Absent;
pub struct Present;

pub struct Application<HTTP, UDP> {
    http_address: Option<String>,
    udp_address: Option<String>,
    _http: PhantomData<HTTP>,
    _udp: PhantomData<UDP>,
}

pub struct ApplicationBuilder<HTTP = Absent, UDP = Absent> {
    http_address: Option<String>,
    udp_address: Option<String>,
    _http: PhantomData<HTTP>,
    _udp: PhantomData<UDP>,
}

trait GracefulShutdown {
    async fn ctrl_c() -> ();
    async fn terminate() -> ();
    async fn shutdown() -> ();
}

impl<HTTP, UDP> GracefulShutdown for Application<HTTP, UDP> {
    async fn ctrl_c() {
        tokio::signal::ctrl_c()
            .await
            .expect("ERROR: Failed to install Ctrl+C handler");
    }

    async fn terminate() {
        #[cfg(not(unix))]
        std::future::pending::<()>().await;

        #[cfg(unix)]
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("ERROR: Failed to install SIGNAL handler")
            .recv()
            .await;
    }

    async fn shutdown() -> () {
        tokio::select! {
            _ = Self::ctrl_c() => { println!("\nINFO: Gracefully exiting (CTRL+C)"); },
            _ = Self::terminate() => { println!("\nINFO: Gracefully exiting (SIGTERM)"); },
        }
    }
}

impl<HTTP, UDP> Application<HTTP, UDP> {
    pub async fn with_http(self) -> Application<Present, UDP> {
        match &self.http_address {
            Some(address) => println!("INFO: Listening on http://{}", address),
            None => println!("INFO: HTTP listener is not enabled... omitting"),
        };

        Application {
            http_address: self.http_address,
            udp_address: self.udp_address,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    pub async fn with_udp(self) -> Application<HTTP, Present> {
        match &self.udp_address {
            Some(address) => println!("INFO: Listening on udp://{}", address),
            None => println!("INFO: UDP listener is not enabled... omitting"),
        };

        Application {
            http_address: self.http_address,
            udp_address: self.udp_address,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    pub async fn run(self) {
        let database = ReDBProvider::new();
        let registry = Registry::new(database);
        let services = Arc::new(Services::new(registry));

        if let Some(http_address) = &self.http_address {
            let listener = TcpListener::bind(http_address).await.unwrap();
            WebDriver::new(listener).spawn(Arc::clone(&services));
        }

        if let Some(udp_address) = &self.udp_address {
            let socket = UdpSocket::bind(udp_address).await.unwrap();
            WorkerDriver::new(socket).spawn(Arc::clone(&services));
        }

        Self::shutdown().await;
    }
}

impl ApplicationBuilder<Absent, Absent> {
    pub fn new() -> Self {
        Self {
            http_address: None,
            udp_address: None,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }
}

impl<HTTP, UDP> ApplicationBuilder<HTTP, UDP> {
    pub fn http_address(self, address: &str) -> ApplicationBuilder<Present, UDP> {
        ApplicationBuilder {
            http_address: Some(address.to_string()),
            udp_address: self.udp_address,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    pub fn udp_address(self, address: &str) -> ApplicationBuilder<HTTP, Present> {
        ApplicationBuilder {
            http_address: self.http_address,
            udp_address: Some(address.to_string()),
            _http: PhantomData,
            _udp: PhantomData,
        }
    }

    pub fn build(self) -> Application<HTTP, UDP> {
        Application {
            http_address: self.http_address,
            udp_address: self.udp_address,
            _http: PhantomData,
            _udp: PhantomData,
        }
    }
}
