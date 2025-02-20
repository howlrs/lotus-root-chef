use log::{error, info, log_enabled};
use serde::{Deserialize, Serialize};

use crate::{
    board,
    target::{exchange, order},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Controller {
    pub is_running: bool,
    // 対象取引所
    pub exchange: exchange::Config,
    // 対象板
    pub board: board::filter::Config,
    // 対象取引
    pub order: order::Config,
}

impl Controller {
    pub fn ok(&self) -> Result<(), &str> {
        if self.is_running {
            Err("already running")
        } else if !self.exchange.is_ok() {
            Err("exchange setting is empty")
        } else if !self.board.is_ok() {
            Err("board setting is empty")
        } else if !self.order.is_ok() {
            Err("order setting is empty")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Logger {
    pub log: Vec<Log>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Log {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

impl Logger {
    pub fn new(log: Option<Log>) -> Self {
        match log {
            Some(log) => Logger { log: vec![log] },
            None => Logger { log: vec![] },
        }
    }

    pub fn add(&mut self, log: Log) {
        if log_enabled!(log::Level::Info) {
            match log.level.as_str() {
                "info" => info!("{}", log.message),
                "error" => error!("{}", log.message),
                "success" => info!("{}", log.message),
                _ => (),
            }
        }

        self.log.push(log);
    }

    pub fn clear(&mut self) {
        self.log.clear();
    }
}
