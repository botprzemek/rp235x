//! WebSocket client adapter and types for the Lapapi desktop application.

use core::net::SocketAddr;

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use net::layout::PACKET_SIZE;
use net::{ClientHandler, ClientPacket, Device, DeviceType, RawConnection, ServerPacket};
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys;

#[cfg(target_arch = "wasm32")]
const SERIAL_NUMBER_STORAGE_KEY: &str = "lapapi_serial_number";

pub fn load_or_create_browser_device() -> Device {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = web_sys::window().and_then(|window| window.local_storage().ok().flatten());

        if let Some(ref store) = storage
            && let Ok(Some(saved_serial_number)) = store.get_item(SERIAL_NUMBER_STORAGE_KEY)
            && let Ok(device) =
                Device::try_new(DeviceType::WebBrowser, &saved_serial_number, "lapapi-app")
        {
            return device;
        }

        let suffix = (js_sys::Math::random() * 10000.0) as u16 % 10000;
        let serial_number = format!("sn-lap-{suffix:04}");

        if let Some(store) = storage {
            let _ = store.set_item(SERIAL_NUMBER_STORAGE_KEY, &serial_number);
        }

        Device::try_new(DeviceType::WebBrowser, &serial_number, "lapapi-app")
            .expect("valid browser DeviceConfig")
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Device::try_new(DeviceType::WebBrowser, "sn-lap-0000", "lapapi-app")
            .expect("valid browser DeviceConfig")
    }
}

/// Adapter bridging browser [`WebSocket`] channels to [`RawConnection`].
pub struct BrowserWebSocketAdapter {
    outgoing_tx: UnboundedSender<[u8; PACKET_SIZE]>,
    incoming_rx: UnboundedReceiver<[u8; PACKET_SIZE]>,
}

impl BrowserWebSocketAdapter {
    #[must_use]
    pub fn new(
        outgoing_tx: UnboundedSender<[u8; PACKET_SIZE]>,
        incoming_rx: UnboundedReceiver<[u8; PACKET_SIZE]>,
    ) -> Self {
        Self {
            outgoing_tx,
            incoming_rx,
        }
    }
}

impl RawConnection for BrowserWebSocketAdapter {
    async fn send(&mut self, data: &[u8; PACKET_SIZE]) -> Result<(), ()> {
        self.outgoing_tx.unbounded_send(*data).map_err(|_| ())
    }

    async fn receive(&mut self, buffer: &mut [u8; PACKET_SIZE]) -> Result<usize, ()> {
        match self.incoming_rx.next().await {
            Some(packet) => {
                *buffer = packet;
                Ok(PACKET_SIZE)
            }
            None => Err(()),
        }
    }
}

/// [`ClientHandler`] implementation that forwards received [`ServerPacket`]s to the UI task.
pub struct LapapiClientHandler {
    packet_tx: UnboundedSender<ServerPacket>,
}

impl LapapiClientHandler {
    #[must_use]
    pub fn new(packet_tx: UnboundedSender<ServerPacket>) -> Self {
        Self { packet_tx }
    }
}

impl ClientHandler for LapapiClientHandler {
    async fn handle_packet(
        &mut self,
        packet: ServerPacket,
        _address: Option<SocketAddr>,
    ) -> Option<ClientPacket> {
        let _ = self.packet_tx.unbounded_send(packet);
        None
    }
}
