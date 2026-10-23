use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use basketball::{Discipline, Game, Scorable, Score, State, Team};
use tokio::sync::{broadcast, watch};

use crate::{
    adapters::repositories::{GameRepository, Repository},
    services::Observable,
};

const TICK_RATE: Duration = Duration::from_nanos(1_000_000_000 / 60);

#[derive(Clone)]
pub struct GameService {
    #[allow(dead_code)]
    game_repository: GameRepository,
    state: watch::Sender<Vec<Game>>,
    cache: Arc<RwLock<Vec<Option<Game>>>>,
    broadcast: broadcast::Sender<Vec<Game>>,
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

        let mut cache_vec: Vec<Option<Game>> = vec![None; 2048];
        for g in &initial_games {
            if (g.id() as usize) < cache_vec.len() {
                cache_vec[g.id() as usize] = Some(*g);
            }
        }
        let cache = Arc::new(RwLock::new(cache_vec));

        let (state, mut state_rx) = watch::channel(initial_games);
        let (broadcast, _broadcast_rx) = broadcast::channel(4096);

        // Asynchroniczny zapis do ReDB w tle (bez blokowania pętli czasu rzeczywistego 60 Hz)
        let repo_writer = game_repository.clone();
        tokio::spawn(async move {
            let mut db_interval = tokio::time::interval(Duration::from_millis(250));
            db_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                if state_rx.changed().await.is_err() {
                    break;
                }
                db_interval.tick().await;
                let snapshot = state_rx.borrow_and_update().clone();
                let repo = repo_writer.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    repo.upsert(snapshot);
                })
                .await;
            }
        });

        // Niezależny zegar czasu rzeczywistego (Instant::now()) 60 Hz dla wszystkich aktywnych gier
        let clock_cache = Arc::clone(&cache);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(TICK_RATE);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut last_instant = Instant::now();

            loop {
                interval.tick().await;
                let now = Instant::now();
                // Ograniczenie catch-up w przypadku chwilowego przeciążenia schedulera
                let elapsed = now
                    .duration_since(last_instant)
                    .min(Duration::from_millis(50));
                last_instant = now;

                let mut games = clock_cache.write().unwrap();
                for game in games.iter_mut().flatten() {
                    if game.get_state() != State::Running {
                        continue;
                    }

                    game.tick(elapsed);
                }
            }
        });

        Self {
            game_repository,
            state,
            cache,
            broadcast,
        }
    }

    pub fn get_all(&self) -> Vec<Game> {
        self.state.borrow().clone()
    }

    pub fn get_by_id(&self, id: u16) -> Option<Game> {
        let cache = self.cache.read().unwrap();
        cache.get(id as usize).copied().flatten()
    }

    pub fn get(&self) -> Game {
        self.get_by_id(1)
            .unwrap_or_else(|| Game::with_id(1, Discipline::Fiba5vs5))
    }

    pub fn create_game(&self, discipline: Discipline) -> Game {
        let mut created = Game::with_id(1, discipline);
        let mut snapshot = Vec::new();

        self.state.send_modify(|games| {
            let next_id = games
                .iter()
                .map(|g| g.id())
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            created = Game::with_id(next_id, discipline);
            games.push(created);
            snapshot = games.clone();
        });

        {
            let mut cache = self.cache.write().unwrap();
            if (created.id() as usize) < cache.len() {
                cache[created.id() as usize] = Some(created);
            }
        }

        if self.broadcast.receiver_count() > 0 {
            let _ = self.broadcast.send(snapshot);
        }

        created
    }

    pub fn modify_game<F>(&self, game_id: u16, f: F) -> Option<Game>
    where
        F: FnOnce(&mut Game),
    {
        let mut updated_game = None;
        let mut snapshot = Vec::new();

        self.state.send_modify(|games| {
            if let Some(game) = games.iter_mut().find(|g| g.id() == game_id) {
                f(game);
                updated_game = Some(*game);
            } else if let Some(first) = games.first_mut() {
                f(first);
                updated_game = Some(*first);
            }
            snapshot = games.clone();
        });

        if let Some(game) = updated_game {
            let mut cache = self.cache.write().unwrap();
            if (game.id() as usize) < cache.len() {
                cache[game.id() as usize] = Some(game);
            }
        }

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
