#[cfg(target_arch = "wasm32")]
use cc_gui::App;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    eframe::start_web(canvas_id, Box::new(App::new()))
}
