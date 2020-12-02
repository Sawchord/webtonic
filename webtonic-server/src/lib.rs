use futures::StreamExt;
use http::request::Request;
use std::net::SocketAddr;
use tonic::body::BoxBody;
use tower_service::Service;
use warp::ws::WebSocket;

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

pub struct Server<B: Service<Request<BoxBody>>>(B);

impl<B: Service<Request<BoxBody>>> Server<B> {
    pub fn build(&mut self, service: B) -> Self {
        Self(service)
    }

    pub async fn serve<A: Into<SocketAddr>>(addr: A) -> Result<(), ()> {
        warp::serve(
            warp::path!(/)
                .and(warp::ws())
                .map(|ws: warp::ws::Ws| ws.on_upgrade(move |socket| handle_connection(socket))),
        )
        .run(addr)
        .await;

        Ok(())
    }
}

async fn handle_connection(ws: WebSocket) {
    let (ws_tx, ws_rx) = ws.split();
    todo!()
}
