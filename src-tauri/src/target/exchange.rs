use serde::{Deserialize, Serialize};
use tokio::task::{JoinError, JoinHandle};

use crate::target::exchanges::{
    bybit,
    models::{Orderboard, Position, Ticker},
};

use crate::target::exchanges;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum ExchangeName {
    // default
    #[serde(rename = "bybit")]
    #[default]
    Bybit,
    #[serde(rename = "bitbank")]
    Bitbank,
    #[serde(rename = "bitflyer")]
    Bitflyer,
}

impl ExchangeName {
    pub fn as_str(&self) -> &str {
        match self {
            ExchangeName::Bybit => "bybit",
            ExchangeName::Bitbank => "bitbank",
            ExchangeName::Bitflyer => "bitflyer",
        }
    }
}

impl From<&str> for ExchangeName {
    fn from(s: &str) -> Self {
        let binding = s.to_lowercase();
        let s = binding.as_str();
        match s {
            "bybit" => ExchangeName::Bybit,
            "bitbank" => ExchangeName::Bitbank,
            "bitflyer" => ExchangeName::Bitflyer,
            _ => ExchangeName::Bybit,
        }
    }
}

impl From<String> for ExchangeName {
    fn from(s: String) -> Self {
        let binding = s.to_lowercase();
        let s = binding.as_str();
        match s {
            "bybit" => ExchangeName::Bybit,
            "bitbank" => ExchangeName::Bitbank,
            "bitflyer" => ExchangeName::Bitflyer,
            _ => ExchangeName::Bybit,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: ExchangeName,
    pub key: String,
    pub secret: String,
    pub passphrase: Option<String>,
    pub category: Option<String>,
}

impl Config {
    pub fn new(
        name: ExchangeName,
        key: String,
        secret: String,
        passphrase: Option<String>,
    ) -> Self {
        Config {
            name,
            key,
            secret,
            passphrase,
            category: None,
        }
    }

    pub fn is_ok(&self) -> bool {
        match self.name {
            ExchangeName::Bybit => {
                if self.key.is_empty() || self.secret.is_empty() {
                    return false;
                }
            }
            ExchangeName::Bitbank => {
                if self.key.is_empty() || self.secret.is_empty() {
                    return false;
                }
            }
            ExchangeName::Bitflyer => {
                if self.key.is_empty() || self.secret.is_empty() {
                    return false;
                }
            }
        }

        true
    }

    pub async fn ticker(
        &self,
        exchange: ExchangeName,
        symbol: String,
        tx_ws: tokio::sync::mpsc::Sender<Ticker>,
        rx_rest: tokio::sync::mpsc::Receiver<()>,
        tx_rest: tokio::sync::broadcast::Sender<Ticker>,
    ) -> Result<JoinHandle<()>, JoinError> {
        let cloned_tx_ws = tx_ws.clone();
        let cloned_tx_rest = tx_rest.clone();

        let handle = match exchange {
            ExchangeName::Bybit => {
                let category = "linear".to_string();
                let symbol = symbol.clone();
                let by = bybit::BybitClient::new(None, None, category, symbol);
                by.public_ticker(cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
            ExchangeName::Bitbank => {
                // [TODO]
                let category = "linear".to_string();
                let symbol = symbol.clone();
                let by = bybit::BybitClient::new(None, None, category, symbol);
                by.public_ticker(cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
            ExchangeName::Bitflyer => {
                // [TODO]
                let category = "linear".to_string();
                let symbol = symbol.clone();
                let by = bybit::BybitClient::new(None, None, category, symbol);
                by.public_ticker(cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
        };

        Ok(handle)
    }

    pub async fn orderboard(
        &self,
        exchange: ExchangeName,
        symbol: String,
        tx_ws: tokio::sync::mpsc::Sender<Orderboard>,
        rx_rest: tokio::sync::mpsc::Receiver<()>,
        tx_rest: tokio::sync::broadcast::Sender<Orderboard>,
    ) -> Result<JoinHandle<()>, JoinError> {
        let cloned_tx_ws = tx_ws.clone();
        let cloned_tx_rest = tx_rest.clone();

        let handle = match exchange {
            ExchangeName::Bybit => {
                let category = "linear".to_string();
                let symbol = symbol.clone();
                let depth = 500;
                let by = bybit::BybitClient::new(None, None, category, symbol);
                by.public_orderboard(Some(depth), cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
            ExchangeName::Bitbank => {
                // [TODO]
                let category = "linear".to_string();
                let symbol = symbol.clone();
                let depth = 500;
                let by = bybit::BybitClient::new(None, None, category, symbol);
                by.public_orderboard(Some(depth), cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
            ExchangeName::Bitflyer => {
                // [TODO]
                let category = "linear".to_string();
                let symbol = symbol.clone();
                let depth = 500;
                let by = bybit::BybitClient::new(None, None, category, symbol);
                by.public_orderboard(Some(depth), cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
        };

        Ok(handle)
    }

    pub async fn position(
        &self,
        exchange: ExchangeName,
        symbol: String,
        tx_ws: tokio::sync::mpsc::Sender<Vec<Position>>,
        rx_rest: tokio::sync::mpsc::Receiver<()>,
        tx_rest: tokio::sync::broadcast::Sender<Vec<Position>>,
    ) -> Result<JoinHandle<()>, JoinError> {
        let cloned_tx_ws = tx_ws.clone();
        let cloned_tx_rest = tx_rest.clone();

        let handle = match exchange {
            ExchangeName::Bybit => {
                let category = self.category.clone().unwrap_or("spot".to_string());
                let symbol = symbol.clone();
                let key = self.key.clone();
                let secret = self.secret.clone();
                let by = bybit::BybitClient::new(Some(key), Some(secret), category, symbol);
                by.private_position(cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
            ExchangeName::Bitbank => {
                // [TODO]
                // rx_rest通知を受けて、REST APIで取得
                // tx_restに送信
                let category = "linear".to_string();
                let symbol = symbol.clone();
                let by = bybit::BybitClient::new(None, None, category, symbol);
                by.private_position(cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
            ExchangeName::Bitflyer => {
                // [TODO]
                let category = "linear".to_string();
                let symbol = symbol.clone();
                let by = bybit::BybitClient::new(None, None, category, symbol);
                by.private_position(cloned_tx_ws, rx_rest, cloned_tx_rest)
                    .await
                    .unwrap()
            }
        };

        Ok(handle)
    }

    pub async fn instruments(&self) -> Result<Vec<exchanges::models::Instrument>, String> {
        match self.name {
            ExchangeName::Bybit => {
                match exchanges::bybit::instruments("linear".to_string()).await {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e.to_string()),
                }
            }
            ExchangeName::Bitbank => Ok(vec![]),
            ExchangeName::Bitflyer => Ok(vec![]),
        }
    }

    pub async fn ticker_info(
        &self,
        exchange_name: ExchangeName,
        symbol: String,
    ) -> Result<exchanges::models::Ticker, String> {
        match exchange_name {
            ExchangeName::Bybit => {
                match exchanges::bybit::ticker("linear".to_string(), symbol).await {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e.to_string()),
                }
            }
            ExchangeName::Bitbank => Ok(exchanges::models::Ticker::default()),
            ExchangeName::Bitflyer => Ok(exchanges::models::Ticker::default()),
        }
    }
}
