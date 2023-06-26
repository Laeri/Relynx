mod drag_and_drop;
use crate::client::options::ClientOptions;
use crate::client::Client;
use crate::config::{load_collection_config, save_collection_config, save_workspace};
use crate::error::{DisplayErrorKind, FrontendError};
use crate::import::{
    create_jetbrains_collection, import_jetbrains_folder, postman, LoadRequestsResult,
    RELYNX_IGNORE_FILE,
};
use crate::model::{
    AddCollectionsResult, Collection, CollectionConfig, Environment, ImportCollectionResult,
    RequestModel, RequestResult, RunRequestCommand, SaveRequestCommand, Workspace,
};
use crate::sanitize::sanitize_filename_with_options;
use crate::tree::{GroupOptions, RequestTreeNode, DEFAULT_OPTIONS};
pub use drag_and_drop::{
    drag_and_drop, reorder_nodes_within_parent, DragAndDropParams, DragAndDropResult,
    ReorderNodesParams,
};
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
    crate::config::load_workspace().map_err(Into::<rspc::Error>::into)
}

#[tauri::command]
pub fn remove_collection(collection: Collection) -> Result<Workspace, rspc::Error> {
    // @TODO error could not remove collection
    let mut workspace: Workspace =
        crate::config::load_workspace().map_err(Into::<rspc::Error>::into)?;

    let position = workspace
        .collections
        .iter()
        .position(|current| current.path == collection.path)
        // @TODO ERROR
        .ok_or(FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not remove collection from workspace".to_string(),
        ))?;

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
pub fn is_directory_empty(path: PathBuf) -> Result<bool, rspc::Error> {
    let it = std::fs::read_dir(&path);
    if it.is_err() {
        return Ok(false);
    }
    Ok(it.unwrap().count() == 0)
}

#[tauri::command]
pub fn update_workspace(workspace: Workspace) -> Result<(), rspc::Error> {
    crate::config::save_workspace(&workspace).map_err(Into::into)
}

#[tauri::command]
pub fn add_existing_collections(
    path: PathBuf,
    mut workspace: Workspace,
) -> Result<AddCollectionsResult, rspc::Error> {
    let mut collection_config_files: Vec<std::path::PathBuf> = Vec::new();
    for entry in WalkDir::new(&path).into_iter().flatten() {
        if entry.file_type().is_file() {
            let filename = entry.file_name();
            // is there a json file for this collection?
            if matches!(
                filename.to_str().unwrap_or(""),
                crate::config::COLLECTION_CONFIGFILE
            ) {
                collection_config_files.push(entry.into_path())
            }
        }
    }

    let mut errored_paths: Vec<PathBuf> = Vec::new();
    let mut configs: Vec<CollectionConfig> = Vec::new();
    let mut collections: Vec<Collection> = Vec::new();

    for config_path in collection_config_files.iter() {
        let content = std::fs::read_to_string(config_path);
        if content.is_err() {
            errored_paths.push(config_path.clone());
            continue;
        }
        let content = content.unwrap();
        let deserialized = serde_json::from_str::<CollectionConfig>(&content);
        if let Ok(config) = deserialized {
            let path_buf = config_path
                .parent()
                .map(|path| path.to_owned())
                .unwrap_or_default();
            let collection = Collection {
                name: config.name.clone(),
                path: path_buf,
                description: "".to_string(),
                current_env_name: "".to_string(),
                import_warnings: Vec::new(),
                path_exists: true,
            };
            configs.push(config);
            collections.push(collection);
        } else {
            errored_paths.push(config_path.clone());
            continue;
        }
    }

    // if no collection is found (aka no .relynx json file) then add the chosen path as a
    // collection itself and generate a new config file
    if collections.is_empty() {
        let mut name = path
            .file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or("New Collection".to_string())
            .to_uppercase();
        let existing_names = workspace
            .collections
            .iter()
            .map(|col| col.name.clone())
            .collect::<Vec<String>>();
        let mut index = 0;
        loop {
            let current = format!("{}_{}", name, index);
            if !existing_names.contains(&current) {
                name = current;
                break;
            }
            index += 1;
        }
        let collection = create_jetbrains_collection(path, name);
        if collection.is_ok() {
            collections.push(collection.unwrap());
        }
    }

    let mut num_imported = collections.len() as u32;

    // @TODO check that none are already within the workspace!
    workspace.collections.append(&mut collections);

    // @TODO: log error
    if let Err(_) = save_workspace(&workspace) {
        num_imported = 0;
    }

    Ok(AddCollectionsResult {
        workspace,
        num_imported, // @TODO error?
        errored_collections: errored_paths,
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
    pub import_postman_path: PathBuf,
    pub import_result_path: PathBuf,
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct AddExistingCollectionsParams {
    pub path: PathBuf,
    pub workspace: Workspace,
}

#[tauri::command]
pub fn import_postman_collection(
    workspace: Workspace,
    import_postman_path: PathBuf,
    import_result_path: PathBuf,
) -> Result<ImportCollectionResult, rspc::Error> {
    postman::import(workspace, import_postman_path, import_result_path).map_err(Into::into)
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct ImportJetbrainsHttpFolderParams {
    pub workspace: Workspace,
    pub import_jetbrains_folder: PathBuf,
    pub collection_name: String,
}

#[tauri::command]
pub fn import_jetbrains_folder_command(
    params: ImportJetbrainsHttpFolderParams,
) -> Result<Workspace, rspc::Error> {
    let result = import_jetbrains_folder(
        params.workspace,
        params.import_jetbrains_folder,
        params.collection_name,
    );
    result.map_err(Into::into)
}

#[tauri::command]
pub fn run_request(request_command: RunRequestCommand) -> Result<RequestResult, rspc::Error> {
    println!("Command: {:?}", request_command);
    // @TODO: cookie input file...
    // @TODO: handle intellij redirect options
    let mut client = Client::new(None);
    let mut options = ClientOptions::default();
    options.follow_location = !request_command
        .request
        .settings
        .no_redirect
        .unwrap_or(false);

    // @TODO: set options from request settings
    let call = client
        .execute(&request_command.request, &options)
        .map_err(|http_err| {
            eprintln!("ERROR: {:?}", http_err);
            // @TODO:
            FrontendError::new_with_message(
                DisplayErrorKind::RequestSendError,
                "There was an error when sending the request.",
            )
        })?;

    // @TODO: what if it is not utf, what if the result is just binary, then displaying it would
    // not make sense, maybe add an option that you don't want to see the actual result together
    // with the redirect options
    let mut request_result = RequestResult {
        id: uuid::Uuid::new_v4().to_string(),
        result: String::from_utf8(call.response.body.to_vec()).unwrap_or_default(), // @TODO: handle non
        // utf8 result
        status_code: call.response.status.to_string(),
        // @TODO @CHECK is it secs or millis?
        total_time: call.timings.total.as_secs_f64(), // @TODO check how fine grained this should be
        content_type: call
            .response
            .headers
            .iter()
            .find(|h| h.key.to_lowercase() == "content-type")
            .map(|h| h.value.clone()),
        // @TODO: @CHECK why is it f64?
        total_result_size: call.response.body.len() as f64,
        warnings: vec![],
        result_file: None,
        result_file_folder: None,
    };

    let redirect_response = &request_command.request.redirect_response;
    if redirect_response.save_response {
        if redirect_response.save_path.is_none() {
            request_result.warnings.push("Could not save the response to file as no path is present. Configure the response path in the request's settings or choose that the result should not be saved to a file.".to_string());
        } else {
            // @TODO: maybe make the path private and only allow access over this method
            let absolute_path = redirect_response
                .get_absolute_path(&request_command.request)
                .clone()
                .unwrap_or(PathBuf::from("request_result"));

            // @TODO: emit a warning if we could not save the file
            let result = std::fs::write(&absolute_path, call.response.body);
            request_result.result_file = Some(absolute_path.clone());
            let parent_path = absolute_path.parent();
            request_result.result_file_folder = parent_path.map(|p| p.to_path_buf());

            if result.is_err() {
                // @TODO: log error
                eprintln!("ERROR: {:?}", result.unwrap_err());
                let msg = format!(
                    "Could not save the response to file '{}'. Check if the folder of the file exists and that you have permissions to write to it.",
                    absolute_path.to_string_lossy()
                );
                request_result.warnings.push(msg);
            }
        }
    }
    Ok(request_result)
}

#[tauri::command]
pub fn save_request(command: SaveRequestCommand) -> Result<String, rspc::Error> {
    let SaveRequestCommand {
        requests,
        collection: _,
        request_name: _,
    } = command;

    if requests.is_empty() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::SaveRequestError,
            "Tried to save empty request list".to_string(),
        )
        .into());
    }

    let file_path = requests[0].rest_file_path.clone();
    let requests: Vec<Request> = requests.into_iter().map(Into::into).collect();

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

    // also check if name changed, because then the request will reside in a new file based on its
    // new name and we have to remove the old file as a new file was created by the Serializer
    // beforehand
    /* @TODO: if the name of a request changes do we also want to update hte filename? It shouldn't
       * be necessary but it might lead to weird cases if it doesn't... @IMPORTANT let model = requests[0].clone();
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
    */
    Ok(file_path)
}

#[tauri::command]
pub fn copy_to_clipboard(string: String) -> Result<(), rspc::Error> {
    let context = RELYNX_CONTEXT.lock().unwrap();
    let app_handle = context.app_handle.as_ref().unwrap();
    let clipboard = &mut app_handle.clipboard_manager();
    clipboard.write_text(string.clone()).map_err(|_err| {
        // @TODO: handle error
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
                check_path.to_string_lossy()
            ),
        )
        .into());
    }

    if !check_path.is_dir() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::InvalidOpenPath,
            format!(
                "Cannot open folder: '{}' in explorer as it is not a directory",
                check_path.to_string_lossy()
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
    request_name: String,
    requests_in_same_file: Vec<RequestModel>,
}

#[tauri::command]
pub fn add_request_node(params: AddRequestNodeParams) -> Result<RequestTreeNode, rspc::Error> {
    let AddRequestNodeParams {
        collection,
        parent,
        request_name,
        requests_in_same_file,
    } = params;

    // @TODO
    let (file_model, node) = if parent.is_file_group {
        // new request is added last within the file
        let mut models = requests_in_same_file;

        let request_model =
            RequestModel::new(request_name, &PathBuf::from(parent.filepath.clone()));

        // if parent is a file group then the request has the same path as it is in the same file
        let new_request_tree_node = Ok(RequestTreeNode::new_request_node(
            request_model.clone(),
            parent.filepath.clone(),
        ));

        models.push(request_model);
        let requests: Vec<http_rest_file::model::Request> =
            models.into_iter().map(Into::into).collect();
        (
            HttpRestFile {
                requests,
                path: Box::new(PathBuf::from(&parent.filepath)),
                errs: vec![],
                extension: Some(http_rest_file::model::HttpRestFileExtension::Http),
            },
            new_request_tree_node,
        )
    } else {
        let request_path = RequestModel::create_request_path(&request_name, parent.filepath.into());

        let new_request = RequestModel::new(request_name, &request_path);
        // @TODO: check if any node with same name exists and return
        let new_request_tree_node = Ok(RequestTreeNode::new_request_node(
            new_request.clone(),
            request_path.to_string_lossy().to_string(),
        ));

        if request_path.exists() {
            let msg = format!(
                "Cannot save new request to path: '{}' as the file already exists",
                request_path.to_string_lossy()
            );

            return Err(FrontendError::new_with_message(
                DisplayErrorKind::RequestFileAlreadyExists,
                msg,
            )
            .into());
        }
        (
            HttpRestFile {
                requests: vec![new_request.into()],
                path: Box::new(request_path),
                errs: vec![],
                extension: Some(http_rest_file::model::HttpRestFileExtension::Http),
            },
            new_request_tree_node,
        )
    };

    if !file_model.path.starts_with(&collection.path) {
        let msg = "Could not crate new request".to_string();
        return Err(FrontendError::new_with_message(DisplayErrorKind::Generic, msg).into());
    }

    match Serializer::serialize_to_file(&file_model) {
        Ok(()) => return node,
        // @TODO: handle error
        Err(_err) => {}
    }
    node
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

    if folder_name.is_empty() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::AddGroupNodeError,
            "Cannot add a group node with no name".to_string(),
        )
        .into());
    }

    if !params
        .parent
        .filepath
        .starts_with(&params.collection.path.to_string_lossy().to_string())
    {
        let msg = format!(
            "The path: '{}' is not within the collection: '{}'",
            params.parent.filepath,
            params.collection.path.to_string_lossy()
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

    let parent_path = PathBuf::from(params.parent.filepath);
    if !parent_path.exists() {
        let msg = format!(
            "The parent's path of the new group does not exist or has been removed. Cannot create new folder in nonexisting directory. Path: '{}'",
            parent_path.to_string_lossy()
        );
        return Err(
            FrontendError::new_with_message(DisplayErrorKind::AddGroupNodeError, msg).into(),
        );
    }

    let path = parent_path.join(folder_name);
    if path.exists() {
        let msg = format!(
            "There exists already a file/folder with path: {}",
            path.to_string_lossy()
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
                path.to_string_lossy()
            );
            Err(FrontendError::new_with_message(DisplayErrorKind::AddGroupNodeError, msg).into())
        }
    }
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct ValidateGroupNameParams {
    old_path: PathBuf,
    new_name: String,
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct ValidateGroupNameResult {
    sanitized_name: String,
    new_path: PathBuf,
    path_exists_already: bool,
}

#[tauri::command]
pub fn validate_group_name(
    params: ValidateGroupNameParams,
) -> Result<ValidateGroupNameResult, rspc::Error> {
    let ValidateGroupNameParams { old_path, new_name } = params;

    let sanitized_name = sanitize_filename_with_options(new_name, DEFAULT_OPTIONS);

    let new_path = old_path
        .parent()
        .map(|parent_path| parent_path.join(&sanitized_name))
        .ok_or_else(|| {
            FrontendError::new_with_message(
                DisplayErrorKind::Generic,
                "Something is wrong with the path of the group.",
            )
        })?;

    Ok(ValidateGroupNameResult {
        sanitized_name,
        path_exists_already: new_path.exists(),
        new_path,
    })
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct RenameGroupParams {
    collection_path: PathBuf,
    old_path: PathBuf,
    new_name: String,
}

#[tauri::command]
pub fn rename_group(params: RenameGroupParams) -> Result<PathBuf, rspc::Error> {
    let RenameGroupParams {
        collection_path,
        old_path,
        new_name,
    } = params;

    let new_path = old_path
        .parent()
        .map(|parent| parent.join(new_name))
        .ok_or_else(|| {
            return FrontendError::new_with_message(
                DisplayErrorKind::Generic,
                "Could not rename the group",
            );
        })
        .map_err(Into::<rspc::Error>::into)?;

    if !(old_path.starts_with(&collection_path) && new_path.starts_with(&collection_path)) {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not rename the group",
        )
        .into());
    }

    if !old_path.exists() {
        let msg = format!(
            "The group's path: '{}' does not exist",
            old_path.to_string_lossy()
        );
        return Err(FrontendError::new_with_message(DisplayErrorKind::Generic, msg).into());
    }

    if new_path.exists() {
        let msg = format!(
            "Cannot rename the group as there exists already a group with path: '{}'",
            new_path.to_string_lossy()
        );
        return Err(FrontendError::new_with_message(DisplayErrorKind::Generic, msg).into());
    }

    std::fs::rename(&old_path, &new_path)
        .map_err(|_err| {
            // @TODO: Log error
            return FrontendError::new_with_message(
                DisplayErrorKind::Generic,
                "Could not rename the group",
            );
        })
        .map_err(Into::<rspc::Error>::into)?;

    Ok(new_path)
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

    if node.filepath.is_empty() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::NodeDeleteError,
            "The node you want to delete has an invalid file path".to_string(),
        )
        .into());
    }

    if !node
        .filepath
        .starts_with(&collection.path.to_string_lossy().to_string())
    {
        let msg =
            format!(
            "The node: '{}' with path: '{}' is not within collection: '{}', collectionPath: '{}'",
            node.name, node.filepath, collection.name, collection.path.to_string_lossy()
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
    if node.request.is_some() {
        // @TODO: log error
        std::fs::remove_file(&node.filepath)
        // @TODO: check that only requests/folders are within the group so that nothing wanted is
        // removed as well
    } else {
        std::fs::remove_dir_all(&node.filepath)
    }
    .map_err(|err| {
        let msg = format!("Could not delete node: {}, err: {}", node.name, err);
        Into::<rspc::Error>::into(FrontendError::new_with_message(
            DisplayErrorKind::NodeDeleteError,
            msg,
        ))
    })?;

    let mut collection_config =
        load_collection_config(&collection.get_config_file_path()).unwrap_or_default();
    collection_config.path_orders.remove(&node.filepath);
    // @TODO log error
    let _ignored = save_collection_config(&collection_config, &collection.get_config_file_path());

    Ok(())
}

#[tauri::command]
pub fn load_environments(collection_path: PathBuf) -> Result<Vec<Environment>, rspc::Error> {
    crate::environment::load_environments(collection_path).map_err(Into::into)
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct SaveEnvironmentsParams {
    collection_path: PathBuf,
    environments: Vec<Environment>,
}

#[tauri::command]
pub fn save_environments(params: SaveEnvironmentsParams) -> Result<(), rspc::Error> {
    crate::environment::save_environments(params.collection_path, params.environments)
        .map_err(Into::into)
}

#[tauri::command]
pub fn get_response_filepath(request_path: PathBuf) -> Result<PathBuf, rspc::Error> {
    let error = FrontendError::new_with_message(
        DisplayErrorKind::Generic,
        "Could not resolve the file path",
    );
    let request_folder = request_path.parent().ok_or(error.clone())?;

    let mut file_dialog_builder = tauri::api::dialog::blocking::FileDialogBuilder::default();
    if let Some(parent) = request_path.parent() {
        file_dialog_builder = file_dialog_builder.set_directory(parent);
    }
    let filepath = file_dialog_builder
        .save_file()
        .ok_or(rspc::Error::from(FrontendError::new(
            DisplayErrorKind::NoPathChosen,
        )))?;

    filepath
        .strip_prefix(request_folder)
        .map(|res| res.to_owned())
        .map_err(|_err| error)
        .map_err(Into::into)
}

#[tauri::command]
pub fn validate_response_filepath(response_file_path: PathBuf) -> Result<bool, rspc::Error> {
    // the file path is valid if its parent (folder) exists, the file itself does not need to exist
    // it will be created when a response is saved
    // @TODO: error, use a different one?
    let valid = response_file_path
        .parent()
        .map(|folder| folder.exists())
        .unwrap_or(false);
    Ok(valid)
}

#[tauri::command]
/// A folder is hidden if it contains a '.relynxignore' file (constant), then it will not be
/// displayed as a group
pub fn hide_group(path: PathBuf) -> Result<(), rspc::Error> {
    if !path.exists() || !path.is_dir() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not hide group".to_string(),
        )
        .into());
    }

    let ignorefile_path = path.join(RELYNX_IGNORE_FILE);
    std::fs::write(ignorefile_path, "").map_err(|_err| {
        // @TODO: log error
        Into::<rspc::Error>::into(FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not hide group",
        ))
    })?;

    Ok(())
}
