use std::fs::File;

use env_logger::{Builder, Target};
use log::error;
use serde_json::{json, Value};

pub fn err_response_handler(msg: &str, cause: &str) -> Value {
    error!("response error: {}: {}", msg, cause);
    json!({"msg": msg, "cause": cause})
}

pub fn init_logger(output_filepath: &str) {
    let is_test = std::env::var("IS_TEST").unwrap_or_default() == "true";

    // 環境変数 RUST_LOG の値を取得（未指定の場合は "debug" と仮定）
    let log_level = std::env::var("RUST_LOG").unwrap_or("debug".to_string());
    let mut builder =
        Builder::from_env(env_logger::Env::default().default_filter_or(log_level.clone()));

    if log_level.eq_ignore_ascii_case("error") {
        println!("RUST_LOG is error");
        // RUST_LOG が "error" の場合、ファイル出力を設定
        let file = File::create(output_filepath).expect("ログファイルの作成に失敗しました");
        // env_logger では Box<dyn Write + Send> としてファイルを指定
        builder.target(Target::Pipe(Box::new(file)));
    } else {
        println!("RUST_LOG is {}", log_level);
        // それ以外はコンソール出力（Stdout）を使用
        builder.target(Target::Stdout);
    }

    // Builder の設定後にログ初期化を実施
    builder.init();

    // ログ出力のテスト
    log::error!("error message, is_test: {}", is_test);
    log::warn!("warn message, is_test: {}", is_test);
    log::info!("info message, is_test: {}", is_test);
    log::debug!("debug message, is_test: {}", is_test);
    log::trace!("trace message, is_test: {}", is_test);
}
