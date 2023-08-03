// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod client;
mod commands;
mod config;
mod cookie_jar;
mod environment;
mod error;
mod import;
mod license;
mod model;
mod pathdiff;
mod runner;
mod sanitize;
mod tree;

use commands::{
    add_existing_collections, add_group_node, add_request_node, choose_file_relative_to,
    copy_logfile_content_to_clipboard, copy_to_clipboard, delete_node, drag_and_drop,
    get_app_environment, get_cookie_jar_command, get_log_path_command, get_response_filepath,
    hide_group, import_jetbrains_folder_command, import_postman_collection, is_directory_empty,
    is_signature_valid, load_environments, load_license_data_command, load_requests_for_collection,
    load_workspace, open_folder_native, remove_collection, rename_group,
    reorder_nodes_within_parent, run_request, save_cookie_jar_command, save_environments,
    save_license_data_command, save_request, select_directory, select_file, update_workspace,
    validate_group_name, validate_response_filepath, AddExistingCollectionsParams,
    AddGroupNodeParams, AddRequestNodeParams, ChooseFileRelativeToParams, DeleteNodeParams,
    DragAndDropParams, ImportJetbrainsHttpFolderParams, ImportPostmanCommandParams,
    RenameGroupParams, ReorderNodesParams, SaveEnvironmentsParams, ValidateGroupNameParams,
    RELYNX_CONTEXT,
};
use config::get_data_dir;
use cookie_jar::{GetCookieJarParams, SaveCookieJarParams};
use license::LicenseData;
use log::LevelFilter;
use model::{Collection, RunRequestCommand, SaveRequestCommand, Workspace};
use rspc::Router;
use std::{path::PathBuf, sync::Arc};
use tauri::Manager;
use tauri_plugin_log::LogTarget;

static mut LICENSE_PUB_KEY: String = String::new();
static mut LICENSE_PRIV_KEY: String = String::new();

#[allow(clippy::result_unit_err)]
pub fn get_license_pub_key() -> Result<String, ()> {
    let key = unsafe { LICENSE_PUB_KEY.clone() };
    if key.is_empty() {
        return Err(());
    }
    Ok(key)
}

pub fn get_license_priv_key() -> String {
    unsafe { LICENSE_PRIV_KEY.clone() }
}

fn router() -> Arc<Router> {
    let router =
        Router::new()
            // change the bindings filename to your liking
            .config(rspc::Config::new().export_ts_bindings("../src/bindings.d.ts"))
            //.query("greet", |t| t(|_, name: String| greet(&name)))
            .query("load_workspace", |t| t(|_, ()| load_workspace()))
            .query("remove_collection", |t| {
                t(|_, collection: Collection| remove_collection(collection))
            })
            .query("select_directory", |t| t(|_, ()| select_directory()))
            .query("select_file", |t| t(|_, ()| select_file(None)))
            .query("is_directory_empty", |t| {
                t(|_, path: PathBuf| is_directory_empty(path))
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
            .query("import_jetbrains_folder", |t| {
                t(|_, params: ImportJetbrainsHttpFolderParams| {
                    import_jetbrains_folder_command(params)
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
                t(|_, path: PathBuf| {
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
            .query("load_environments", |t| {
                t(|_, collection_path: PathBuf| load_environments(collection_path))
            })
            .query("save_environments", |t| {
                t(|_, params: SaveEnvironmentsParams| save_environments(params))
            })
            .query("get_response_filepath", |t| {
                t(|_, params: PathBuf| get_response_filepath(params))
            })
            .query("validate_response_filepath", |t| {
                t(|_, params: PathBuf| validate_response_filepath(params))
            })
            .query("validate_group_name", |t| {
                t(|_, params: ValidateGroupNameParams| validate_group_name(params))
            })
            .query("rename_group", |t| {
                t(|_, params: RenameGroupParams| rename_group(params))
            })
            .query("hide_group", |t| t(|_, params: PathBuf| hide_group(params)))
            .query("choose_file_relative_to", |t| {
                t(|_, params: ChooseFileRelativeToParams| choose_file_relative_to(params))
            })
            .query("load_license_data", |t| {
                t(|_, ()| load_license_data_command())
            })
            .mutation("save_license_data", |t| {
                t(|_, params: LicenseData| save_license_data_command(&params))
            })
            .query("is_signature_valid", |t| {
                t(|_, params: LicenseData| is_signature_valid(&params))
            })
            .query("get_app_environment", |t| t(|_, ()| get_app_environment()))
            .query("get_log_path", |t| t(|_, ()| get_log_path_command()))
            .query("copy_logfile_content_to_clipboard", |t| {
                t(|_, ()| copy_logfile_content_to_clipboard())
            })
            .query("get_cookie_jar", |t| {
                t(|_, params: GetCookieJarParams| get_cookie_jar_command(params))
            })
            .mutation("save_cookie_jar", |t| {
                t(|_, params: SaveCookieJarParams| save_cookie_jar_command(params))
            })
            .build();
    Arc::new(router)
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let context = tauri::generate_context!();
    let app_builder = tauri::Builder::default();
    let log_level_filter = if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };
    let log_dir = get_data_dir().map(|dir| dir.join("logs"));
    let mut log_targets = vec![LogTarget::Stdout];
    if let Some(log_dir) = log_dir {
        log_targets = vec![LogTarget::Folder(log_dir), LogTarget::Stdout];
    }
    let app = app_builder
        .plugin(rspc::integrations::tauri::plugin(router(), || {}))
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets(log_targets)
                .level(log_level_filter)
                .build(),
        )
        .build(context)
        .expect("error while running tauri application");

    // @TODO: This is a workaround to access app_handle within router. Maybe with later rspc version we
    // can do this (https://github.com/oscartbeaumont/rspc/issues/163)
    let mut data = RELYNX_CONTEXT.lock().unwrap();
    data.app_handle = Some(app.app_handle());
    std::mem::drop(data);

    let pub_key_path = if cfg!(debug_assertions) {
        "resources/dev.pub_key.pem"
    } else {
        "resources/prod.pub_key.pem"
    };
    if cfg!(debug_assertions) {
        let settings_resource_path = app
            .path_resolver()
            .resolve_resource(pub_key_path)
            .expect("failed to resolve resource for public key");
        let content = std::fs::read_to_string(settings_resource_path).unwrap_or(String::new());
        unsafe {
            LICENSE_PUB_KEY = content;
        }
    } else {
        // @TODO:
    }
    //
    app.run(|_, _| {});
}
