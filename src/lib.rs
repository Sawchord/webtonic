mod request;

use core::fmt;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::future::{BoxFuture, FutureExt};
use http::request::Request;
use http::response::Response;
use http::uri::Uri;
use std::error::Error;
use tonic::body::BoxBody;
use tonic::client::GrpcService;
use web_sys::Request as JsRequest;

use crate::request::req_to_js_req;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebTonic<'a> {
    uri: Uri,
    _dat: PhantomData<&'a ()>,
}

impl<'a> GrpcService<BoxBody> for WebTonic<'a> {
    type ResponseBody = BoxBody;
    type Error = WebTonicError;
    type Future = WebTonicFuture<'a>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // How do we implement this?
        todo!()
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        let uri_clone = self.uri.clone();
        WebTonicFuture::Request((async { req_to_js_req(uri_clone, request).await }).boxed())
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

pub enum WebTonicFuture<'a> {
    Request(BoxFuture<'a, Result<JsRequest, WebTonicError>>),
}
impl<'a> Future for WebTonicFuture<'a> {
    type Output = Result<
        Response<<WebTonic<'a> as GrpcService<BoxBody>>::ResponseBody>,
        <WebTonic<'a> as GrpcService<BoxBody>>::Error,
    >;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut() {
            WebTonicFuture::Request(future) => match future.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(request) => match request {
                    Err(err) => Poll::Ready(Err(err)),
                    // TODO: Do the actual fetch
                    Ok(_req) => Poll::Pending,
                },
            },
        }
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
