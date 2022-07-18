#![cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowExtWebSys;
use winit::window::Window;

use super::graphics;

fn init_logs() {
    // Start the panic hook if enabled
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    // Start the logger if enabled
    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Debug).unwrap();
}

/**
 * Prevents weird scrolling issues in mobile web
 */
fn disable_touch_events(window: &Window) {
    let canvas = web_sys::Element::from(window.canvas());
    let closure = Closure::wrap(Box::new(|event: web_sys::Event| {
        event.prevent_default();
    }) as Box<dyn Fn(web_sys::Event)>);
    canvas
        .add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())
        .expect("Failed to add touchstart listener");
    canvas
        .add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())
        .expect("Failed to add touchmove listener");
    canvas
        .add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())
        .expect("Failed to add touchend listener");
    canvas
        .add_event_listener_with_callback("touchcancel", closure.as_ref().unchecked_ref())
        .expect("Failed to add touchcancel listener");
    // If you don't add this the closure will be destroyed and do nothing
    closure.forget();
}

fn start_web_window() -> (Window, EventLoop<()>) {
    // Start the event loop
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    disable_touch_events(&window);

    let web_sys_window = web_sys::window().expect("Failed to get web_sys window");
    window.set_inner_size(LogicalSize::new(
        web_sys_window
            .inner_width()
            .unwrap_or(wasm_bindgen::JsValue::from(1024))
            .as_f64()
            .map_or(1024, |f| f as u32),
        web_sys_window
            .inner_height()
            .unwrap_or(wasm_bindgen::JsValue::from(768))
            .as_f64()
            .map_or(768, |f| f as u32),
    ));

    // On wasm, append the canvas to the document body
    web_sys_window
        .document()
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");
    (window, event_loop)
}

pub fn run() {
    init_logs();

    let (window, event_loop) = start_web_window();
    // wasm_bindgen_futures::spawn_local(test_game_loop());
    wasm_bindgen_futures::spawn_local(graphics::run_loop(event_loop, window));
}
