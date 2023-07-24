use std::fs;
use std::path::PathBuf;

use crate::error::RelynxError;
use crate::model::{CollectionConfig, Workspace};
use directories::ProjectDirs;

pub const WORKSPACE_FILENAME: &str = "workspace.json";
pub const COLLECTION_CONFIGFILE: &str = "relynx.collection.json";

fn get_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("app", "relynx", "relynx")
}

pub fn get_data_dir() -> Option<std::path::PathBuf> {
    get_dirs().map(|dirs| dirs.data_local_dir().to_path_buf())
}

pub fn get_log_dir() -> Option<std::path::PathBuf> {
    get_data_dir().map(|dir| dir.join("logs"))
}

// the file 'relynx.log' is a convention by the tauri log plugin
pub fn get_log_filepath() -> Option<std::path::PathBuf> {
    get_log_dir().map(|dir| dir.join("relynx.log"))
}

pub fn get_config_dir() -> Option<std::path::PathBuf> {
    get_dirs().map(|dirs| dirs.config_dir().to_path_buf())
}

pub fn load_workspace() -> Result<Workspace, RelynxError> {
    // @TODO @ERR could not use ProjectDirs
    let config_dir = get_config_dir().ok_or(RelynxError::LoadWorkspaceError)?;

    let workspace_file_path = config_dir.join(WORKSPACE_FILENAME);

    if !config_dir.exists() {
        // @TODO @ERR could not create workspace folder
        fs::create_dir(config_dir).map_err(|io_err| {
            log::error!("Io Error: {:?}", io_err);
            log::error!("Could not create config dir when loading the workspace");
            RelynxError::LoadWorkspaceError
        })?;
    }

    if !workspace_file_path.exists() {
        save_workspace(&Workspace::default())?;
    }

    let content = std::fs::read_to_string(&workspace_file_path).map_err(|io_err| {
        log::error!(
            "Cannot read workspace to string, path: '{}'",
            workspace_file_path.to_string_lossy()
        );
        log::error!("Io Error: {:?}", io_err);
        RelynxError::LoadWorkspaceError
    })?;

    let mut workspace: Workspace = serde_json::from_str(&content).map_err(|err| {
        log::error!("Serde Error: {:?}", err);
        log::error!("Could not deserialize workspace: '{:?}'", content);
        RelynxError::LoadWorkspaceError
    })?;

    // for each collection check if the path of it's folder actually exists
    // use this in the frontend to mark them as not available until the path is fixed
    workspace.collections.iter_mut().for_each(|collection| {
        collection.path_exists = collection.path.exists();
    });

    Ok(workspace)
}

pub fn save_workspace(workspace: &Workspace) -> Result<(), RelynxError> {
    let config_dir = get_config_dir().ok_or(RelynxError::SaveWorkspaceError)?;
    let workspace_file_path = config_dir.join(WORKSPACE_FILENAME);
    let default_str =
        serde_json::to_string_pretty::<Workspace>(workspace).map_err(|serde_err| {
            log::error!("Serde Error: {:?}", serde_err);
            log::error!(
                "Could not serialize workspace to string, workspace: {:?}",
                workspace
            );
            RelynxError::SaveWorkspaceError
        })?;
    fs::write(&workspace_file_path, default_str).map_err(|io_err| {
        log::error!("Io Error: {:?}", io_err);
        log::error!(
            "Could not write workspace string to file, path: {:?}",
            workspace_file_path.display()
        );

        RelynxError::SaveWorkspaceError
    })?;

    // update the name within the relynx.collection.json file within the folder
    // the workspace is in another directory (.config folder), the name is duplicated within
    // collection in the workspace and collection config file, this makes it simpler as most often
    // we just need the collection from the workspace but we also need to save the name itself in
    // the collection config json if someone else imports the collection (after a git sync for
    // example)
    workspace.collections.iter().for_each(|collection| {
        let config_path = collection.get_config_file_path();
        // @TODO: handle error
        if let Ok(mut collection_config) = load_collection_config(&config_path) {
            collection_config.name = collection.name.clone();
            // @TODO: handle error
            let _result = save_collection_config(&collection_config, &config_path);
        }
    });
    Ok(())
}

pub fn load_collection_config(config_file_path: &PathBuf) -> Result<CollectionConfig, RelynxError> {
    let config_file_path = if config_file_path.is_dir() {
        config_file_path.join(COLLECTION_CONFIGFILE)
    } else {
        config_file_path.clone()
    };
    let content = std::fs::read_to_string(&config_file_path).map_err(|err| {
        log::error!("Io Error: {:?}", err);
        log::error!(
            "Could not read config content to str, path: '{:?}'",
            config_file_path
        );
        RelynxError::InvalidCollectionConfig(config_file_path.to_string_lossy().to_string())
    })?;
    let collection_config: CollectionConfig = serde_json::from_str(&content).map_err(|err| {
        log::error!("Could not deserialize content, err: {:?}", err);
        log::error!("Content: '{:}'", content);
        RelynxError::InvalidCollectionConfig(config_file_path.to_string_lossy().to_string())
    })?;
    Ok(collection_config)
}

pub fn save_collection_config(
    collection_config: &CollectionConfig,
    path: &PathBuf,
) -> Result<(), RelynxError> {
    let str = serde_json::to_string_pretty(collection_config).map_err(|err| {
        log::error!("Could not pretty print collection_config to json");
        log::error!("Serde error: {:?}", err);
        log::error!("Config: {:?}", collection_config);
        RelynxError::SerializeCollectionConfigError(collection_config.name.clone())
    })?;

    std::fs::write(path, str).map_err(|err| {
        log::error!("Io Error: {:?}", err);
        log::error!(
            "Could not write collection config to file: {}",
            path.to_string_lossy()
        );
        RelynxError::SerializeCollectionConfigError(collection_config.name.clone())
    })
}
