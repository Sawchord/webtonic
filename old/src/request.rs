use http::{request::Request, uri::Uri};
use http_body::Body;
use js_sys::Uint8Array;
use tonic::body::BoxBody;
use wasm_bindgen::JsValue;
use web_sys::{
    console, Headers as JsHeaders, Request as JsRequest, RequestCache, RequestInit, RequestMode,
};

use crate::WebTonicError;

pub(crate) async fn req_to_js_req(
    uri: Uri,
    mut req: Request<BoxBody>,
) -> Result<JsRequest, WebTonicError> {
    // Body
    let mut body_vec: Vec<u8> = vec![];
    let body_bytes = req.body_mut();
    while let Some(bytes) = body_bytes.data().await {
        body_vec.extend_from_slice(&bytes.unwrap());
    }
    let js_body = Uint8Array::from(&body_vec[..]);

    // Parse Uri
    let mut full_uri = "".to_string();
    full_uri.push_str(&format!("{}", uri));
    // TODO: Remove double slash in resource
    full_uri.push_str(&format!("{}", req.uri()));
    console::log_1(&JsValue::from_str(&format!("{}", full_uri)));

    // Method
    let method = req.method().as_str().to_string();

    // Build Headers
    let js_headers = JsHeaders::new().unwrap();
    for (header_name, header_value) in req.headers() {
        js_headers
            .append(
                header_name.as_str(),
                header_value
                    .to_str()
                    .map_err(|_| WebTonicError::InvalidUrl)?,
            )
            .unwrap();
    }

    // Version??
    // How do we guarantee http 2.0?

    // What do we do with these settings?
    // They should be set to sensible values here
    // Mode
    // Cache
    // Credentials
    // Referrer
    // Integrity

    Ok(JsRequest::new_with_str_and_init(
        &full_uri,
        &RequestInit::new()
            .method(&method)
            .mode(RequestMode::NoCors)
            .cache(RequestCache::NoCache)
            .headers(&JsValue::from(js_headers))
            .body(Some(&js_body)),
    )
    .unwrap())
}

// TODO: Test
