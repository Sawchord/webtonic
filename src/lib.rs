mod request;
mod response;

use core::{
    fmt,
    marker::PhantomData,
    task::{Context, Poll},
};
use futures::future::{FutureExt, LocalBoxFuture};
use http::{request::Request, response::Response, uri::Uri};
use std::error::Error;
use tonic::{body::BoxBody, client::GrpcService};
//use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;
//use web_sys::console;

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
    type Future = LocalBoxFuture<'a, Result<Response<BoxBody>, WebTonicError>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // We return an ok, because we are essentially always ready to poll
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        let uri_clone = self.uri.clone();
        (async { call(uri_clone, request).await }).boxed_local()
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

async fn call(uri: Uri, request: Request<BoxBody>) -> Result<Response<BoxBody>, WebTonicError> {
    let request = req_to_js_req(uri, request).await?;
    let response = JsFuture::from(window().unwrap().fetch_with_request(&request)).await;
    js_res_to_res(response).await
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
