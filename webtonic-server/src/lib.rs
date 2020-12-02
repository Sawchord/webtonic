use futures::StreamExt;
use http::request::Request;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
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

#[derive(Clone, Debug)]
pub struct Server<B: Service<Request<BoxBody>>>(Arc<RwLock<B>>);

impl<B: Service<Request<BoxBody>>> Server<B> {
    pub fn build(service: B) -> Self {
        Self(Arc::new(RwLock::new(service)))
    }

    pub async fn serve<A: Into<SocketAddr>>(&self, addr: A) -> Result<(), ()> {
        warp::serve(
            warp::path::end()
                .and(warp::ws()) //.and(self.clone())
                .map(|ws: warp::ws::Ws| ws.on_upgrade(move |socket| handle_connection(socket))),
        )
        .run(addr)
        .await;

        Ok(())
    }
}

async fn handle_connection(ws: WebSocket) {
    let (_ws_tx, _ws_rx) = ws.split();
    todo!()
}
