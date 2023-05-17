// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;
mod model;

use model::Workspace;
use rspc::Router;
use std::sync::Arc; // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use tauri_plugin_log::LogTarget;

#[tauri::command]
fn load_workspace() -> Result<Workspace, rspc::Error> {
    config::load_workspace().map_err(Into::into)
}

struct Context;

fn router() -> Arc<Router<Context>> {
    let router = Router::new()
        // change the bindings filename to your liking
        .config(rspc::Config::new().export_ts_bindings("../src/bindings.d.ts"))
        //.query("greet", |t| t(|_, name: String| greet(&name)))
        .query("load_workspace", |t| t(|_, ()| load_workspace()))
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
