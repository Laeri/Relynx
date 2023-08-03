use std::time::Duration;

use http_rest_file::model::HttpVersion;

use crate::model::Header;

use super::{certificate::Certificate, request::Request, timings::Timings};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub version: HttpVersion,
    pub status: u32,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
    pub duration: Duration,
    pub url: String,
    /// The end-user certificate, in the response certificate chain
    pub certificate: Option<Certificate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub request: Request,
    pub response: Response,
    pub timings: Timings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestCookie {
    pub name: String,
    pub value: String,
}

pub fn parse_cookies(s: &str) -> Vec<RequestCookie> {
    s.split(';').map(|t| parse_cookie(t.trim())).collect()
}

fn parse_cookie(s: &str) -> RequestCookie {
    match s.find('=') {
        Some(i) => RequestCookie {
            name: s.split_at(i).0.to_string(),
            value: s.split_at(i + 1).1.to_string(),
        },
        None => RequestCookie {
            name: s.to_string(),
            value: "".to_string(),
        },
    }
}

impl std::fmt::Display for RequestCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}
