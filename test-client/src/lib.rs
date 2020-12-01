#[cfg(test)]
mod tests {
    pub mod hello_world {
        tonic::include_proto!("helloworld");
    }

    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
