pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    use webtonic::WebTonic;
    wasm_bindgen_test_configure!(run_in_browser);
    use hello_world::HelloRequest;

    #[wasm_bindgen_test]
    async fn hello_world() {
        let client = WebTonic::from_static("http://[::1]:50051");
        let mut client = hello_world::greeter_client::GreeterClient::new(client);

        let request = tonic::Request::new(HelloRequest {
            name: "WebTonic".into(),
        });

        let response = client.say_hello(request).await.unwrap().into_inner();
        assert_eq!(response.message, "Hello, WebTonic");
    }
}
