use std::{collections::HashMap, path::PathBuf};

use http_rest_file::model::{
    DataSource, DispositionField, Header as HttpRestFileHeader, HttpMethod, HttpRestFile,
    HttpRestFileExtension, HttpVersion, Multipart as HttpRestfileMultipart, Request,
    RequestBody as HttpRestFileBody, RequestLine, RequestSettings, SaveResponse, UrlEncodedParam,
    WithDefault,
};

use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Type, Default, Debug)]
pub struct Workspace {
    pub collections: Vec<Collection>,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub enum AppEnvironment {
    DEVELOPMENT,
    PRODUCTION,
}

type ISO8601 = String;

#[derive(Serialize, Deserialize, Type, Default, Debug)]
pub struct LicenseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_signature: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_start: Option<ISO8601>,
}

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub path: PathBuf,
    pub current_env_name: String,
    pub description: String,
    pub import_warnings: Vec<ImportWarning>,
    #[serde(default = "default_path_exists")]
    pub path_exists: bool,
}

fn default_path_exists() -> bool {
    true
}

impl Collection {
    pub fn get_config_file_path(&self) -> PathBuf {
        Collection::config_file_path(&PathBuf::from(&self.path))
    }

    pub fn config_file_path(collection_folder_path: &PathBuf) -> PathBuf {
        collection_folder_path.join(COLLECTION_CONFIGFILE)
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub enum MessageSeverity {
    #[serde(rename = "warn")]
    INFO,

    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "success")]
    SUCCESS,

    #[serde(rename = "error")]
    ERROR,
}

// @TODO
#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct ImportWarning {
    pub rest_file_path: String,
    pub is_group: bool,
    pub message: Option<String>,
    pub severity: Option<MessageSeverity>,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct AddCollectionsResult {
    pub workspace: Workspace,
    pub num_imported: u32,
    pub errored_collections: Vec<PathBuf>, // @TODO
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ImportCollectionResult {
    pub collection: Collection,
    // @TODO: environment
    // @TODO: requestTree
    // @TODO: collectionConfig
}

// order of children within a collection cannot be saved as they are just files/folders in a file
// system which does not have an order. It maps paths to the respective order *within* a parent
// this is only for requests or groups within a group. For filegroups the order is dependent on the
// order of the requests within a file
pub type PathOrder = HashMap<PathBuf, u32>;
pub type EnvName = String;
pub type EnvVarDescriptions = HashMap<EnvName, Vec<SingleEnvVarDescription>>;

#[derive(Serialize, Deserialize, Type, Default, Debug)]
pub struct CollectionConfig {
    pub name: String,
    pub path_orders: PathOrder,
    pub env_var_descriptions: EnvVarDescriptions,
}

pub type Uuid = String;

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub struct QueryParam {
    key: String,
    value: String,
    active: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RequestFileModel {
    pub id: Uuid,
    pub path: String,
    pub requests: Vec<RequestModel>,
}

impl From<HttpRestFile> for RequestFileModel {
    fn from(value: HttpRestFile) -> Self {
        RequestFileModel {
            id: uuid::Uuid::new_v4().to_string(),
            path: value.path.to_string_lossy().to_string(),
            requests: value
                .requests
                .into_iter()
                .map(|request| request_to_request_model(request, value.path.as_ref()))
                .collect::<Vec<RequestModel>>(),
        }
    }
}

impl From<RequestFileModel> for HttpRestFile {
    fn from(value: RequestFileModel) -> Self {
        From::<&RequestFileModel>::from(&value)
    }
}

impl From<&RequestFileModel> for HttpRestFile {
    fn from(value: &RequestFileModel) -> Self {
        HttpRestFile {
            requests: value
                .requests
                .iter()
                .map(Into::into)
                .collect::<Vec<Request>>(),
            errs: vec![],
            path: Box::new(std::path::PathBuf::from(value.path.clone())),
            extension: Some(HttpRestFileExtension::Http),
        }
    }
}

pub fn query_params_from_url(url: &str) -> Vec<QueryParam> {
    if !url.contains('?') {
        return vec![];
    }

    let mut split = url.split("?");
    let (_, query) = (split.next(), split.next());
    if query.is_none() {
        return vec![];
    }
    let parts = query.unwrap().split("&");
    return parts
        .into_iter()
        .map(|part| {
            let mut part_split = part.split("=");
            let (key, value) = (
                part_split.next().map(ToString::to_string),
                part_split.next().map(ToString::to_string),
            );
            return QueryParam {
                key: key.unwrap_or_default(),
                value: value.unwrap_or_default(),
                active: true,
            };
        })
        .collect::<Vec<QueryParam>>();
}

pub fn request_to_request_model(
    value: http_rest_file::model::Request,
    path: &std::path::PathBuf,
) -> RequestModel {
    let redirect_response = match value.save_response {
        Some(SaveResponse::NewFileIfExists(ref path)) => RedirectResponse {
            save_response: true,
            save_path: Some(path.clone()),
            overwrite: false,
        },
        Some(SaveResponse::RewriteFile(ref path)) => RedirectResponse {
            save_response: true,
            save_path: Some(path.clone()),
            overwrite: true,
        },
        None => RedirectResponse {
            save_response: false,
            save_path: None,
            overwrite: true,
        },
    };

    RequestModel {
        id: uuid::Uuid::new_v4().to_string(),
        name: value.name.clone().unwrap_or(String::new()),
        description: value.get_comment_text().unwrap_or(String::new()),
        method: value.request_line.method.unwrap_or_default(),
        http_version: value.request_line.http_version.into(),
        url: value.request_line.target.to_string(),
        rest_file_path: path.to_owned(),
        body: (&value.body).into(),
        query_params: query_params_from_url(&value.request_line.target.to_string()),
        headers: value.headers.iter().map(Into::into).collect(),
        settings: value.settings,
        redirect_response,
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub active: bool,
}

//@TODO: put this into http_header
impl Header {
    /// Parses an HTTP header line received from the server
    /// It does not panic. Just returns `None` if it can not be parsed.
    pub fn parse(line: &str) -> Option<Header> {
        match line.find(':') {
            Some(index) => {
                let (name, value) = line.split_at(index);
                Some(Header::new(name.trim(), value[1..].trim()))
            }
            None => None,
        }
    }
}

impl Header {
    pub fn new<S, T>(key: S, value: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        Header {
            key: key.into(),
            value: value.into(),
            active: true,
        }
    }
}

impl From<&HttpRestFileHeader> for Header {
    fn from(value: &HttpRestFileHeader) -> Self {
        Header {
            key: value.key.clone(),
            value: value.value.clone(),
            active: true,
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub enum RequestBody {
    None,
    Multipart {
        boundary: String,
        parts: Vec<Multipart>,
    },
    UrlEncoded {
        url_encoded_params: Vec<UrlEncodedParam>,
    },
    //@TODO
    Raw {
        data: DataSource<String>,
    },
}

impl RequestBody {
    pub fn is_none(&self) -> bool {
        return matches!(self, RequestBody::None);
    }
    pub fn is_multipart(&self) -> bool {
        return matches!(self, RequestBody::Multipart { .. });
    }
    pub fn is_url_encoded(&self) -> bool {
        return matches!(self, RequestBody::UrlEncoded { .. });
    }
    pub fn is_raw(&self) -> bool {
        return matches!(self, RequestBody::Raw { .. });
    }
}

impl From<&HttpRestFileBody> for RequestBody {
    fn from(value: &HttpRestFileBody) -> Self {
        match value {
            HttpRestFileBody::None => RequestBody::None,
            HttpRestFileBody::Multipart { boundary, parts } => RequestBody::Multipart {
                boundary: boundary.clone(),
                parts: parts.iter().map(Into::into).collect(),
            },
            HttpRestFileBody::UrlEncoded { url_encoded_params } => RequestBody::UrlEncoded {
                url_encoded_params: url_encoded_params.clone(),
            },
            HttpRestFileBody::Raw { data } => RequestBody::Raw { data: data.clone() },
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub struct Multipart {
    pub data: DataSource<String>,
    pub disposition: DispositionField,
    pub headers: Vec<Header>,
}

impl From<&HttpRestfileMultipart> for Multipart {
    fn from(value: &HttpRestfileMultipart) -> Self {
        Multipart {
            data: value.data.clone(),
            disposition: value.disposition.clone(),
            headers: value.headers.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub struct RedirectResponse {
    // save response result to file or not?
    pub save_response: bool,
    // if the response is saved in which path
    pub save_path: Option<PathBuf>,
    // if the respones is saved and the file exists already overwrite or create new files?
    pub overwrite: bool,
}

impl RedirectResponse {
    pub fn no_save() -> Self {
        RedirectResponse {
            save_response: false,
            save_path: None,
            overwrite: true,
        }
    }

    pub fn get_absolute_path(&self, request: &RequestModel) -> Option<PathBuf> {
        let request_path = PathBuf::from(&request.rest_file_path);
        let request_folder = request_path.parent()?;
        let save_path = self.save_path.as_ref()?;
        Some(request_folder.join(save_path))
    }
}

impl Default for RedirectResponse {
    fn default() -> Self {
        RedirectResponse {
            save_response: false,
            save_path: None,
            overwrite: true,
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub struct RequestModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub method: HttpMethod,
    pub url: String,
    pub query_params: Vec<QueryParam>,
    pub headers: Vec<Header>,
    pub body: RequestBody,
    pub rest_file_path: PathBuf,
    pub http_version: Replaced<HttpVersion>,
    pub settings: RequestSettings,
    pub redirect_response: RedirectResponse,
}

const DEFAULT_HTTP_EXTENSION: &str = "http";

impl Default for RequestModel {
    fn default() -> Self {
        RequestModel {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Request".to_string(),
            description: String::new(),
            method: HttpMethod::default(),
            url: String::new(),
            query_params: vec![],
            headers: vec![],
            body: RequestBody::None,
            rest_file_path: PathBuf::new(),
            http_version: Replaced {
                value: HttpVersion::default(),
                is_replaced: true,
            },
            settings: RequestSettings::default(),
            redirect_response: RedirectResponse::default(),
        }
    }
}

pub enum GetHeadersOption {
    /// Returns each header value as it is, if it contains multiple values within one header
    /// comma separated such as 'SomeHeader: value1,value2,value3' the value is returned as it is
    /// 'value1,value2,value3'
    JustValues,
    /// If a header value contains multiple comma separated values, split them and return each as
    /// its own header 'value1,value2,value3' -> ['value1', 'value2', 'value3']
    SplitMultiple,
}

impl RequestModel {
    pub fn get_url_with_env(
        &self,
        remove_inactive_params: bool,
        env: Option<&Environment>,
    ) -> String {
        if env.is_none() {
            return self.url.clone();
        }
        let env = env.unwrap();
        let url = env.replace_values_in_str(&self.url);
        println!("url: {:?}, replaced: {:?}", self.url, url);

        let host_header = self.get_header_values("Host", GetHeadersOption::JustValues);
        let mut url_with_host: Option<String> = None;
        if !host_header.is_empty() {
            url_with_host = Some(host_header[0].to_string() + &url);
        }

        let url: Result<Url, ()> = match Url::parse(&url) {
            Ok(parsed_url) => Ok(parsed_url),
            Err(_err) => {
                if let Some(ref url_with_host) = url_with_host {
                    Url::parse(url_with_host).map_err(|_err| ())
                } else {
                    Err(())
                }
            }
        };
        if url.is_err() {
            return url_with_host.unwrap_or(self.url.clone());
        }

        let mut url = url.unwrap();

        if remove_inactive_params {
            if self.query_params.is_empty() {
                url.set_query(None);
            } else {
                let query = self.get_query_string_with_env(Some(env));
                url.set_query(Some(&query));
            }
        }

        url.to_string()
    }

    pub fn get_query_params_with_env(&self, env: Option<&Environment>) -> Vec<QueryParam> {
        if env.is_none() {
            return self.query_params.clone();
        }
        let env = env.unwrap();
        self.query_params
            .iter()
            .map(|query_param| {
                let mut new_param = query_param.clone();
                new_param.value = env.replace_values_in_str(&new_param.value);
                new_param
            })
            .collect()
    }

    pub fn get_query_string_with_env(&self, env: Option<&Environment>) -> String {
        let params = self.get_query_params_with_env(env);
        params
            .iter()
            .map(|param| format!("{}={}", param.key, param.value))
            .collect::<Vec<String>>()
            .join("&")
    }

    pub fn get_headers_with_env(&self, env: Option<&Environment>) -> Vec<Header> {
        if env.is_none() {
            return self.headers.clone();
        }
        let env = env.unwrap();
        self.headers
            .iter()
            .map(|header| {
                let mut new_header = header.clone();
                new_header.value = env.replace_values_in_str(&new_header.value);
                new_header
            })
            .collect()
    }

    pub fn get_url_encoded_params_with_env(
        &self,
        env: Option<&Environment>,
    ) -> Option<Vec<UrlEncodedParam>> {
        if let RequestBody::UrlEncoded {
            ref url_encoded_params,
        } = self.body
        {
            if env.is_none() {
                Some(
                    url_encoded_params
                        .iter()
                        .map(|param| param.clone())
                        .collect(),
                )
            } else {
                let env = env.unwrap();
                Some(
                    url_encoded_params
                        .iter()
                        .map(|param| {
                            let mut new_param = param.clone();
                            new_param.value = env.replace_values_in_str(&new_param.value);
                            new_param
                        })
                        .collect(),
                )
            }
        } else {
            None
        }
    }

    pub fn get_request_file_path(&self, parent_path: &str) -> String {
        let parent_path = std::path::Path::new(parent_path);
        let previous_path = std::path::PathBuf::from(&self.rest_file_path);
        let file_name = previous_path.file_name().unwrap();
        let path = parent_path
            .join(std::path::Path::new(&self.rest_file_path))
            .join(file_name);
        path.to_string_lossy().to_string()
    }

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

    pub fn has_header(&self, key: &str) -> bool {
        let key_lowercase = key.to_lowercase();
        self.headers
            .iter()
            .any(|header| header.key.to_lowercase() == key_lowercase)
    }

    pub fn get_request_file_name(&self) -> String {
        let path = PathBuf::from(&self.rest_file_path);
        return path.file_name().unwrap().to_string_lossy().to_string();
    }

    pub fn create_request_path(
        request_name: &str,
        parent_path: std::path::PathBuf,
    ) -> std::path::PathBuf {
        let file_name = sanitize_filename_with_options(request_name, DEFAULT_OPTIONS);
        let mut result = parent_path.join(file_name);
        result.set_extension(DEFAULT_HTTP_EXTENSION);
        result
    }
    pub fn new(name: String, path: &std::path::Path) -> Self {
        RequestModel {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            method: HttpMethod::GET,
            url: String::new(),
            query_params: vec![],
            headers: vec![],
            body: RequestBody::None,
            rest_file_path: path.to_owned(),
            http_version: Replaced {
                value: HttpVersion::default(),
                is_replaced: true,
            },
            settings: RequestSettings::default(),
            redirect_response: RedirectResponse::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub struct Replaced<T> {
    pub value: T,
    pub is_replaced: bool,
}

impl<T> From<WithDefault<T>> for Replaced<T> {
    fn from(value: WithDefault<T>) -> Self {
        match value {
            WithDefault::Some(value) => Replaced {
                value,
                is_replaced: false,
            },
            WithDefault::Default(value) => Replaced {
                value,
                is_replaced: true,
            },
            WithDefault::DefaultFn(default_fn) => Replaced {
                value: default_fn(),
                is_replaced: true,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct EnvironmentVariable {
    pub name: String,
    pub initial_value: String,
    pub current_value: Option<String>,
    pub description: Option<String>,
}

impl EnvironmentVariable {
    pub fn new<S, T>(name: S, initial_value: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        EnvironmentVariable {
            name: name.into(),
            initial_value: initial_value.into(),
            current_value: None,
            description: None,
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct EnvironmentSecret {
    pub name: String,
    pub initial_value: String,
    pub current_value: Option<String>,
    pub description: Option<String>,
    pub persist_to_file: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct SingleEnvVarDescription {
    pub env_var_name: String,
    pub description: String,
    pub is_secret: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Environment {
    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
    pub secrets: Vec<EnvironmentSecret>,
}

impl Environment {
    pub fn new(name: String) -> Self {
        Environment {
            name,
            variables: vec![],
            secrets: vec![],
        }
    }

    pub fn replace_values_in_str(&self, str: &str) -> String {
        let mut result = str.to_string();
        for variable in &self.variables {
            let replace_key = "{{".to_string() + &variable.name + "}}";
            if result.contains(&replace_key) {
                let value = variable
                    .current_value
                    .as_ref()
                    .unwrap_or(&variable.initial_value);
                result = result.replace(&replace_key, value);
            }
        }

        for secret in &self.secrets {
            let replace_key = format!("{{{}}}", secret.name);
            if result.contains(&replace_key) {
                let value = secret
                    .current_value
                    .as_ref()
                    .unwrap_or(&secret.initial_value);
                result = result.replace(&replace_key, value);
            }
        }
        result
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RunRequestCommand {
    pub request: RequestModel,
    pub environment: Option<Environment>,
}

pub type ContentType = String;
pub type StatusCode = String;

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RequestResult {
    pub id: Uuid, // internal id, not from request itself
    pub result: String,
    pub status_code: StatusCode,
    pub total_time: f64,
    pub total_result_size: f64,
    pub content_type: Option<ContentType>,
    pub warnings: Vec<String>,
    pub result_file: Option<PathBuf>,
    pub result_file_folder: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct SaveRequestCommand {
    pub requests: Vec<RequestModel>,
    pub collection: Collection,
    pub old_name: String,
}

impl From<&Header> for http_rest_file::model::Header {
    fn from(value: &Header) -> Self {
        http_rest_file::model::Header {
            value: value.value.clone(),
            key: value.key.clone(),
        }
    }
}
impl From<&Multipart> for http_rest_file::model::Multipart {
    fn from(value: &Multipart) -> Self {
        http_rest_file::model::Multipart {
            data: value.data.clone(),
            headers: value.headers.iter().map(Into::into).collect(),
            disposition: value.disposition.clone(),
        }
    }
}
use http_rest_file::model::RequestBody as RestFileBody;
use url::Url;

use crate::{
    config::COLLECTION_CONFIGFILE, sanitize::sanitize_filename_with_options, tree::DEFAULT_OPTIONS,
};
impl From<RequestBody> for http_rest_file::model::RequestBody {
    fn from(value: RequestBody) -> Self {
        match value {
            RequestBody::None => RestFileBody::None,
            RequestBody::Raw { data } => RestFileBody::Raw { data },
            RequestBody::UrlEncoded { url_encoded_params } => {
                RestFileBody::UrlEncoded { url_encoded_params }
            }
            RequestBody::Multipart { boundary, parts } => RestFileBody::Multipart {
                boundary,
                parts: parts.iter().map(Into::into).collect(),
            },
        }
    }
}

impl From<RequestModel> for http_rest_file::model::Request {
    fn from(value: RequestModel) -> Self {
        From::<&RequestModel>::from(&value)
    }
}
impl From<&RequestModel> for http_rest_file::model::Request {
    fn from(value: &RequestModel) -> Self {
        let http_version = match value.http_version.clone() {
            Replaced {
                value,
                is_replaced: false,
            } => WithDefault::Some(value),
            Replaced {
                value,
                is_replaced: true,
            } => WithDefault::Default(value),
        };
        let comments: Vec<http_rest_file::model::Comment> = match &value.description[..] {
            "" => vec![],
            description => description
                .split('\n')
                .map(|str| http_rest_file::model::Comment {
                    kind: http_rest_file::model::CommentKind::DoubleSlash,
                    value: str.to_string(),
                })
                .collect(),
        };
        let target = value.url.as_str().into();

        let save_response = match value.redirect_response {
            RedirectResponse {
                save_response: false,
                ..
            } => None,
            RedirectResponse {
                ref save_path,
                overwrite,
                ..
            } => {
                if overwrite {
                    Some(SaveResponse::RewriteFile(
                        save_path.clone().unwrap_or(PathBuf::new()),
                    ))
                } else {
                    Some(SaveResponse::NewFileIfExists(
                        save_path.clone().unwrap_or(PathBuf::new()),
                    ))
                }
            }
        };
        // filter out headers which have no key
        let headers: Vec<http_rest_file::model::Header> = value
            .headers
            .iter()
            .filter(|header| !header.key.is_empty())
            .map(Into::into)
            .collect();
        http_rest_file::model::Request {
            name: Some(value.name.clone()),
            request_line: RequestLine {
                method: WithDefault::Some(value.method.clone()),
                http_version,
                target,
            },
            body: value.body.clone().into(),
            headers,
            comments,
            settings: value.settings.clone(),
            pre_request_script: None,
            response_handler: None,
            save_response,
        }
    }
}
