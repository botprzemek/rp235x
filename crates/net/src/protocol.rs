use crate::data::{Device, HandshakeAck, Heartbeat, HeartbeatAck};
use crate::layout::DATA_SIZE;
use core::future::Future;

pub trait Role: Send + Sync + 'static {}

pub struct HmiRole;
impl Role for HmiRole {}

pub struct WorkerRole;
impl Role for WorkerRole {}

pub struct MasterRole;
impl Role for MasterRole {}

pub trait SendHandshake {
    type Error;
    fn send_handshake(
        &mut self,
        device: &Device,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveHandshake {
    type Error;
    fn on_handshake(
        &mut self,
        device: &Device,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendHandshakeAck {
    type Error;
    fn send_handshake_ack(
        &mut self,
        ack: &HandshakeAck,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveHandshakeAck {
    type Error;
    fn on_handshake_ack(
        &mut self,
        ack: &HandshakeAck,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendHeartbeat {
    type Error;
    fn send_heartbeat(
        &mut self,
        heartbeat: &Heartbeat,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveHeartbeat {
    type Error;
    fn on_heartbeat(
        &mut self,
        heartbeat: &Heartbeat,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendHeartbeatAck {
    type Error;
    fn send_heartbeat_ack(
        &mut self,
        ack: &HeartbeatAck,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveHeartbeatAck {
    type Error;
    fn on_heartbeat_ack(
        &mut self,
        ack: &HeartbeatAck,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameCreate {
    type Error;
    fn send_game_create(
        &mut self,
        discipline: u8,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameCreate {
    type Error;
    fn on_game_create(
        &mut self,
        discipline: u8,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameDelete {
    type Error;
    fn send_game_delete(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameDelete {
    type Error;
    fn on_game_delete(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameStart {
    type Error;
    fn send_game_start(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameStart {
    type Error;
    fn on_game_start(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameResume {
    type Error;
    fn send_game_resume(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameResume {
    type Error;
    fn on_game_resume(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGamePause {
    type Error;
    fn send_game_pause(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGamePause {
    type Error;
    fn on_game_pause(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameEnd {
    type Error;
    fn send_game_end(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameEnd {
    type Error;
    fn on_game_end(&mut self, game_id: u16)
    -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameReset {
    type Error;
    fn send_game_reset(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameReset {
    type Error;
    fn on_game_reset(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameSelect {
    type Error;
    fn send_game_select(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameSelect {
    type Error;
    fn on_game_select(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameScoreHomeFT {
    type Error;
    fn send_score_home_ft(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameScoreHomeFT {
    type Error;
    fn on_score_home_ft(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameScoreHome2FG {
    type Error;
    fn send_score_home_2fg(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameScoreHome2FG {
    type Error;
    fn on_score_home_2fg(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameScoreHome3FG {
    type Error;
    fn send_score_home_3fg(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameScoreHome3FG {
    type Error;
    fn on_score_home_3fg(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameScoreAwayFT {
    type Error;
    fn send_score_away_ft(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameScoreAwayFT {
    type Error;
    fn on_score_away_ft(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameScoreAway2FG {
    type Error;
    fn send_score_away_2fg(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameScoreAway2FG {
    type Error;
    fn on_score_away_2fg(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameScoreAway3FG {
    type Error;
    fn send_score_away_3fg(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameScoreAway3FG {
    type Error;
    fn on_score_away_3fg(
        &mut self,
        game_id: u16,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendDeviceDisconnect {
    type Error;
    fn send_device_disconnect(
        &mut self,
        target_device: &Device,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveDeviceDisconnect {
    type Error;
    fn on_device_disconnect(
        &mut self,
        target_device: &Device,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendDeviceAssignGame {
    type Error;
    fn send_device_assign_game(
        &mut self,
        target_device: &Device,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveDeviceAssignGame {
    type Error;
    fn on_device_assign_game(
        &mut self,
        target_device: &Device,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameUpdate {
    type Error;
    fn send_game_update(
        &mut self,
        game_payload: &[u8; DATA_SIZE],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameUpdate {
    type Error;
    fn on_game_update(
        &mut self,
        game_payload: &[u8; DATA_SIZE],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendGameListUpdate {
    type Error;
    fn send_game_list_update(
        &mut self,
        data: &[u8; DATA_SIZE],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveGameListUpdate {
    type Error;
    fn on_game_list_update(
        &mut self,
        data: &[u8; DATA_SIZE],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait SendDeviceUpdate {
    type Error;
    fn send_device_update(
        &mut self,
        device: &Device,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ReceiveDeviceUpdate {
    type Error;
    fn on_device_update(
        &mut self,
        device: &Device,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ClientReaderProtocol:
    SendHandshake + ReceiveHandshakeAck + SendHeartbeat + ReceiveHeartbeatAck + ReceiveGameUpdate
{
}

impl<T> ClientReaderProtocol for T where
    T: SendHandshake
        + ReceiveHandshakeAck
        + SendHeartbeat
        + ReceiveHeartbeatAck
        + ReceiveGameUpdate
{
}

pub trait ClientWriterProtocol:
    SendHandshake
    + ReceiveHandshakeAck
    + SendHeartbeat
    + ReceiveHeartbeatAck
    + SendGameStart
    + SendGamePause
    + SendGameResume
    + SendGameEnd
    + SendGameReset
    + SendGameSelect
    + SendGameCreate
    + SendGameDelete
    + SendGameScoreHomeFT
    + SendGameScoreHome2FG
    + SendGameScoreHome3FG
    + SendGameScoreAwayFT
    + SendGameScoreAway2FG
    + SendGameScoreAway3FG
    + SendDeviceAssignGame
    + SendDeviceDisconnect
    + ReceiveGameUpdate
    + ReceiveGameListUpdate
    + ReceiveDeviceUpdate
{
}

impl<T> ClientWriterProtocol for T where
    T: SendHandshake
        + ReceiveHandshakeAck
        + SendHeartbeat
        + ReceiveHeartbeatAck
        + SendGameStart
        + SendGamePause
        + SendGameResume
        + SendGameEnd
        + SendGameReset
        + SendGameSelect
        + SendGameCreate
        + SendGameDelete
        + SendGameScoreHomeFT
        + SendGameScoreHome2FG
        + SendGameScoreHome3FG
        + SendGameScoreAwayFT
        + SendGameScoreAway2FG
        + SendGameScoreAway3FG
        + SendDeviceAssignGame
        + SendDeviceDisconnect
        + ReceiveGameUpdate
        + ReceiveGameListUpdate
        + ReceiveDeviceUpdate
{
}

pub trait ServerReaderProtocol:
    ReceiveHandshake
    + SendHandshakeAck
    + ReceiveHeartbeat
    + SendHeartbeatAck
    + ReceiveGameStart
    + ReceiveGamePause
    + ReceiveGameResume
    + ReceiveGameEnd
    + ReceiveGameReset
    + ReceiveGameSelect
    + ReceiveGameCreate
    + ReceiveGameDelete
    + ReceiveGameScoreHomeFT
    + ReceiveGameScoreHome2FG
    + ReceiveGameScoreHome3FG
    + ReceiveGameScoreAwayFT
    + ReceiveGameScoreAway2FG
    + ReceiveGameScoreAway3FG
    + ReceiveDeviceAssignGame
    + ReceiveDeviceDisconnect
    + SendGameUpdate
    + SendGameListUpdate
    + SendDeviceUpdate
{
}

impl<T> ServerReaderProtocol for T where
    T: ReceiveHandshake
        + SendHandshakeAck
        + ReceiveHeartbeat
        + SendHeartbeatAck
        + ReceiveGameStart
        + ReceiveGamePause
        + ReceiveGameResume
        + ReceiveGameEnd
        + ReceiveGameReset
        + ReceiveGameSelect
        + ReceiveGameCreate
        + ReceiveGameDelete
        + ReceiveGameScoreHomeFT
        + ReceiveGameScoreHome2FG
        + ReceiveGameScoreHome3FG
        + ReceiveGameScoreAwayFT
        + ReceiveGameScoreAway2FG
        + ReceiveGameScoreAway3FG
        + ReceiveDeviceAssignGame
        + ReceiveDeviceDisconnect
        + SendGameUpdate
        + SendGameListUpdate
        + SendDeviceUpdate
{
}
