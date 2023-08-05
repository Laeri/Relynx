mod drag_and_drop;
use crate::client::error::HttpError;
use crate::client::options::ClientOptions;
use crate::client::Client;
use crate::config::{
    get_log_filepath, load_collection_config, save_collection_config, save_workspace,
};
use crate::cookie_jar::{
    load_cookie_jar, save_cookie_jar, update_cookie_jar, update_request_with_cookie_jar,
    CookieJarPath, GetCookieJarParams, SaveCookieJarParams,
};
use crate::error::RelynxError;
use crate::import::{
    create_jetbrains_collection, import_jetbrains_folder, postman, LoadRequestsResult,
    RELYNX_IGNORE_FILE,
};
use crate::license::{self, verify_signature};
use crate::model::{
    AddCollectionsResult, AppEnvironment, Collection, CollectionConfig, CookieJar, Environment,
    ImportCollectionResult, RequestModel, RequestResult, RunLogger, RunRequestCommand,
    SaveRequestCommand, Workspace,
};
use crate::pathdiff::diff_paths;
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
use license::LicenseData;
use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};
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
    let mut workspace: Workspace =
        crate::config::load_workspace().map_err(Into::<rspc::Error>::into)?;

    let position = workspace
        .collections
        .iter()
        .position(|current| current.path == collection.path)
        .ok_or(RelynxError::RemoveCollectionError)
        .map_err(|err| {
            log::error!("Could not remove collection as it could not be found");
            err
        })?;

    workspace.collections.remove(position);

    crate::config::save_workspace(&workspace).map_err(|err| {
        log::error!("Could not save collection after remove_collection");
        log::error!("Error: {:?}", err);
        RelynxError::RemoveCollectionError
    })?;
    Ok(workspace)
}

#[tauri::command]
pub fn select_directory() -> Result<Option<PathBuf>, rspc::Error> {
    let result = tauri::api::dialog::blocking::FileDialogBuilder::default().pick_folder();
    match result {
        Some(folder_path) => Ok(Some(folder_path)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn select_file(default_folder: Option<&Path>) -> Result<Option<PathBuf>, rspc::Error> {
    let mut builder = tauri::api::dialog::blocking::FileDialogBuilder::default();
    if let Some(default_folder) = default_folder {
        builder = builder.set_directory(default_folder);
    }
    let result = builder.pick_file();

    match result {
        Some(folder_path) => Ok(Some(folder_path)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn is_directory_empty(path: PathBuf) -> Result<bool, rspc::Error> {
    let it = std::fs::read_dir(path);
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

    for config_path in dbg!(collection_config_files).iter() {
        let content = std::fs::read_to_string(config_path);
        if content.is_err() {
            errored_paths.push(config_path.clone());
            continue;
        }
        let content = content.unwrap();
        let deserialized = dbg!(serde_json::from_str::<CollectionConfig>(&content));
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
            .unwrap_or("New Collection".to_string());
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
        if let Ok(collection) = create_jetbrains_collection(path, name) {
            collections.push(collection);
        }
    }

    let mut num_imported = collections.len() as u32;

    let collection_names: Vec<String> = collections.iter().map(|col| col.name.clone()).collect();

    // @TODO check that none are already within the workspace!
    workspace.collections.append(&mut collections);

    if let Err(err) = save_workspace(&workspace) {
        num_imported = 0;
        log::error!(
            "Could not save workspace in add existing collections, err: {:?}",
            err
        );
    }

    Ok(AddCollectionsResult {
        workspace,
        num_imported,
        errored_collections: errored_paths,
        collection_names,
    })
}

#[tauri::command]
pub fn load_requests_for_collection(
    collection: Collection,
) -> Result<LoadRequestsResult, rspc::Error> {
    dbg!(crate::import::load_requests_for_collection(&collection).map_err(|err| err.into()))
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
pub fn run_request(mut request_command: RunRequestCommand) -> Result<RequestResult, rspc::Error> {
    // @TODO: cookie input file...
    // @TODO: handle intellij redirect options
    let mut client = Client::new(None);
    let follow_location = !request_command
        .request
        .settings
        .no_redirect
        .unwrap_or(false);

    let options = ClientOptions {
        follow_location,
        ..Default::default()
    };

    let mut cookie_jar: Option<CookieJar> = None;

    let allow_cookie_jar = !request_command
        .request
        .settings
        .no_cookie_jar
        .unwrap_or(false);

    let no_log = request_command.request.settings.no_log.unwrap_or(false);
    let logger = RunLogger::new(no_log);

    if allow_cookie_jar {
        cookie_jar = load_cookie_jar(&CookieJarPath::CollectionFolderPath(
            request_command.collection.path.clone(),
        ))
        .ok();
        if cookie_jar.is_some() {
            let _ = update_request_with_cookie_jar(
                &mut request_command.request,
                cookie_jar.as_ref().unwrap(),
                request_command.environment.as_ref(),
                &logger,
            );
        }
    }

    let calls = client
        .execute(
            &request_command.request,
            &options,
            request_command.environment.as_ref(),
            &logger,
        )
        .map_err(|http_err: HttpError| {
            if !no_log {
                logger.log_error(format!("Http error occurred: {:?}", http_err));
                logger.log_error(format!("Option: {:?}", options));
                logger.log_error(format!("Request: {:?}", request_command.request));
                if let Some(environment) = request_command.environment {
                    // @TODO
                    log::error!("Environment name: {}", environment.name);
                    log::error!("Public environment values: {:?}", environment.variables);
                    log::error!(
                        "Secret names (values are not logged!) {:?}",
                        environment.secrets.into_iter().map(|secret| secret.name)
                    );
                } else {
                    log::error!("Environment: None");
                }
            }
            if !no_log {
                match http_err {
                    HttpError::InvalidUrl(url) => RelynxError::RequestSendErrorWithMsg(format!(
                        "Something is wrong with the url: {}",
                        url
                    )),
                    _ => RelynxError::RequestSendErrorGeneric,
                }
            } else {
                RelynxError::RequestSendErrorGeneric
            }
        })?;

    let call = calls.last().unwrap();

    #[allow(clippy::unnecessary_unwrap)]
    if allow_cookie_jar {
        let cookie_jar = cookie_jar.or_else(|| {
            load_cookie_jar(&CookieJarPath::CollectionFolderPath(
                request_command.collection.path.clone(),
            ))
            .ok()
        });

        if cookie_jar.is_some() {
            let cookie_jar = cookie_jar.expect("cookie jar is present");
            let _ = update_cookie_jar(&request_command.collection, &cookie_jar, &calls).map_err(
                |err| {
                    logger.log_error("Could not update cookie jar after execute request!");
                    logger.log_error(format!("Error {:?}", err));
                },
            );
        }
    }

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

    if let Some(ref save_response) = request_command.request.save_response {
        if save_response.is_path_empty() {
            request_result.warnings.push("Could not save the response to file as no path is present. Configure the response path in the request's settings or choose that the result should not be saved to a file.".to_string());
        } else {
            // @TODO: maybe make the path private and only allow access over this method
            let absolute_path = save_response
                .get_absolute_path(&request_command.request)
                .unwrap_or(PathBuf::from("request_result"));

            // @TODO: emit a warning if we could not save the file
            let result = std::fs::write(&absolute_path, &call.response.body);
            request_result.result_file = Some(absolute_path.clone());
            let parent_path = absolute_path.parent();
            request_result.result_file_folder = parent_path.map(|p| p.to_path_buf());

            if result.is_err() {
                log::error!(
                    "Could not save response of request to file {}",
                    absolute_path.display()
                );
                let msg = format!(
                    "Could not save the response to file '{}'. Check if the folder of the file exists and that you have permissions to write to it.",
                    absolute_path.display()
                );
                request_result.warnings.push(msg);
            }
        }
    }
    Ok(request_result)
}

#[tauri::command]
pub fn save_request(command: SaveRequestCommand) -> Result<PathBuf, rspc::Error> {
    let SaveRequestCommand {
        requests,
        collection,
        old_name,
    } = dbg!(command);
    log::debug!("SAVE_REQUEST");

    if requests.is_empty() {
        log::error!("Tried to save empty request list");
        return Err(RelynxError::SaveRequestError.into());
    }

    let old_path = requests[0].rest_file_path.clone();

    // if there are more than ony request we change a request within a file group, so the path will
    // be the same and does not change, if there is one request and its name changed we have to
    // create a new file
    let file_path = if requests.len() == 1 && requests[0].name != old_name {
        let request = &requests[0];

        // otherwise we will create a new file with the request and afterwards delete the old one
        let new_filename = sanitize_filename_with_options(&request.name, DEFAULT_OPTIONS)
            + &HttpRestFileExtension::get_default_extension();
        let parent_path = request.rest_file_path.parent().ok_or_else(|| {
            log::error!(
                "Could not save request as restfilepath: '{}' has no valid parent path.",
                request.rest_file_path.display()
            );
            Into::<rspc::Error>::into(RelynxError::SaveRequestError)
        })?;
        let new_path = parent_path.join(new_filename);

        // if the new path exists already keep the old one
        if new_path.exists() {
            return Err(RelynxError::RequestFileAlreadyExists(
                new_path.to_string_lossy().to_string(),
            )
            .into());
        }
        new_path
    } else {
        requests[0].rest_file_path.clone()
    };

    if !old_path.starts_with(&collection.path) || !file_path.starts_with(&collection.path) {
        log::error!(
            "Could not update requests: '{:?}' as it does not seem to belong to collection: '{:?}'",
            requests,
            collection
        );
        return Err(RelynxError::SaveRequestError.into());
    }

    let requests: Vec<Request> = requests.into_iter().map(Into::into).collect();

    let file_model = http_rest_file::model::HttpRestFile {
        errs: vec![],
        requests,
        path: Box::new(file_path.clone()),
        extension: Some(HttpRestFileExtension::Http),
    };

    Serializer::serialize_to_file(&file_model).map_err(|err| {
        log::error!(
            "Serializing in save request failed for file_model: {:?}",
            file_model
        );
        log::error!("Serde error: {:?}", err);
        Into::<rspc::Error>::into(RelynxError::SaveRequestError)
    })?;

    if file_path != old_path {
        //if the name of a request changes do we also want to update hte filename? It shouldn't
        // be necessary but it might lead to weird cases if it doesn't... @IMPORTANT let model = requests[0].clone();
        std::fs::remove_file(&old_path).map_err(|err| {
            log::error!(
                "When renaming the request could not remove old file at path: {}",
                old_path.display()
            );
            log::error!("Io Error: {:?}", err);
            Into::<rspc::Error>::into(RelynxError::SaveRequestError)
        })?;
    }

    Ok(dbg!(file_path))
}

#[tauri::command]
pub fn copy_to_clipboard(string: String) -> Result<(), rspc::Error> {
    let context = RELYNX_CONTEXT.lock().unwrap();
    let app_handle = context.app_handle.as_ref().unwrap();
    let clipboard = &mut app_handle.clipboard_manager();
    clipboard.write_text(string.clone()).map_err(|err| {
        log::error!("Could not copy to clipboard, content: {}", string);
        log::error!("Error: {:?}", err);
        RelynxError::CopyToClipboardError.into()
    })
}

#[tauri::command]
pub fn open_folder_native(
    app_handle: &tauri::AppHandle,
    check_path: &Path,
) -> Result<(), rspc::Error> {
    // @TODO: we might want to restrict the path to collection folders
    if !check_path.exists() {
        log::error!(
            "Cannot open folder: '{}' in explorer as it does not exist",
            check_path.display()
        );
        return Err(
            RelynxError::OpenFolderNativeError(check_path.to_string_lossy().to_string()).into(),
        );
    }

    if !check_path.is_dir() {
        log::error!(
            "Folder to open is not a directory! Folder: '{}'",
            check_path.display()
        );
        return Err(
            RelynxError::OpenFolderNativeError(check_path.to_string_lossy().to_string()).into(),
        );
    }
    let shell_scope = app_handle.shell_scope();

    match shell::open(&shell_scope, check_path.to_string_lossy(), None) {
        tauri::api::Result::Ok(_) => Ok(()),
        tauri::api::Result::Err(_err) => {
            log::error!(
                "shell::open for open folder not working with path: {}",
                check_path.display()
            );
            Err(RelynxError::InvalidOpenPath(check_path.to_string_lossy().to_string()).into())
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

        let request_model = RequestModel::new(request_name, &parent.filepath);

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
        let request_path = RequestModel::create_request_path(&request_name, parent.filepath);

        let new_request = RequestModel::new(request_name, &request_path);
        // @TODO: check if any node with same name exists and return
        let new_request_tree_node = Ok(RequestTreeNode::new_request_node(
            new_request.clone(),
            request_path.clone(),
        ));

        if request_path.exists() {
            log::error!(
                "Cannot save request to path: '{}' as the file already exists.",
                request_path.display()
            );
            return Err(RelynxError::RequestFileAlreadyExists(
                request_path.to_string_lossy().to_string(),
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
        log::error!("Cannot create reques tas the file_models path is not within the collection.");
        log::error!("FileModel: {:?}", file_model);
        log::error!("Collection: {:?}", collection);
        return Err(RelynxError::RequestCreateError.into());
    }

    match Serializer::serialize_to_file(&file_model) {
        Ok(()) => node,
        Err(err) => {
            log::error!("Could not serialize to file");
            log::error!("Err: {:?}", err);
            Err(RelynxError::RequestCreateError.into())
        }
    }
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct AddGroupNodeParams {
    collection: Collection,
    parent: RequestTreeNode,
    group_name: String,
}

#[tauri::command]
pub fn add_group_node(params: AddGroupNodeParams) -> Result<RequestTreeNode, rspc::Error> {
    let folder_name = sanitize_filename_with_options(&params.group_name, DEFAULT_OPTIONS);

    if folder_name.is_empty() {
        log::error!("Cannot add a group node with no name");
        log::error!("Params: {:?}", params);
        return Err(RelynxError::CreateNewGroupGeneric.into());
    }

    if !params.parent.filepath.starts_with(&params.collection.path) {
        log::error!(
            "The path: '{}' is not within the collection: '{}'",
            params.parent.filepath.display(),
            params.collection.path.display()
        );
        log::error!("Params: {:?}", params);

        return Err(RelynxError::CreateNewGroupGeneric.into());
    }

    if params
        .parent
        .children
        .iter()
        .any(|child| child.name == folder_name)
    {
        log::error!("There exists already a node with the same name in the parent");
        log::error!("Params: {:?}", params);
        return Err(RelynxError::GroupNameAlreadyExistsInParent(folder_name).into());
    }

    let parent_path = params.parent.filepath;
    if !parent_path.exists() {
        log::error!("The parent's path of the new group does not exist or has been removed. Cannot create new folder in nonexisting directory. Path: '{}'",
parent_path.display()
        );
        return Err(RelynxError::CreateNewGroupGeneric.into());
    }

    let path = parent_path.join(folder_name);
    if path.exists() {
        log::error!(
            "There exists already a file/folder with path: {}",
            path.display()
        );
        return Err(
            RelynxError::GroupFolderAlreadyExists(path.to_string_lossy().to_string()).into(),
        );
    }

    match std::fs::create_dir(&path) {
        Ok(()) => Ok(RequestTreeNode::new_group(GroupOptions::FullPath(path))),
        Err(err) => {
            log::error!(
                "Add group node, could not create new folder: '{}'",
                path.display()
            );
            log::error!("Io Error: {:?}", err);
            Err(RelynxError::CreateNewGroupError(path.to_string_lossy().to_string()).into())
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

    let sanitized_name = sanitize_filename_with_options(&new_name, DEFAULT_OPTIONS);

    let new_path = old_path
        .parent()
        .map(|parent_path| parent_path.join(&sanitized_name))
        .ok_or_else(|| RelynxError::InvalidGroupName(new_name.clone(), sanitized_name.clone()))?;

    Ok(ValidateGroupNameResult {
        sanitized_name,
        path_exists_already: new_path.exists(),
        new_path,
    })
}

#[derive(Serialize, Deserialize, rspc::Type, Debug, Clone)]
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
    } = params.clone();

    let new_name_sanitized = sanitize_filename_with_options(&new_name, DEFAULT_OPTIONS);

    let new_path = old_path
        .parent()
        .map(|parent| parent.join(&new_name_sanitized))
        .ok_or_else(|| {
            log::error!(
                "Could not rename group as path join does not work, new_name: {:?}, sanitized: {:?}",
                &new_name,
                &new_name_sanitized
            );
            log::error!("Params: {:?}", params);
            RelynxError::RenameGroupError(new_name.clone())
        })
        .map_err(Into::<rspc::Error>::into)?;

    if !(old_path.starts_with(&collection_path) && new_path.starts_with(&collection_path)) {
        log::error!("Old or new path not within collection");
        log::error!("new path: {}", new_path.display());
        log::error!("Params: {:?}", params);
        return Err(RelynxError::RenameGroupError(new_name).into());
    }

    if !old_path.exists() {
        log::error!("The group's path: '{}' does not exist", old_path.display());
        log::error!("Params: {:?}", params);
        return Err(RelynxError::RenameGroupError(new_name).into());
    }

    if new_path.exists() {
        log::error!(
            "Cannot rename the group as there exists already a group with path: '{}'",
            new_path.display()
        );
        log::error!("Params: {:?}", params);
        return Err(RelynxError::RenameGroupError(new_name).into());
    }

    std::fs::rename(&old_path, &new_path)
        .map_err(|err| {
            log::error!(
                "Rename Group: Could not rename '{}' to '{}",
                old_path.display(),
                new_path.display()
            );
            log::error!("IO Error: {:?}", err);
            RelynxError::RenameGroupError(new_name)
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

    if node.filepath.as_os_str().is_empty() {
        log::error!(
            "Node filepath is not valid for deletion. collection: '{:?}', node: '{:?}', file_node: '{:?}'",
            collection, node, params.file_node
        );
        return Err(RelynxError::DeleteNodeError.into());
    }

    if !node.filepath.starts_with(&collection.path) {
        log::error!(
            "The node: '{}' with path: '{}' is not within collection: '{}', collectionPath: '{}'",
            node.name,
            node.filepath.display(),
            collection.name,
            collection.path.display()
        );
        return Err(RelynxError::DeleteNodeError.into());
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
            path: Box::new(file_node.filepath.clone()),
        };
        match Serializer::serialize_to_file(&file_model) {
            Ok(_) => return Ok(()),
            Err(err) => {
                log::error!(
                    "Could not remove request: '{}' from file: '{}",
                    node.name,
                    file_node.filepath.display()
                );
                log::error!("Io Error: {:?}", err);
                return Err(Into::<rspc::Error>::into(RelynxError::DeleteNodeError));
            }
        };
    }

    // @TODO: if the node contains multiple children maybe ask if they want to delete everything
    // otherwise we delete either a single request (one file), or a directory with all its children
    if node.request.is_some() {
        std::fs::remove_file(&node.filepath)
        // @TODO: check that only requests/folders are within the group so that nothing wanted is
        // removed as well
    } else {
        std::fs::remove_dir_all(&node.filepath)
    }
    .map_err(|err| {
        log::error!("Could not delete node: {}, err: {}", node.name, err);
        log::error!("Io Error: {:?}", err);
        Into::<rspc::Error>::into(RelynxError::DeleteNodeError)
    })?;

    let mut collection_config =
        load_collection_config(&collection.get_config_file_path()).unwrap_or_default();
    collection_config.path_orders.remove(&node.filepath);
    // here we do not want to return an error as the actual deletion worked, just the config could
    // not be updated which is not critical
    let ignored = save_collection_config(&collection_config, &collection.get_config_file_path());
    if ignored.is_err() {
        log::error!("Could not save collection config after deleting node!");
        log::error!("Io Error: {:?}", ignored.unwrap_err());
    }

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
pub fn get_response_filepath(request_path: PathBuf) -> Result<Option<PathBuf>, rspc::Error> {
    let request_folder = request_path.parent().ok_or({
        log::error!(
            "Could not resolve response path relative to request. request_path: {}",
            request_path.display()
        );
        RelynxError::RelativeResponsePathError
    })?;

    let mut file_dialog_builder = tauri::api::dialog::blocking::FileDialogBuilder::default();
    if let Some(parent) = request_path.parent() {
        file_dialog_builder = file_dialog_builder.set_directory(parent);
    }
    let filepath = file_dialog_builder.save_file();
    // canceled / none chosen
    if filepath.is_none() {
        return Ok(None);
    }
    let filepath = filepath.unwrap();

    filepath
        .strip_prefix(request_folder)
        .map(|res| Some(res.to_owned()))
        .map_err(|err| {
            log::error!("Filepath is not relative could not strip prefix. Filepath: '{}', request_folder: '{}'", filepath.display(), request_folder.display());
            log::error!("Err: {:?}", err);
            RelynxError::RelativeResponsePathError
        })
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
        log::error!(
            "Could not hide group, path exists: {}, path is dir: {}",
            path.exists(),
            path.is_dir()
        );

        return Err(RelynxError::HideGroupError.into());
    }

    let ignorefile_path = path.join(RELYNX_IGNORE_FILE);
    std::fs::write(ignorefile_path, "").map_err(|err| {
        log::error!("Could not write relynx ignore file for hiding groups to file system!");
        log::error!("Io Error: {:?}", err);
        Into::<rspc::Error>::into(RelynxError::HideGroupError)
    })?;

    Ok(())
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct ChooseFileRelativeToParams {
    base_path: PathBuf,
}

#[tauri::command]
pub fn choose_file_relative_to(
    params: ChooseFileRelativeToParams,
) -> Result<Option<PathBuf>, rspc::Error> {
    let base_path_folder = if params.base_path.is_dir() {
        Some(params.base_path.as_path())
    } else {
        params.base_path.parent().to_owned()
    }
    .unwrap();
    let chosen_file = select_file(Some(&base_path_folder))?;
    // nothing chosen, cancelled
    if chosen_file.is_none() {
        return Ok(None);
    }
    let chosen_file = chosen_file.unwrap();

    if let Some(relative_path) = diff_paths(&chosen_file, &base_path_folder) {
        Ok(Some(relative_path))
    } else {
        log::error!(
            "Could not create relative file paths for chosen_file: '{}', params.base_path: '{}', basepath_folder: '{}'",
            chosen_file.display(),
            params.base_path.display(),
            base_path_folder.display()
        );
        return Err(RelynxError::RelativePathChoiceError(
            chosen_file.to_string_lossy().to_string(),
            params.base_path.to_string_lossy().to_string(),
        )
        .into());
    }
}

#[tauri::command]
pub fn load_license_data_command() -> Result<LicenseData, rspc::Error> {
    license::get_license_data().map_err(Into::into)
}

#[tauri::command]
pub fn save_license_data_command(params: &LicenseData) -> Result<(), rspc::Error> {
    license::save_license_data(params).map_err(Into::into)
}

#[tauri::command]
pub fn is_signature_valid(license_data: &LicenseData) -> Result<bool, rspc::Error> {
    verify_signature(license_data).map_err(Into::into)
}

#[tauri::command]
pub fn get_app_environment() -> Result<AppEnvironment, rspc::Error> {
    if cfg!(debug_assertions) {
        Ok(AppEnvironment::Development)
    } else {
        Ok(AppEnvironment::Production)
    }
}

#[tauri::command]
pub fn get_log_path_command() -> Result<PathBuf, rspc::Error> {
    get_log_filepath().ok_or(RelynxError::LogFolderMissing.into())
}

#[tauri::command]
pub fn copy_logfile_content_to_clipboard() -> Result<(), rspc::Error> {
    let logpath =
        get_log_filepath().ok_or(Into::<rspc::Error>::into(RelynxError::LogFolderMissing))?;
    let log_content = std::fs::read_to_string(logpath).map_err(|err| {
        log::error!("Cannot open log file!");
        log::error!("Err: {:?}", err);
        Into::<rspc::Error>::into(RelynxError::LogFolderMissing)
    })?;
    copy_to_clipboard(log_content)
}

#[tauri::command]
pub fn get_cookie_jar_command(params: GetCookieJarParams) -> Result<CookieJar, rspc::Error> {
    dbg!(
        load_cookie_jar(&CookieJarPath::CollectionFolderPath(params.collection.path))
            .map_err(Into::<rspc::Error>::into)
    )
}

#[tauri::command]
pub fn save_cookie_jar_command(params: SaveCookieJarParams) -> Result<(), rspc::Error> {
    let path = if let Some(ref path) = params.cookie_jar.path {
        CookieJarPath::CookieJarFilePath(path.clone())
    } else {
        CookieJarPath::CollectionFolderPath(params.collection.path)
    };
    save_cookie_jar(path, &params.cookie_jar).map_err(Into::<rspc::Error>::into)
}
