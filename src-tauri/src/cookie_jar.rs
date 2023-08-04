use std::{path::PathBuf, str::FromStr};

use crate::{
    client::client_model::Call,
    error::RelynxError,
    model::{Collection, Cookie, CookieJar, Environment, RequestModel, RunLogger}, import::RELYNX_IGNORE_FILE,
};
use http_rest_file::parser::Uri;
use serde::{Deserialize, Serialize};
pub type CollectionPath = PathBuf;

const IDEA_FOLDER_NAME: &str = ".idea";
const HTTP_REQ_FOLDER_NAME: &str = "httpRequests";
const COOKIE_JAR_FILENAME: &str = "http-client.cookies";

/// https://www.jetbrains.com/help/idea/http-client-in-product-code-editor.html#manage_cookies
/// There should be a `.idea/httpRequests`
fn find_idea_cookie_jar_path(collection_path: &CollectionPath) -> PathBuf {
    let mut current = Some(collection_path.clone());
    let mut idea_path: Option<PathBuf> = None;
    while current.is_some() {
        let check_path = current.as_ref().unwrap().join(IDEA_FOLDER_NAME);
        if check_path.exists() {
            idea_path = Some(check_path);
            break;
        }
        current = current.unwrap().parent().map(|p| p.to_path_buf());
    }
    let idea_path = idea_path.unwrap_or(collection_path.clone());

    idea_path
        .join(HTTP_REQ_FOLDER_NAME)
        .join(COOKIE_JAR_FILENAME)
}

#[derive(Serialize, Deserialize, rspc::Type, Debug, PartialEq)]
pub enum CookieJarPath {
    CollectionFolderPath(PathBuf),
    CookieJarFilePath(PathBuf),
}

#[derive(Serialize, Deserialize, rspc::Type, Debug, PartialEq)]
pub struct SaveCookieJarParams {
    pub cookie_jar: CookieJar,
    pub collection: Collection,
}

#[derive(Serialize, Deserialize, rspc::Type, Debug, PartialEq)]
pub struct GetCookieJarParams {
    pub collection: Collection,
}

pub fn load_cookie_jar(path: &CookieJarPath) -> Result<CookieJar, RelynxError> {
    let path = match path {
        CookieJarPath::CookieJarFilePath(path) => path.clone(),
        CookieJarPath::CollectionFolderPath(collection_path) => {
            find_idea_cookie_jar_path(collection_path)
        }
    };

    // ok if the path does not exist we do not load any cookies
    if !path.exists() {
        return Ok(CookieJar {
            path: None,
            cookies: Vec::new(),
        });
    }

    let content = std::fs::read_to_string(&path).map_err(|err| {
        log::error!(
            "Could not read cookie jar file to string, path: '{}'",
            path.display()
        );
        log::error!("Io Error: {:?}", err);
        RelynxError::LoadCookieJarError(path.to_string_lossy().to_string())
    })?;

    let mut cookies: Vec<Cookie> = Vec::new();

    // @TODO: parse each cookie
    // domain, path, name, value, date
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match parse_cookie_str(trimmed) {
            Ok(cookie) => {
                cookies.push(cookie);
            }
            // @TODO: display error / warning in ui
            Err(err) => {
                log::error!("Could not parse cookie on line {}", trimmed);
                log::error!("Cookie parse error: {:?}", err);
            }
        }
    }
    Ok(CookieJar {
        path: Some(path),
        cookies,
    })
}

pub fn save_cookie_jar(
    cookie_jar_path: CookieJarPath,
    cookie_jar: &CookieJar,
) -> Result<(), RelynxError> {
    let path = match cookie_jar_path {
        CookieJarPath::CookieJarFilePath(path) => path,
        CookieJarPath::CollectionFolderPath(collection_path) => {
            find_idea_cookie_jar_path(&collection_path)
        }
    };

    let cookie_dir = path
        .parent()
        .ok_or(RelynxError::SaveCookieJarErrorGeneric)?;

    if !cookie_dir.exists() {
        log::info!(
            "Cookie directory does not exist yet, creating:  '{}'",
            cookie_dir.display()
        );
        std::fs::create_dir_all(cookie_dir).map_err(|err| {
            log::error!("Could not save cookies to cookie jar as parent directory does not exist and could not be created.");
            log::error!("Cookie jar dir: '{}'", cookie_dir.display());
            log::error!("Io Error: {:?}", err);
            RelynxError::SaveCookieJarErrorGeneric
        })?;
        // also create ignore file
        let ignore_file = path.join(RELYNX_IGNORE_FILE);
        let _ = std::fs::write(ignore_file, "").map_err(|err| {
            log::error!("Could not create ignore file in new cookie jar folder");
            log::error!("Error: {:?}", err);
        });
    }

    let cookie_strings: Vec<String> = cookie_jar
        .cookies
        .iter()
        .map(|cookie| cookie.to_string())
        .collect();
    let content: String = cookie_strings.join("\n");
    std::fs::write(path, content).map_err(|err| {
        log::error!("Could not save cookie jar");
        log::error!("Io error: {:?}", err);
        RelynxError::SaveCookieJarErrorGeneric
    })
}

pub fn parse_cookie_str(str: &str) -> Result<Cookie, ()> {
    let mut parts = str.split(' ');
    let domain = parts.next().ok_or(())?.to_string();
    let path = parts.next().ok_or(())?.to_string();
    #[allow(clippy::map_flatten)]
    let (name, value) = parts
        .next()
        .map(|name_eq_value| {
            let mut parts = name_eq_value.split('=');
            let name = parts.next();
            let value = parts.next();
            if name.is_none() || value.is_none() {
                return None;
            }
            Some((name.unwrap().to_string(), value.unwrap().to_string()))
        })
        .flatten()
        .ok_or(())?;
    let expires: String = parts
        .map(str::to_string)
        .collect::<Vec<String>>()
        .join(" ");
    if expires.is_empty() {
        return Err(());
    }
    Ok(Cookie {
        domain,
        path,
        name,
        value,
        expires,
    })
}

pub fn save_cookies_to_jar(
    collection: &Collection,
    cookies: Vec<Cookie>,
    cookie_jar: &CookieJar,
) -> Result<CookieJar, RelynxError> {
    let mut cookie_jar = cookie_jar.clone();
    for cookie in cookies {
        let position = cookie_jar.cookies.iter().position(|current| {
            cookie.domain == current.domain
                && cookie.path == current.path
                && cookie.name == current.name
        });

        if let Some(position) = position {
            cookie_jar.cookies[position] = cookie;
        } else {
            cookie_jar.cookies.push(cookie);
        };
    }

    let path = cookie_jar
        .path
        .clone()
        .unwrap_or_else(|| find_idea_cookie_jar_path(&collection.path));
    save_cookie_jar(CookieJarPath::CookieJarFilePath(path), &cookie_jar).map_err(|_err| {
        log::error!("Could not update new cookies to cookie jar in 'save cookies to jar'");
        RelynxError::UpdateCookieJarError
    })?;

    Ok(cookie_jar)
}

pub fn update_cookie_jar(
    collection: &Collection,
    cookie_jar: &CookieJar,
    calls: &[Call],
) -> Result<(), RelynxError> {
    let mut errored = false;
    calls.iter().for_each(|call| {
        let cookies: Vec<crate::model::Cookie> = call
            .response
            .headers
            .iter()
            .filter(|header| header.key.to_lowercase() == "set-cookie")
            .filter_map(|set_cookie_header| {
                cookie::Cookie::parse(set_cookie_header.value.to_string())
                    .map(Into::<crate::model::Cookie>::into)
                    .ok()
            })
            .collect();
        if !cookies.is_empty() {
            let _ = save_cookies_to_jar(collection, cookies, cookie_jar).map_err(|err| {
                log::error!("Could not save cookie to jar!");
                errored = true;
                err
            });
        }
    });
    if errored {
        Err(RelynxError::UpdateCookieJarError)
    } else {
        Ok(())
    }
}

pub fn update_request_with_cookie_jar(
    request: &mut RequestModel,
    cookie_jar: &CookieJar,
    environment: Option<&Environment>,
    logger: &RunLogger,
) -> Result<(), RelynxError> {
    let url = request.get_url_with_env(true, environment);
    let parsed = Uri::from_str(&url).map_err(|err| {
        logger.log_error(format!(
            "Could not parse url: '{}' when trying to set cookie jar cookies on request",
            url
        ));
        logger.log_error(format!("Parse error {:?}", err));
        logger.log_error(format!("Request: {:?}", request));
        RelynxError::UpdateRequestWithCookieJar
    })?;
    let cookies: Vec<String> = cookie_jar
        .cookies
        .iter()
        .filter(|cookie| {
            cookie.path == parsed.path() && cookie.domain == parsed.host().unwrap_or("")
        })
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect();
    let cookies = cookies.join(";");
    if let Some(cookie_header) = request
        .headers
        .iter_mut()
        .find(|header| header.key.to_lowercase() == "cookie")
    {
        cookie_header.value.push(';');
        cookie_header.value.push_str(&cookies);
    } else {
        request.headers.push(crate::model::Header {
            key: "Cookie".to_string(),
            value: cookies,
            active: true,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{cookie_jar::parse_cookie_str, model::Cookie};

    #[test]
    pub fn test_parse_cookie_str() {
        let cookie_string = "testibus.com /test name=value Wed, 02 Aug 2023 20:32:00";
        let cookie = parse_cookie_str(cookie_string).expect("cookie is parsed");
        assert_eq!(
            Into::<Cookie>::into(cookie.clone()),
            Cookie {
                domain: "testibus.com".to_string(),
                path: "/test".to_string(),
                name: "name".to_string(),
                value: "value".to_string(),
                expires: "Wed, 02 Aug 2023 20:32:00".to_string(),
            },
        );
        assert_eq!(cookie.to_string(), cookie_string);
    }
}
