use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use cookie::{time::format_description, Expiration};
use http_rest_file::model::{
    DispositionField, Header as HttpRestFileHeader, HttpMethod, HttpRestFile,
    HttpRestFileExtension, HttpVersion, Multipart as HttpRestfileMultipart, PreRequestScript,
    Request, RequestBody as HttpRestFileBody, RequestLine, RequestSettings, ResponseHandler,
    SaveResponse as RestFileSaveResponse, UrlEncodedParam, WithDefault,
};
use rspc::Type;
use serde::{Deserialize, Serialize};
use typed_path::{UnixEncoding, UnixPathBuf, WindowsEncoding};

#[derive(Serialize, Deserialize, Type, Default, Debug)]
pub struct Workspace {
    pub collections: Vec<Collection>,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub enum AppEnvironment {
    Development,
    Production,
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq)]
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

    pub fn config_file_path(collection_folder_path: &Path) -> PathBuf {
        collection_folder_path.join(COLLECTION_CONFIGFILE)
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq)]
pub enum MessageSeverity {
    #[serde(rename = "warn")]
    Info,

    #[serde(rename = "warn")]
    Warn,

    #[serde(rename = "success")]
    Success,

    #[serde(rename = "error")]
    Error,
}

// @TODO
#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq)]
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
    pub errored_collections: Vec<PathBuf>,
    pub collection_names: Vec<String>, // all added collections
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

    let mut split = url.split('?');
    let (_, query) = (split.next(), split.next());
    if query.is_none() {
        return vec![];
    }
    let parts = query.unwrap().split('&');
    parts
        .into_iter()
        .map(|part| {
            let mut part_split = part.split('=');
            let (key, value) = (
                part_split.next().map(ToString::to_string),
                part_split.next().map(ToString::to_string),
            );
            QueryParam {
                key: key.unwrap_or_default(),
                value: value.unwrap_or_default(),
                active: true,
            }
        })
        .collect::<Vec<QueryParam>>()
}

pub fn request_to_request_model(
    value: http_rest_file::model::Request,
    path: &std::path::PathBuf,
) -> RequestModel {
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
        save_response: value.save_response.map(Into::<SaveResponse>::into),
        pre_request_script: value.pre_request_script,
        response_handler: value.response_handler,
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub active: bool,
}

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

    pub fn content_type_multipart(boundary: &str) -> Self {
        Header {
            key: "Content-Type".to_string(),
            value: format!("multipart/form-data; boundary=\"{}\"", boundary),
            active: true,
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
    Raw {
        data: DataSource<String>,
    },
}

impl RequestBody {
    pub fn is_none(&self) -> bool {
        matches!(self, RequestBody::None)
    }
    pub fn is_multipart(&self) -> bool {
        matches!(self, RequestBody::Multipart { .. })
    }
    pub fn is_url_encoded(&self) -> bool {
        matches!(self, RequestBody::UrlEncoded { .. })
    }
    pub fn is_raw(&self) -> bool {
        matches!(self, RequestBody::Raw { .. })
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
            HttpRestFileBody::Raw { data } => RequestBody::Raw {
                data: data.clone().into(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Type, Serialize, Deserialize)]
pub enum DataSource<T> {
    Raw(T),
    FromFilepath(String),
}

impl ToString for DataSource<String> {
    fn to_string(&self) -> String {
        match self {
            Self::Raw(str) => str.to_string(),
            Self::FromFilepath(path) => format!("< {}", path),
        }
    }
}

impl DataSource<String> {
    pub fn get_abs_path_relative_to(&self, request: &RequestModel) -> Option<PathBuf> {
        let request_folder = request.rest_file_path.parent()?;
        return self.get_abs_path(request_folder);
    }
    pub fn get_abs_path(&self, base: &Path) -> Option<PathBuf> {
        if let DataSource::FromFilepath(ref pathstr) = self {
            let mut abs_path = PathBuf::from(pathstr);
            if abs_path.is_relative() {
                abs_path = base.join(abs_path);
            }
            // workaround, if the path does not exist try a
            if !abs_path.exists() {
                abs_path = base.to_owned();
                if cfg!(windows) {
                    // we are on windows, so the request path is a regular windows path but the
                    // datasources filepath may be from linux
                    let unix_path = UnixPathBuf::from(pathstr);
                    let path_parts: Vec<String> = unix_path
                        .components()
                        .map(|c| {
                            c.as_path::<WindowsEncoding>()
                                .to_str()
                                .unwrap_or("")
                                .to_string()
                        })
                        .collect();
                    abs_path.extend(path_parts.into_iter());
                } else if cfg!(unix) {
                    // we are on linux, so the request path is a regular linux path but the
                    // datasources filepath may be from windows
                    let win_path = typed_path::PathBuf::<WindowsEncoding>::from(pathstr);
                    let path_parts: Vec<String> = win_path
                        .components()
                        .map(|c| {
                            let part = c
                                .as_path::<UnixEncoding>()
                                .to_str()
                                .unwrap_or("")
                                .to_string();
                            if part.starts_with("\"") && part.ends_with("\"") {
                                part[1..(part.len() - 2)].to_string()
                            } else {
                                part
                            }
                        })
                        .collect();
                    abs_path.extend(path_parts.into_iter());
                }
            }
            Some(dbg!(abs_path))
        } else {
            None
        }
    }
}

impl<T> From<http_rest_file::model::DataSource<T>> for DataSource<T> {
    fn from(value: http_rest_file::model::DataSource<T>) -> Self {
        match value {
            http_rest_file::model::DataSource::FromFilepath(filepath) => {
                Self::FromFilepath(filepath)
            }
            http_rest_file::model::DataSource::Raw(raw) => Self::Raw(raw),
        }
    }
}

impl<T> From<DataSource<T>> for http_rest_file::model::DataSource<T> {
    fn from(value: DataSource<T>) -> Self {
        match value {
            DataSource::Raw(raw) => http_rest_file::model::DataSource::Raw(raw),
            DataSource::FromFilepath(filepath) => {
                http_rest_file::model::DataSource::FromFilepath(filepath)
            }
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
            data: value.data.clone().into(),
            disposition: value.disposition.clone(),
            headers: value.headers.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, PartialEq, Eq)]
pub enum SaveResponse {
    // save the response into a new file if there exists already an existing save (use incremental
    // numbering for filename)
    NewFileIfExists(std::path::PathBuf),
    // save the response to a file and overwrite it if present
    RewriteFile(std::path::PathBuf),
}

impl SaveResponse {
    pub fn get_path(&self) -> &PathBuf {
        match self {
            SaveResponse::RewriteFile(path) => path,
            SaveResponse::NewFileIfExists(path) => path,
        }
    }
    pub fn is_path_empty(&self) -> bool {
        self.get_path().to_string_lossy().is_empty()
    }

    pub fn get_absolute_path(&self, request: &RequestModel) -> Option<PathBuf> {
        let path = self.get_path();
        let request_path = PathBuf::from(&request.rest_file_path);
        let request_folder = request_path.parent()?;
        if path.is_absolute() {
            Some(path.clone())
        } else {
            Some(request_folder.join(path))
        }
    }
}

impl From<RestFileSaveResponse> for SaveResponse {
    fn from(value: RestFileSaveResponse) -> Self {
        match value {
            RestFileSaveResponse::RewriteFile(path) => SaveResponse::RewriteFile(path),
            RestFileSaveResponse::NewFileIfExists(path) => SaveResponse::NewFileIfExists(path),
        }
    }
}

impl From<SaveResponse> for RestFileSaveResponse {
    fn from(value: SaveResponse) -> Self {
        match value {
            SaveResponse::RewriteFile(path) => RestFileSaveResponse::RewriteFile(path),
            SaveResponse::NewFileIfExists(path) => RestFileSaveResponse::NewFileIfExists(path),
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
    pub save_response: Option<SaveResponse>,
    pub pre_request_script: Option<PreRequestScript>,
    pub response_handler: Option<ResponseHandler>,
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
            save_response: None,
            pre_request_script: None,
            response_handler: None,
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
            if let Some(env) = env {
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
            } else {
                Some(url_encoded_params.clone())
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
            save_response: None,
            pre_request_script: None,
            response_handler: None,
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
    pub collection: Collection,
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
            data: value.data.clone().into(),
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
            RequestBody::Raw { data } => RestFileBody::Raw { data: data.into() },
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
            pre_request_script: value.pre_request_script.clone(),
            response_handler: value.response_handler.clone(),
            save_response: value
                .save_response
                .clone()
                .map(Into::<RestFileSaveResponse>::into),
        }
    }
}

pub struct RunLogger {
    no_log: bool,
}

impl RunLogger {
    pub fn new(no_log: bool) -> Self {
        log::warn!("Not logging current request as no_log is set!");
        RunLogger { no_log }
    }

    pub fn log_error<S: AsRef<str>>(&self, msg: S) {
        if self.no_log {
            return;
        }
        log::error!("{}", msg.as_ref());
    }

    pub fn log_info<S: AsRef<str>>(&self, msg: S) {
        if self.no_log {
            return;
        }
        log::info!("{}", msg.as_ref());
    }

    pub fn log_debug<S: AsRef<str>>(&self, msg: S) {
        if self.no_log {
            return;
        }
        log::debug!("{}", msg.as_ref());
    }
}

#[derive(Serialize, Deserialize, Type, Debug, PartialEq, Clone)]
pub struct CookieJar {
    pub path: Option<PathBuf>,
    pub cookies: Vec<Cookie>,
}

#[derive(Serialize, Deserialize, Type, Debug, PartialEq, Clone)]
pub struct Cookie {
    pub domain: String,
    pub path: String,
    pub name: String,
    pub value: String,
    pub expires: String,
}

impl<'c> From<cookie::Cookie<'c>> for Cookie {
    fn from(value: cookie::Cookie) -> Self {
        let binding = format_description::parse("%a, %d %b %Y %H:%M:%S GMT")
            .expect("valid format description");
        let format = binding.first().expect("only one format");
        Self {
            domain: value
                .domain()
                .map(|str| str.to_string())
                .unwrap_or_default(),
            name: value.name().to_string(),
            value: value.value().to_string(),
            path: value.path().map(|str| str.to_string()).unwrap_or_default(),
            expires: value
                .expires()
                .map(|expiration: Expiration| {
                    if let Expiration::DateTime(expiration_date) = expiration {
                        expiration_date.format(format).unwrap_or_default()
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default(),
        }
    }
}

impl ToString for Cookie {
    fn to_string(&self) -> String {
        format!(
            "{} {} {}={} {}",
            self.domain, self.path, self.name, self.value, self.expires
        )
    }
}
