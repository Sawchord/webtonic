use futures::StreamExt;
use http::request::Request;
use std::net::SocketAddr;
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

pub struct Server<B: Service<Request<BoxBody>>>(B);

impl<B: Service<Request<BoxBody>>> Server<B> {
    pub fn build(&mut self, service: B) -> Self {
        Self(service)
    }

    pub async fn serve<A: Into<SocketAddr>>(addr: A) -> Result<(), ()> {
        warp::serve(
            warp::path("/")
                .and(warp::ws())
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

#[cfg(test)]
mod tests {
    mod hello_world {
        tonic::include_proto!("helloworld");
    }
    use super::*;
    use tonic::{Request, Response, Status};

    use hello_world::greeter_server::{Greeter, GreeterServer};
    use hello_world::{HelloReply, HelloRequest};

    #[derive(Default)]
    pub struct MyGreeter {}

    #[tonic::async_trait]
    impl Greeter for MyGreeter {
        async fn say_hello(
            &self,
            request: Request<HelloRequest>,
        ) -> Result<Response<HelloReply>, Status> {
            println!("Got a request from {:?}", request.remote_addr());
            let reply = HelloReply {
                message: format!("Hello {}!", request.into_inner().name),
            };
            Ok(Response::new(reply))
        }
    }

    fn compile() {
        //Server::build<>
    }
}
