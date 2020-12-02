#![allow(dead_code, unused_imports)]

mod websocket;

use bytes::BytesMut;
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use futures::{future::LocalBoxFuture, FutureExt};
use http::{request::Request, response::Response};
use prost::Message;
use tonic::{body::BoxBody, client::GrpcService};
use wasm_bindgen::JsValue;
use web_sys::console;
use webtonic_proto::{Reply, WebTonicError};

use crate::websocket::WebSocketConnector;

pub(crate) fn console_log(s: &str) {
    console::log_1(&JsValue::from_str(s));
}

pub struct Client<'a> {
    ws: WebSocketConnector,
    _a: PhantomData<&'a ()>,
}

impl Client<'static> {
    pub async fn connect(uri: &str) -> Result<Self, WebTonicError> {
        let ws = WebSocketConnector::connect(uri).await?;
        Ok(Self {
            ws,
            _a: PhantomData,
        })
    }
}

impl<'a> GrpcService<BoxBody> for Client<'a> {
    type ResponseBody = BoxBody;
    type Error = WebTonicError;
    type Future = LocalBoxFuture<'a, Result<Response<BoxBody>, WebTonicError>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // We return an ok, because we are essentially always ready to poll
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        let ws_clone = self.ws.clone();
        (async { call(ws_clone, request).await }).boxed_local()
    }
}

async fn call(
    ws: WebSocketConnector,
    request: Request<BoxBody>,
) -> Result<Response<BoxBody>, WebTonicError> {
    //TODO: Error handling
    // Parse request into bytes
    let request = webtonic_proto::http_request_to_call(request).await;
    let mut msg = BytesMut::new();
    request.encode(&mut msg).unwrap();

    // Make the request
    let msg = ws.send(&msg.into()).await?;

    // Parse response
    let reply = Reply::decode(msg).unwrap();
    let response = webtonic_proto::reply_to_http_response(reply).unwrap();

    // Return
    Ok(response)
}
