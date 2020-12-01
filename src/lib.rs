mod request;

use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use http::request::Request;
use http::response::Response;
use std::error::Error;
use tonic::body::BoxBody;
use tonic::client::GrpcService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebTonic;

impl GrpcService<BoxBody> for WebTonic {
    type ResponseBody = BoxBody;
    type Error = WebTonicError;
    type Future = WebTonicFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, _request: Request<BoxBody>) -> Self::Future {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebTonicError {
    InvalidUrl,
}
impl Error for WebTonicError {}
impl fmt::Display for WebTonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct WebTonicFuture;
impl Future for WebTonicFuture {
    type Output = Result<
        Response<<WebTonic as GrpcService<BoxBody>>::ResponseBody>,
        <WebTonic as GrpcService<BoxBody>>::Error,
    >;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
