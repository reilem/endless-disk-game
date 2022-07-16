#![cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoop;
use winit::platform::web::WindowExtWebSys;

use super::graphics;

pub fn run() {
    // Start the panic hook if enabled
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    // Start the logger if enabled
    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Debug).unwrap();

    // Start the event loop
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    // On wasm, append the canvas to the document body
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");
    wasm_bindgen_futures::spawn_local(graphics::run_loop(event_loop, window));
}
