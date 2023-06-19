// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod client;
mod commands;
mod config;
mod environment;
mod error;
mod import;
mod model;
mod sanitize;
mod serialize;
mod tree;

use commands::{
    add_existing_collections, add_group_node, add_request_node, validate_response_filepath,
    copy_to_clipboard, delete_node, drag_and_drop, get_response_filepath,
    import_postman_collection, is_directory_empty, load_environments, load_requests_for_collection,
    load_workspace, open_folder_native, remove_collection, reorder_nodes_within_parent,
    run_request, save_environments, save_request, select_directory, select_file, update_workspace,
    AddExistingCollectionsParams, AddGroupNodeParams, AddRequestNodeParams, DeleteNodeParams,
    DragAndDropParams, ImportPostmanCommandParams, ReorderNodesParams, SaveEnvironmentsParams,
    RELYNX_CONTEXT,
};
use model::{Collection, RunRequestCommand, SaveRequestCommand, Workspace};
use rspc::Router;
use std::{path::PathBuf, sync::Arc};
use tauri::Manager;
use tauri_plugin_log::LogTarget;

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
        .build();
    Arc::new(router)
}

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
