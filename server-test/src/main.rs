use crate::greeter_server::{Greeter, GreeterServer};
use tonic::{Request, Response, Status};

tonic::include_proto!("helloworld");

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    //let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    //println!("GreeterServer listening on {}", addr);

    // Server::builder()
    //     .add_service(GreeterServer::new(greeter))
    //     .serve(addr)
    //     .await?;
    webtonic_server::Server::build(GreeterServer::new(greeter))
        .serve(([127, 0, 0, 1], 1337))
        .await
        .unwrap();

    Ok(())
}
