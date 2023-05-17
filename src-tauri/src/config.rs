use std::fs;

use crate::error::{DisplayErrorKind, FrontendError};
use crate::model::Workspace;
use directories::ProjectDirs;

const WORKSPACE_FILENAME: &str = "workspace.json";

pub fn load_workspace() -> Result<Workspace, FrontendError> {
    // @TODO @ERR could not use ProjectDirs
    let project_dirs = ProjectDirs::from("app", "relynx", "relynx")
        .ok_or(FrontendError::new(DisplayErrorKind::LoadWorkspaceError))?;

    let config_dir = dbg!(project_dirs.config_dir());
    if !config_dir.exists() {
        // @TODO @ERR could not create workspace folder
        dbg!(fs::create_dir(config_dir)
            .map_err(|_io_err| FrontendError::new(DisplayErrorKind::LoadWorkspaceError)))?;
    }

    let workspace_file_path = dbg!(config_dir.join(WORKSPACE_FILENAME));

    if !workspace_file_path.exists() {
        let default_str = serde_json::to_string::<Workspace>(&Workspace::default())
            .map_err(|_serde_err| FrontendError::new(DisplayErrorKind::SerializeWorkspaceError))?;
        dbg!(fs::write(workspace_file_path.clone(), default_str)
            .map_err(|_io_err| FrontendError::new(DisplayErrorKind::SaveWorkspaceError)))?;
    }

    let content = dbg!(std::fs::read_to_string(workspace_file_path)
        .map_err(|_io_err| FrontendError::new(DisplayErrorKind::ReadWorkspaceFileError)))?;
    let workspace: Workspace = dbg!(serde_json::from_str(&content)
        .map_err(|_err| FrontendError::new(DisplayErrorKind::DeserializeWorkspaceError)))?;

    Ok(workspace)
}
