tonic::include_proto!("helloworld");
tonic::include_proto!("grpc.examples.echo");

#[cfg(test)]
mod tests {
    wasm_bindgen_test_configure!(run_in_browser);
    use super::*;
    use wasm_bindgen_test::*;
    use webtonic_client::Client;

    #[wasm_bindgen_test]
    async fn hello_world() {
        let client = Client::connect("ws://localhost:1337").await.unwrap();
        let mut client = greeter_client::GreeterClient::new(client);

        let request = tonic::Request::new(HelloRequest {
            name: "WebTonic".into(),
        });

        let response = client.say_hello(request).await.unwrap().into_inner();
        assert_eq!(response.message, "Hello WebTonic!");
    }

    #[wasm_bindgen_test]
    async fn echo_unary() {
        let client = Client::connect("ws://localhost:1337").await.unwrap();
        let mut client = echo_client::EchoClient::new(client);

        let request = tonic::Request::new(EchoRequest {
            message: "Echo Test".to_string(),
        });

        let response = client.unary_echo(request).await.unwrap().into_inner();
        assert_eq!(response.message, "Echo Test");
    }
}
