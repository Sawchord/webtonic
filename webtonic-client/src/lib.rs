mod websocket;

use bytes::BytesMut;
use core::{
    marker::PhantomData,
    task::{Context, Poll},
};
use futures::{future::LocalBoxFuture, FutureExt};
use http::{request::Request, response::Response};
use prost::Message;
use tonic::{body::BoxBody, client::GrpcService};
use wasm_bindgen::JsValue;
use web_sys::console;
use webtonic_proto::{Reply, WebTonicError};

use crate::websocket::WebSocketConnector;

pub(crate) fn console_log(s: &str) {
    console::log_1(&JsValue::from_str(s));
}

/// A websocket-tunneled, browser enabled tonic client.
///
/// This client can be used in place of tonic's
/// [`Channel`](https://docs.rs/tonic/0.3.1/tonic/transport/struct.Channel.html).
/// It tunnels the request through a websocket connection to the server that reconstructs them and send them
/// to their respective handlers.
///
/// # Cryptography
/// This transport implementation does not directly support encryption.
/// It is however possible to encrypt the websocket connection itself.
/// However, client authentication is not possible that way.
///
/// # Example
/// Assuming we have the
/// [echo example](https://github.com/hyperium/tonic/blob/master/examples/proto/helloworld/helloworld.proto)
/// in scope, we can instanciate a connection like so:
///
/// ```
/// let client = Client::connect("ws://localhost:8080").await.unwrap();
/// let mut client = greeter_client::GreeterClient::new(client);
///
/// let request = tonic::Request::new(HelloRequest {
///    name: "WebTonic".into(),
/// });
///
/// let response = client.say_hello(request).await.unwrap().into_inner();
/// assert_eq!(response.message, "Hello WebTonic!");
/// ```
#[derive(Debug, Clone)]
pub struct Client<'a> {
    ws: WebSocketConnector,
    _a: PhantomData<&'a ()>,
}

impl Client<'static> {
    /// Connects the client to the endpoint.
    ///
    /// # Arguments
    /// - `uri`: The uri to connect to.
    /// **Note**: The sceme is either `ws://` or `wss://`, depending wether encryption is used or not.
    ///
    /// # Returns
    /// - A [`Client`](Client) on success.
    /// - [`WebTonicError::InvalidUrl`](WebTonicError::InvalidUrl), if the url is malformed.
    /// - [`WebTonicError::ConnectionError`](WebTonicError::InvalidUrl), if the endpoint can not be reached.
    ///
    /// # Example
    /// ```
    /// let client = Client::connect("ws://localhost:1337").await.unwrap();
    /// ```
    pub async fn connect(uri: &str) -> Result<Self, WebTonicError> {
        let ws = WebSocketConnector::connect(uri).await?;
        Ok(Self {
            ws,
            _a: PhantomData,
        })
    }
}

impl<'a> GrpcService<BoxBody> for Client<'a> {
    type ResponseBody = BoxBody;
    type Error = WebTonicError;
    type Future = LocalBoxFuture<'a, Result<Response<BoxBody>, WebTonicError>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // We return an ok, because we are essentially always ready to poll
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        let ws_clone = self.ws.clone();
        (async { call(ws_clone, request).await }).boxed_local()
    }
}

async fn call(
    ws: WebSocketConnector,
    mut request: Request<BoxBody>,
) -> Result<Response<BoxBody>, WebTonicError> {
    // Parse request into bytes
    let request = webtonic_proto::http_request_to_call(&mut request).await;
    let mut msg = BytesMut::new();
    request
        .encode(&mut msg)
        .map_err(|_| WebTonicError::EncodingError)?;

    // Make the request
    let msg = ws.send(&msg.into()).await?;

    // Parse response
    let reply = Reply::decode(msg).map_err(|_| WebTonicError::DecodingError)?;
    let response =
        webtonic_proto::reply_to_http_response(reply).ok_or(WebTonicError::DecodingError)?;

    // Return
    Ok(response)
}
