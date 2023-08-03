/*
* This file has been taken and modified from the project: https://github.com/Orange-OpenSource/hurl.
* Changes include using our own domain model in certain places and modifying request execute logic
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

mod certificate;
pub mod client_model;
mod easy_ext;
pub mod error;
pub mod options;
mod request;
mod timings;

use self::certificate::Certificate;
use self::client_model::{parse_cookies, Call, RequestCookie, Response};
use self::error::HttpError;
use self::options::{ClientOptions, Verbosity};
use self::timings::Timings;
use crate::model::{
    Environment, GetHeadersOption, Header, Multipart, RequestBody, RequestModel, RunLogger,
};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::Utc;
use curl::easy::{self, List, SslOpt};
use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};
use http_rest_file::model::{DataSource, HttpMethod, HttpVersion, UrlEncodedParam};
use std::io::Read;
use std::path::PathBuf;

/// Defines an HTTP client to execute HTTP requests.
///
/// Most of the methods are delegated to libcurl functions, while some
/// features are implemented "by hand" (like retry, redirection etc...)
#[derive(Debug)]
pub struct Client {
    /// The handle to libcurl binding
    handle: Box<easy::Easy>,
}

// @TODO: use our own Response struct?
impl Response {
    pub fn get_header_values(&self, key: &str, options: GetHeadersOption) -> Vec<String> {
        let lowercase_key = key.to_lowercase();
        let headers = self
            .headers
            .iter()
            .filter(|header| header.key.to_lowercase() == lowercase_key);

        match options {
            GetHeadersOption::JustValues => headers.map(|header| header.value.clone()).collect(),
            GetHeadersOption::SplitMultiple => headers
                .flat_map(|header| header.value.split(','))
                .map(str::to_string)
                .collect(),
        }
    }
}

impl RequestModel {
    fn cookies(&self) -> Vec<RequestCookie> {
        let header_values = self.get_header_values("Cookie", GetHeadersOption::SplitMultiple);
        header_values
            .iter()
            .flat_map(|header_val| parse_cookies(header_val))
            .collect()
    }
}

impl Client {
    /// Creates HTTP Hurl client.
    pub fn new(cookie_input_file: Option<String>) -> Client {
        let mut h = easy::Easy::new();

        // Set handle attributes
        // that are not affected by reset

        // Activate cookie storage
        // with or without persistence (empty string)
        h.cookie_file(cookie_input_file.unwrap_or_default())
            .unwrap();

        Client {
            handle: Box::new(h),
        }
    }

    pub fn execute(
        &mut self,
        request_model: &RequestModel,
        options: &ClientOptions,
        environment: Option<&Environment>,
        logger: &RunLogger,
    ) -> Result<Vec<Call>, HttpError> {
        let calls = if options.follow_location {
            logger.log_info("Run request: follow location");
            self.execute_with_redirect(request_model, options, environment, logger)
        } else {
            logger.log_info("Run request  no redirect following");
            self.execute_without_redirect(request_model, options, environment, logger)
                .map(|result_call| vec![result_call])
        }?;

        Ok(calls)
    }

    // Executes an HTTP request `request_spec`, optionally follows redirection and returns a
    // list of pair of [`Request`], [`Response`].
    fn execute_with_redirect(
        &mut self,
        request_model: &RequestModel,
        options: &ClientOptions,
        environment: Option<&Environment>,
        logger: &RunLogger, //logger: &Logger,
    ) -> Result<Vec<Call>, HttpError> {
        let mut calls = vec![];

        // Unfortunately, follow-location feature from libcurl can not be used
        // libcurl returns a single list of headers for the 2 responses
        // Hurl needs to keep everything.
        let mut request_model = request_model.clone();
        let mut redirect_count = 0;
        loop {
            let call =
                self.execute_without_redirect(&request_model, options, environment, logger)?;
            let base_url = call.request.base_url()?;
            let redirect_url = self.get_follow_location(&call.response, &base_url);
            calls.push(call);
            if !options.follow_location || redirect_url.is_none() {
                break;
            }
            let redirect_url = redirect_url.unwrap();

            logger.log_debug(format!("=> Redirect to {redirect_url}").as_str());

            redirect_count += 1;
            if let Some(max_redirect) = options.max_redirect {
                if redirect_count > max_redirect {
                    logger.log_error("Too many redirects present");
                    return Err(HttpError::TooManyRedirect);
                }
            }
            request_model = request_model.clone();
            request_model.url = redirect_url;
        }
        Ok(calls)
    }

    /// Executes an HTTP request `request_spec`, without following redirection and returns a
    /// pair of [`Request`], [`Response`].
    fn execute_without_redirect(
        &mut self,
        request_model: &RequestModel,
        options: &ClientOptions,
        environment: Option<&Environment>,
        logger: &RunLogger,
    ) -> Result<Call, HttpError> {
        // Set handle attributes that have not been set or reset.

        // We force libcurl verbose mode regardless of Hurl verbose option to be able
        // to capture HTTP request headers in libcurl `debug_function`. That's the only
        // way to get access to the outgoing headers.
        self.handle.verbose(true).unwrap();

        // Activates the access of certificates info chain after a transfer has been executed.
        self.handle.certinfo(true).unwrap();

        if !options.connects_to.is_empty() {
            let connects = to_list(&options.connects_to);
            self.handle.connect_to(connects).unwrap();
        }
        if !options.resolves.is_empty() {
            let resolves = to_list(&options.resolves);
            self.handle.resolve(resolves).unwrap();
        }
        self.handle.ssl_verify_host(!options.insecure).unwrap();
        self.handle.ssl_verify_peer(!options.insecure).unwrap();
        if let Some(cacert_file) = options.cacert_file.clone() {
            self.handle.cainfo(cacert_file).unwrap();
            self.handle.ssl_cert_type("PEM").unwrap();
        }
        if let Some(client_cert_file) = options.client_cert_file.clone() {
            self.handle.ssl_cert(client_cert_file).unwrap();
            self.handle.ssl_cert_type("PEM").unwrap();
        }
        if let Some(client_key_file) = options.client_key_file.clone() {
            self.handle.ssl_key(client_key_file).unwrap();
            self.handle.ssl_cert_type("PEM").unwrap();
        }
        if let Some(proxy) = options.proxy.clone() {
            self.handle.proxy(proxy.as_str()).unwrap();
        }
        if let Some(s) = options.no_proxy.clone() {
            self.handle.noproxy(s.as_str()).unwrap();
        }
        self.handle.timeout(options.timeout).unwrap();
        self.handle
            .connect_timeout(options.connect_timeout)
            .unwrap();

        self.set_ssl_options(options.ssl_no_revoke);

        let url = request_model.get_url_with_env(true, environment);
        logger.log_debug(format!("=>Url: {}", url).as_str());

        self.handle.url(url.as_str()).unwrap();
        let method = &request_model.method;
        self.set_method(method);
        self.set_cookies(&request_model.cookies());
        if let RequestBody::UrlEncoded { .. } = request_model.body {
            self.set_form_url_encoded(
                &request_model
                    .get_url_encoded_params_with_env(environment)
                    .unwrap(),
            );
        }
        if let RequestBody::Multipart {
            ref boundary,
            ref parts,
        } = request_model.body
        {
            self.set_multipart(boundary, parts, logger)?;
        }

        let request_body_bytes: Option<Vec<u8>> = match request_model.body {
            RequestBody::None => None,
            RequestBody::Raw { ref data } => match data {
                DataSource::Raw(ref raw_data) => Some(raw_data.as_bytes().to_vec()),
                DataSource::FromFilepath(ref filepath) => {
                    let filepath = PathBuf::from(filepath);
                    let content = std::fs::read(&filepath).map_err(|err| {
                        log::error!(
                            "File not present for multipart body, path: {}",
                            filepath.display()
                        );
                        log::error!("Io Error: {:?}", err);
                        HttpError::CouldNotReadBodyFile(filepath)
                    })?;
                    Some(content)
                }
            },
            RequestBody::Multipart { .. } => None, // Nothing done here, handled separately
            RequestBody::UrlEncoded { .. } => None, // Nothing done here, handled separately
        };

        if let Some(ref bytes) = request_body_bytes {
            self.set_body(bytes);
        }
        //let mut request_body_bytes: &[u8] = &request_model.body.bytes();
        self.set_headers(request_model, options, environment);

        let start = Utc::now();
        let verbose = options.verbosity.is_some();
        let very_verbose = options.verbosity == Some(Verbosity::VeryVerbose);
        let mut request_headers: Vec<Header> = vec![];
        let mut status_lines = vec![];
        let mut response_headers = vec![];
        let has_body_data = request_body_bytes.is_some()
            || request_model.body.is_url_encoded()
            || request_model.body.is_multipart();

        // `request_body` are request body bytes computed by libcurl (the real bytes sent over the wire)
        // whereas`request_spec_body` are request body bytes provided by Hurl user. For instance, if user uses
        // a [FormParam] section, `request_body` is empty whereas libcurl sent a url-form encoded list
        // of key-value.
        let mut request_body = Vec::<u8>::new();
        let mut response_body = Vec::<u8>::new();

        if *method == HttpMethod::HEAD {
            self.handle.nobody(true).unwrap();
        }
        {
            let mut transfer = self.handle.transfer();
            if request_body_bytes.is_some() {
                transfer
                    .read_function(|buf| {
                        Ok((&request_body_bytes.as_ref().unwrap()[..])
                            .read(buf)
                            .unwrap_or(0))
                    })
                    .unwrap();
            }

            transfer
                .debug_function(|info_type, data| match info_type {
                    // Return all request headers (not one by one)
                    easy::InfoType::HeaderOut => {
                        let mut lines = split_lines(data);
                        logger.log_debug(
                            format!("deasy infotype headerout lines: {:?}", &lines[0]).as_str(),
                        );

                        // Extracts request headers from libcurl debug info.
                        lines.pop().unwrap(); // Remove last empty line.
                        lines.remove(0); // Remove method/path/version line.
                        for line in lines {
                            if let Some(header) = Header::parse(&line) {
                                request_headers.push(header);
                            }
                        }

                        // If we don't send any data, we log headers and empty body here
                        // instead of relying on libcurl computing body in easy::InfoType::DataOut.
                        // because libcurl dont call easy::InfoType::DataOut if there is no data
                        // to send.
                        if !has_body_data && verbose {
                            //@TODO: what do they do here? we create a debug request? let debug_request = RequestModel {
                            //     url: url.to_string(),
                            //     method: method.to_string(),
                            //     headers: request_headers.clone(),
                            //     body: Vec::new(),
                            // };
                            // @TODO: for header in &debug_request.headers {
                            //     logger.debug_header_out(&header.name, &header.value);
                            // }
                            // logger.info(">");

                            // if very_verbose {
                            //     debug_request.log_body(logger);
                            // }
                        }
                    }
                    // We use this callback to get the real body bytes sent by libcurl.
                    easy::InfoType::DataOut => {
                        // Extracts request body from libcurl debug info.
                        request_body.extend(data);
                        //@TODO if verbose {
                        //     let debug_request = Request {
                        //         url: url.to_string(),
                        //         method: method.to_string(),
                        //         headers: request_headers.clone(),
                        //         body: Vec::from(data),
                        //     };
                        //     for header in &debug_request.headers {
                        //         logger.debug_header_out(&header.name, &header.value);
                        //     }
                        //     logger.info(">");

                        //     if very_verbose {
                        //         debug_request.log_body(logger);
                        //     }
                        // }
                    }
                    // Curl debug logs
                    easy::InfoType::Text => {
                        let len = data.len();
                        if very_verbose && len > 0 {
                            let text = std::str::from_utf8(&data[..len - 1]);
                            if let Ok(text) = text {
                                logger.log_debug(format!("Curl text: {}", text).as_str());
                            }
                        }
                    }
                    _ => {}
                })
                .unwrap();
            transfer
                .header_function(|h| {
                    if let Some(s) = decode_header(h, logger) {
                        if s.starts_with("HTTP/") {
                            status_lines.push(s);
                        } else {
                            response_headers.push(s)
                        }
                    }
                    true
                })
                .unwrap();

            transfer
                .write_function(|data| {
                    response_body.extend(data);
                    Ok(data.len())
                })
                .unwrap();

            if let Err(e) = transfer.perform() {
                let code = e.code() as i32; // due to windows build
                let description = match e.extra_description() {
                    None => e.description().to_string(),
                    Some(s) => s.to_string(),
                };
                return Err(HttpError::Libcurl {
                    code,
                    description,
                    url,
                });
            }
        }

        let status = self.handle.response_code().unwrap();
        // TODO: explain why status_lines is Vec ?
        let version = match status_lines.last() {
            None => return Err(HttpError::StatuslineIsMissing { url }),
            Some(status_line) => self.parse_response_version(status_line)?,
        };
        let headers = self.parse_response_headers(&response_headers);
        let _length = response_body.len();
        let certificate = if let Some(cert_info) = easy_ext::get_certinfo(&self.handle)? {
            match Certificate::try_from(cert_info) {
                Ok(value) => Some(value),
                Err(message) => {
                    logger.log_error(format!("can not parse certificate - {message}").as_str());
                    None
                }
            }
        } else {
            None
        };
        let stop = Utc::now();
        let duration = (stop - start).to_std().unwrap();
        let timings = Timings::new(&mut self.handle, start, stop);

        self.handle.reset();

        let request = self::request::Request {
            url: url.clone(),
            method: method.to_string(),
            headers: request_headers
                .iter()
                .map(|relynx_header| crate::client::request::Header {
                    name: relynx_header.key.to_string(),
                    value: relynx_header.value.to_string(),
                })
                .collect(),
            body: request_body,
        };
        let response = Response {
            version,
            status,
            headers,
            body: response_body,
            duration,
            url,
            certificate,
        };

        if verbose {
            // FIXME: the cast to u64 seems not necessary.
            //  If we dont cast from u128 and try to format! or println!
            //  we have a segfault on Alpine Docker images and Rust 1.68.0, whereas it was
            //  ok with Rust >= 1.67.0.
            // @TODO: let duration = duration.as_millis() as u64;
            // logger.debug_important(
            //     format!("Response: (received {length} bytes in {duration} ms)").as_str(),
            // );
            // logger.debug("");

            // // FIXME: Explain why there may be multiple status line
            // status_lines
            //     .iter()
            //     .filter(|s| s.starts_with("HTTP/"))
            //     .for_each(|s| logger.debug_status_version_in(s.trim()));

            // for header in &response.headers {
            //     logger.debug_header_in(&header.name, &header.value);
            // }
            // logger.info("<");
            // if very_verbose {
            //     response.log_body(logger);
            // }
        }

        Ok(Call {
            request,
            response,
            timings,
        })
    }

    /// Sets HTTP method.
    fn set_method(&mut self, method: &HttpMethod) {
        self.handle
            .custom_request(method.to_string().as_str())
            .unwrap()
    }

    /// Sets HTTP headers.
    fn set_headers(
        &mut self,
        request: &RequestModel,
        options: &ClientOptions,
        env: Option<&Environment>,
    ) {
        let mut list = easy::List::new();

        for header in &request.get_headers_with_env(env) {
            list.append(format!("{}: {}", header.key, header.value).as_str())
                .unwrap();
        }

        // @TODO: we do not store the content type somewhere else, only the header is present
        // @TODO: maybe create a helper function for that
        // if !request.has_header("Content-Type") {
        //     if let Some(ref s) = request.get_content_type() {
        //         list.append(format!("Content-Type: {s}").as_str()).unwrap();
        //     } else {
        //         // We remove default Content-Type headers added by curl because we want
        //         // to explicitly manage this header.
        //         // For instance, with --data option, curl will send a 'Content-type: application/x-www-form-urlencoded'
        //         // header.
        //         list.append("Content-Type:").unwrap();
        //     }
        // }

        // @TODO: does curl just insert an expect header? no other way to work around this?
        if request
            .get_header_values("Expect", GetHeadersOption::JustValues)
            .is_empty()
        {
            // We remove default Expect headers added by curl because we want
            // to explicitly manage this header.
            list.append("Expect:").unwrap(); // remove header Expect
        }

        // @TODO: do we need the user agent or can we just leave it be? does curl inject one themselves?
        if request
            .get_header_values("User-Agent", GetHeadersOption::JustValues)
            .is_empty()
        {
            let user_agent = match options.user_agent {
                Some(ref u) => u.clone(),
                None => "relynx".to_string(),
            };
            list.append(format!("User-Agent: {user_agent}").as_str())
                .unwrap();
        }

        if let Some(ref user) = options.user {
            let user = user.as_bytes();
            let authorization = general_purpose::STANDARD.encode(user);
            if request
                .get_header_values("Authorization", GetHeadersOption::JustValues)
                .is_empty()
            {
                list.append(format!("Authorization: Basic {authorization}").as_str())
                    .unwrap();
            }
        }
        if options.compressed
            && request
                .get_header_values("Accept-Encoding", GetHeadersOption::JustValues)
                .is_empty()
        {
            list.append("Accept-Encoding: gzip, deflate, br").unwrap();
        }

        self.handle.http_headers(list).unwrap();
    }

    /// Sets request cookies.
    fn set_cookies(&mut self, cookies: &[RequestCookie]) {
        let s = cookies
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("; ");
        if !s.is_empty() {
            self.handle.cookie(s.as_str()).unwrap();
        }
    }

    /// Sets form params.
    fn set_form_url_encoded(&mut self, params: &[UrlEncodedParam]) {
        if !params.is_empty() {
            let s = self.url_encode_params(params);
            self.handle.post_fields_copy(s.as_str().as_bytes()).unwrap();
            //self.handle.write_function(sink);
        }
    }

    /// Sets multipart form data.
    fn set_multipart(
        &mut self,
        _boundary: &str,
        parts: &[Multipart],
        logger: &RunLogger,
    ) -> Result<(), HttpError> {
        let mut form = easy::Form::new();
        for part in parts {
            let contents = match part.data {
                http_rest_file::model::DataSource::Raw(ref raw_data) => {
                    raw_data.as_bytes().to_vec()
                }
                http_rest_file::model::DataSource::FromFilepath(ref path) => {
                    std::fs::read(&std::path::PathBuf::from(path)).map_err(|err| {
                        logger.log_error(format!(
                            "Could not read of file: '{}'. Check if the file exists!",
                            path
                        ));
                        logger.log_error(format!("Error: {:?}", err));
                        HttpError::CouldNotReadBodyPartFromFile(PathBuf::from(path))
                    })?
                }
            };
            // @TODO error log
            let mut curl_part = form.part(&part.disposition.name);
            // @TODO: filename
            curl_part.contents(&contents);
            // @TODO: set multipart boundary yourself!

            if !part.headers.is_empty() {
                let mut list = easy::List::new();
                for header in part.headers.iter() {
                    list.append(format!("{}: {}", header.key, header.value).as_str())
                        .unwrap();
                }
                curl_part.content_header(list);
            }

            //@TODO what about this... .buffer(filename, data.clone())
            //                    .content_type(content_type)

            curl_part.add().map_err(|err| {
                log::error!("Could not add multipart form parameters");
                log::error!("Curl form error: {:?}", err);
                HttpError::FormError
            })?;
        }

        self.handle.httppost(form).unwrap();
        Ok(())
    }

    /// Sets request body.
    fn set_body(&mut self, data: &[u8]) {
        if !data.is_empty() {
            self.handle.post(true).unwrap();
            self.handle.post_field_size(data.len() as u64).unwrap();
        }
    }

    /// Sets SSL options
    fn set_ssl_options(&mut self, no_revoke: bool) {
        let mut ssl_opt = SslOpt::new();
        ssl_opt.no_revoke(no_revoke);
        self.handle.ssl_options(&ssl_opt).unwrap();
    }

    /// URL encodes parameters.
    fn url_encode_params(&mut self, params: &[UrlEncodedParam]) -> String {
        params
            .iter()
            .map(|p| {
                let value = self.handle.url_encode(p.value.as_bytes());
                format!("{}={}", p.key, value)
            })
            .collect::<Vec<String>>()
            .join("&")
    }

    /// Parses HTTP response version.
    fn parse_response_version(&mut self, line: &str) -> Result<HttpVersion, HttpError> {
        line.parse::<HttpVersion>().map_err(|err| {
            log::error!("Could not parse http response version");
            log::error!("{:?}", err);
            HttpError::CouldNotParseResponse
        })
    }

    /// Parse headers from libcurl responses.
    fn parse_response_headers(&mut self, lines: &[String]) -> Vec<Header> {
        let mut headers: Vec<Header> = vec![];
        for line in lines {
            if let Some(header) = Header::parse(line) {
                headers.push(header);
            }
        }
        headers
    }

    /// Retrieves an optional location to follow
    ///
    /// You need:
    /// 1. the option follow_location set to true
    /// 2. a 3xx response code
    /// 3. a header Location
    fn get_follow_location(&mut self, response: &Response, base_url: &str) -> Option<String> {
        let response_code = response.status;
        if !(300..400).contains(&response_code) {
            return None;
        }
        let location = match response
            .get_header_values("Location", GetHeadersOption::SplitMultiple)
            .get(0)
        {
            None => return None,
            Some(value) => get_redirect_url(value, base_url),
        };

        if location.is_empty() {
            None
        } else {
            Some(location)
        }
    }
}

/// Returns the redirect url.
fn get_redirect_url(location: &str, base_url: &str) -> String {
    if location.starts_with('/') {
        format!("{base_url}{location}")
    } else {
        location.to_string()
    }
}

/// Splits an array of bytes into HTTP lines (\r\n separator).
fn split_lines(data: &[u8]) -> Vec<String> {
    let mut lines = vec![];
    let mut start = 0;
    let mut i = 0;
    while i < (data.len() - 1) {
        if data[i] == 13 && data[i + 1] == 10 {
            if let Ok(s) = std::str::from_utf8(&data[start..i]) {
                lines.push(s.to_string());
            }
            start = i + 2;
            i += 2;
        } else {
            i += 1;
        }
    }
    lines
}

/// Decodes optionally header value as text with UTF-8 or ISO-8859-1 encoding.
pub fn decode_header(data: &[u8], logger: &RunLogger) -> Option<String> {
    match std::str::from_utf8(data) {
        Ok(s) => Some(s.to_string()),
        Err(_) => match ISO_8859_1.decode(data, DecoderTrap::Strict) {
            Ok(s) => Some(s),
            Err(_) => {
                logger.log_error(format!(
                    "Error decoding header both UTF-8 and ISO-8859-1 {:?}",
                    data
                ));
                None
            }
        },
    }
}

/// Converts a list of [`String`] to a libcurl's list of strings.
fn to_list(items: &[String]) -> List {
    let mut list = List::new();
    items.iter().for_each(|l| list.append(l).unwrap());
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        assert_eq!(
            Header::parse("Foo: Bar\r\n").unwrap(),
            Header::new("Foo", "Bar")
        );
        assert_eq!(
            Header::parse("Location: http://localhost:8000/redirected\r\n").unwrap(),
            Header::new("Location", "http://localhost:8000/redirected")
        );
        assert!(Header::parse("Foo").is_none());
    }

    #[test]
    fn test_split_lines_header() {
        let data = b"GET /hello HTTP/1.1\r\nHost: localhost:8000\r\n\r\n";
        let lines = split_lines(data);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines.get(0).unwrap().as_str(), "GET /hello HTTP/1.1");
        assert_eq!(lines.get(1).unwrap().as_str(), "Host: localhost:8000");
        assert_eq!(lines.get(2).unwrap().as_str(), "");
    }

    #[test]
    fn test_redirect_url() {
        assert_eq!(
            get_redirect_url("http://localhost:8000/redirected", "http://localhost:8000"),
            "http://localhost:8000/redirected".to_string()
        );
        assert_eq!(
            get_redirect_url("/redirected", "http://localhost:8000"),
            "http://localhost:8000/redirected".to_string()
        );
    }
}
