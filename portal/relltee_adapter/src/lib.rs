use wasm_bindgen::prelude::*;
use web_sys;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    web_sys::console::info_1(&JsValue::from_str("Hello world... from WASM!"));

    Ok(())
}
