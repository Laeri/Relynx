use crate::config::{load_collection_config, save_collection_config};
use crate::error::{DisplayErrorKind, FrontendError};
use crate::import::LoadRequestsResult;
use crate::model::{
    AddCollectionsResult, Collection, CollectionConfig, ImportCollectionResult, RequestModel,
    RequestResult, RunRequestCommand, SaveRequestCommand, Workspace,
};
use crate::sanitize::sanitize_filename_with_options;
use crate::tree::{correct_children_paths, GroupOptions, RequestTreeNode, DEFAULT_OPTIONS};
use http_rest_file::{
    model::{HttpRestFile, HttpRestFileExtension, Request},
    Serializer,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex; // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use tauri::{api::shell, Manager};
use tauri::{AppHandle, ClipboardManager};
use walkdir::WalkDir;

pub struct Context {
    pub app_handle: Option<AppHandle>,
}

// @TODO: This is a workaround to access app_handle within router. Maybe with later rspc version we
// can do this (https://github.com/oscartbeaumont/rspc/issues/163)
pub static RELYNX_CONTEXT: Mutex<Context> = Mutex::new(Context { app_handle: None });

#[tauri::command]
pub fn load_workspace() -> Result<Workspace, rspc::Error> {
    crate::config::load_workspace().map_err(Into::into)
}

#[tauri::command]
pub fn remove_collection(collection: Collection) -> Result<Workspace, rspc::Error> {
    // @TODO error could not remove collection
    let mut workspace: Workspace =
        crate::config::load_workspace().map_err(Into::<rspc::Error>::into)?;

    let position = workspace
        .collections
        .iter()
        .position(|current| current.path != collection.path)
        // @TODO ERROR
        .ok_or(FrontendError::new(DisplayErrorKind::Generic))?;

    workspace.collections.remove(position);

    crate::config::save_workspace(&workspace)?;
    Ok(workspace)
}

// @TODO validate paths, createCollection -> should be empty...
#[tauri::command]
pub fn select_directory() -> Result<String, rspc::Error> {
    let result = tauri::api::dialog::blocking::FileDialogBuilder::default().pick_folder();
    match result {
        Some(folder_path) => Ok(folder_path.to_str().unwrap_or("").to_owned()),
        _ => Err(FrontendError::new(DisplayErrorKind::NoPathChosen).into()),
    }
}

// @TODO
#[tauri::command]
pub fn select_file() -> Result<String, rspc::Error> {
    let result = tauri::api::dialog::blocking::FileDialogBuilder::default().pick_file();
    match result {
        Some(folder_path) => Ok(folder_path.to_str().unwrap_or("").to_owned()),
        _ => Err(FrontendError::new(DisplayErrorKind::NoPathChosen).into()),
    }
}

#[tauri::command]
pub fn is_directory_empty(path: String) -> Result<bool, rspc::Error> {
    let path = std::path::Path::new(&path);
    Ok(path.exists())
}

#[tauri::command]
pub fn update_workspace(workspace: Workspace) -> Result<(), rspc::Error> {
    crate::config::save_workspace(&workspace).map_err(Into::into)
}

#[tauri::command]
pub fn add_existing_collections(
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
                    crate::config::COLLECTION_CONFIGFILE
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
pub fn load_requests_for_collection(
    collection: Collection,
) -> Result<LoadRequestsResult, rspc::Error> {
    crate::import::load_requests_for_collection(&collection).map_err(|err| err.into())
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct ImportPostmanCommandParams {
    pub workspace: Workspace,
    pub import_postman_path: String,
    pub import_result_path: String,
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct AddExistingCollectionsParams {
    pub path: String,
    pub workspace: Workspace,
}

#[tauri::command]
pub fn import_postman_collection(
    workspace: Workspace,
    import_postman_path: String,
    import_result_path: String,
) -> Result<ImportCollectionResult, rspc::Error> {
    todo!("Implement");
    //import::postman::import(workspace, import_postman_path, import_result_path).map_err(Into::into)
}

#[tauri::command]
pub fn run_request(request_command: RunRequestCommand) -> Result<RequestResult, rspc::Error> {
    todo!("implement")
}

#[tauri::command]
pub fn save_request(command: SaveRequestCommand) -> Result<String, rspc::Error> {
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
pub fn copy_to_clipboard(string: String) -> Result<(), rspc::Error> {
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
pub fn open_folder_native(app_handle: &tauri::AppHandle, path: &String) -> Result<(), rspc::Error> {
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
pub fn add_request_node(params: AddRequestNodeParams) -> Result<RequestTreeNode, rspc::Error> {
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
pub fn add_group_node(params: AddGroupNodeParams) -> Result<RequestTreeNode, rspc::Error> {
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
pub fn delete_node(params: DeleteNodeParams) -> Result<(), rspc::Error> {
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
            Err(_err) => {
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
pub fn drag_and_drop(params: DragAndDropParams) -> Result<DragAndDropResult, rspc::Error> {
    let DragAndDropParams {
        collection,
        mut drag_node_parent,
        mut drag_node,
        mut drop_node,
        drop_index,
    } = params;

    println!("DROP INDEX: {:?}", drop_index);
    if collection.path == "" {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Invalid collection given, collection has no path",
        )
        .into());
    }

    if !drag_node.filepath.starts_with(&collection.path) {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "The drag node is not within the given collection",
        )
        .into());
    }

    if !drop_node.filepath.starts_with(&collection.path) {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "The drop node is not within the given collection",
        )
        .into());
    }

    if drag_node.filepath == drop_node.filepath {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Cannot drag node unto itself",
        )
        .into());
    }

    if drop_node.request.is_some() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Can only drop node into a group and not into a request",
        )
        .into());
    }

    // cannot drag a regular group into a file group, only requests can be put into file groups
    if drop_node.is_file_group && drag_node.request.is_none() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Cannot drop a group node into a file group.",
        )
        .into());
    }

    if drop_node.any_child_with_name(&drag_node.name) {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "There exists already a node within the same name in the new parent. Rename the node first before dragging it."
        )
        .into());
    }

    // remove node from parent
    let in_parent_pos = drag_node_parent
        .children
        .iter()
        .position(|child| child.id == drag_node.id)
        .unwrap();
    println!("PARENT_POS: {:?}", in_parent_pos);
    drag_node_parent.children.remove(in_parent_pos);

    // create at new location
    let new_path = if drag_node.is_folder() {
        let new_path = PathBuf::from(&drop_node.filepath).join(&drag_node.name);
        std::fs::create_dir(&new_path).map_err(|_err| {
            Into::<rspc::Error>::into(FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Could not create new folder when dragging folder group",
            ))
        })?;
        new_path
    } else {
        // Create new file
        // If we drop into a file group the parent (aka the file group node) has to be resaved with the
        // new requests
        println!("DROP NODE: {:?}", drop_node);
        let request_file = if drop_node.is_file_group {
            // we drop a request into a file that already has requests, therefore save new file with
            // new request and remove old file
            let request_file: Result<HttpRestFile, ()> = (&drop_node).try_into();
            if request_file.is_err() {
                return Err(FrontendError::new_with_message(
                    DisplayErrorKind::DragAndDropError,
                    "Cannot convert request into a request file",
                )
                .into());
            }
            request_file.unwrap()
        } else {
            // otherwise we have moved a request into a folder and just need to save a new request
            // file
            let request_file: Result<HttpRestFile, ()> = (&drag_node).try_into();

            if request_file.is_err() {
                eprintln!("ERROR {:?}", request_file.unwrap_err());
                return Err(FrontendError::new_with_message(
                    DisplayErrorKind::DragAndDropError,
                    "Cannot convert request into a request file",
                )
                .into());
            }
            // we need to take the new path the drag node would have after dragging
            let mut request_file = request_file.unwrap();
            request_file.path = Box::new(PathBuf::from(&drop_node.filepath).join(&drag_node.name));

            request_file
        };
        Serializer::serialize_to_file(&request_file).map_err(|_err| {
            // @TODO: log serialize error
            Into::<rspc::Error>::into(FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Could not write request to file. The request may be malformed.",
            ))
        })?;
        (*request_file.path).clone()
    };

    // Remove old file
    // If the parents drag node was a file group we need to remove the file from the group and
    // resave the group. If it was a single request we need to remove the old file
    if drag_node_parent.is_file_group {
        let drag_node_pos = drag_node_parent
            .children
            .iter()
            .position(|child| child.id == drag_node.id);
        match drag_node_pos {
            Some(pos) => {
                drag_node_parent.children.remove(pos);
            }
            None => { /* @TODO handle error / warning?*/ }
        }

        // @TODO: @ERROR
        // if we dragged out of a file node and there are no requests left then remove the file
        if drag_node_parent.children.is_empty() {
            std::fs::remove_file(drag_node_parent.filepath).map_err(|_err| {
                Into::<rspc::Error>::into(FrontendError::new_with_message(
                    DisplayErrorKind::DragAndDropError,
                    "Could not remove old request file for drag node",
                ))
            })?;
        } else {
            // @TODO this might only be a warning?
            let new_drag_node_file = (&drag_node_parent).try_into().map_err(|_err| {
                Into::<rspc::Error>::into(FrontendError::new_with_message(
                    DisplayErrorKind::DragAndDropError,
                    "Could not remove drag node from old parent.",
                ))
            });
            // @TODO log error
            Serializer::serialize_to_file(new_drag_node_file.as_ref().unwrap()).map_err(
                |_err| {
                    Into::<rspc::Error>::into(FrontendError::new_with_message(
                        DisplayErrorKind::DragAndDropError,
                        "Could not save drag node's parent to file for updating",
                    ))
                },
            )?;
        }
    } else {
        // we have a request file or folder that was moved, so remove the old path
        let remove_path: PathBuf = drag_node.filepath.into();

        println!("Remove path: {:?}", remove_path);

        if !remove_path.exists() {
            return Err(Into::<rspc::Error>::into(FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Could not save drag node's parent to file for updating",
            )));
        }

        if dbg!(drag_node.is_file_group || drag_node.request.is_some()) {
            std::fs::remove_file(remove_path)
        } else {
            std::fs::remove_dir(remove_path)
        }
        .map_err(|_err| {
            // @TODO: log error
            Into::<rspc::Error>::into(FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Could not remove old request file for drag node",
            ))
        })?;
    };

    // update paths
    if drop_node.is_file_group {
        drag_node.filepath = drop_node.filepath.clone();
    } else {
        drag_node.filepath = new_path.to_string_lossy().to_string()
    }

    if drop_index >= drop_node.children.len() as u32 {
        drop_node.children.push(drag_node);
    } else {
        drop_node.children.insert(drop_index as usize, drag_node);
    }

    correct_children_paths(&mut drop_node);
    // @TODO: update child order in collection config
    /*  let collection_config =
    load_collection_config(&collection.get_config_file_path()).unwrap_or_default(); */

    // if we removed the last child from the drag node then we need to remove its file as well
    let remove_drag_node_parent =
        drag_node_parent.is_file_group && drag_node_parent.children.is_empty();
    Ok(DragAndDropResult {
        new_drop_node: drop_node,
        remove_drag_node_parent,
    })
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct ReorderNodesParams {
    // @TODO
    collection: Collection,
    drag_node: RequestTreeNode,
    drop_node: RequestTreeNode,
    // needs u32 type as rspc cannot export larger types as there is an issue with json parsing for
    // large numbers
    drop_index: u32,
}

#[tauri::command]
pub fn reorder_nodes_within_parent(
    params: ReorderNodesParams,
) -> Result<RequestTreeNode, rspc::Error> {
    let ReorderNodesParams {
        collection,
        mut drag_node,
        mut drop_node,
        mut drop_index,
    } = params;

    let mut drop_index = drop_index as usize;

    let drop_node_path = PathBuf::from(&drop_node.filepath);
    if !drop_node_path.exists() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::ReorderError,
            "Reorder parent does not exist anymore. The file/folder might have been removed."
                .to_string(),
        )
        .into());
    }

    let position = drop_node
        .children
        .iter()
        .position(|child| child.id == drag_node.id);
    if position.is_none() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::ReorderError,
            "Could not reorder items".to_string(),
        )
        .into());
    }
    let position = position.unwrap();
    if drop_index > position {
        drop_index -= 1;
    }
    drop_node.children.remove(position);
    drop_node.children.insert(drop_index, drag_node);

    // if a request is dragged within a file group its ordering within changed and we have to
    // resave the file
    if drop_node.is_file_group {
        let rest_file: HttpRestFile = (&drop_node).try_into().unwrap();
        Serializer::serialize_to_file(&rest_file).map_err(|_err| {
            FrontendError::new_with_message(
                DisplayErrorKind::ReorderError,
                "Could not persist updated requests".to_string(),
            )
        })?;
    } else {
        // we dragged a folder/file within a folder, we only have to store the updated pathordering
        // @TODO: load CollectionConfig, update orderings for all paths within parent_node
        let config_file_path = collection.get_config_file_path();
        let mut collection_config = load_collection_config(&config_file_path).unwrap_or_default();
        let path_orders = &mut collection_config.path_orders;
        drop_node
            .children
            .iter()
            .enumerate()
            .for_each(|(index, child)| {
                path_orders.insert(child.filepath.clone(), index as u32);
            });
        save_collection_config(&collection_config, &config_file_path)?;
    }

    Ok(drop_node)
}
