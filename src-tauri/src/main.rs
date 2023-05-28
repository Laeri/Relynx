// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;
mod import;
mod model;

use error::{DisplayErrorKind, FrontendError};
use import::LoadRequestsResult;
use model::{
    AddCollectionsResult, Collection, CollectionConfig, ImportCollectionResult, RequestModel,
    RequestResult, RunRequestCommand, SaveRequestCommand, Workspace,
};
use rspc::Router;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
}; // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use tauri::{api::shell, AppHandle, Manager};
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

#[tauri::command]
fn load_requests_for_collection(collection: Collection) -> Result<LoadRequestsResult, rspc::Error> {
    import::load_requests_for_collection(&collection).map_err(|err| err.into())
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
    //import::postman::import(workspace, import_postman_path, import_result_path).map_err(Into::into)
}

#[tauri::command]
fn run_request(request_command: RunRequestCommand) -> Result<RequestResult, rspc::Error> {
    todo!("implement")
}

#[tauri::command]
fn save_request(command: SaveRequestCommand) -> Result<RequestModel, rspc::Error> {
    todo!("implement")
}

#[tauri::command]
fn copy_to_clipboard(string: String) -> Result<(), rspc::Error> {
    todo!("implement")
}

#[tauri::command]
fn open_folder_native(app_handle: &tauri::AppHandle, path: &String) -> Result<(), rspc::Error> {
    // @TODO: we might want to restrict the path to collection folders
    let check_path = std::path::PathBuf::from(path.clone());
    if !check_path.exists() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::InvalidOpenPath,
            format!(
                "Cannot open folder: '{}' in explorer as it does not exist",
                check_path.to_string_lossy().to_owned()
            ),
        )
        .into());
    }

    if !check_path.is_dir() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::InvalidOpenPath,
            format!(
                "Cannot open folder: '{}' in explorer as it is not a directory",
                check_path.to_string_lossy().to_owned()
            ),
        )
        .into());
    }
    let shell_scope = app_handle.shell_scope();

    match shell::open(&shell_scope, path, None) {
        tauri::api::Result::Ok(_) => Ok(()),
        tauri::api::Result::Err(err) => {
            // @TODO: log error
            Err(FrontendError::new_with_message(
                DisplayErrorKind::InvalidOpenPath,
                format!("Could not open directory: {}", path),
            )
            .into())
        }
    }
}

fn router() -> Arc<Router> {
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
            t(|_, params: AddExistingCollectionsParams| {
                add_existing_collections(params.path, params.workspace)
            })
        })
        .query("load_requests_for_collection", |t| {
            t(|_, collection: Collection| load_requests_for_collection(collection))
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
        .query("run_request", |t| {
            t(|_, command: RunRequestCommand| run_request(command))
        })
        .query("save_request", |t| {
            t(|_, command: SaveRequestCommand| save_request(command))
        })
        .query("copy_to_clipboard", |t| {
            t(|_, string: String| copy_to_clipboard(string))
        })
        .query("open_folder_native", |t| {
            t(|_, path: String| {
                let mutex = RELYNX_CONTEXT.lock().unwrap();
                let handle = mutex.app_handle.as_ref().unwrap();
                open_folder_native(handle, &path)
            })
        })
        .build();
    Arc::new(router)
}

pub struct Context {
    pub app_handle: Option<AppHandle>,
}

// @TODO: This is a workaround to access app_handle within router. Maybe with later rspc version we
// can do this (https://github.com/oscartbeaumont/rspc/issues/163)
static RELYNX_CONTEXT: Mutex<Context> = Mutex::new(Context { app_handle: None });

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let context = tauri::generate_context!();
    let app = tauri::Builder::default();
    let app = app
        .plugin(rspc::integrations::tauri::plugin(router(), || {}))
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .build(),
        )
        .build(context)
        .expect("error while running tauri application");

    // @TODO: This is a workaround to access app_handle within router. Maybe with later rspc version we
    // can do this (https://github.com/oscartbeaumont/rspc/issues/163)
    let mut data = RELYNX_CONTEXT.lock().unwrap();
    data.app_handle = Some(app.app_handle());
    std::mem::drop(data);
    app.run(|_, _| {});
}
