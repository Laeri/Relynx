// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;
mod model;

use error::{DisplayErrorKind, FrontendError};
use model::{
    AddCollectionsResult, Collection, CollectionConfig, ImportCollectionResult, Workspace,
};
use rspc::Router;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc}; // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use tauri_plugin_log::LogTarget;
use walkdir::WalkDir;

#[tauri::command]
fn load_workspace() -> Result<Workspace, rspc::Error> {
    config::load_workspace().map_err(Into::into)
}

#[tauri::command]
fn remove_collection(collection: Collection) -> Result<Workspace, rspc::Error> {
    // @TODO error could not remove collection
    let mut workspace: Workspace = config::load_workspace().map_err(Into::<rspc::Error>::into)?;

    let position = workspace
        .collections
        .iter()
        .position(|current| current.path != collection.path)
        // @TODO ERROR
        .ok_or(FrontendError::new(DisplayErrorKind::Generic))?;

    workspace.collections.remove(position);
    config::save_workspace(&workspace)?;
    Ok(workspace)
}

// @TODO validate paths, createCollection -> should be empty...
#[tauri::command]
fn select_directory() -> Result<String, rspc::Error> {
    let result = tauri::api::dialog::blocking::FileDialogBuilder::default().pick_folder();
    match result {
        Some(folder_path) => Ok(folder_path.to_str().unwrap_or("").to_owned()),
        _ => Err(FrontendError::new(DisplayErrorKind::NoPathChosen).into()),
    }
}

// @TODO
#[tauri::command]
fn select_file() -> Result<String, rspc::Error> {
    let result = tauri::api::dialog::blocking::FileDialogBuilder::default().pick_file();
    match result {
        Some(folder_path) => Ok(folder_path.to_str().unwrap_or("").to_owned()),
        _ => Err(FrontendError::new(DisplayErrorKind::NoPathChosen).into()),
    }
}

#[tauri::command]
fn is_directory_empty(path: String) -> Result<bool, rspc::Error> {
    let path = std::path::Path::new(&path);
    Ok(path.exists())
}

#[tauri::command]
fn update_workspace(workspace: Workspace) -> Result<(), rspc::Error> {
    config::save_workspace(&workspace).map_err(Into::into)
}

#[tauri::command]
fn add_existing_collections(
    path: String,
    mut workspace: Workspace,
) -> Result<AddCollectionsResult, rspc::Error> {
    let mut collection_config_files: Vec<std::path::PathBuf> = Vec::new();
    for entry in WalkDir::new(path) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let filename = entry.file_name();
                // is there a json file for this collection?
                if matches!(
                    filename.to_str().unwrap_or(""),
                    config::COLLECTION_CONFIGFILE
                ) {
                    collection_config_files.push(std::path::PathBuf::from(filename))
                }
            }
        }
    }

    let mut errored_paths: Vec<&PathBuf> = Vec::new();
    let mut configs: Vec<CollectionConfig> = Vec::new();
    let mut collections: Vec<Collection> = Vec::new();

    for config_path in collection_config_files.iter() {
        let content = std::fs::read_to_string(config_path);
        if content.is_err() {
            errored_paths.push(config_path);
            continue;
        }
        let content = content.unwrap();
        if let Ok(config) = serde_json::from_str::<CollectionConfig>(&content) {
            let collection = Collection {
                name: config.name.clone(),
                path: config_path
                    .parent()
                    .map(|path| path.to_str())
                    .flatten()
                    .unwrap_or("")
                    .to_string(),
                description: "".to_string(),
                current_env_name: "".to_string(),
                import_warnings: Vec::new(),
            };
            configs.push(config);
            collections.push(collection);
        } else {
            errored_paths.push(config_path);
            continue;
        }
    }

    // @TODO check that none are already within the workspace!
    workspace.collections.append(&mut collections);

    Ok(AddCollectionsResult {
        workspace,
        num_imported: collections.len() as i32, // @TODO error?
        errored_collections: errored_paths
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(), // @TODO error?
        any_collections_found: collections.len() > 0,
    })
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
struct ImportPostmanCommandParams {
    pub workspace: Workspace,
    pub import_postman_path: String,
    pub import_result_path: String,
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
struct AddExistingCollectionsParams {
    pub path: String,
    pub workspace: Workspace,
}

#[tauri::command]
fn import_postman_collection(
    workspace: Workspace,
    import_postman_path: String,
    import_result_path: String,
) -> Result<ImportCollectionResult, rspc::Error> {
    todo!("Implement");
}

struct Context;

fn router() -> Arc<Router<Context>> {
    let router = Router::new()
        // change the bindings filename to your liking
        .config(rspc::Config::new().export_ts_bindings("../src/bindings.d.ts"))
        //.query("greet", |t| t(|_, name: String| greet(&name)))
        .query("load_workspace", |t| t(|_, ()| load_workspace()))
        .query("remove_collection", |t| {
            t(|_, collection: Collection| remove_collection(collection))
        })
        .query("select_directory", |t| t(|_, ()| select_directory()))
        .query("select_file", |t| t(|_, ()| select_file()))
        .query("is_directory_empty", |t| {
            t(|_, path: String| is_directory_empty(path))
        })
        .query("update_workspace", |t| {
            t(|_, workspace: Workspace| update_workspace(workspace))
        })
        .query("add_existing_collections", |t| {
            t(|_, params: AddExistingCollectionsParams | add_existing_collections(params.path, params.workspace))
        })
        .query("import_postman_collection", |t| {
            t(|_, params: ImportPostmanCommandParams| {
                import_postman_collection(
                    params.workspace,
                    params.import_postman_path,
                    params.import_result_path,
                )
            })
        })
        .build();
    Arc::new(router)
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    tauri::Builder::default()
        .plugin(rspc::integrations::tauri::plugin(router(), || Context))
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
