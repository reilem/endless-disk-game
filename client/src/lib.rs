#![cfg(target_arch = "wasm32")]
#[cfg(feature = "log")]
use log::Level;
use wasm_bindgen::prelude::*;

mod client_wasm;
mod graphics;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // Start the panic hook if enabled
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    // Start the logger if enabled
    #[cfg(feature = "console_log")]
    console_log::init_with_level(Level::Debug).unwrap();

    client_wasm::run();

    Ok(())
}
