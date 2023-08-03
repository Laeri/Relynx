/*
* This file has been taken and modified from the project: https://github.com/Orange-OpenSource/hurl.
* Changes include omitting unneeded code snippets as well as modifications to the domain model
* to match the requirements of relynx.app.
* See the copyright below.
*
* Hurl (https://hurl.dev)
* Copyright (C) 2023 Orange
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*          http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*
*/
use url::Url;

use super::error::HttpError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub value: String,
}

/// Represents an HTTP header
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl Request {
    /// Returns the base url http(s)://host(:port)
    pub fn base_url(&self) -> Result<String, HttpError> {
        // FIXME: is it possible to do it with libcurl?
        let url = match Url::parse(&self.url) {
            Ok(url) => url,
            Err(_) => return Err(HttpError::InvalidUrl(self.url.clone())),
        };
        let base_url = format!(
            "{}://{}{}",
            url.scheme(),
            url.host().unwrap(),
            if let Some(port) = url.port() {
                format!(":{port}")
            } else {
                "".to_string()
            }
        );
        Ok(base_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url() {
        assert_eq!(
            Request {
                url: "http://localhost".to_string(),
                method: "".to_string(),
                headers: vec![],
                body: vec![],
            }
            .base_url()
            .unwrap(),
            "http://localhost".to_string()
        );
        assert_eq!(
            Request {
                url: "http://localhost:8000/redirect-relative".to_string(),
                method: "".to_string(),
                headers: vec![],
                body: vec![],
            }
            .base_url()
            .unwrap(),
            "http://localhost:8000".to_string()
        );
        assert_eq!(
            Request {
                url: "https://localhost:8000".to_string(),
                method: "".to_string(),
                headers: vec![],
                body: vec![],
            }
            .base_url()
            .unwrap(),
            "https://localhost:8000".to_string()
        );
    }
}
