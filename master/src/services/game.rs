use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::adapters::repositories::{GameRepository, Repository};
use crate::services::Observable;

use basketball::{Discipline, Game, Scorable, Score, Scores, State, Team};
use net::layout::PACKET_SIZE;
use net::{ServerEvent, ServerPacket};
use tokio::sync::{broadcast, watch};
use tokio::task;
use tokio::time::{self, MissedTickBehavior};

const TICK_RATE: Duration = Duration::from_nanos(1_000_000_000 / 60);

const FRAME_PRUNE_TICKS: u32 = 600;

pub type GameFrame = Arc<[u8; PACKET_SIZE]>;

type FrameChannels = Arc<RwLock<HashMap<u16, watch::Sender<Option<GameFrame>>>>>;

#[derive(Clone, Copy)]
struct ClockAdvanceOutcome {
    ticked: bool,
    state_changed: bool,
}

#[derive(Clone, Copy)]
struct ActiveGame {
    game: Game,
    last_tick: Option<Instant>,
}

impl ActiveGame {
    fn new(game: Game, now: Instant) -> Self {
        let last_tick = if game.state() == State::Running {
            Some(now)
        } else {
            None
        };
        Self { game, last_tick }
    }

    fn advance_clock(&mut self, now: Instant) -> ClockAdvanceOutcome {
        if self.game.state() != State::Running {
            self.last_tick = None;
            return ClockAdvanceOutcome {
                ticked: false,
                state_changed: false,
            };
        }

        let last = self.last_tick.get_or_insert(now);
        let elapsed_millis = now.duration_since(*last).as_millis() as u64;
        if elapsed_millis == 0 {
            return ClockAdvanceOutcome {
                ticked: false,
                state_changed: false,
            };
        }

        let previous_state = self.game.state();
        let previous_quarter = self.game.quarter();

        *last += Duration::from_millis(elapsed_millis);
        self.game.tick(Duration::from_millis(elapsed_millis));

        if self.game.state() != State::Running {
            self.last_tick = None;
        }

        let state_changed =
            previous_state != self.game.state() || previous_quarter != self.game.quarter();
        ClockAdvanceOutcome {
            ticked: true,
            state_changed,
        }
    }
}

#[derive(Clone)]
pub struct GameService {
    #[allow(dead_code)]
    game_repository: GameRepository,
    state: watch::Sender<Vec<Game>>,
    cache: Arc<RwLock<Vec<Option<ActiveGame>>>>,
    database_dirty: Arc<AtomicBool>,
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

    #[must_use]
    pub fn new(game_repository: GameRepository) -> Self {
        let mut initial_games = game_repository.select();
        if initial_games.is_empty() {
            let default_game = Game::new_with_id(1, Discipline::Fiba5v5);
            initial_games.push(default_game);
            game_repository.upsert(&initial_games);
        }

        let now = Instant::now();
        let mut cache_vec: Vec<Option<ActiveGame>> = vec![None; 64];
        for game in &initial_games {
            if (game.id() as usize) < cache_vec.len() {
                cache_vec[game.id() as usize] = Some(ActiveGame::new(*game, now));
            }
        }
        let cache = Arc::new(RwLock::new(cache_vec));
        let database_dirty = Arc::new(AtomicBool::new(false));
        let frames: FrameChannels = Arc::new(RwLock::new(HashMap::new()));

        let (state, _state_rx) = watch::channel(initial_games);
        let (broadcast, _broadcast_rx) = broadcast::channel(64);

        let repository_writer = game_repository.clone();
        let database_cache = Arc::clone(&cache);
        let database_flag = Arc::clone(&database_dirty);
        tokio::spawn(async move {
            let mut database_interval = time::interval(Duration::from_millis(250));
            database_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                database_interval.tick().await;
                if !database_flag.swap(false, Ordering::AcqRel) {
                    continue;
                }

                let snapshot: Vec<Game> = {
                    let games = database_cache
                        .read()
                        .expect("game cache lock should not be poisoned");
                    games.iter().flatten().map(|entry| entry.game).collect()
                };

                let repository = repository_writer.clone();
                let _ = task::spawn_blocking(move || {
                    repository.upsert(&snapshot);
                })
                .await;
            }
        });

        let clock_cache = Arc::clone(&cache);
        let clock_state = state.clone();
        let clock_broadcast = broadcast.clone();
        let clock_database_dirty = Arc::clone(&database_dirty);
        let clock_frames = Arc::clone(&frames);
        tokio::spawn(async move {
            let mut interval = time::interval(TICK_RATE);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let mut sequence_id: u16 = 0;
            let mut tick_counter: u32 = 0;

            loop {
                interval.tick().await;
                let now = Instant::now();
                let mut any_ticked = false;
                let mut any_state_changed = false;
                let mut snapshot = Vec::new();

                {
                    let mut games = clock_cache
                        .write()
                        .expect("game cache lock should not be poisoned");
                    let active_entries = games.iter_mut().flatten();
                    for entry in active_entries {
                        let outcome = entry.advance_clock(now);
                        any_ticked |= outcome.ticked;
                        any_state_changed |= outcome.state_changed;
                    }
                    if any_state_changed {
                        snapshot = games.iter().flatten().map(|entry| entry.game).collect();
                    }
                }

                if any_ticked {
                    clock_database_dirty.store(true, Ordering::Release);
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
                if tick_counter.is_multiple_of(FRAME_PRUNE_TICKS) {
                    clock_frames
                        .write()
                        .expect("frame channels lock should not be poisoned")
                        .retain(|_, sender| sender.receiver_count() > 0);
                }
            }
        });

        Self {
            game_repository,
            state,
            cache,
            database_dirty,
            broadcast,
            frames: Arc::clone(&frames),
        }
    }

    fn publish_frames(
        cache: &RwLock<Vec<Option<ActiveGame>>>,
        frames: &RwLock<HashMap<u16, watch::Sender<Option<GameFrame>>>>,
        sequence_id: u16,
    ) {
        let frames = frames
            .read()
            .expect("frame channels lock should not be poisoned");
        if frames.is_empty() {
            return;
        }

        let cache = cache
            .read()
            .expect("game cache lock should not be poisoned");
        for (&game_id, sender) in frames.iter() {
            if sender.receiver_count() == 0 {
                continue;
            }

            if let Some(entry) = cache.get(game_id as usize).and_then(Option::as_ref) {
                let packet =
                    ServerPacket::new(ServerEvent::GameUpdate, sequence_id, entry.game.to_bytes());
                sender.send_replace(Some(Arc::new(packet.to_bytes())));
            }
        }
    }

    #[must_use]
    pub fn subscribe_frames(&self, game_id: u16) -> watch::Receiver<Option<GameFrame>> {
        if let Some(sender) = self
            .frames
            .read()
            .expect("frame channels lock should not be poisoned")
            .get(&game_id)
        {
            return sender.subscribe();
        }

        self.frames
            .write()
            .expect("frame channels lock should not be poisoned")
            .entry(game_id)
            .or_insert_with(|| watch::channel(None).0)
            .subscribe()
    }

    #[must_use]
    pub fn all(&self) -> Vec<Game> {
        let cache = self
            .cache
            .read()
            .expect("game cache lock should not be poisoned");
        cache.iter().flatten().map(|entry| entry.game).collect()
    }

    #[must_use]
    pub fn by_id(&self, id: u16) -> Option<Game> {
        let cache = self
            .cache
            .read()
            .expect("game cache lock should not be poisoned");
        cache
            .get(id as usize)
            .and_then(|entry| entry.as_ref().map(|active| active.game))
    }

    #[must_use]
    pub fn default_game(&self) -> Game {
        self.by_id(1)
            .unwrap_or_else(|| Game::new_with_id(1, Discipline::Fiba5v5))
    }

    #[must_use]
    pub fn create_game(&self, discipline: Discipline) -> Game {
        let now = Instant::now();
        let created;
        let snapshot: Vec<Game>;

        {
            let mut cache = self
                .cache
                .write()
                .expect("game cache lock should not be poisoned");
            let next_id = cache
                .iter()
                .flatten()
                .map(|entry| entry.game.id())
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            created = Game::new_with_id(next_id, discipline);
            if (next_id as usize) < cache.len() {
                cache[next_id as usize] = Some(ActiveGame::new(created, now));
            }
            snapshot = cache.iter().flatten().map(|entry| entry.game).collect();
        }

        self.database_dirty.store(true, Ordering::Release);
        let _ = self.state.send(snapshot.clone());

        if self.broadcast.receiver_count() > 0 {
            let _ = self.broadcast.send(snapshot);
        }

        created
    }

    pub fn modify_game<F>(&self, game_id: u16, modifier: F) -> Option<Game>
    where
        F: FnOnce(&mut Game),
    {
        let now = Instant::now();
        let mut updated_game = None;
        let snapshot: Vec<Game>;

        {
            let mut cache = self
                .cache
                .write()
                .expect("game cache lock should not be poisoned");
            let target_index = if (game_id as usize) < cache.len() {
                cache[game_id as usize].map(|_| game_id as usize)
            } else {
                None
            }
            .or_else(|| cache.iter().position(Option::is_some));

            if let Some(index) = target_index
                && let Some(entry) = cache[index].as_mut()
            {
                entry.advance_clock(now);
                let previous_state = entry.game.state();
                let previous_regulation = entry.game.regulation_millis();

                modifier(&mut entry.game);

                if entry.game.state() == State::Running {
                    if previous_state != State::Running
                        || previous_regulation != entry.game.regulation_millis()
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

        self.database_dirty.store(true, Ordering::Release);
        let _ = self.state.send(snapshot.clone());

        if self.broadcast.receiver_count() > 0
            && let Err(error) = self.broadcast.send(snapshot)
        {
            eprintln!("ERROR: Failed to broadcast: {error}");
        }

        updated_game
    }

    pub async fn scores(&self) -> Scores {
        let game = self.default_game();
        game.scores()
    }

    pub async fn score(&self, team: Team) -> u16 {
        let game = self.default_game();
        game.score(team)
    }

    pub async fn add_score(&self, team: Team, score: Score) {
        let id = self.default_game().id();
        self.add_score_for(id, team, score).await;
    }

    pub async fn add_score_for(&self, game_id: u16, team: Team, score: Score) {
        self.modify_game(game_id, |game| game.add_score(team, score));
    }

    pub async fn resume(&self) {
        let id = self.default_game().id();
        self.resume_for(id).await;
    }

    pub async fn resume_for(&self, game_id: u16) {
        self.modify_game(game_id, Game::resume);
    }

    pub async fn pause(&self) {
        let id = self.default_game().id();
        self.pause_for(id).await;
    }

    pub async fn pause_for(&self, game_id: u16) {
        self.modify_game(game_id, Game::pause);
    }

    pub async fn end_for(&self, game_id: u16) {
        self.modify_game(game_id, |game| game.set_state(State::GameEnd));
    }

    pub async fn reset_for(&self, game_id: u16) {
        self.modify_game(game_id, |game| {
            *game = Game::new_with_id(game.id(), game.discipline());
        });
    }
}
