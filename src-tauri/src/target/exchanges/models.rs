use serde::{Deserialize, Serialize};

use crate::{
    board::book::Book,
    target::{
        exchange::{Config, ExchangeName},
        exchanges::bybit::BybitClient,
    },
};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum BookSide {
    #[default]
    #[serde(rename = "bid")]
    Bid,
    #[serde(rename = "ask")]
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum OrderSide {
    #[default]
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

#[derive(Debug, Clone, Default)]
pub struct OrderParams {
    pub order_id: Option<String>,
    pub side: OrderSide,
    pub price: f64,
    pub qty: f64,
    pub is_post_only: bool,
}

pub trait OrderClient {
    fn new_for_order_client(
        key: String,
        secret: String,
        passphrase: Option<String>,
        category: Option<String>,
        symbol: String,
    ) -> Self;
    async fn cancel(&self, order_id: String) -> Result<(), String>;
    async fn order(&self, params: &OrderParams) -> Result<String, String>;
}

// 複数の取引所情報を管理する型（enumで各取引所のオブジェクトを保持）
pub enum ToExchange {
    None,
    Bybit(BybitClient),
    // Bitbank(BitbankClient),
}

impl ToExchange {
    pub fn create_client(exchange_config: &Config, symbol: String) -> Self {
        match exchange_config.name {
            ExchangeName::Bybit => {
                let client = <BybitClient as OrderClient>::new_for_order_client(
                    exchange_config.key.clone(),
                    exchange_config.secret.clone(),
                    None,
                    exchange_config.category.clone(),
                    symbol.clone(),
                );
                ToExchange::Bybit(client)
            }
            ExchangeName::Bitbank => ToExchange::None,
            ExchangeName::Bitflyer => ToExchange::None,
        }
    }
    pub async fn cancel_order(&self, order_id: String) -> Result<(), String> {
        match self {
            ToExchange::Bybit(client) => client.cancel(order_id).await,
            // ToExchange::Bitbank(client) => client.cancel(order_id),
            _ => Ok(()),
        }
    }

    pub async fn place_order(&self, params: &OrderParams) -> Result<String, String> {
        match self {
            ToExchange::Bybit(client) => client.order(params).await, // ToExchange::Bitbank(client) => client.order(order_id, price, qty),
            _ => Ok("".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    pub ltp: f64,
    pub volume24h: f64,
    pub price_tick: f64,
    pub size_tick: f64,
    pub size_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ticker {
    pub symbol: String,
    pub ltp: f64,
    pub volume24h: f64,
    pub best_ask: f64,
    pub best_bid: f64,
}

impl Ticker {
    pub fn new(symbol: String, ltp: f64, volume24h: f64, best_ask: f64, best_bid: f64) -> Self {
        Ticker {
            symbol,
            ltp,
            volume24h,
            best_ask,
            best_bid,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DataType {
    // default
    #[default]
    Snapshot,
    UpdateDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Orderboard {
    pub symbol: String,
    pub data_type: DataType,
    pub a: Vec<Book>,
    pub b: Vec<Book>,
    pub t: Option<i64>,
    pub u: Option<i64>,
}

impl Orderboard {
    pub fn new(
        data_type: DataType,
        symbol: String,
        a: Vec<Book>,
        b: Vec<Book>,
        t: Option<i64>,
        u: Option<i64>,
    ) -> Self {
        Orderboard {
            data_type,
            symbol,
            a,
            b,
            t,
            u,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub order_id: String,
    pub side: String,
    pub qty: f64,
    pub price: f64,
    pub pnl: f64,
}
