

use http_rest_file::model::ParseError;
use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct FrontendError {
    pub kind: DisplayErrorKind,
    pub message: Option<String>,
}

impl FrontendError {
    pub fn new(kind: DisplayErrorKind) -> Self {
        FrontendError {
            kind,
            message: None,
        }
    }

    pub fn new_with_message<T>(kind: DisplayErrorKind, message: T) -> Self
    where
        T: Into<String>,
    {
        FrontendError {
            kind,
            message: Some(message.into()),
        }
    }
}

impl Default for FrontendError {
    fn default() -> Self {
        FrontendError {
            kind: DisplayErrorKind::Generic,
            message: Some("".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, Copy)]
pub enum DisplayErrorKind {
    Generic,
    LoadWorkspaceError,
    ReadWorkspaceFileError,
    DeserializeWorkspaceError,
    SerializeWorkspaceError,
    SaveWorkspaceError,
    NoPathChosen,
    ImportPostmanError,
    ParseError,
    InvalidOpenPath,
    CopyToClipboardError,
    RequestFileAlreadyExists,
    NodeDeleteError,
    SaveRequestError,
    RemoveOldRequestFile,
    AddGroupNodeError,
    DragAndDropError,
    InvalidCollectionConfig,
    ReorderError,
    UnsupportedImportFormat,
    ImportSerializeError,
    LoadEnvironmentsError,
    SaveEnvironmentsError,
    RequestFileMissing,
    CurlError,
    RequestSendError,
}

impl std::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("tmp")
    }
}

// @TODO: error with a cause
impl std::error::Error for FrontendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<FrontendError> for rspc::Error {
    fn from(frontend_error: FrontendError) -> Self {
        rspc::Error::with_cause::<FrontendError>(
            rspc::ErrorCode::InternalServerError,
            frontend_error
                .message
                .as_ref()
                .unwrap_or(&String::new())
                .clone(),
            frontend_error,
        )
    }
}

impl From<ParseError> for FrontendError {
    fn from(value: ParseError) -> Self {
        FrontendError {
            kind: DisplayErrorKind::ParseError,
            message: Some(value.message),
        }
    }
}
