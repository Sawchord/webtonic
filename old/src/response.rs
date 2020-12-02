use bytes::Bytes;
use core::{
    convert::TryFrom,
    pin::Pin,
    task::{Context, Poll},
};
use http::{header::HeaderMap, response::Response, status::StatusCode};
use http_body::Body;
use js_sys::{ArrayBuffer, Error as JsError, Uint8Array};
use tonic::{body::BoxBody, Status};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, Blob, Response as JsResponse};

use crate::WebTonicError;

#[allow(dead_code)]
pub(crate) async fn js_res_to_res(
    response: Result<JsValue, JsValue>,
) -> Result<Response<BoxBody>, WebTonicError> {
    // Catch errors that happened during fetch
    let response = match response {
        Err(error) => {
            let err = JsError::from(error);
            return Err(WebTonicError::FetchError(String::from(err.to_string())));
        }
        Ok(response) => JsResponse::from(response),
    };

    // Check the http response
    // FIXME: Needed? Instead just return the status
    if !response.ok() {
        return Err(WebTonicError::HttpError(response.status()));
    }

    // TODO: How to get headers?
    console::log_1(&JsValue::from_str(&format!("{:?}", response.headers())));
    //for header in response.headers().keys() {}

    // Status
    let status_code = match StatusCode::try_from(response.status()) {
        Ok(status_code) => status_code,
        Err(_) => return Err(WebTonicError::HttpError(response.status())),
    };

    // Parse the Body
    // NOTE: We wait here for the body to be streamed completely and then return it on first poll
    // in ResponseBody. This is not the most performant solution
    // TODO: Error handling
    let blob = Blob::from(JsFuture::from(response.blob().unwrap()).await.unwrap());
    let array_buffer = ArrayBuffer::from(JsFuture::from(blob.array_buffer()).await.unwrap());
    let array = Uint8Array::new(&JsValue::from(array_buffer));
    let body = Bytes::from(array.to_vec());

    Ok(Response::builder()
        .status(status_code)
        .body(BoxBody::new(ResponseBody::new(body)?))
        .unwrap())
}

struct ResponseBody {
    body: Bytes,
}

impl ResponseBody {
    fn new(body: Bytes) -> Result<Self, WebTonicError> {
        Ok(Self { body })
    }
}

impl Body for ResponseBody {
    type Data = Bytes;
    type Error = Status;

    fn poll_data(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Poll::Ready(Some(Ok(self.get_mut().body.clone())))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        todo!("trailer polling is unimplemented")
    }
}

// TODO: Tests
