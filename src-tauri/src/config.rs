use std::fs;
use std::path::PathBuf;

use crate::error::{DisplayErrorKind, FrontendError};
use crate::model::{CollectionConfig, Workspace};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

pub const WORKSPACE_FILENAME: &str = "workspace.json";
pub const COLLECTION_CONFIGFILE: &str = "relynx.collection.json";


fn get_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("app", "relynx", "relynx")
}

pub fn get_data_dir() -> Option<std::path::PathBuf> {
    get_dirs().map(|dirs| dirs.data_local_dir().to_path_buf())
}
pub fn get_config_dir() -> Option<std::path::PathBuf> {
    get_dirs().map(|dirs| dirs.config_dir().to_path_buf())
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
    let mut workspace: Workspace = dbg!(serde_json::from_str(&content)
        .map_err(|_err| FrontendError::new(DisplayErrorKind::DeserializeWorkspaceError)))?;

    // for each collection check if the path of it's folder actually exists
    // use this in the frontend to mark them as not available until the path is fixed
    workspace.collections.iter_mut().for_each(|collection| {
        collection.path_exists = collection.path.exists();
    });

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

    // update the name within the relynx.collection.json file within the folder
    // the workspace is in another directory (.config folder), the name is duplicated within
    // collection in the workspace and collection config file, this makes it simpler as most often
    // we just need the collection from the workspace but we also need to save the name itself in
    // the collection config json if someone else imports the collection (after a git sync for
    // example)
    workspace.collections.iter().for_each(|collection| {
        let config_path = collection.get_config_file_path();
        let collection_config = load_collection_config(&config_path);
        // @TODO: handle error
        if collection_config.is_ok() {
            let mut collection_config = collection_config.unwrap();
            collection_config.name = collection.name.clone();
            // @TODO: handle error
            let _result = save_collection_config(&collection_config, &config_path);
        }
    });
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
