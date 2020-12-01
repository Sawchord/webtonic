use bytes::Bytes;
use core::{
    convert::TryFrom,
    pin::Pin,
    task::{Context, Poll},
};
use http::{header::HeaderMap, response::Response, status::StatusCode};
use http_body::Body;
use js_sys::Error as JsError;
use tonic::{body::BoxBody, Status};
use wasm_bindgen::JsValue;
use web_sys::Response as JsResponse;

use crate::WebTonicError;

#[allow(dead_code)]
pub(crate) fn js_res_to_res(
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

    // Status
    let status_code = StatusCode::try_from(response.status()).unwrap();

    Ok(Response::builder()
        .status(status_code)
        .body(BoxBody::new(ResponseBody))
        .unwrap())
}

struct ResponseBody;

impl ResponseBody {
    fn _new(_response: &JsResponse) -> Result<Self, WebTonicError> {
        todo!()
    }
}

impl Body for ResponseBody {
    type Data = Bytes;
    type Error = Status;

    fn poll_data(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        todo!()
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        todo!()
    }
}
