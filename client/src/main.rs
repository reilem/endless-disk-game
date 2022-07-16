mod client_main;
mod graphics;

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    print!("Starting non-wasm project...");
    // Start logger
    env_logger::init();
    client_main::run();
}

#[cfg(target_arch = "wasm32")]
pub fn main() {
    // Just here so that the rust-analyzer doesn't complain about there not being a main
}
