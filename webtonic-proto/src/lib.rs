//! [request]: https://docs.rs/http/0.2.1/http/request/struct.Request.html
//! [response]: https://docs.rs/http/0.2.1/http/response/struct.Response.html
//! [prost-crate]: https://github.com/danburkert/prost
//!
//! This crate contains all the part of the `WebTonic` implementation, that is shared by both
//! the server and the client.
//!
//! The crate is encoding [`Requests`][request]  into [`Calls`](Call) and [`Responses`][response]
//! into [`Replies`](Reply), using [`Prost`][prost] messages itself.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use bytes::{Buf, Bytes};
use core::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};
use http::{
    header::{HeaderMap, HeaderName, HeaderValue},
    method::Method as HttpMethod,
    request::Request as HttpRequest,
    response::Response as HttpResponse,
    version::Version,
};
use http_body::Body as HttpBody;
use prost::{Enumeration, Message};
use std::error::Error;
use tonic::body::BoxBody;

/// The error type of `WebTonic`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebTonicError {
    /// The url entered is not a valid url.
    InvalidUrl,

    /// The endpoint could not connect to the supplied url.
    ConnectionError,

    /// Error while encoding a `Request` or `Response`.
    ///
    /// This is likely a bug or implementation shortcomming of `WebTonic`.
    EncodingError,

    /// Failed to decode a received packet.
    ///
    /// Likely the other side sent a malformed packet.
    DecodingError,

    /// The connection was closed unexpectedly.
    ConnectionClosed,
}

impl Error for WebTonicError {}
impl fmt::Display for WebTonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Message)]
struct Header {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(string, tag = "2")]
    value: String,
}

#[derive(Clone, PartialEq, Message)]
struct Body {
    #[prost(bytes, tag = "1")]
    body: Vec<u8>,
    #[prost(message, repeated, tag = "2")]
    trailers: Vec<Header>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
enum Method {
    Get = 0,
    Head = 1,
    Post = 2,
    Put = 3,
    Delete = 4,
    Connect = 5,
    Options = 6,
    Trace = 7,
    Patch = 8,
}

#[derive(Clone, PartialEq, Message)]
struct Request {
    #[prost(string, tag = "1")]
    uri: String,
    #[prost(enumeration = "Method", tag = "2")]
    method: i32,
    #[prost(message, repeated, tag = "3")]
    headers: Vec<Header>,
}

#[derive(Clone, PartialEq, Message)]
struct Response {
    #[prost(uint32, tag = "1")]
    status: u32,
    #[prost(message, repeated, tag = "2")]
    headers: Vec<Header>,
}

/// A protobuf encodable internal representation of
/// a [`Request`](https://docs.rs/http/0.2.1/http/request/struct.Request.html).
#[derive(Clone, PartialEq, Message)]
pub struct Call {
    #[prost(message, tag = "1")]
    request: Option<Request>,
    #[prost(message, tag = "2")]
    body: Option<Body>,
}

/// A protobuf encodable representation of a [`Response`](https://docs.rs/http/0.2.1/http/response/struct.Response.html).
#[derive(Clone, PartialEq, Message)]
pub struct Reply {
    #[prost(message, tag = "1")]
    response: Option<Response>,
    #[prost(message, tag = "2")]
    body: Option<Body>,
}

/// Parses a [`Request`](https://docs.rs/http/0.2.1/http/request/struct.Request.html) into [`Call`](Call).
///
/// # Arguments
/// - `request`: the [`Request`](https://docs.rs/http/0.2.1/http/request/struct.Request.html) to parse
///
/// # Returns
/// - the protobuf encodable [`Call`](Call) object
pub async fn http_request_to_call(request: &mut HttpRequest<BoxBody>) -> Call {
    let body = http_body_to_body(request).await;
    let request = Some(Request {
        uri: format!("{:?}", request.uri()),
        method: http_method_to_method(request.method()) as i32,
        headers: http_headers_to_headers(request.headers()),
    });

    Call { request, body }
}

/// Parses a [`Call`](Call) into a [`Request`](https://docs.rs/http/0.2.1/http/request/struct.Request.html).
///
/// # Arguments
/// - `call`: The [`Call`](Call) to parse
///
/// # Returns
/// - `Some(request)`, if parsing succeeds.
/// - `None`, if parsing fails.
pub fn call_to_http_request(call: Call) -> Option<HttpRequest<BoxBody>> {
    use http::request::Builder;

    let request = match call.request {
        Some(request) => request,
        None => return None,
    };

    let mut builder = Builder::new()
        .version(Version::HTTP_2)
        .method(method_to_http_method(
            Method::from_i32(request.method).unwrap(),
        ))
        .uri(request.uri);

    for header in request.headers {
        builder = builder.header(
            HeaderName::from_bytes(header.name.as_bytes()).unwrap(),
            HeaderValue::from_str(header.value.as_str()).unwrap(),
        )
    }

    builder
        .body(match call.body {
            Some(body) => BoxBody::new(body),
            None => BoxBody::new(Body::empty()),
        })
        .ok()
}

/// Parse a [`Response`](https://docs.rs/http/0.2.1/http/response/struct.Response.html)
/// into a [`Reply`](Reply).
///
/// # Arguments
/// - `response`: the [`Response`](https://docs.rs/http/0.2.1/http/response/struct.Response.html) to parse
///
/// # Returns
/// - the protobuf encodable [`Reply`](Reply) object
pub async fn http_response_to_reply(response: &mut HttpResponse<BoxBody>) -> Reply {
    let body = http_body_to_body(response).await;

    let response = Some(Response {
        status: response.status().as_u16() as u32,
        headers: http_headers_to_headers(response.headers()),
    });

    Reply { response, body }
}

/// Parse a [`Reply`](Reply) into a [`Response`](https://docs.rs/http/0.2.1/http/response/struct.Response.html).
///
/// # Arguments
/// - `reply`: The [`Reply`](Reply) to parse
///
/// # Returns
/// - `Some(response)`, if parsing the
/// [`Response`](https://docs.rs/http/0.2.1/http/response/struct.Response.html) succeded
/// - `None`, if parsing failed
pub fn reply_to_http_response(reply: Reply) -> Option<HttpResponse<BoxBody>> {
    use http::response::Builder;

    let response = match reply.response {
        Some(response) => response,
        None => return None,
    };

    let mut builder = Builder::new()
        .version(Version::HTTP_2)
        .status(response.status as u16);

    for header in response.headers {
        builder = builder.header(
            HeaderName::from_bytes(header.name.as_bytes()).unwrap(),
            HeaderValue::from_str(header.value.as_str()).unwrap(),
        )
    }
    builder
        .body(match reply.body {
            Some(body) => BoxBody::new(body),
            None => BoxBody::new(Body::empty()),
        })
        .ok()
}

fn http_headers_to_headers(headers: &HeaderMap) -> Vec<Header> {
    headers
        .iter()
        .map(|(header_name, header_value)| Header {
            name: header_name.as_str().to_string(),
            value: header_value.to_str().unwrap().to_string(),
        })
        .collect()
}

fn http_method_to_method(method: &HttpMethod) -> Method {
    match *method {
        HttpMethod::GET => Method::Get,
        HttpMethod::HEAD => Method::Head,
        HttpMethod::POST => Method::Post,
        HttpMethod::PUT => Method::Put,
        HttpMethod::DELETE => Method::Delete,
        HttpMethod::CONNECT => Method::Connect,
        HttpMethod::OPTIONS => Method::Options,
        HttpMethod::TRACE => Method::Trace,
        HttpMethod::PATCH => Method::Patch,
        _ => panic!(),
    }
}

async fn http_body_to_body<B: HttpBody + Unpin>(body: &mut B) -> Option<Body> {
    let trailers = match body.trailers().await {
        Ok(Some(trailers)) => Some(http_headers_to_headers(&trailers)),
        Ok(None) => None,
        Err(_) => None,
    };

    let body = match body.data().await {
        Some(Ok(mut body)) => Some(body.to_bytes().to_vec()),
        _ => None,
    };

    match (body, trailers) {
        (None, None) => None,
        (Some(body), None) => Some(Body {
            body,
            trailers: vec![],
        }),
        (None, Some(trailers)) => Some(Body {
            body: vec![],
            trailers,
        }),
        (Some(body), Some(trailers)) => Some(Body { body, trailers }),
    }
}

fn method_to_http_method(method: Method) -> HttpMethod {
    match method {
        Method::Get => HttpMethod::GET,
        Method::Head => HttpMethod::HEAD,
        Method::Post => HttpMethod::POST,
        Method::Put => HttpMethod::PUT,
        Method::Delete => HttpMethod::DELETE,
        Method::Connect => HttpMethod::CONNECT,
        Method::Options => HttpMethod::OPTIONS,
        Method::Trace => HttpMethod::TRACE,
        Method::Patch => HttpMethod::PATCH,
    }
}

impl HttpBody for Body {
    type Data = Bytes;
    type Error = tonic::Status;

    fn poll_data(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        if !self.body.is_empty() {
            let poll = Poll::Ready(Some(Ok(Bytes::from(self.body.clone()))));
            self.get_mut().body = vec![];
            poll
        } else {
            Poll::Ready(None)
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        if !self.trailers.is_empty() {
            let mut res = HeaderMap::new();

            for trailer in &self.trailers {
                res.append(
                    HeaderName::from_bytes(trailer.name.as_bytes()).unwrap(),
                    HeaderValue::from_str(trailer.value.as_str()).unwrap(),
                );
            }
            self.get_mut().trailers.clear();

            Poll::Ready(Ok(Some(res)))
        } else {
            Poll::Ready(Ok(None))
        }
    }
}

impl Body {
    fn empty() -> Self {
        Self {
            body: vec![],
            trailers: vec![],
        }
    }
}
