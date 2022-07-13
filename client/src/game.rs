#![cfg(target_arch = "wasm32")]
use game_loop::*;
use wasm_bindgen::prelude::*;
use web_sys::console;

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
    let game_state = GameState::new();

    let _g = game_loop(game_state, 60, 0.1, |g| game_update(g), |g| game_render(g));
}

fn game_update(g: &mut GameLoop<GameState, Time, ()>) {
    console::log_1(&JsValue::from(format!("Game update {}", g.game.name)));
}

fn game_render(g: &mut GameLoop<GameState, Time, ()>) {
    console::log_1(&JsValue::from(format!("Game render {}", g.game.name)));
}
