mod graphics;

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    print!("Starting non-wasm project...");
    graphics::run();
}

#[cfg(target_arch = "wasm32")]
pub fn main() {}
