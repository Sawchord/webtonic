#![allow(dead_code, unused_imports)]

use wasm_bindgen::JsValue;
use web_sys::console;

mod websocket;

pub(crate) fn console_log(s: &str) {
    console::log_1(&JsValue::from_str(s));
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
