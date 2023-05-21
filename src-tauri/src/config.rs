use std::fs;

use crate::error::{DisplayErrorKind, FrontendError};
use crate::model::Workspace;
use directories::ProjectDirs;

pub const WORKSPACE_FILENAME: &str = "workspace.json";
pub const COLLECTION_CONFIGFILE: &str = "relynx.collection.json";


pub fn get_config_dir() -> Option<std::path::PathBuf> {
    ProjectDirs::from("app", "relynx", "relynx").map(|dirs| dirs.config_dir().to_path_buf())
}

pub fn load_workspace() -> Result<Workspace, FrontendError> {
    // @TODO @ERR could not use ProjectDirs
    let config_dir =
        get_config_dir().ok_or(FrontendError::new(DisplayErrorKind::LoadWorkspaceError))?;

    let workspace_file_path = config_dir.join(WORKSPACE_FILENAME);

    if !config_dir.exists() {
        // @TODO @ERR could not create workspace folder
        dbg!(fs::create_dir(config_dir)
            .map_err(|_io_err| FrontendError::new(DisplayErrorKind::LoadWorkspaceError)))?;
    }

    if !workspace_file_path.exists() {
        save_workspace(&Workspace::default())?;
    }

    let content = dbg!(std::fs::read_to_string(workspace_file_path)
        .map_err(|_io_err| FrontendError::new(DisplayErrorKind::ReadWorkspaceFileError)))?;
    let workspace: Workspace = dbg!(serde_json::from_str(&content)
        .map_err(|_err| FrontendError::new(DisplayErrorKind::DeserializeWorkspaceError)))?;

    Ok(workspace)
}

pub fn save_workspace(workspace: &Workspace) -> Result<(), FrontendError> {
    let config_dir =
        get_config_dir().ok_or(FrontendError::new(DisplayErrorKind::LoadWorkspaceError))?;
    let workspace_file_path = dbg!(config_dir.join(WORKSPACE_FILENAME));
    let default_str = serde_json::to_string::<Workspace>(workspace)
        .map_err(|_serde_err| FrontendError::new(DisplayErrorKind::SerializeWorkspaceError))?;
    fs::write(workspace_file_path.clone(), default_str)
        .map_err(|_io_err| FrontendError::new(DisplayErrorKind::SaveWorkspaceError))?;
    Ok(())
}
