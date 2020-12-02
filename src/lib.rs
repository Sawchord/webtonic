mod request;
mod response;

use core::{
    fmt,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use futures::future::{FutureExt, LocalBoxFuture};
use http::{request::Request, response::Response, uri::Uri};
use std::error::Error;
use tonic::{body::BoxBody, client::GrpcService};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, window, Request as JsRequest};

use crate::{request::req_to_js_req, response::js_res_to_res};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebTonic<'a> {
    uri: Uri,
    _dat: PhantomData<&'a ()>,
}

impl WebTonic<'_> {
    pub fn from_static(s: &'static str) -> Self {
        Self {
            uri: Uri::from_static(s),
            _dat: PhantomData,
        }
    }
}

impl<'a> GrpcService<BoxBody> for WebTonic<'a> {
    type ResponseBody = BoxBody;
    type Error = WebTonicError;
    type Future = WebTonicFuture<'a>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // We return an ok, because we are essentially always ready to poll
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        let uri_clone = self.uri.clone();
        WebTonicFuture::Request((async { req_to_js_req(uri_clone, request).await }).boxed_local())
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
    Request(LocalBoxFuture<'a, Result<JsRequest, WebTonicError>>),
    Fetch(JsFuture),
    Response(LocalBoxFuture<'a, Result<Response<BoxBody>, WebTonicError>>),
}
impl<'a> Future for WebTonicFuture<'a> {
    type Output = Result<
        Response<<WebTonic<'a> as GrpcService<BoxBody>>::ResponseBody>,
        <WebTonic<'a> as GrpcService<BoxBody>>::Error,
    >;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        console::log_1(&JsValue::from_str(&"polling"));
        let inner = self.get_mut();
        match inner {
            WebTonicFuture::Request(future) => match future.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(request) => match request {
                    Err(err) => Poll::Ready(Err(err)),
                    Ok(request) => {
                        // Do the fetch
                        console::log_1(&JsValue::from_str(&"fetching"));
                        let fetch = JsFuture::from(window().unwrap().fetch_with_request(&request));
                        *inner = WebTonicFuture::Fetch(fetch);
                        Poll::Pending
                    }
                },
            },
            WebTonicFuture::Fetch(future) => match future.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(response) => {
                    console::log_1(&JsValue::from_str(&"parsing response"));
                    let fut = js_res_to_res(response);
                    *inner = WebTonicFuture::Response((async { fut.await }).boxed_local());
                    Poll::Pending
                }
            },
            WebTonicFuture::Response(future) => match future.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(response) => {
                    console::log_1(&JsValue::from_str(&"fetching"));
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
