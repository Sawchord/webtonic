extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use bytes::Buf;
use core::default::Default;
use http::{header::HeaderMap, method::Method as HttpMethod, request::Request as HttpRequest};
use http_body::Body as HttpBody;
use prost;
use prost::{Enumeration, Message};
use tonic::body::BoxBody;

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
    match method {
        &HttpMethod::GET => Method::Get,
        &HttpMethod::HEAD => Method::Head,
        &HttpMethod::POST => Method::Post,
        &HttpMethod::PUT => Method::Put,
        &HttpMethod::DELETE => Method::Delete,
        &HttpMethod::CONNECT => Method::Connect,
        &HttpMethod::OPTIONS => Method::Options,
        &HttpMethod::PATCH => Method::Patch,
        _ => panic!(),
    }
}
