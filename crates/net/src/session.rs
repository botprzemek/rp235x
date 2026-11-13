use core::marker::PhantomData;

use crate::data::{Handshake, Heartbeat};
use crate::event::{ClientEvent, ServerEvent};
use crate::game::{GameIdle, GameSession};
use crate::layout::DATA_SIZE;
use crate::packet::{ClientPacket, ServerPacket};
use crate::transport::{PacketReceiver, PacketSender};

pub struct Disconnected;
pub struct Connected;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SessionError {
    InvalidPacket(&'static str),
    UnexpectedEvent(ServerEvent),
}

pub struct Session<T, State> {
    transport: T,
    sequence_id: u16,
    session_token: u32,
    _state: PhantomData<State>,
}

impl<T, State> Session<T, State> {
    #[must_use]
    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    #[must_use]
    pub fn into_transport(self) -> T {
        self.transport
    }

    #[must_use]
    pub fn sequence_id(&self) -> u16 {
        self.sequence_id
    }

    pub(crate) fn next_sequence_id(&mut self) -> u16 {
        let id = self.sequence_id;
        self.sequence_id = self.sequence_id.wrapping_add(1);
        id
    }
}

impl<T> Session<T, Disconnected> {
    #[must_use]
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            sequence_id: 0,
            session_token: 0,
            _state: PhantomData,
        }
    }
}

impl<T: PacketSender + PacketReceiver> Session<T, Disconnected> {
    pub async fn handshake(
        mut self,
        handshake: &Handshake,
    ) -> Result<Session<T, Connected>, (Self, SessionError)> {
        let mut data = [0u8; DATA_SIZE];
        let bytes = handshake.to_bytes();
        let len = bytes.len().min(DATA_SIZE);
        data[..len].copy_from_slice(&bytes[..len]);

        let seq = self.next_sequence_id();
        let packet = ClientPacket::with_session(
            ClientEvent::HandshakeRequest,
            seq,
            0,
            len as u8,
            data,
        );
        self.transport.send_packet(&packet.to_bytes()).await;

        let raw = match self.transport.receive_packet().await {
            Ok(raw) => raw,
            Err(err) => return Err((self, SessionError::InvalidPacket(err))),
        };

        let response = match ServerPacket::from_bytes(raw.bytes) {
            Ok(pkt) => pkt,
            Err(err) => return Err((self, SessionError::InvalidPacket(err))),
        };

        if response.event() != ServerEvent::HandshakeAck {
            return Err((self, SessionError::UnexpectedEvent(response.event())));
        }

        let session_token = response.session_token();

        Ok(Session {
            transport: self.transport,
            sequence_id: self.sequence_id,
            session_token,
            _state: PhantomData,
        })
    }
}

impl<T> Session<T, Connected> {
    #[must_use]
    pub fn session_token(&self) -> u32 {
        self.session_token
    }

    #[must_use]
    pub fn into_game(self, game_id: u16) -> GameSession<T, GameIdle> {
        GameSession::new(self, game_id)
    }

    pub(crate) fn build_packet(
        &mut self,
        event: ClientEvent,
        payload_len: u8,
        data: [u8; DATA_SIZE],
    ) -> ClientPacket {
        let seq = self.next_sequence_id();
        ClientPacket::with_session(event, seq, self.session_token, payload_len, data)
    }
}

impl<T: PacketSender> Session<T, Connected> {
    pub(crate) async fn send_event(
        &mut self,
        event: ClientEvent,
        payload_len: u8,
        data: [u8; DATA_SIZE],
    ) {
        let packet = self.build_packet(event, payload_len, data);
        self.transport.send_packet(&packet.to_bytes()).await;
    }

    pub async fn heartbeat(&mut self, heartbeat: &Heartbeat) {
        let mut data = [0u8; DATA_SIZE];
        let bytes = heartbeat.to_bytes();
        let len = bytes.len().min(DATA_SIZE);
        data[..len].copy_from_slice(&bytes[..len]);
        self.send_event(ClientEvent::HandshakeHeartbeat, len as u8, data)
            .await;
    }

    pub async fn create_game(&mut self, discipline: u8) {
        let mut data = [0u8; DATA_SIZE];
        data[0] = discipline;
        self.send_event(ClientEvent::GameCreate, 1, data).await;
    }

    pub async fn delete_game(&mut self, game_id: u16) {
        let mut data = [0u8; DATA_SIZE];
        data[..2].copy_from_slice(&game_id.to_le_bytes());
        self.send_event(ClientEvent::GameDelete, 2, data).await;
    }

    pub async fn select_game(&mut self, game_id: u16) {
        let mut data = [0u8; DATA_SIZE];
        data[..2].copy_from_slice(&game_id.to_le_bytes());
        self.send_event(ClientEvent::GameSelect, 2, data).await;
    }

    pub async fn disconnect(mut self) -> Session<T, Disconnected> {
        self.send_event(ClientEvent::DeviceDisconnect, 0, [0u8; DATA_SIZE])
            .await;
        Session {
            transport: self.transport,
            sequence_id: self.sequence_id,
            session_token: 0,
            _state: PhantomData,
        }
    }
}
