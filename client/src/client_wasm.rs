#![cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::Element;
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
    console_log::init_with_level(log::Level::Warn).unwrap(); // INFO can cause frame drops in web
}

/**
 * Prevents weird scrolling issues in mobile web
 */
fn disable_touch_events(element: &Element) {
    let closure = Closure::wrap(Box::new(|event: web_sys::Event| {
        event.prevent_default();
    }) as Box<dyn Fn(web_sys::Event)>);
    add_event_listener(element, "touchstart", &closure);
    add_event_listener(element, "touchmove", &closure);
    add_event_listener(element, "touchend", &closure);
    add_event_listener(element, "touchcancel", &closure);
    // If you don't add this the closure will be destroyed and do nothing
    closure.forget();
}

// TODO: maybe add this as a trait?
// ?Sized relaxes the size restriction of this generic type and allows the Closure type to be defined generically
fn add_event_listener<T: ?Sized>(element: &Element, listener_name: &str, closure: &Closure<T>) {
    element
        .add_event_listener_with_callback(listener_name, closure.as_ref().unchecked_ref())
        .unwrap_or_else(|err| panic!("Failed to add {} listener: {:?}", listener_name, err));
}

fn resize_window(window: &Window, web_window: &web_sys::Window) {
    window.set_inner_size(LogicalSize::new(
        web_window
            .inner_width()
            .unwrap_or_else(|_err| wasm_bindgen::JsValue::from(1024))
            .as_f64()
            .map_or(1024, |f| f as u32),
        web_window
            .inner_height()
            .unwrap_or_else(|_err| wasm_bindgen::JsValue::from(768))
            .as_f64()
            .map_or(768, |f| f as u32),
    ));
}

fn start_web_window() -> (Window, EventLoop<()>) {
    // Start the event loop
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    let web_window = web_sys::window().expect("Failed to get web window");
    let canvas = web_sys::Element::from(window.canvas());
    disable_touch_events(&canvas);
    resize_window(&window, &web_window);

    // On wasm, append the canvas to the document body
    web_window
        .document()
        .and_then(|doc| doc.body())
        .and_then(|body| body.append_child(&canvas).ok())
        .expect("couldn't append canvas to document body");
    (window, event_loop)
}

pub fn run() {
    init_logs();

    let (window, event_loop) = start_web_window();
    // wasm_bindgen_futures::spawn_local(test_game_loop());
    wasm_bindgen_futures::spawn_local(graphics::run_loop(event_loop, window));
}
