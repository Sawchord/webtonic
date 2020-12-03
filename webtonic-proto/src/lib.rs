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
    header::HeaderMap, method::Method as HttpMethod, request::Request as HttpRequest,
    response::Response as HttpResponse, version::Version,
};
use http_body::Body as HttpBody;
use prost::{Enumeration, Message};
use std::error::Error;
use tonic::body::BoxBody;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebTonicError {
    InvalidUrl,
    ConnectionError,
    FetchError(String),
    HttpError(u16),
}
impl Error for WebTonicError {}
impl fmt::Display for WebTonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Header {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub value: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Body {
    #[prost(bytes, tag = "1")]
    body: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum Method {
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
pub struct Request {
    #[prost(string, tag = "1")]
    pub uri: String,
    #[prost(enumeration = "Method", tag = "2")]
    pub method: i32,
    #[prost(message, repeated, tag = "3")]
    pub headers: Vec<Header>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Response {
    #[prost(uint32, tag = "1")]
    pub status: u32,
    #[prost(message, repeated, tag = "2")]
    pub headers: Vec<Header>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Call {
    #[prost(message, tag = "1")]
    pub request: Option<Request>,
    #[prost(message, tag = "2")]
    pub body: Option<Body>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Reply {
    #[prost(message, tag = "1")]
    pub response: Option<Response>,
    #[prost(message, tag = "2")]
    pub body: Option<Body>,
}

pub async fn http_request_to_call(mut request: HttpRequest<BoxBody>) -> Call {
    Call {
        request: Some(Request {
            uri: format!("{:?}", request.uri()),
            method: http_method_to_method(request.method()) as i32,
            headers: http_headers_to_headers(request.headers()),
        }),
        body: match request.body_mut().data().await {
            Some(Ok(mut body)) => Some(Body {
                body: body.to_bytes().to_vec(),
            }),
            _ => None,
        },
    }
}

pub fn call_to_http_request(call: Call) -> Option<HttpRequest<BoxBody>> {
    use http::header::{HeaderName, HeaderValue};
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
            None => BoxBody::new(Body { body: vec![] }),
        })
        .ok()
}

pub async fn http_response_to_reply(response: &mut HttpResponse<BoxBody>) -> Reply {
    Reply {
        response: Some(Response {
            status: response.status().as_u16() as u32,
            headers: http_headers_to_headers(response.headers()),
        }),
        body: match response.body_mut().data().await {
            Some(Ok(mut body)) => Some(Body {
                body: body.to_bytes().to_vec(),
            }),
            _ => None,
        },
    }
}

pub fn reply_to_http_response(reply: Reply) -> Option<HttpResponse<BoxBody>> {
    use http::header::{HeaderName, HeaderValue};
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
            None => BoxBody::new(Body { body: vec![] }),
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
        //todo!("trailer polling is unimplemented")
        Poll::Ready(Ok(None))
    }
}
