use core::marker::{Send, Sync};
use futures::StreamExt;
use http::request::Request;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tonic::body::BoxBody;
use tower_service::Service;
use warp::{ws::WebSocket, Filter};

//use core::future::Future;
//use core::pin::Pin;
//use std::error::Error;
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

impl<B: Service<Request<BoxBody>> + Sync + Send> Server<B> {
    pub fn build(service: B) -> Self {
        Self(Arc::new(Mutex::new(service)))
    }

    pub async fn serve<A: Into<SocketAddr>>(self, addr: A) -> Result<(), ()> {
        warp::serve(
            warp::path::end()
                .and(warp::ws())
                .map(|ws: warp::ws::Ws| ws.on_upgrade(|socket| handle_connection(socket))),
        )
        .run(addr)
        .await;

        Ok(())
    }
}
async fn handle_connection(
    //async fn handle_connection<B: Service<Request<BoxBody>> + Sync + Send>(
    ws: WebSocket,
    //server: &Server<B>,
) {
    let (_ws_tx, _ws_rx) = ws.split();
    todo!()
}
