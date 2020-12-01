mod request;
mod response;

use core::{
    fmt,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use futures::future::{BoxFuture, FutureExt};
use http::{request::Request, response::Response, uri::Uri};
use std::error::Error;
use tonic::{body::BoxBody, client::GrpcService};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Request as JsRequest};

use crate::{request::req_to_js_req, response::js_res_to_res};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebTonic<'a> {
    uri: Uri,
    _dat: PhantomData<&'a ()>,
}

// TODO: Constructors

impl<'a> GrpcService<BoxBody> for WebTonic<'a> {
    type ResponseBody = BoxBody;
    type Error = WebTonicError;
    type Future = WebTonicFuture<'a>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // How do we implement this?
        todo!("poll ready unimplemented")
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        let uri_clone = self.uri.clone();
        WebTonicFuture::Request((async { req_to_js_req(uri_clone, request).await }).boxed())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebTonicError {
    InvalidUrl,
    FetchError(String),
    HttpError(u16),
}
impl Error for WebTonicError {}
impl fmt::Display for WebTonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// TODO: Add inner to hide the internals
pub enum WebTonicFuture<'a> {
    Request(BoxFuture<'a, Result<JsRequest, WebTonicError>>),
    Fetch(JsFuture),
}
impl<'a> Future for WebTonicFuture<'a> {
    type Output = Result<
        Response<<WebTonic<'a> as GrpcService<BoxBody>>::ResponseBody>,
        <WebTonic<'a> as GrpcService<BoxBody>>::Error,
    >;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = self.get_mut();
        match inner {
            WebTonicFuture::Request(future) => match future.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(request) => match request {
                    Err(err) => Poll::Ready(Err(err)),
                    Ok(request) => {
                        // Do the fetch
                        let fetch = JsFuture::from(window().unwrap().fetch_with_request(&request));
                        *inner = WebTonicFuture::Fetch(fetch);
                        Poll::Pending
                    }
                },
            },
            WebTonicFuture::Fetch(future) => match future.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(response) => {
                    // Parse response
                    let response = js_res_to_res(response);
                    Poll::Ready(response)
                }
            },
            #[allow(unreachable_patterns)]
            _ => todo!(),
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
