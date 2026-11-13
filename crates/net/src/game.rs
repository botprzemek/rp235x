use core::marker::PhantomData;

use crate::event::ClientEvent;
use crate::layout::DATA_SIZE;
use crate::session::{Connected, Session};
use crate::transport::PacketSender;

pub struct GameIdle;
pub struct GameRunning;
pub struct GamePaused;
pub struct GameEnded;
pub struct GameSession<T, State> {
    session: Session<T, Connected>,
    game_id: u16,
    _state: PhantomData<State>,
}

impl<T, State> GameSession<T, State> {
    pub(crate) fn new(session: Session<T, Connected>, game_id: u16) -> Self {
        Self {
            session,
            game_id,
            _state: PhantomData,
        }
    }

    #[must_use]
    pub fn game_id(&self) -> u16 {
        self.game_id
    }

    #[must_use]
    pub fn session(&self) -> &Session<T, Connected> {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut Session<T, Connected> {
        &mut self.session
    }

    #[must_use]
    pub fn into_session(self) -> Session<T, Connected> {
        self.session
    }

    fn game_id_payload(&self) -> [u8; DATA_SIZE] {
        let mut data = [0u8; DATA_SIZE];
        data[..2].copy_from_slice(&self.game_id.to_le_bytes());
        data
    }

    fn transition<Next>(self) -> GameSession<T, Next> {
        GameSession {
            session: self.session,
            game_id: self.game_id,
            _state: PhantomData,
        }
    }
}

impl<T: PacketSender> GameSession<T, GameIdle> {
    pub async fn start(mut self) -> GameSession<T, GameRunning> {
        let payload = self.game_id_payload();
        self.session.send_event(ClientEvent::GameStart, 2, payload).await;
        self.transition()
    }
}

impl<T: PacketSender> GameSession<T, GameRunning> {
    pub async fn score_home_ft(&mut self) {
        let payload = self.game_id_payload();
        self.session
            .send_event(ClientEvent::GameScoreHomeFT, 2, payload)
            .await;
    }

    pub async fn score_home_2fg(&mut self) {
        let payload = self.game_id_payload();
        self.session
            .send_event(ClientEvent::GameScoreHome2FG, 2, payload)
            .await;
    }

    pub async fn score_home_3fg(&mut self) {
        let payload = self.game_id_payload();
        self.session
            .send_event(ClientEvent::GameScoreHome3FG, 2, payload)
            .await;
    }

    pub async fn score_away_ft(&mut self) {
        let payload = self.game_id_payload();
        self.session
            .send_event(ClientEvent::GameScoreAwayFT, 2, payload)
            .await;
    }

    pub async fn score_away_2fg(&mut self) {
        let payload = self.game_id_payload();
        self.session
            .send_event(ClientEvent::GameScoreAway2FG, 2, payload)
            .await;
    }

    pub async fn score_away_3fg(&mut self) {
        let payload = self.game_id_payload();
        self.session
            .send_event(ClientEvent::GameScoreAway3FG, 2, payload)
            .await;
    }

    pub async fn pause(mut self) -> GameSession<T, GamePaused> {
        let payload = self.game_id_payload();
        self.session.send_event(ClientEvent::GamePause, 2, payload).await;
        self.transition()
    }

    pub async fn end(mut self) -> GameSession<T, GameEnded> {
        let payload = self.game_id_payload();
        self.session.send_event(ClientEvent::GameEnd, 2, payload).await;
        self.transition()
    }
}

impl<T: PacketSender> GameSession<T, GamePaused> {
    pub async fn resume(mut self) -> GameSession<T, GameRunning> {
        let payload = self.game_id_payload();
        self.session.send_event(ClientEvent::GameResume, 2, payload).await;
        self.transition()
    }

    pub async fn end(mut self) -> GameSession<T, GameEnded> {
        let payload = self.game_id_payload();
        self.session.send_event(ClientEvent::GameEnd, 2, payload).await;
        self.transition()
    }
}

impl<T: PacketSender> GameSession<T, GameEnded> {
    pub async fn reset(mut self) -> GameSession<T, GameIdle> {
        let payload = self.game_id_payload();
        self.session.send_event(ClientEvent::GameReset, 2, payload).await;
        self.transition()
    }
}
