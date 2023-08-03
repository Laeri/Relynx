/*
* This file has been taken and modified from the project: https://github.com/Orange-OpenSource/hurl.
* Changes include omitting unneeded code snippets as well as modifications to the domain model
* to match the requirements of relynx.app.
* See the copyright below.

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

use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HttpError {
    #[error("Could not parse response")]
    CouldNotParseResponse,
    #[error("Could not uncompress response, description: '{description}'")]
    CouldNotUncompressResponse { description: String },
    #[error("Invalid charset: '{charset}'")]
    InvalidCharset { charset: String },
    #[error("Invalid decoding, charset: '{charset}'")]
    InvalidDecoding { charset: String },
    #[error("Libcurl error, code: '{code}', description: '{description}', url: '{url}'")]
    Libcurl {
        code: i32,
        description: String,
        url: String,
    },
    #[error("Status line is missing, url: '{url}'")]
    StatuslineIsMissing { url: String },
    #[error("Too many redirects")]
    TooManyRedirect,
    #[error("Unsupported encoding, description: '{description}'")]
    UnsupportedContentEncoding { description: String },

    #[error("Invalid url: '{0}'")]
    InvalidUrl(String),

    #[error("The body file does not exist: '{0}'")]
    CouldNotReadBodyFile(PathBuf),

    #[error("The request body multipart file does not exist: '{0}'")]
    CouldNotReadBodyPartFromFile(PathBuf),

    #[error("Form error")]
    FormError,
}

impl From<curl::Error> for HttpError {
    fn from(err: curl::Error) -> Self {
        let code = err.code() as i32;
        let description = err.description().to_string();
        let url = "".to_string();
        HttpError::Libcurl {
            code,
            description,
            url,
        }
    }
}
