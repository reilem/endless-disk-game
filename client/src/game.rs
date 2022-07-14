#![cfg(target_arch = "wasm32")]
use game_loop::*;
use log::{debug, Level};

struct GameState {
    name: String,
}

impl GameState {
    pub fn new() -> Self {
        return GameState {
            name: "Default".to_string(),
        };
    }
}

pub fn run() {
    console_log::init_with_level(Level::Debug).unwrap();
    let game_state = GameState::new();

    let _g = game_loop(game_state, 60, 0.1, |g| game_update(g), |g| game_render(g));
}

fn game_update(g: &mut GameLoop<GameState, Time, ()>) {
    debug!("Game update {}", g.game.name);
}

fn game_render(g: &mut GameLoop<GameState, Time, ()>) {
    debug!("Game render {}", g.game.name);
}
