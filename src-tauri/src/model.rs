use http_rest_file::model::{
    DataSource, DispositionField, Header as HttpRestFileHeader, HttpMethod, HttpRestFile,
    HttpVersion, Multipart as HttpRestfileMultipart, RequestBody as HttpRestFileBody,
    RequestSettings, WithDefault,
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

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct CollectionConfig {
    pub name: String,
}

type Uuid = String;

#[derive(Serialize, Deserialize, Type, Debug)]
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

fn request_to_request_model(value: http_rest_file::model::Request, path: String) -> RequestModel {
    let url = value.request_line.target.to_string();
    // @TODO: parse query from url !
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

#[derive(Serialize, Deserialize, Type, Debug)]
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

#[derive(Serialize, Deserialize, Type, Debug)]
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

#[derive(Serialize, Deserialize, Type, Debug)]
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

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RequestModel {
    pub id: Uuid,
    name: String,
    description: String,
    method: HttpMethod,
    url: String,
    query_params: Vec<QueryParam>,
    headers: Vec<Header>,
    body: RequestBody,
    rest_file_path: String,
    http_version: Replaced<HttpVersion>,
    settings: RequestSettings,
}

#[derive(Serialize, Deserialize, Type, Debug)]
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
