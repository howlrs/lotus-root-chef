use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::target::exchanges::models::OrderSide;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub symbol: String,

    pub side: OrderSide,
    pub size: f64,
    pub is_post_only: bool,

    pub tick_size: f64,
    pub interval_sec: i64,
}

impl Config {
    #[allow(unused)]
    pub fn new(symbol: String, size: f64, side: OrderSide) -> Self {
        Config {
            symbol,
            side,
            size,
            is_post_only: true,

            tick_size: 0.01,

            interval_sec: 5,
        }
    }

    pub fn to_order_info(&self) -> OrderInfo {
        OrderInfo {
            order_id: None,
            price: None,
            qty: self.size,
            interval_sec: self.interval_sec,
            latest_at: None,
        }
    }

    pub fn is_ok(&self) -> bool {
        if self.symbol.is_empty() || self.size <= 0.0 {
            return false;
        }

        true
    }

    // 対象の板の価格より有利な価格を出力
    pub fn add_tick_size(&self, price: f64) -> f64 {
        match self.side {
            OrderSide::Buy => price + self.tick_size,
            OrderSide::Sell => price - self.tick_size,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OrderInfo {
    pub order_id: Option<String>,
    pub price: Option<f64>,
    pub qty: f64,
    pub interval_sec: i64,
    pub latest_at: Option<DateTime<chrono::Utc>>,
}

impl OrderInfo {
    #[allow(unused)]
    pub fn new() -> Self {
        OrderInfo {
            order_id: None,
            price: None,
            qty: 0.0,
            interval_sec: 5,
            latest_at: None,
        }
    }

    // interval_sec以上経過しているか
    pub fn is_allowed(&self) -> bool {
        // interval_sec以上経過しているか
        let now = chrono::Utc::now();
        if let Some(prev) = self.latest_at {
            let diff = now.signed_duration_since(prev).num_seconds();
            if diff < self.interval_sec {
                return false;
            }
        }

        true
    }

    pub fn set_order(&mut self, order_id: String) {
        self.order_id = Some(order_id);
        self.latest_at = Some(chrono::Utc::now());
    }

    pub fn set_error_order(&mut self) {
        self.latest_at = Some(chrono::Utc::now());
    }
}
