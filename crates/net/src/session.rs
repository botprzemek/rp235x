//! Type-state network session abstractions for client and server nodes.
//!
//! Enforces protocol state transitions at compile time:
//! - Disconnected -> Connecting -> Established -> Disconnected
//! - Strict handling of Handshake, Heartbeat, and Game updates.

use crate::data::{Device, HandshakeAck, Heartbeat, HeartbeatAck};
use crate::event::{ClientEvent, ServerEvent};
use crate::layout::DATA_SIZE;
use crate::node::NetError;
use crate::packet::{ClientPacket, ServerPacket};

/// Marker types representing client session states.
pub mod client_state {
    /// Client is disconnected and has no active session.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Disconnected;

    /// Client has transmitted a `HandshakeRequest` and awaits `HandshakeAck`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Connecting {
        pub sequence_id: u16,
    }

    /// Client has received `HandshakeAck`, holds an active `session_token`,
    /// and is permitted to send heartbeats and receive game updates.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Established {
        pub session_token: u32,
        pub assigned_game_id: u16,
        pub sequence_id: u16,
    }
}

/// Generic client-side session tracker in state `S`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientSession<S> {
    state: S,
}

impl Default for ClientSession<client_state::Disconnected> {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientSession<client_state::Disconnected> {
    /// Creates a new client session in [`client_state::Disconnected`] state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: client_state::Disconnected,
        }
    }

    /// Initiates a handshake by producing a `HandshakeRequest` packet and transitioning to `Connecting`.
    pub fn start_handshake(
        self,
        device: &Device,
        sequence_id: u16,
    ) -> (ClientSession<client_state::Connecting>, ClientPacket) {
        let packet = ClientPacket::new(ClientEvent::HandshakeRequest, sequence_id, device.to_bytes());
        (
            ClientSession {
                state: client_state::Connecting { sequence_id },
            },
            packet,
        )
    }
}

impl ClientSession<client_state::Connecting> {
    /// Validates an incoming [`ServerPacket`] expecting `HandshakeAck`.
    /// On success, transitions into [`client_state::Established`].
    pub fn receive_handshake_ack(
        self,
        packet: &ServerPacket,
    ) -> Result<ClientSession<client_state::Established>, NetError> {
        if packet.event() != ServerEvent::HandshakeAck {
            return Err(NetError::InvalidPacket("expected HandshakeAck"));
        }

        let session_token = packet.session_token();
        if session_token == 0 {
            return Err(NetError::InvalidPacket("received null session token"));
        }

        let ack = HandshakeAck::from_bytes(&packet.data())
            .map_err(|_| NetError::InvalidPacket("invalid HandshakeAck payload"))?;

        Ok(ClientSession {
            state: client_state::Established {
                session_token,
                assigned_game_id: ack.assigned_game_id(),
                sequence_id: self.state.sequence_id,
            },
        })
    }

    /// Aborts or times out connection attempt, returning to [`client_state::Disconnected`].
    pub fn abort(self) -> ClientSession<client_state::Disconnected> {
        ClientSession::new()
    }
}

impl ClientSession<client_state::Established> {
    /// Returns the active 32-bit session token.
    #[must_use]
    pub const fn session_token(&self) -> u32 {
        self.state.session_token
    }

    /// Returns the game ID assigned to this worker node.
    #[must_use]
    pub const fn assigned_game_id(&self) -> u16 {
        self.state.assigned_game_id
    }

    /// Produces a new lightweight `HandshakeHeartbeat` packet with current session token.
    pub fn send_heartbeat(&mut self) -> ClientPacket {
        self.state.sequence_id = self.state.sequence_id.wrapping_add(1);
        let heartbeat = Heartbeat::new(0);
        ClientPacket::with_session(
            ClientEvent::HandshakeHeartbeat,
            self.state.sequence_id,
            self.state.session_token,
            Heartbeat::PAYLOAD_LEN,
            heartbeat.to_bytes(),
        )
    }

    /// Validates an incoming `HeartbeatAck` packet against the current session token.
    pub fn receive_heartbeat_ack(
        &mut self,
        packet: &ServerPacket,
    ) -> Result<HeartbeatAck, NetError> {
        if packet.event() != ServerEvent::HeartbeatAck {
            return Err(NetError::InvalidPacket("expected HeartbeatAck"));
        }
        if packet.session_token() != self.state.session_token {
            return Err(NetError::InvalidPacket("session token mismatch"));
        }

        HeartbeatAck::from_bytes(&packet.data())
            .map_err(|_| NetError::InvalidPacket("invalid HeartbeatAck payload"))
    }

    /// Validates an incoming `GameUpdate` packet against the current session token.
    pub fn receive_game_update(
        &mut self,
        packet: &ServerPacket,
    ) -> Result<[u8; DATA_SIZE], NetError> {
        if packet.event() != ServerEvent::GameUpdate {
            return Err(NetError::InvalidPacket("expected GameUpdate"));
        }
        if packet.session_token() != self.state.session_token {
            return Err(NetError::InvalidPacket("session token mismatch"));
        }

        Ok(packet.data())
    }

    /// Explicitly disconnects or reacts to a session timeout (e.g. 9s without Ack),
    /// dropping session token and returning to [`client_state::Disconnected`].
    pub fn disconnect(self) -> ClientSession<client_state::Disconnected> {
        ClientSession::new()
    }
}

/// Marker types representing server worker-registration states.
pub mod server_state {
    /// Worker is not registered or session has timed out.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Unregistered;

    /// Worker is registered and authenticated by `session_token`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Registered {
        pub session_token: u32,
        pub assigned_game_id: u16,
        pub sequence_id: u16,
    }
}

/// Generic server-side session tracker for a single remote worker node in state `S`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerSession<S> {
    state: S,
}

impl Default for ServerSession<server_state::Unregistered> {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerSession<server_state::Unregistered> {
    /// Creates a new unregistered server session tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: server_state::Unregistered,
        }
    }

    /// Validates an inbound `HandshakeRequest` packet.
    /// On success, creates a session token and transitions into [`server_state::Registered`].
    pub fn accept_handshake(
        self,
        packet: &ClientPacket,
        session_token: u32,
        assigned_game_id: u16,
    ) -> Result<(ServerSession<server_state::Registered>, ServerPacket, Device), NetError> {
        if packet.event() != ClientEvent::HandshakeRequest {
            return Err(NetError::InvalidPacket("expected HandshakeRequest"));
        }

        let device = Device::try_from(*packet)
            .map_err(|_| NetError::InvalidPacket("invalid Device handshake payload"))?;

        let seq = packet.sequence_id().wrapping_add(1);
        let ack_payload = HandshakeAck::new(assigned_game_id, 0);

        let ack_packet = ServerPacket::with_session(
            ServerEvent::HandshakeAck,
            seq,
            session_token,
            HandshakeAck::PAYLOAD_LEN,
            ack_payload.to_bytes(),
        );

        let next_session = ServerSession {
            state: server_state::Registered {
                session_token,
                assigned_game_id,
                sequence_id: seq,
            },
        };

        Ok((next_session, ack_packet, device))
    }
}

impl ServerSession<server_state::Registered> {
    /// Returns the session token for this worker.
    #[must_use]
    pub const fn session_token(&self) -> u32 {
        self.state.session_token
    }

    /// Validates an inbound heartbeat packet from the worker.
    pub fn receive_heartbeat(&mut self, packet: &ClientPacket) -> Result<Heartbeat, NetError> {
        if packet.event() != ClientEvent::HandshakeHeartbeat {
            return Err(NetError::InvalidPacket("expected HandshakeHeartbeat"));
        }
        if packet.session_token() != self.state.session_token {
            return Err(NetError::InvalidPacket("session token mismatch"));
        }

        Heartbeat::from_bytes(&packet.data())
            .map_err(|_| NetError::InvalidPacket("invalid Heartbeat payload"))
    }

    /// Generates a `HeartbeatAck` packet in response to a heartbeat.
    pub fn send_heartbeat_ack(&mut self, latest_game_seq: u16) -> ServerPacket {
        self.state.sequence_id = self.state.sequence_id.wrapping_add(1);
        let ack_payload = HeartbeatAck::new(latest_game_seq, 0);
        ServerPacket::with_session(
            ServerEvent::HeartbeatAck,
            self.state.sequence_id,
            self.state.session_token,
            HeartbeatAck::PAYLOAD_LEN,
            ack_payload.to_bytes(),
        )
    }

    /// Generates a `GameUpdate` packet containing game payload data.
    pub fn send_game_update(&mut self, game_bytes: [u8; DATA_SIZE]) -> ServerPacket {
        self.state.sequence_id = self.state.sequence_id.wrapping_add(1);
        ServerPacket::with_session(
            ServerEvent::GameUpdate,
            self.state.sequence_id,
            self.state.session_token,
            DATA_SIZE as u8,
            game_bytes,
        )
    }

    /// Terminates or marks the worker session as offline on timeout.
    pub fn expire(self) -> ServerSession<server_state::Unregistered> {
        ServerSession::new()
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod session_tests;
