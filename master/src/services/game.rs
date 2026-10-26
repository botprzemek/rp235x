use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use basketball::{Discipline, Game, Scorable, Score, State, Team};
use net::{ServerEvent, ServerPacket, layout::PACKET_SIZE};
use tokio::sync::{broadcast, watch};

use crate::{
    adapters::repositories::{GameRepository, Repository},
    services::Observable,
};

const TICK_RATE: Duration = Duration::from_nanos(1_000_000_000 / 60);
/// How often (in ticks) frame channels without subscribers are pruned.
const FRAME_PRUNE_TICKS: u32 = 600;

/// A fully serialized `ServerEvent::GameUpdate` packet, shared by every
/// connection observing the same game.
pub type GameFrame = Arc<[u8; PACKET_SIZE]>;

type FrameChannels = Arc<RwLock<HashMap<u16, watch::Sender<Option<GameFrame>>>>>;

#[derive(Clone, Copy)]
struct ActiveGame {
    game: Game,
    last_tick: Option<Instant>,
}

impl ActiveGame {
    fn new(game: Game, now: Instant) -> Self {
        let last_tick = if game.get_state() == State::Running {
            Some(now)
        } else {
            None
        };
        Self { game, last_tick }
    }

    /// Aktualizuje czas gry względem monotonicznego zegara `Instant`.
    /// Przesuwa `last_tick` wyłącznie o pełne przepracowane milisekundy,
    /// zachowując ułamki milisekund (< 1 ms) do kolejnego cyklu (zerowy dryf).
    fn advance_clock(&mut self, now: Instant) -> (bool, bool) {
        if self.game.get_state() != State::Running {
            self.last_tick = None;
            return (false, false);
        }

        let last = self.last_tick.get_or_insert(now);
        let elapsed_ms = now.duration_since(*last).as_millis() as u64;
        if elapsed_ms == 0 {
            return (false, false);
        }

        let prev_state = self.game.get_state();
        let prev_quarter = self.game.get_quarter();

        *last += Duration::from_millis(elapsed_ms);
        self.game.tick(Duration::from_millis(elapsed_ms));

        if self.game.get_state() != State::Running {
            self.last_tick = None;
        }

        let state_changed =
            prev_state != self.game.get_state() || prev_quarter != self.game.get_quarter();
        (true, state_changed)
    }
}

#[derive(Clone)]
pub struct GameService {
    #[allow(dead_code)]
    game_repository: GameRepository,
    state: watch::Sender<Vec<Game>>,
    cache: Arc<RwLock<Vec<Option<ActiveGame>>>>,
    db_dirty: Arc<AtomicBool>,
    broadcast: broadcast::Sender<Vec<Game>>,
    frames: FrameChannels,
}

impl Observable<Vec<Game>> for GameService {
    fn subscribe(&self) -> broadcast::Receiver<Vec<Game>> {
        self.broadcast.subscribe()
    }

    fn watch(&self) -> watch::Receiver<Vec<Game>> {
        self.state.subscribe()
    }
}

impl GameService {
    pub fn new(game_repository: GameRepository) -> Self {
        let mut initial_games = game_repository.select();
        if initial_games.is_empty() {
            let default_game = Game::with_id(1, Discipline::Fiba5vs5);
            initial_games.push(default_game);
            game_repository.upsert(initial_games.clone());
        }

        let now = Instant::now();
        let mut cache_vec: Vec<Option<ActiveGame>> = vec![None; 64];
        for g in &initial_games {
            if (g.id() as usize) < cache_vec.len() {
                cache_vec[g.id() as usize] = Some(ActiveGame::new(*g, now));
            }
        }
        let cache = Arc::new(RwLock::new(cache_vec));
        let db_dirty = Arc::new(AtomicBool::new(false));
        let frames: FrameChannels = Arc::new(RwLock::new(HashMap::new()));

        let (state, _state_rx) = watch::channel(initial_games);
        let (broadcast, _broadcast_rx) = broadcast::channel(64);

        // Asynchroniczny zapis do ReDB w tle (bez blokowania pętli czasu rzeczywistego 60 Hz)
        let repo_writer = game_repository.clone();
        let db_cache = Arc::clone(&cache);
        let db_flag = Arc::clone(&db_dirty);
        tokio::spawn(async move {
            let mut db_interval = tokio::time::interval(Duration::from_millis(250));
            db_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                db_interval.tick().await;
                if !db_flag.swap(false, Ordering::AcqRel) {
                    continue;
                }

                let snapshot: Vec<Game> = {
                    let games = db_cache.read().unwrap();
                    games.iter().flatten().map(|entry| entry.game).collect()
                };

                let repo = repo_writer.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    repo.upsert(snapshot);
                })
                .await;
            }
        });

        // Niezależny zegar czasu rzeczywistego (Instant::now()) 60 Hz bez utraty sub-milisekund
        let clock_cache = Arc::clone(&cache);
        let clock_state = state.clone();
        let clock_broadcast = broadcast.clone();
        let clock_db_dirty = Arc::clone(&db_dirty);
        let clock_frames = Arc::clone(&frames);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(TICK_RATE);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut sequence_id: u8 = 0;
            let mut tick_counter: u32 = 0;

            loop {
                interval.tick().await;
                let now = Instant::now();
                let mut any_ticked = false;
                let mut any_state_changed = false;
                let mut snapshot = Vec::new();

                {
                    let mut games = clock_cache.write().unwrap();
                    for entry in games.iter_mut().flatten() {
                        let (ticked, state_changed) = entry.advance_clock(now);
                        any_ticked |= ticked;
                        any_state_changed |= state_changed;
                    }
                    if any_state_changed {
                        snapshot = games.iter().flatten().map(|entry| entry.game).collect();
                    }
                }

                if any_ticked {
                    clock_db_dirty.store(true, Ordering::Release);
                }

                if any_state_changed {
                    let _ = clock_state.send(snapshot.clone());
                    if clock_broadcast.receiver_count() > 0 {
                        let _ = clock_broadcast.send(snapshot);
                    }
                }

                Self::publish_frames(&clock_cache, &clock_frames, sequence_id);
                sequence_id = sequence_id.wrapping_add(1);

                tick_counter = tick_counter.wrapping_add(1);
                if tick_counter % FRAME_PRUNE_TICKS == 0 {
                    clock_frames
                        .write()
                        .unwrap()
                        .retain(|_, tx| tx.receiver_count() > 0);
                }
            }
        });

        Self {
            game_repository,
            state,
            cache,
            db_dirty,
            broadcast,
            frames: Arc::clone(&frames),
        }
    }

    /// Serializes every observed game once and publishes the shared frame to
    /// all of its subscribers.
    fn publish_frames(
        cache: &RwLock<Vec<Option<ActiveGame>>>,
        frames: &RwLock<HashMap<u16, watch::Sender<Option<GameFrame>>>>,
        sequence_id: u8,
    ) {
        let frames = frames.read().unwrap();
        if frames.is_empty() {
            return;
        }

        let cache = cache.read().unwrap();
        for (&game_id, tx) in frames.iter() {
            if tx.receiver_count() == 0 {
                continue;
            }

            if let Some(entry) = cache.get(game_id as usize).and_then(|entry| entry.as_ref()) {
                let packet =
                    ServerPacket::new(ServerEvent::GameUpdate, sequence_id, entry.game.to_bytes());
                tx.send_replace(Some(Arc::new(packet.to_bytes())));
            }
        }
    }

    /// Subscribes to the 60 Hz stream of pre-serialized `GameUpdate` frames
    /// for a single game. Frames are latest-value only: slow consumers skip
    /// stale frames instead of queueing them.
    pub fn subscribe_frames(&self, game_id: u16) -> watch::Receiver<Option<GameFrame>> {
        if let Some(tx) = self.frames.read().unwrap().get(&game_id) {
            return tx.subscribe();
        }

        self.frames
            .write()
            .unwrap()
            .entry(game_id)
            .or_insert_with(|| watch::channel(None).0)
            .subscribe()
    }

    pub fn get_all(&self) -> Vec<Game> {
        let cache = self.cache.read().unwrap();
        cache.iter().flatten().map(|entry| entry.game).collect()
    }

    pub fn get_by_id(&self, id: u16) -> Option<Game> {
        let cache = self.cache.read().unwrap();
        cache
            .get(id as usize)
            .and_then(|entry| entry.as_ref().map(|e| e.game))
    }

    pub fn get(&self) -> Game {
        self.get_by_id(1)
            .unwrap_or_else(|| Game::with_id(1, Discipline::Fiba5vs5))
    }

    pub fn create_game(&self, discipline: Discipline) -> Game {
        let now = Instant::now();
        let created;
        let snapshot: Vec<Game>;

        {
            let mut cache = self.cache.write().unwrap();
            let next_id = cache
                .iter()
                .flatten()
                .map(|entry| entry.game.id())
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            created = Game::with_id(next_id, discipline);
            if (next_id as usize) < cache.len() {
                cache[next_id as usize] = Some(ActiveGame::new(created, now));
            }
            snapshot = cache.iter().flatten().map(|entry| entry.game).collect();
        }

        self.db_dirty.store(true, Ordering::Release);
        let _ = self.state.send(snapshot.clone());

        if self.broadcast.receiver_count() > 0 {
            let _ = self.broadcast.send(snapshot);
        }

        created
    }

    pub fn modify_game<F>(&self, game_id: u16, f: F) -> Option<Game>
    where
        F: FnOnce(&mut Game),
    {
        let now = Instant::now();
        let mut updated_game = None;
        let snapshot: Vec<Game>;

        {
            let mut cache = self.cache.write().unwrap();
            let target_idx =
                if (game_id as usize) < cache.len() && cache[game_id as usize].is_some() {
                    Some(game_id as usize)
                } else {
                    cache.iter().position(|entry| entry.is_some())
                };

            if let Some(idx) = target_idx
                && let Some(entry) = cache[idx].as_mut()
            {
                entry.advance_clock(now);
                let prev_state = entry.game.get_state();
                let prev_reg = entry.game.regulation_millis();

                f(&mut entry.game);

                if entry.game.get_state() == State::Running {
                    if prev_state != State::Running
                        || prev_reg != entry.game.regulation_millis()
                        || entry.last_tick.is_none()
                    {
                        entry.last_tick = Some(now);
                    }
                } else {
                    entry.last_tick = None;
                }

                updated_game = Some(entry.game);
            }

            snapshot = cache.iter().flatten().map(|entry| entry.game).collect();
        }

        self.db_dirty.store(true, Ordering::Release);
        let _ = self.state.send(snapshot.clone());

        if self.broadcast.receiver_count() > 0
            && let Err(error) = self.broadcast.send(snapshot)
        {
            eprintln!("ERROR: Failed to broadcast: {}", error);
        };

        updated_game
    }

    pub async fn get_scores(&self) -> (u16, u16) {
        let game = self.get();

        game.get_scores()
    }

    pub async fn get_score(&self, team: Team) -> u16 {
        let game = self.get();

        game.get_score(team)
    }

    pub async fn add_score(&self, team: Team, score: Score) {
        let id = self.get().id();
        self.add_score_for(id, team, score).await;
    }

    pub async fn add_score_for(&self, game_id: u16, team: Team, score: Score) {
        self.modify_game(game_id, |game| game.score(team, score));
    }

    pub async fn resume(&self) {
        let id = self.get().id();
        self.resume_for(id).await;
    }

    pub async fn resume_for(&self, game_id: u16) {
        self.modify_game(game_id, |game| game.resume());
    }

    pub async fn pause(&self) {
        let id = self.get().id();
        self.pause_for(id).await;
    }

    pub async fn pause_for(&self, game_id: u16) {
        self.modify_game(game_id, |game| game.pause());
    }

    pub async fn end_for(&self, game_id: u16) {
        self.modify_game(game_id, |game| game.set_state(State::GameEnd));
    }

    pub async fn reset_for(&self, game_id: u16) {
        self.modify_game(game_id, |game| {
            *game = Game::with_id(game.id(), game.discipline());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use basketball::Quarter;

    #[test]
    fn test_zero_drift_over_12_minutes_at_60hz() {
        let start = Instant::now();
        let mut game = Game::with_id(1, Discipline::Fiba5vs5); // 12 minut = 720_000 ms
        game.resume();

        let mut active = ActiveGame::new(game, start);
        let mut current = start;

        // Symulujemy 719 sekund w krokach 16_666_666 ns (~60 Hz z ułamkami nanosekund)
        let total_ns: u64 = 719 * 1_000_000_000;
        let step_ns: u64 = 16_666_666;
        let steps = total_ns / step_ns;
        let remainder_ns = total_ns % step_ns;

        for _ in 0..steps {
            current += Duration::from_nanos(step_ns);
            active.advance_clock(current);
        }
        current += Duration::from_nanos(remainder_ns);
        active.advance_clock(current);

        // Po dokładnie 719 sekundach (719_000 ms) powinno zostać dokładnie 1000 ms (1.00 s)
        assert_eq!(active.game.regulation_millis(), 1_000);
        assert_eq!(active.game.get_state(), State::Running);

        // Doliczamy ostatnią sekundę (1000 ms) -> koniec kwarty Q1 i przejście do Q2 (Paused)
        current += Duration::from_millis(1_000);
        active.advance_clock(current);

        assert_eq!(active.game.get_state(), State::Paused);
        assert_eq!(active.game.get_quarter(), Quarter::Q2);
        assert_eq!(active.game.regulation_millis(), 12 * 60 * 1_000);
    }
}
