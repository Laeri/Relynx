use std::{collections::HashMap, path::PathBuf};

use http_rest_file::model::{
    DataSource, DispositionField, Header as HttpRestFileHeader, HttpMethod, HttpRestFile,
    HttpRestFileExtension, HttpVersion, Multipart as HttpRestfileMultipart, Request,
    RequestBody as HttpRestFileBody, RequestLine, RequestSettings, WithDefault,
};
use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Workspace {
    pub collections: Vec<Collection>,
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            collections: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Collection {
    pub name: String,
    pub path: String,
    pub current_env_name: String,
    pub description: String,
    pub import_warnings: Vec<ImportWarning>,
}

impl Collection {
    pub fn get_config_file_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.path);
        path.join(COLLECTION_CONFIGFILE)
    }
}

// @TODO
#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ImportWarning {
    rest_file_path: String,
    request_name: String, // @TODO: check if not identifiable by id
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct AddCollectionsResult {
    pub workspace: Workspace,
    pub any_collections_found: bool,
    pub num_imported: i32,
    pub errored_collections: Vec<String>, // @TODO
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ImportCollectionResult {
    pub collection: Collection,
    // @TODO: environment
    // @TODO: requestTree
    // @TODO: collectionConfig
    pub import_warnings: Vec<ImportWarning>,
}

// order of children within a collection cannot be saved as they are just files/folders in a file
// system which does not have an order. It maps paths to the respective order *within* a parent
// this is only for requests or groups within a group. For filegroups the order is dependent on the
// order of the requests within a file
type PathOrder = HashMap<String, u32>;

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct CollectionConfig {
    pub name: String,
    pub path_orders: PathOrder,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        CollectionConfig {
            name: String::new(),
            path_orders: HashMap::new(),
        }
    }
}

pub type Uuid = String;

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
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
                .map(|request| {
                    request_to_request_model(request, value.path.to_string_lossy().to_string())
                })
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

pub fn request_to_request_model(
    value: http_rest_file::model::Request,
    path: String,
) -> RequestModel {
    RequestModel {
        id: uuid::Uuid::new_v4().to_string(),
        name: value.name.clone().unwrap_or(String::new()),
        description: value.get_comment_text().clone().unwrap_or(String::new()),
        method: value.request_line.method.unwrap_or_default(),
        http_version: value.request_line.http_version.into(),
        url: value.request_line.target.to_string(),
        rest_file_path: path,
        body: (&value.body).into(),
        query_params: Vec::new(), // @TODO parse qurey from url and remove
        headers: value.headers.iter().map(Into::into).collect(),
        settings: value.settings,
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct Header {
    key: String,
    value: String,
    active: bool,
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

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub enum RequestBody {
    None,
    Multipart {
        boundary: String,
        parts: Vec<Multipart>,
    },
    //@TODO
    Text {
        data: DataSource<String>,
    },
}

impl From<&HttpRestFileBody> for RequestBody {
    fn from(value: &HttpRestFileBody) -> Self {
        match value {
            HttpRestFileBody::None => RequestBody::None,
            HttpRestFileBody::Multipart { boundary, parts } => RequestBody::Multipart {
                boundary: boundary.clone(),
                parts: parts.iter().map(Into::into).collect(),
            },
            HttpRestFileBody::Text { data } => RequestBody::Text { data: data.clone() },
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct Multipart {
    pub name: String,
    pub data: DataSource<String>,
    pub fields: Vec<DispositionField>,
    pub headers: Vec<Header>,
}

impl From<&HttpRestfileMultipart> for Multipart {
    fn from(value: &HttpRestfileMultipart) -> Self {
        Multipart {
            name: value.name.clone(),
            data: value.data.clone(),
            fields: value.fields.clone(),
            headers: value.headers.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct RequestModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub method: HttpMethod,
    pub url: String,
    pub query_params: Vec<QueryParam>,
    pub headers: Vec<Header>,
    pub body: RequestBody,
    pub rest_file_path: String,
    pub http_version: Replaced<HttpVersion>,
    pub settings: RequestSettings,
}

impl RequestModel {
    pub fn get_request_file_path(&self, parent_path: String) -> String {
        let parent_path = std::path::Path::new(&parent_path);
        // @TODO files filenamify
        let path = parent_path.join(std::path::Path::new(&self.name));
        path.to_string_lossy().to_string()
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct Replaced<T> {
    value: T,
    is_replaced: bool,
}

impl<T> Replaced<T> {
    fn unwrap(self) -> T {
        self.value
    }

    fn is_replaced(&self) -> bool {
        self.is_replaced
    }

    fn as_ref<'a>(&'a self) -> &'a T {
        &self.value
    }
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
    name: String,
    initial_value: String,
    current_value: String,
    description: String,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct EnvironmentSecret {
    name: String,
    initial_value: String,
    current_value: String,
    description: String,
    persist_to_file: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct EnvVarDescription {
    env_var_name: String,
    description: String,
    is_secret: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Environment {
    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
    pub secrets: Vec<EnvironmentSecret>,
    pub env_var_descriptions: Vec<EnvVarDescription>,
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
    pub result: String,
    pub status_code: StatusCode,
    pub total_time: f64,
    pub total_result_size: f64,
    pub content_type: ContentType,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct SaveRequestCommand {
    pub requests: Vec<RequestModel>,
    pub collection: Collection,
    pub request_name: String,
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
            name: value.name.clone(),
            fields: value.fields.clone(),
        }
    }
}
use http_rest_file::model::RequestBody as RestFileBody;

use crate::config::COLLECTION_CONFIGFILE;
impl From<RequestBody> for http_rest_file::model::RequestBody {
    fn from(value: RequestBody) -> Self {
        match value {
            RequestBody::None => RestFileBody::None,
            RequestBody::Text { data } => RestFileBody::Text { data },
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
            } => WithDefault::Some(value.clone()),
            Replaced {
                value,
                is_replaced: true,
            } => WithDefault::Default(value.clone()),
        };
        let comments: Vec<http_rest_file::model::Comment> = match &value.description[..] {
            "" => vec![],
            description => description
                .split("\n")
                .into_iter()
                .map(|str| http_rest_file::model::Comment {
                    kind: http_rest_file::model::CommentKind::DoubleSlash,
                    value: str.to_string(),
                })
                .collect(),
        };
        let target = value.url.as_str().into();
        http_rest_file::model::Request {
            name: Some(value.name.clone()),
            request_line: RequestLine {
                method: WithDefault::Some(value.method.clone()),
                http_version,
                target,
            },
            body: value.body.clone().into(),
            headers: value.headers.iter().map(Into::into).collect(),
            comments,
            settings: value.settings.clone(),
            redirect: None, // @TODO
            pre_request_script: None,
            response_handler: None,
        }
    }
}
