use bytes::{Bytes, BytesMut};
use core::marker::{Send, Sync};
use futures::StreamExt;
use http::{request::Request, response::Response};
use prost::Message as ProstMessage;
use std::{collections::BTreeMap, net::SocketAddr};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tonic::{body::BoxBody, transport::NamedService, Status};
use tower_service::Service;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};
use webtonic_proto::Call;

// use core::future::Future;
// use core::pin::Pin;
// use std::error::Error;
// type FullService = Box<
//     dyn Service<
//         Request<BoxBody>,
//         Response = Response<BoxBody>,
//         Error = Box<dyn Error>,
//         Future = Pin<Box<dyn Future<Output = Result<Response<BoxBody>, Box<dyn Error>>> + Send>>,
//     >,
// >;

type ServiceList<T> = BTreeMap<&'static str, T>;

#[derive(Debug, Clone)]
pub struct Server<B>(ServiceList<B>);

impl<B: Clone> Server<B> {
    pub fn builder() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add_service(mut self, service: B) -> Self
    where
        B: NamedService,
    {
        // TODO: Error out if path is taken?
        self.0.insert(<B as NamedService>::NAME, service);
        self
    }

    pub async fn serve<A>(self, addr: A) -> Result<(), ()>
    where
        A: Into<SocketAddr>,
        B: Service<Request<BoxBody>, Response = Response<BoxBody>> + Sync + Send + 'static,
        <B as Service<Request<BoxBody>>>::Future: Send,
    {
        let server_clone = warp::any().map(move || self.clone());

        warp::serve(warp::path::end().and(warp::ws()).and(server_clone).map(
            |ws: warp::ws::Ws, server_clone| {
                ws.on_upgrade(|socket| Self::handle_connection(socket, server_clone))
            },
        ))
        .run(addr)
        .await;

        Ok(())
    }

    async fn handle_connection(ws: WebSocket, server: Server<B>)
    where
        B: Service<Request<BoxBody>, Response = Response<BoxBody>>,
    {
        log::debug!("opening a new connection");

        let (ws_tx, mut ws_rx) = ws.split();
        let (tx, rx) = unbounded_channel();
        // Create outbound task
        tokio::task::spawn(rx.forward(ws_tx));

        while let Some(msg) = ws_rx.next().await {
            log::debug!("received message {:?}", msg);

            // Try to send status error
            // If even that fails, end task
            macro_rules! status_err {
                ($status: expr) => {
                    match return_status(&tx, $status).await {
                        true => continue,
                        false => break,
                    }
                };
            }

            // Check that we got a message and it is binary
            let msg = match msg {
                Ok(msg) => {
                    if msg.is_binary() {
                        Bytes::from(msg.into_bytes())
                    } else if msg.is_close() {
                        log::debug!("channel was closed");
                        break;
                    } else {
                        status_err!(Status::invalid_argument(
                            "websocket messages must be sent in binary"
                        ))
                    }
                }
                Err(e) => status_err!(Status::internal(&format!(
                    "error on the websocket channel {:?}",
                    e
                ))),
            };

            // Parse message first into protobuf then into http request
            let call = match Call::decode(msg) {
                Ok(call) => call,
                Err(e) => status_err!(Status::internal(&format!("failed to decode call {:?}", e))),
            };
            let call = webtonic_proto::call_to_http_request(call).unwrap();

            // Get the path to the requested service
            let path: &str = call
                .uri()
                .path()
                .split("/")
                .collect::<Vec<&str>>()
                .get(1)
                .unwrap_or(&&"/");
            log::debug!("request to path {:?}", path);

            // Call the inner service
            let mut server_clone = match server.0.get(path) {
                Some(server) => server.clone(),
                None => status_err!(Status::unimplemented("")),
            };
            let mut response = match server_clone.call(call).await {
                Ok(response) => response,
                Err(_e) => {
                    panic!("Tonic services never error");
                }
            };
            log::debug!("got response {:?}", response);

            // Turn reply first into protobuf, then into message
            let reply = webtonic_proto::http_response_to_reply(&mut response).await;
            let mut msg = BytesMut::new();
            match reply.encode(&mut msg) {
                Ok(()) => (),
                Err(e) => status_err!(Status::internal(&format!("failed to decode reply {:?}", e))),
            };
            let msg = Message::binary(msg.as_ref());

            // Return the message
            log::debug!("sending response {:?}", msg);
            match tx.send(Ok(msg)) {
                Ok(()) => (),
                Err(e) => {
                    log::warn!("stream no longer exists {:?}", e);
                    break;
                }
            }
        }
    }
}

async fn return_status(tx: &UnboundedSender<Result<Message, warp::Error>>, status: Status) -> bool {
    log::warn!("error while processing msg, returning status {:?}", status);
    let mut response = status.to_http();

    let reply = webtonic_proto::http_response_to_reply(&mut response).await;
    let mut msg = BytesMut::new();
    reply.encode(&mut msg).unwrap();
    let msg = Message::binary(msg.as_ref());

    match tx.send(Ok(msg)) {
        Ok(()) => true,
        Err(_) => false,
    }
}
