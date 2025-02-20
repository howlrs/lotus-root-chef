use log::{error, trace};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::{JoinError, JoinHandle};

use serde_json::{json, Value};

use tauri::State;

use crate::funcs::task::Log;
use crate::target::exchange::ExchangeName;

mod board;
mod funcs;
mod target;

struct AppState {
    controller: funcs::task::Controller,

    workers: Option<Workers>,

    logger: Option<Arc<RwLock<funcs::task::Logger>>>,
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
                        trace!("error: {:?}", e);
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
) -> Result<funcs::task::Controller, Value> {
    let (cloned_controller, cloned_logger) = {
        let mut w = state.write().await;
        if !w.controller.is_ok() {
            return Err(err_response_handler(
                "controller is not ok, please check value",
                json!(w.controller).as_str().unwrap(),
            ));
        }

        // すでに実行してるWorkerがあれば停止
        if let Some(mut workers) = w.workers.take() {
            println!("workers[{}] is done, abort_all", workers.handles.len());
            workers.abort_all().await.unwrap();
            w.workers = None;
        }

        let set_log = Some(Log {
            level: "info".to_string(),
            message: "start_controller".to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
        });
        let logger = Arc::new(RwLock::new(funcs::task::Logger::new(set_log.clone())));
        w.logger = Some(logger.clone());

        (Arc::new(RwLock::new(w.controller.clone())), logger.clone())
    };

    let handles = funcs::task::runner(cloned_controller.clone(), cloned_logger.clone())
        .await
        .unwrap();

    // worker
    let mut workers = Workers::new();
    workers.extend(handles);

    let mut w = state.write().await;
    w.workers = Some(workers);
    let mut controller = w.controller.clone();
    drop(w);

    controller.is_running = true;
    Ok(controller)
}

#[tauri::command]
async fn stop_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<funcs::task::Controller, Value> {
    // workers
    let mut w = state.write().await;
    let mut workers = match w.workers.take() {
        Some(v) => v,
        None => {
            return Err(err_response_handler(
                "workers is not found, please ",
                "runner is not running, please start runner",
            ));
        }
    };
    w.workers = None;
    let mut controller = w.controller.clone();
    drop(w);

    // abort all workers
    match workers.abort_all().await {
        Ok(v) => {
            trace!("done: {:?}", v);
            controller.is_running = false;
            Ok(controller)
        }
        Err(e) => {
            error!("error: {:?}", e);
            Err(err_response_handler(
                "abort is failed, workers is not found",
                &e.to_string(),
            ))
        }
    }
}

#[tauri::command]
async fn post_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
    value: Value,
) -> Result<funcs::task::Controller, Value> {
    // value bind to Controller
    let controller: funcs::task::Controller = match serde_json::from_value(value.clone()) {
        Ok(v) => v,
        Err(e) => {
            return Err(err_response_handler(
                "controller is invalid",
                &e.to_string(),
            ));
        }
    };

    if !controller.is_ok() {
        // 更新が行われない場合はエラー
        // - controllerが実行中の場合
        if controller.is_running {
            return Err(err_response_handler(
                "controller is running, please stop controller",
                "runner is running, please stop_controller",
            ));
        } else {
            // - controllerが不正な値の場合
            return Err(err_response_handler(
                "controller is not update, please check value",
                &value.to_string(),
            ));
        }
    }

    let mut w = state.write().await;
    w.controller = controller.clone();

    Ok(controller)
}

#[tauri::command(rename_all = "snake_case")]
async fn get_controller(state: State<'_, Arc<RwLock<AppState>>>) -> Result<Value, Value> {
    let r = state.read().await;
    let controller = r.controller.clone();

    Ok(json!(controller))
}

#[tauri::command]
async fn put_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
    value: Value,
) -> Result<funcs::task::Controller, Value> {
    // value bind to Controller
    let controller: funcs::task::Controller = serde_json::from_value(value).unwrap();
    trace!("{:?}", controller);

    let mut w = state.write().await;
    w.controller = controller.clone();

    Ok(controller)
}

#[tauri::command]
async fn delete_controller(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<funcs::task::Controller, Value> {
    let controller = funcs::task::Controller::default();
    let mut w = state.write().await;
    w.controller = controller.clone();

    Ok(controller)
}

#[tauri::command(rename_all = "snake_case")]
async fn get_instruments(
    exchange_name: String,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<Value, Value> {
    let r = state.read().await;

    let instruments: Vec<target::exchanges::models::Instrument> =
        match r.controller.exchange.instruments().await {
            Ok(v) => v,
            Err(e) => {
                return Err(err_response_handler(
                    "instruments is not found",
                    &e.to_string(),
                ))
            }
        };

    Ok(json!(instruments))
}

#[tauri::command(rename_all = "snake_case")]
async fn get_ticker(
    mut exchange_name: String,
    symbol: String,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<Value, Value> {
    let r = state.read().await;

    if exchange_name.is_empty() {
        exchange_name = r.controller.exchange.name.as_str().to_string();
    }

    let ticker: target::exchanges::models::Ticker = match r
        .controller
        .exchange
        .ticker_info(ExchangeName::from(exchange_name), symbol.clone())
        .await
    {
        Ok(v) => v,
        Err(e) => return Err(err_response_handler("ticker is not found", &e.to_string())),
    };

    Ok(json!(ticker))
}

#[tauri::command]
async fn get_logger(state: State<'_, Arc<RwLock<AppState>>>) -> Result<Value, Value> {
    let logger = {
        let r = state.write().await;
        r.logger.clone()
    };

    match logger {
        Some(logger_arc) => {
            let mut guarded_logger = logger_arc.write().await;
            let send_logger = guarded_logger.clone();
            guarded_logger.clear();

            Ok(json!(send_logger.log))
        }
        None => Err(json!(funcs::task::Logger::new(None))),
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
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let use_state = Arc::new(RwLock::new(AppState {
        controller: funcs::task::Controller::default(),
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

fn err_response_handler(msg: &str, cause: &str) -> Value {
    json!({"msg": msg, "cause": cause})
}
