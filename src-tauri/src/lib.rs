use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::funcs::client;
use crate::funcs::utils;

mod api;
mod board;
mod funcs;
mod target;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::init_logger("output.log");

    let use_state = Arc::new(RwLock::new(api::invokers::AppState {
        controller: client::Controller::default(),
        workers: None,
        logger: None,
    }));

    let app_use_state = use_state.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_use_state)
        .invoke_handler(tauri::generate_handler![
            api::invokers::start_controller,
            api::invokers::stop_controller,
            api::invokers::post_controller,
            api::invokers::get_controller,
            api::invokers::put_controller,
            api::invokers::delete_controller,
            api::invokers::get_instruments,
            api::invokers::get_ticker,
            api::invokers::get_logger,
            api::invokers::clear_logger
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
