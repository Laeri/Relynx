use std::fs;
use std::path::PathBuf;

use crate::error::{DisplayErrorKind, FrontendError};
use crate::model::{CollectionConfig, Workspace};
use directories::ProjectDirs;
use http_rest_file::model::HttpRestFileExtension;

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

    let content = std::fs::read_to_string(workspace_file_path)
        .map_err(|_io_err| FrontendError::new(DisplayErrorKind::ReadWorkspaceFileError))?;
    let workspace: Workspace = dbg!(serde_json::from_str(&content)
        .map_err(|_err| FrontendError::new(DisplayErrorKind::DeserializeWorkspaceError)))?;

    Ok(workspace)
}

pub fn save_workspace(workspace: &Workspace) -> Result<(), FrontendError> {
    let config_dir =
        get_config_dir().ok_or(FrontendError::new(DisplayErrorKind::LoadWorkspaceError))?;
    let workspace_file_path = dbg!(config_dir.join(WORKSPACE_FILENAME));
    let default_str = serde_json::to_string_pretty::<Workspace>(workspace)
        .map_err(|_serde_err| FrontendError::new(DisplayErrorKind::SerializeWorkspaceError))?;
    fs::write(workspace_file_path, default_str)
        .map_err(|_io_err| FrontendError::new(DisplayErrorKind::SaveWorkspaceError))?;
    Ok(())
}

pub fn load_collection_config(
    config_file_path: &PathBuf,
) -> Result<CollectionConfig, FrontendError> {
    let content = std::fs::read_to_string(config_file_path).map_err(|_err| {
        // @TODO: log error
        FrontendError::new_with_message(
            DisplayErrorKind::InvalidCollectionConfig,
            format!(
                "Could not load collection at path: '{}'",
                config_file_path.to_string_lossy()
            ),
        )
    })?;
    let collection_config: CollectionConfig = serde_json::from_str(&content).map_err(|_err| {
        FrontendError::new_with_message(
            DisplayErrorKind::InvalidCollectionConfig,
            format!(
                "Could not load collection at path: '{}'",
                config_file_path.to_string_lossy()
            ),
        )
    })?;
    Ok(collection_config)
}

pub fn save_collection_config(
    collection_config: &CollectionConfig,
    path: &PathBuf,
) -> Result<(), FrontendError> {
    let str = serde_json::to_string_pretty(collection_config).map_err(|_err| {
        let msg = format!(
            "Could not serialize collection configuration for collection: {}",
            collection_config.name
        );
        FrontendError::new_with_message(DisplayErrorKind::InvalidCollectionConfig, msg)
    })?;

    std::fs::write(path, str).map_err(|_err| {
        let msg = format!(
            "Could not write collection configuration to file. Collection config: {}, path: {}",
            collection_config.name,
            path.to_string_lossy()
        );
        FrontendError::new_with_message(DisplayErrorKind::InvalidCollectionConfig, msg)
    })
}
