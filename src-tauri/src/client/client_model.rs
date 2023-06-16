use std::{str::FromStr, time::Duration};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cookie {
    pub domain: String,
    pub include_subdomain: String,
    pub path: String,
    pub https: String,
    pub expires: String,
    pub name: String,
    pub value: String,
    pub http_only: bool,
}
impl std::fmt::Display for Cookie {
    /// Formats this cookie using Netscape cookie format.
    ///
    /// <http://www.cookiecentral.com/faq/#3.5>
    ///
    /// > The layout of Netscape's cookies.txt file is such that each line contains one name-value
    /// > pair. An example cookies.txt file may have an entry that looks like this:
    /// >
    /// > `.netscape.com     TRUE   /  FALSE  946684799   NETSCAPE_ID  100103`
    /// >
    /// > Each line represents a single piece of stored information. A tab is inserted between each
    /// > of the fields.
    /// > From left-to-right, here is what each field represents:
    /// > - domain - The domain that created AND that can read the variable.
    /// > - flag - A TRUE/FALSE value indicating if all machines within a given domain can access
    /// > the variable. This value is set automatically by the browser, depending on the value you
    /// > set for domain.
    /// > - path - The path within the domain that the variable is valid for.
    /// > - secure - A TRUE/FALSE value indicating if a secure connection with the domain is
    /// > needed to access the variable.
    /// > - expiration - The UNIX time that the variable will expire on. UNIX time is defined as the
    /// > - number of seconds since Jan 1, 1970 00:00:00 GMT.
    /// > - name - The name of the variable.
    /// > - value - The value of the variable.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}",
            if self.http_only { "#HttpOnly_" } else { "" },
            self.domain,
            self.include_subdomain,
            self.path,
            self.https,
            self.expires,
            self.name,
            self.value
        )
    }
}

impl std::fmt::Display for RequestCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseCookieError {}

impl FromStr for Cookie {
    type Err = ParseCookieError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = s.split_ascii_whitespace().collect::<Vec<&str>>();
        let (http_only, domain) = if let Some(&v) = tokens.first() {
            if let Some(domain) = v.strip_prefix("#HttpOnly_") {
                (true, domain.to_string())
            } else {
                (false, v.to_string())
            }
        } else {
            return Err(ParseCookieError {});
        };
        let include_subdomain = if let Some(&v) = tokens.get(1) {
            v.to_string()
        } else {
            return Err(ParseCookieError {});
        };
        let path = if let Some(&v) = tokens.get(2) {
            v.to_string()
        } else {
            return Err(ParseCookieError {});
        };
        let https = if let Some(&v) = tokens.get(3) {
            v.to_string()
        } else {
            return Err(ParseCookieError {});
        };
        let expires = if let Some(&v) = tokens.get(4) {
            v.to_string()
        } else {
            return Err(ParseCookieError {});
        };
        let name = if let Some(&v) = tokens.get(5) {
            v.to_string()
        } else {
            return Err(ParseCookieError {});
        };
        let value = if let Some(&v) = tokens.get(6) {
            v.to_string()
        } else {
            "".to_string()
        };
        Ok(Cookie {
            domain,
            include_subdomain,
            path,
            https,
            expires,
            name,
            value,
            http_only,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_cookie_from_str() {
        assert_eq!(
            Cookie::from_str("httpbin.org\tFALSE\t/\tFALSE\t0\tcookie1\tvalueA").unwrap(),
            Cookie {
                domain: "httpbin.org".to_string(),
                include_subdomain: "FALSE".to_string(),
                path: "/".to_string(),
                https: "FALSE".to_string(),
                expires: "0".to_string(),
                name: "cookie1".to_string(),
                value: "valueA".to_string(),
                http_only: false,
            }
        );
        assert_eq!(
            Cookie::from_str("localhost\tFALSE\t/\tFALSE\t1\tcookie2\t").unwrap(),
            Cookie {
                domain: "localhost".to_string(),
                include_subdomain: "FALSE".to_string(),
                path: "/".to_string(),
                https: "FALSE".to_string(),
                expires: "1".to_string(),
                name: "cookie2".to_string(),
                value: "".to_string(),
                http_only: false,
            }
        );

        assert_eq!(Cookie::from_str("xxx").err().unwrap(), ParseCookieError {});
    }
}
