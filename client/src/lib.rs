mod client_desktop;
mod client_wasm;
mod graphics;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
        #[cfg(feature = "wee_alloc")]
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
        #[wasm_bindgen(start)]
        pub fn start() -> Result<(), JsValue> {
            log::info!("Starting non-wasm project...");
            client_wasm::run();
            Ok(())
        }
    } else {
        pub fn start() -> Result<(), String> {
            log::info!("Starting non-wasm project...");
            client_desktop::run();
            Ok(())
        }
    }
}
