extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::default::Default;
use prost;
use prost::{Enumeration, Message};

#[derive(Clone, PartialEq, Message)]
pub struct Header {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub value: String,
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
    pub path: String,
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
    #[prost(bytes, tag = "2")]
    pub body: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Reply {
    #[prost(message, tag = "1")]
    pub response: Option<Response>,
    #[prost(bytes, tag = "2")]
    pub body: Vec<u8>,
}
