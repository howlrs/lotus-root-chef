use serde::{Deserialize, Serialize};

use crate::{board::book::Book, target::exchanges::models::BookSide};

// Config
// 額面通りの比較であればf64で十分
// 計算誤差は不要

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub side: BookSide,

    // 指定範囲上限価格
    pub hight: f64,
    // 指定範囲下限価格
    pub low: f64,
    // 指定サイズ
    pub size: f64,
}

impl Config {
    pub fn is_ok(&self) -> bool {
        self.hight > 0.0 && self.low >= 0.0 && self.size > 0.0
    }

    pub fn is_range(&self, book: &Book) -> bool {
        self.hight > book.price && self.low < book.price
    }

    pub fn is_large(&self, book: &Book) -> bool {
        self.size < book.size
    }

    // 自身の注文価格と同じ価格の注文であればtrue
    // 自身の注文価格がなければfalse
    pub fn is_excluded(&self, book: &Book, own_price: Option<f64>) -> bool {
        match own_price {
            Some(price) => price == book.price,
            None => false,
        }
    }
}
