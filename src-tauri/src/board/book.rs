use chrono::{DateTime, Utc};
use log::trace;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    str,
    sync::{Arc, RwLock},
    time::Instant,
};

use crate::{board::filter::Config, target::exchanges::models::BookSide};

#[derive(Debug, Default, Clone)]
pub struct Orderboard {
    ask: Arc<RwLock<BTreeMap<OrderedFloat<f64>, Book>>>,
    bid: Arc<RwLock<BTreeMap<OrderedFloat<f64>, Book>>>,
    update_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl Orderboard {
    pub fn new() -> Self {
        Orderboard {
            ask: Arc::new(RwLock::new(BTreeMap::new())),
            bid: Arc::new(RwLock::new(BTreeMap::new())),
            update_at: Utc::now(),
        }
    }

    pub fn ask(&self) -> BTreeMap<OrderedFloat<f64>, Book> {
        let ask = self.ask.read().unwrap();

        ask.clone()
    }

    pub fn bid(&self) -> BTreeMap<OrderedFloat<f64>, Book> {
        let bid = self.bid.read().unwrap();

        bid.clone()
    }

    pub fn len(&self) -> (usize, usize) {
        let ask = self.ask.read().unwrap();
        let bid = self.bid.read().unwrap();

        (ask.len(), bid.len())
    }

    pub fn best_prices(&self) -> (f64, f64) {
        let ask = self.ask.read().unwrap();
        let bid = self.bid.read().unwrap();

        let ask_price = match ask.iter().next() {
            Some((price, _)) => price.0,
            None => 0.0,
        };

        let bid_price = match bid.iter().next_back() {
            Some((price, _)) => price.0,
            None => 0.0,
        };

        (ask_price, bid_price)
    }

    pub fn best(&self, target_side: BookSide) -> f64 {
        match target_side {
            BookSide::Ask => {
                let ask = self.ask.read().unwrap();

                match ask.iter().next() {
                    Some((price, _)) => price.0,
                    None => 0.0,
                }
            }
            BookSide::Bid => {
                let bid = self.bid.read().unwrap();

                match bid.iter().next_back() {
                    Some((price, _)) => price.0,
                    None => 0.0,
                }
            }
        }
    }

    pub fn extend(&mut self, target_side: BookSide, book: Vec<Book>) -> DateTime<Utc> {
        match target_side {
            BookSide::Ask => self.extend_ask(book),
            BookSide::Bid => self.extend_bid(book),
        }
    }

    pub fn extend_ask(&mut self, ask: Vec<Book>) -> DateTime<Utc> {
        let mut new_book = BTreeMap::new();
        for book in ask {
            new_book.insert(OrderedFloat(book.price), book);
        }

        {
            let mut w = self.ask.write().unwrap();
            w.clear();
            *w = new_book;
        }

        self.update_at = Utc::now();
        self.update_at
    }

    pub fn extend_bid(&mut self, bid: Vec<Book>) -> DateTime<Utc> {
        let mut new_book = BTreeMap::new();
        for book in bid {
            new_book.insert(OrderedFloat(book.price), book);
        }

        {
            let mut w = self.bid.write().unwrap();
            w.clear();
            *w = new_book;
        }

        self.update_at = Utc::now();
        self.update_at
    }

    pub fn update_delta(&mut self, target_side: BookSide, books: Vec<Book>) {
        let mut target_book = match target_side {
            BookSide::Ask => self.ask.write().unwrap(),
            BookSide::Bid => self.bid.write().unwrap(),
        };

        for book in books {
            if book.is_remove() {
                target_book.remove(&OrderedFloat(book.price));
                continue;
            }

            target_book.insert(OrderedFloat(book.price), book);
        }
    }

    pub fn push(&mut self, target_side: BookSide, book: Book) {
        match target_side {
            BookSide::Ask => self.push_to_ask(book),
            BookSide::Bid => self.push_to_bid(book),
        }
    }

    pub fn push_to_ask(&mut self, book: Book) {
        let mut abook = self.ask.write().unwrap();

        if book.is_remove() {
            abook.remove(&OrderedFloat(book.price));
            return;
        }

        abook.insert(OrderedFloat(book.price), book);
    }

    pub fn push_to_bid(&mut self, book: Book) {
        let mut bbook = self.bid.write().unwrap();

        if book.is_remove() {
            bbook.remove(&OrderedFloat(book.price));
            return;
        }

        bbook.insert(OrderedFloat(book.price), book);
    }

    pub fn f64_to_book(&self, price: f64, size: f64) -> Book {
        Book { price, size }
    }

    pub fn string_to_book(&self, price: String, size: String) -> Book {
        let price = price.parse::<f64>().unwrap();
        let size = size.parse::<f64>().unwrap();

        Book { price, size }
    }

    // 対象の板を検出する
    pub fn target_book(
        &self,
        filter_config: &Config,
        prev_own_order_price: Option<f64>,
    ) -> (f64, bool) {
        let start = Instant::now();
        // 複数の filter を連結したクロージャ
        let is_condition = |&(_, book): &(&OrderedFloat<f64>, &Book)| {
            filter_config.is_large(book)
                && filter_config.is_range(book)
                // 自身の注文価格を除外する
                // 自身の板が検知に引っかかる場合は除外し、次の候補を探す
                // これにより、自板の後ろに板が引いたときには引いた価格に注文する
                && !filter_config.is_excluded(book, prev_own_order_price)
        };

        match filter_config.side {
            BookSide::Ask => {
                let abook = {
                    let book = self.ask.read().unwrap();
                    book.clone()
                };

                // 指定条件のフィルタリング適用後の板を取得
                let filtered = abook.iter().filter(is_condition).collect::<Vec<_>>();
                // フィルタリング後の板が空の場合は 0 を返す
                if filtered.is_empty() {
                    return (0.0, false);
                }

                // 昇順で最初の要素を出力する
                // 配列は昇順でソートされているため、最初の要素が最小値
                match filtered.first() {
                    Some((price, book)) => {
                        // BTreeMapのキーが保持するBook.Priceと値が一致するか確認する
                        let price = price.0;
                        if price != book.price {
                            return (0.0, false);
                        }

                        // 経過時間を表示
                        trace!("board search elapsed: {:?}", start.elapsed());

                        (price, true)
                    }
                    None => (0.0, false),
                }
            }
            BookSide::Bid => {
                let bbook = {
                    let book = self.bid.read().unwrap();
                    book.clone()
                };

                // 指定条件のフィルタリング適用後の板を取得
                let filtered = bbook.iter().filter(is_condition).collect::<Vec<_>>();
                // フィルタリング後の板が空の場合は 0 を返す
                if filtered.is_empty() {
                    return (0.0, false);
                }

                // 昇順で最後の要素を出力する
                // 配列は昇順でソートされているため、最後の要素が最大値
                match filtered.last() {
                    Some((price, book)) => {
                        // BTreeMapのキーが保持するBook.Priceと値が一致するか確認する
                        let price = price.0;
                        if price != book.price {
                            return (0.0, false);
                        }

                        // 経過時間を表示
                        trace!("board search elapsed: {:?}", start.elapsed());

                        (price, true)
                    }
                    None => (0.0, false),
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Book {
    pub size: f64,
    pub price: f64,
}

#[allow(dead_code)]
impl Book {
    pub fn new(size: f64, price: f64) -> Self {
        Book { size, price }
    }

    fn is_remove(&self) -> bool {
        self.size.is_nan() || self.size == 0.0
    }

    pub fn is_large(&self, size: f64) -> bool {
        self.size > size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_book_ask_without_prev_exclusion() {
        // 検索対象価格の設定
        let price_min = 1;
        let target_price = 99;
        // 見つかった価格を自分の指値として除外する
        let execuded_price = Some(99.0);
        // 期待する値
        let is_expected = false;
        let expected_price = 0.0;
        let expected_best_ask = 1.0;

        // 対象データの生成
        let mut board = Orderboard::new();
        // 価格が [1.0, 2.0, ..., 100.0] の等差数列で100個のブックを生成します。
        let mut books = Vec::new();
        for i in price_min..=100 {
            // すべてのブックはサイズが1.0で、is_largeの条件を満たしています。
            // 99の倍数のブックのサイズは1.5です。
            // 検索対象を1.0に設定します。
            let size = if i % target_price as usize == 0 {
                1.5
            } else {
                1.0
            };
            books.push(Book::new(size, i as f64));
        }
        board.extend_ask(books);

        // Askサイドのテスト設定を作成します。価格範囲は[1.0, 150.0]、最小サイズは1.0です。
        let config = Config {
            side: BookSide::Ask,
            // 価格範囲
            hight: 150.0,
            low: 1.0,
            // 検索対象サイズ
            size: 1.0,
        };

        // 前回の自身の注文価格は提供されていません。
        let (price, is_found) = board.target_book(&config, execuded_price);
        // Askの場合、最も低い有効な価格が選択されるはずです: 1.0。
        assert_eq!(is_found, is_expected);
        assert_eq!(price, expected_price);

        let best_ask = board.best(BookSide::Ask);
        assert_eq!(best_ask, expected_best_ask);
    }

    #[test]
    fn test_target_book_bid_with_prev_exclusion() {
        // 検索対象価格の設定
        let price_max = 100;
        let target_price = 71;
        // 見つかった価格を自分の指値として除外する
        let execuded_price = None;
        // 期待する値
        let is_expected = true;
        let expected_price = 71.0;
        let expected_best_bid = 100.0;

        // 対象データの生成
        let mut board = Orderboard::new();
        // 価格が [1.0, 2.0, ..., 100.0] の等差数列で100個のブックを生成します。
        let mut books = Vec::new();
        for i in 1..=price_max {
            // すべてのブックはサイズが1.0で、is_largeの条件を満たしています。
            // 99の倍数のブックのサイズは1.5です。
            // 検索対象を1.0に設定します。
            let size = if i % target_price as usize == 0 {
                1.5
            } else {
                1.0
            };
            books.push(Book::new(size, i as f64));
        }
        board.extend_bid(books);

        // Askサイドのテスト設定を作成します。価格範囲は[1.0, 150.0]、最小サイズは1.0です。
        let config = Config {
            side: BookSide::Bid,
            // 価格範囲
            hight: 150.0,
            low: 1.0,
            // 検索対象サイズ
            size: 1.0,
        };

        // 前回の自身の注文価格は提供されていません。
        let (price, is_found) = board.target_book(&config, execuded_price);
        // Askの場合、最も低い有効な価格が選択されるはずです: 1.0。
        assert_eq!(is_found, is_expected);
        assert_eq!(price, expected_price);

        let best_bid = board.best(BookSide::Bid);
        assert_eq!(best_bid, expected_best_bid);
    }
}
