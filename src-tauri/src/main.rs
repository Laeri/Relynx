// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;
mod import;
mod model;
mod sanitize;
mod tree;

use error::{DisplayErrorKind, FrontendError};
use http_rest_file::{
    model::{HttpRestFile, HttpRestFileExtension, Request},
    Serializer,
};
use import::LoadRequestsResult;
use model::{
    AddCollectionsResult, Collection, CollectionConfig, ImportCollectionResult, RequestModel,
    RequestResult, RunRequestCommand, SaveRequestCommand, Workspace,
};
use rspc::Router;
use sanitize::sanitize_filename_with_options;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
}; // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use tauri::{api::shell, AppHandle, ClipboardManager, Manager};
use tauri_plugin_log::LogTarget;
use tree::{GroupOptions, RequestTreeNode, DEFAULT_OPTIONS};
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
fn save_request(command: SaveRequestCommand) -> Result<String, rspc::Error> {
    if command.requests.is_empty() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::SaveRequestError,
            "Tried to save empty request list".to_string(),
        )
        .into());
    }

    let file_path = command.requests[0].rest_file_path.clone();
    let requests: Vec<Request> = command
        .requests
        .clone()
        .into_iter()
        .map(Into::into)
        .collect();
    let file_model = http_rest_file::model::HttpRestFile {
        errs: vec![],
        requests,
        path: Box::new(PathBuf::from(file_path.clone())),
        extension: Some(HttpRestFileExtension::Http),
    };

    Serializer::serialize_to_file(&file_model).map_err(|_err| {
        // @TODO: handle error
        Into::<rspc::Error>::into(FrontendError::new_with_message(
            DisplayErrorKind::SaveRequestError,
            "Could not save request to file".to_string(),
        ))
    })?;

    // if the request is renamed and within a file group (only then are multiple requests present)
    // then just return after renaming the request
    if command.requests.len() > 1 {
        return Ok(file_path);
    }

    // also check if name changed, because then the request will reside in a new file based on its
    // new name and we have to remove the old file as a new file was created by the Serializer
    // beforehand
    let model = command.requests[0].clone();
    if model.name != command.request_name {
        std::fs::remove_file(model.rest_file_path.clone()).map_err(|_err| {
            // @TODO: handle error
            let msg = format!(
                "When renaming the request could not remove old file at path: {}",
                model.rest_file_path
            );
            Into::<rspc::Error>::into(FrontendError::new_with_message(
                DisplayErrorKind::RemoveOldRequestFile,
                msg,
            ))
        })?;
    }

    Ok(file_path)
}

#[tauri::command]
fn copy_to_clipboard(string: String) -> Result<(), rspc::Error> {
    let context = RELYNX_CONTEXT.lock().unwrap();
    let app_handle = context.app_handle.as_ref().unwrap();
    let clipboard = &mut app_handle.clipboard_manager();
    clipboard.write_text(string.clone()).map_err(|err| {
        // @TODO: handle error
        eprintln!("Error on writing to clipboard: {:?}", err);
        FrontendError::new_with_message(
            DisplayErrorKind::CopyToClipboardError,
            format!("Cannot copy content: '{:?}' to clipboard", string),
        )
        .into()
    })
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
        tauri::api::Result::Err(_err) => {
            // @TODO: log error
            Err(FrontendError::new_with_message(
                DisplayErrorKind::InvalidOpenPath,
                format!("Could not open directory: {}", path),
            )
            .into())
        }
    }
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct AddRequestNodeParams {
    collection: Collection,
    parent: RequestTreeNode,
    new_request: RequestModel,
    requests_in_same_file: Vec<RequestModel>,
}

#[tauri::command]
fn add_request_node(params: AddRequestNodeParams) -> Result<RequestTreeNode, rspc::Error> {
    // @TODO
    let (file_model, node) = if params.parent.is_file_group {
        // new request is added last within the file
        let mut models = params.requests_in_same_file;
        let new_request_tree_node = Ok(RequestTreeNode::new_request_node(
            params.new_request.clone(),
            params.parent.filepath.clone(),
        ));

        models.push(params.new_request);
        let requests: Vec<http_rest_file::model::Request> =
            models.into_iter().map(Into::into).collect();
        (
            HttpRestFile {
                requests,
                path: Box::new(PathBuf::from(params.parent.filepath)),
                errs: vec![],
                extension: Some(http_rest_file::model::HttpRestFileExtension::Http),
            },
            new_request_tree_node,
        )
    } else {
        // @TODO: check if any node with same name exists and return
        let request_path = params
            .new_request
            .get_request_file_path(params.parent.filepath);
        let request_path = std::path::PathBuf::from(request_path);
        let new_request_tree_node = Ok(RequestTreeNode::new_request_node(
            params.new_request.clone(),
            request_path.to_string_lossy().to_string(),
        ));

        if request_path.exists() {
            let msg = format!(
                "Cannot save new request to path: '{}' as the file already exists",
                request_path.to_string_lossy().to_string()
            );

            return Err(FrontendError::new_with_message(
                DisplayErrorKind::RequestFileAlreadyExists,
                msg,
            )
            .into());
        }
        (
            HttpRestFile {
                requests: vec![params.new_request.into()],
                path: Box::new(request_path),
                errs: vec![],
                extension: Some(http_rest_file::model::HttpRestFileExtension::Http),
            },
            new_request_tree_node,
        )
    };
    print!("FileModel: {:?}", file_model);
    print!("Node: {:?}", node);
    match Serializer::serialize_to_file(&file_model) {
        Ok(()) => return node,
        // @TODO: handle error
        Err(_err) => {}
    }
    todo!("@TODO");
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct AddGroupNodeParams {
    collection: Collection,
    parent: RequestTreeNode,
    group_name: String,
}

#[tauri::command]
fn add_group_node(params: AddGroupNodeParams) -> Result<RequestTreeNode, rspc::Error> {
    let folder_name = sanitize_filename_with_options(params.group_name, DEFAULT_OPTIONS);

    if folder_name == "" {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::AddGroupNodeError,
            "Cannot add a group node with no name".to_string(),
        )
        .into());
    }

    if !params.parent.filepath.starts_with(&params.collection.path) {
        let msg = format!(
            "The path: '{}' is not within the collection: '{}'",
            params.parent.filepath, params.collection.path
        );
        return Err(
            FrontendError::new_with_message(DisplayErrorKind::AddGroupNodeError, msg).into(),
        );
    }

    if params
        .parent
        .children
        .iter()
        .any(|child| child.name == folder_name)
    {
        let msg = "There exists already a node with the same name in the parent".to_string();
        return Err(
            FrontendError::new_with_message(DisplayErrorKind::AddGroupNodeError, msg).into(),
        );
    }

    let parent_path = PathBuf::from(params.parent.filepath.clone());
    if !parent_path.exists() {
        let msg = format!(
            "The parent's path of the new group does not exist or has been removed. Cannot create new folder in nonexisting directory. Path: '{}'",
            parent_path.to_string_lossy().to_string()
        );
        return Err(
            FrontendError::new_with_message(DisplayErrorKind::AddGroupNodeError, msg).into(),
        );
    }

    let path = parent_path.join(folder_name);
    if path.exists() {
        let msg = format!(
            "There exists already a file/folder with path: {}",
            path.to_string_lossy().to_string()
        );
        return Err(
            FrontendError::new_with_message(DisplayErrorKind::AddGroupNodeError, msg).into(),
        );
    }

    match std::fs::create_dir(path.clone()) {
        Ok(()) => Ok(RequestTreeNode::new_group(GroupOptions::FullPath(
            path.to_string_lossy().to_string(),
        ))),
        Err(_err) => {
            // @TODO: log error
            //
            let msg = format!(
                "Could not create new folder at path: '{}'",
                path.to_string_lossy().to_string()
            );
            Err(FrontendError::new_with_message(DisplayErrorKind::AddGroupNodeError, msg).into())
        }
    }
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct DeleteNodeParams {
    collection: Collection,
    node: RequestTreeNode,
    file_node: Option<RequestTreeNode>,
}
#[tauri::command]
fn delete_node(params: DeleteNodeParams) -> Result<(), rspc::Error> {
    let collection = params.collection;
    let node = params.node;

    if node.filepath == "" {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::NodeDeleteError,
            "The node you want to delete has an invalid file path".to_string(),
        )
        .into());
    }

    if !node.filepath.starts_with(&collection.path) {
        let msg = format!(
            "The node: '{}' with path: '{}' is not within collection: '{}', collectionPath: '{}'",
            node.name, node.filepath, collection.name, collection.path
        );
        return Err(FrontendError::new_with_message(DisplayErrorKind::NodeDeleteError, msg).into());
    }

    // if deleted node is part of a file, then we need to remove it from its request and resave /
    // reserialize all requests within the file
    if let Some(mut file_node) = params.file_node {
        // file_node only has children with requests, so it is safe to unwrap
        file_node.children = file_node
            .children
            .into_iter()
            .filter(|child| child.request.as_ref().unwrap().id != node.request.as_ref().unwrap().id)
            .collect::<Vec<RequestTreeNode>>();
        let file_model = HttpRestFile {
            requests: file_node
                .children
                .into_iter()
                .map(|node| node.request.unwrap().into())
                .collect::<Vec<http_rest_file::model::Request>>(),
            errs: vec![],
            extension: Some(HttpRestFileExtension::Http),
            path: Box::new(PathBuf::from(file_node.filepath.clone())),
        };
        match Serializer::serialize_to_file(&file_model) {
            Ok(_) => return Ok(()),
            Err(err) => {
                let msg = format!(
                    "Could not remove request: '{}' from file: '{}'",
                    node.name, file_node.filepath
                );
                return Err(Into::<rspc::Error>::into(FrontendError::new_with_message(
                    DisplayErrorKind::NodeDeleteError,
                    msg,
                )));
            }
        };
    }

    // @TODO: if the node contains multiple children maybe ask if they want to delete everything
    // otherwise we delete either a single request (one file), or a directory with all its children
    if let Some(_) = node.request {
        // @TODO: log error
        std::fs::remove_file(node.filepath)
        // @TODO: check that only requests/folders are within the group so that nothing wanted is
        // removed as well
    } else {
        std::fs::remove_dir_all(node.filepath)
    }
    .map_err(|err| {
        let msg = format!(
            "Could not delete node: {}, err: {}",
            node.name,
            err.to_string()
        );
        Into::<rspc::Error>::into(FrontendError::new_with_message(
            DisplayErrorKind::NodeDeleteError,
            msg,
        ))
    })?;
    Ok(())
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct DragAndDropParams {
    collection: Collection,
    drag_node_parent: RequestTreeNode,
    drag_node: RequestTreeNode,
    drop_node: RequestTreeNode,
    drop_index: u32,
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct DragAndDropResult {
    // @TODO
    new_drop_node: RequestTreeNode,
    remove_drag_node_parent: bool,
}

#[tauri::command]
fn drag_and_drop(params: DragAndDropParams) -> DragAndDropResult {
    // @TODO
    todo!("@TODO");
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct ReorderNodesParams {
    // @TODO
    collection: Collection,
    drag_node: RequestTreeNode,
    drop_node: RequestTreeNode,
    drop_index: u32,
}

#[tauri::command]
fn reorder_nodes_within_parent(params: ReorderNodesParams) -> RequestTreeNode {
    // @TODO
    todo!("@TODO");
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
        .query("add_request_node", |t| {
            t(|_, params: AddRequestNodeParams| add_request_node(params))
        })
        .query("add_group_node", |t| {
            t(|_, params: AddGroupNodeParams| add_group_node(params))
        })
        .query("delete_node", |t| {
            t(|_, params: DeleteNodeParams| delete_node(params))
        })
        .query("drag_and_drop", |t| {
            t(|_, params: DragAndDropParams| drag_and_drop(params))
        })
        .query("reorder_nodes_within_parent", |t| {
            t(|_, params: ReorderNodesParams| reorder_nodes_within_parent(params))
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
