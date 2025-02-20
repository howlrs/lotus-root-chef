use log::{debug, error};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::{JoinError, JoinHandle};

use serde_json::{json, Value};

use tauri::State;

use crate::funcs::client;
use crate::funcs::utils;
use crate::target::exchange::{get_rest_instruments, get_rest_ticker_info, ExchangeName};

mod board;
mod funcs;
mod target;

struct AppState {
    controller: client::Controller,

    workers: Option<Workers>,

    logger: Option<Arc<RwLock<client::Logger>>>,
}

struct Workers {
    // 返り値を持たない非同期タスクのハンドル
    // .abort() でキャンセル可能
    // .await で終了待ち
    // spawn内で.awaitに対してキャンセル命令を送ることで終了させる
    handles: Vec<JoinHandle<()>>,
}

impl Workers {
    fn new() -> Self {
        Workers {
            handles: Vec::new(),
        }
    }

    fn extend(&mut self, handles: Vec<JoinHandle<()>>) {
        self.handles.extend(handles);
    }

    async fn abort_all(&mut self) -> Result<(), JoinError> {
        for handle in self.handles.drain(..) {
            handle.abort();
            let _ = match handle.await {
                Ok(_) => Ok(()),
                // JoinError::Cancelledはabort()による正常な終了
                Err(e) => {
                    if e.is_cancelled() {
                        Ok(())
                    } else {
                        error!("error: {:?}", e);
                        Err(e)
                    }
                }
            };
        }

        Ok(())
    }
}

#[tauri::command]
async fn start_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<client::Controller, Value> {
    let (cloned_controller, cloned_logger) = {
        let mut w = state.write().await;
        match w.controller.ok() {
            Ok(_) => (),
            Err(e) => {
                return Err(utils::err_response_handler(
                    "controller is not ok, please check value",
                    e,
                ));
            }
        }

        // すでに実行してるWorkerがあれば停止
        if let Some(mut workers) = w.workers.take() {
            debug!("workers[{}] is done, abort_all", workers.handles.len());
            workers.abort_all().await.unwrap();
            w.workers = None;
        }

        let set_log = Some(client::Log {
            level: "info".to_string(),
            message: "start_controller".to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
        });
        let logger = Arc::new(RwLock::new(client::Logger::new(set_log.clone())));
        w.logger = Some(logger.clone());

        (Arc::new(RwLock::new(w.controller.clone())), logger.clone())
    };

    let handles = funcs::task::runner(cloned_controller.clone(), cloned_logger.clone())
        .await
        .unwrap();

    // worker
    let mut workers = Workers::new();
    workers.extend(handles);
    let workers = workers;

    let mut controller = {
        let mut w = state.write().await;
        w.workers = Some(workers);
        w.controller.clone()
    };

    controller.is_running = true;
    Ok(controller)
}

#[tauri::command]
async fn stop_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<client::Controller, Value> {
    // workers
    let (mut controller, mut workers) = {
        let mut w = state.write().await;
        let workers = match w.workers.take() {
            Some(v) => v,
            None => {
                return Err(utils::err_response_handler(
                    "workers is not found, please ",
                    "runner is not running, please start runner",
                ));
            }
        };
        w.workers = None;
        (w.controller.clone(), workers)
    };

    // abort all workers
    match workers.abort_all().await {
        Ok(_) => {
            controller.is_running = false;
            Ok(controller)
        }
        Err(e) => Err(utils::err_response_handler(
            "abort is failed, workers is not found",
            &e.to_string(),
        )),
    }
}

#[tauri::command]
async fn post_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
    value: Value,
) -> Result<Value, Value> {
    // value bind to Controller
    let controller: client::Controller = match serde_json::from_value(value.clone()) {
        Ok(v) => v,
        Err(e) => {
            return Err(utils::err_response_handler(
                "controller is invalid",
                &e.to_string(),
            ));
        }
    };

    match controller.ok() {
        Ok(_) => (),
        Err(e) => {
            return Err(utils::err_response_handler(
                "controller is not ok, please check value",
                e,
            ));
        }
    }

    {
        let mut w = state.write().await;
        w.controller = controller.clone();
    }

    Ok(json!(controller))
}

#[tauri::command(rename_all = "snake_case")]
async fn get_controller(state: State<'_, Arc<RwLock<AppState>>>) -> Result<Value, Value> {
    let controller = {
        let r = state.read().await;
        r.controller.clone()
    };

    Ok(json!(controller))
}

#[tauri::command]
async fn put_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
    value: Value,
) -> Result<Value, Value> {
    // value bind to Controller
    let controller: client::Controller = serde_json::from_value(value).unwrap();
    debug!("put data: {:?}", controller);

    {
        let mut w = state.write().await;
        w.controller = controller.clone();
    }

    Ok(json!(controller))
}

#[tauri::command]
async fn delete_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<client::Controller, Value> {
    let controller = client::Controller::default();
    let mut w = state.write().await;
    w.controller = controller.clone();

    Ok(controller)
}

#[tauri::command(rename_all = "snake_case")]
async fn get_instruments(exchange_name: ExchangeName) -> Result<Value, Value> {
    let instruments = match get_rest_instruments(exchange_name.clone()).await {
        Ok(v) => v,
        Err(e) => {
            return Err(utils::err_response_handler(
                "instruments is not found",
                &e.to_string(),
            ))
        }
    };

    Ok(json!(instruments))
}

#[tauri::command(rename_all = "snake_case")]
async fn get_ticker(exchange_name: ExchangeName, symbol: String) -> Result<Value, Value> {
    let ticker = match get_rest_ticker_info(exchange_name.clone(), symbol.clone()).await {
        Ok(v) => v,
        Err(e) => {
            return Err(utils::err_response_handler(
                "ticker is not found",
                &e.to_string(),
            ))
        }
    };

    Ok(json!(ticker))
}

#[tauri::command]
async fn get_logger(state: State<'_, Arc<RwLock<AppState>>>) -> Result<Value, Value> {
    let read_logger = {
        let r = state.read().await;
        r.logger.clone()
    };

    match read_logger {
        Some(origin_logger) => {
            let mut w_logger = origin_logger.write().await;
            let send_logger = w_logger.clone();
            w_logger.clear();
            let mut w = state.write().await;
            w.logger = Some(origin_logger.clone());

            Ok(json!(send_logger.log))
        }
        None => Err(json!(funcs::client::Logger::new(None))),
    }
}

#[tauri::command]
async fn clear_logger(state: State<'_, Arc<RwLock<AppState>>>) -> Result<(), Value> {
    let mut w = state.write().await;
    w.logger = None;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::init_logger("output.log");

    let use_state = Arc::new(RwLock::new(AppState {
        controller: client::Controller::default(),
        workers: None,
        logger: None,
    }));

    let app_use_state = use_state.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_use_state)
        .invoke_handler(tauri::generate_handler![
            start_controller,
            stop_controller,
            post_controller,
            get_controller,
            put_controller,
            delete_controller,
            get_instruments,
            get_ticker,
            get_logger,
            clear_logger
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
