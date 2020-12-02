use bytes::Bytes;
use core::marker::{Send, Sync};
use futures::StreamExt;
use http::request::Request;
use prost::Message as ProstMessage;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::body::BoxBody;
use tower_service::Service;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};
use webtonic_proto::Call;

// use core::future::Future;
// use core::pin::Pin;
// use http::response::Response;
// use std::error::Error;
// type FullService = Box<
//     dyn Service<
//         Request<BoxBody>,
//         Response = Response<BoxBody>,
//         Error = Box<dyn Error>,
//         Future = Pin<Box<dyn Future<Output = Result<Response<BoxBody>, Box<dyn Error>>>>>,
//     >,
// >;

// TODO: Use tonic::NamedService to get the path right once multiple services are allowed

#[derive(Debug)]
pub struct Server<B>(Arc<Mutex<B>>);

// NOTE: Derived Clone adds Clone constraint on inner, even thought we don't need ot
impl<B> Clone for Server<B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<B: Service<Request<BoxBody>> + Sync + Send + 'static> Server<B> {
    pub fn build(service: B) -> Self {
        Self(Arc::new(Mutex::new(service)))
    }

    pub async fn serve<A: Into<SocketAddr>>(self, addr: A) -> Result<(), ()>
    where
        <B as Service<Request<BoxBody>>>::Future: Send,
    {
        let server_clone = warp::any().map(move || self.clone());

        warp::serve(warp::path::end().and(warp::ws()).and(server_clone).map(
            |ws: warp::ws::Ws, server_clone| {
                ws.on_upgrade(|socket| handle_connection(socket, server_clone))
            },
        ))
        .run(addr)
        .await;

        Ok(())
    }
}
//async fn handle_connection(
async fn handle_connection<B: Service<Request<BoxBody>> + Sync + Send + 'static>(
    ws: WebSocket,
    server: Server<B>,
) {
    let (ws_tx, mut ws_rx) = ws.split();

    while let Some(msg) = ws_rx.next().await {
        // Check that we got a message and it is binary
        let msg = match msg {
            Ok(msg) => {
                if msg.is_binary() {
                    Bytes::from(msg.into_bytes())
                } else {
                    todo!()
                }
            }
            Err(_) => todo!(),
        };

        // Parse message first into protobuf then into http request
        let call = match Call::decode(msg) {
            Ok(call) => call,
            Err(_) => todo!(),
        };
        let call = webtonic_proto::call_to_http_request(call).unwrap();

        // Call the inner service
        let response = match server.0.lock().await.call(call).await {
            Ok(response) => response,
            Err(_) => todo!(),
        };

        // Turn reply first into protobuf, then into message
        //let reply = webtonic_proto::http_response_to_reply(&mut response).await;

        // Return the message
    }
}
