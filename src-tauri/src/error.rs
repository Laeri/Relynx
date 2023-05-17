use rspc;
use rspc::Type;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct FrontendError {
    kind: DisplayErrorKind,
    message: Option<String>,
    #[serde(skip)]
    source: Option<Box<dyn Error>>
}

impl FrontendError {
    pub fn new(kind: DisplayErrorKind) -> Self {
        FrontendError { kind, message: None, source: None }
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone, Copy)]
pub enum DisplayErrorKind {
    LoadWorkspaceError,
    ReadWorkspaceFileError,
    DeserializeWorkspaceError,
    SerializeWorkspaceError,
    SaveWorkspaceError,
}

impl std::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("tmp")
    }
}

impl std::error::Error for FrontendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        
    }
    /* fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {

    } */
}

impl From<FrontendError> for rspc::Error {
    fn from(display_error: FrontendError) -> Self {
        let msg = format!("Error: {:?}", display_error);

        rspc::Error::with_cause::<FrontendError>(
            rspc::ErrorCode::InternalServerError,
            msg.into(),
            display_error,
        )
    }
}
