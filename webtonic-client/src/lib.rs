#![allow(dead_code, unused_imports)]

mod websocket;

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
