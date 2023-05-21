use rspc;
use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct FrontendError {
    kind: DisplayErrorKind,
    message: Option<String>,
}

impl FrontendError {
    pub fn new(kind: DisplayErrorKind) -> Self {
        FrontendError {
            kind,
            message: None,
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
    NoPathChosen
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
