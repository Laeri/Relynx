use std::path::PathBuf;

use rspc::Type;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
/// All these errors will be displayed to the user in the frontend
pub enum RelynxError {
    #[error("Could not remove collection from workspace")]
    RemoveCollectionError,
    #[error("There was an error when sending the request")]
    RequestSendErrorGeneric,
    #[error("{0}")]
    RequestSendErrorWithMsg(String),

    #[error("Could not open directory: '{0}'")]
    InvalidOpenPath(String),

    #[error("Could not delete item")]
    DeleteNodeError,

    #[error("Could not hide group")]
    HideGroupError,

    #[error("Could not select path as '{0}' cannot be relative to '{1}'")]
    RelativePathChoiceError(String, String),

    #[error("Could not load collection at path: '{0}'")]
    InvalidCollectionConfig(String),

    #[error("Could not serialize collection configuration for collection: {0}")]
    SerializeCollectionConfigError(String),

    #[error("Could not save environments to file")]
    SaveEnvironmentsError,

    #[error("Could not drag and drop request to new location. The request may be malformed")]
    DragAndDropRequestError,
    #[error("{0}")]
    DragAndDropError(String),
    #[error("Could not drag request into file group")]
    DragAndDropIntoFilegroupError,
    #[error("Could not drag and drop node")]
    DragAndDropGeneral,

    #[error("Could not reorder nodes within group")]
    ReorderDragAndDropError,

    #[error("Could not verify license")]
    LicenseInvalid,
    #[error("Could not load license data")]
    LoadLicenseDataError,
    #[error("Could not save license data")]
    SaveLicenseDataError,

    #[error("Could not load workspace")]
    LoadWorkspaceError,

    #[error("Could not save workspace")]
    SaveWorkspaceError,

    #[error("Could not import collection")]
    ImportCollectionError,

    #[error("Could not save request")]
    SaveRequestError,

    #[error("Cannot copy content to clipboard")]
    CopyToClipboardError,

    #[error("Could not open folder: '{0}'")]
    OpenFolderNativeError(String),

    #[error("Cannot create new request as the file already exists. File: '{0}'")]
    RequestFileAlreadyExists(String),

    #[error("Could not create new request")]
    RequestCreateError,

    #[error("Could not create new group")]
    CreateNewGroupGeneric,
    #[error("Could not create new group: '{0}'")]
    CreateNewGroupError(String),
    #[error("The new group's folder already exists. Folder: '{0}'")]
    GroupFolderAlreadyExists(String),
    #[error("There exists already a group with the name: '{0}' within the same parent group")]
    GroupNameAlreadyExistsInParent(String),

    #[error("Invalid group name: '{0}'. Cannot create folder with sanitized name: '{1}'")]
    InvalidGroupName(String, String),

    #[error("Could not rename group to: '{0}'")]
    RenameGroupError(String),

    #[error("Could not resolve the path")]
    RelativeResponsePathError,

    #[error("Parse errors occurred for request files")]
    ParseErrorGeneric,

    #[error("Import for postman collection version v1.0.0 is not supported. Try the import with a collection that uses the v2.1.0 json format.")]
    TriedPostmanImportV1_0_0,

    #[error("Import for postman collection version v2.0.0 is not supported. Try the import with a collection that uses the v2.1.0 json format.")]
    TriedPostmanImportV2_0_0,

    #[error("The Postman collection has an invalid format. Could not import collection.")]
    InvalidPostmanCollection,

    #[error("Load environment error")]
    LoadEnvironmentError,

    #[error("Could not locate log folder")]
    LogFolderMissing,

    #[error("Could not open cookie jar as there is not 'http-client.cookies' file present")]
    NoCookieJarFileFound,

    #[error("Could not open cookie jar file from path: '{0}'")]
    LoadCookieJarError(String),

    #[error("Could not save cookie jar file to path: '{0}'")]
    SaveCookieJarError(String),

    #[error("Could not save cookie jar file")]
    SaveCookieJarErrorGeneric,

    #[error("Could not update cookie jar file with new cookies")]
    UpdateCookieJarError,

    #[error("Could not use cookies from the cookie jar when sending a request")]
    UpdateRequestWithCookieJar,
}

impl From<RelynxError> for rspc::Error {
    fn from(error: RelynxError) -> Self {
        rspc::Error::with_cause::<RelynxError>(
            rspc::ErrorCode::InternalServerError,
            error.to_string(),
            error,
        )
    }
}

#[derive(Serialize, Deserialize, Type, Debug, PartialEq)]
pub struct ParseErrorMsg {
    pub filepath: PathBuf,
    pub filename: String,
    pub msg: String,
}
