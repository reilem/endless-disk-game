#![cfg(not(target_arch = "wasm32"))]

mod graphics;

pub fn main() {
    print!("Starting non-wasm project...");
    graphics::run();
}
