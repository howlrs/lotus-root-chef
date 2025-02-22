use chrono::{DateTime, Utc};
use log::info;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    str,
    sync::{Arc, RwLock},
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

        info!("best prices: [ask: {}, bid: {}]", ask_price, bid_price);
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

    pub fn update_at(&mut self) -> DateTime<Utc> {
        self.update_at = Utc::now();
        self.update_at
    }

    pub fn replace(&self, target_side: BookSide, book: Vec<Book>) {
        match target_side {
            BookSide::Ask => self.replace_ask(book),
            BookSide::Bid => self.replace_bid(book),
        }
    }

    pub fn replace_ask(&self, ask: Vec<Book>) {
        let mut new_book = BTreeMap::new();
        for book in ask {
            new_book.insert(OrderedFloat(book.price), book);
        }

        {
            let mut w = self.ask.write().unwrap();
            w.clear();
            *w = new_book;
        }
    }

    pub fn replace_bid(&self, bid: Vec<Book>) {
        let mut new_book = BTreeMap::new();
        for book in bid {
            new_book.insert(OrderedFloat(book.price), book);
        }

        {
            let mut w = self.bid.write().unwrap();
            w.clear();
            *w = new_book;
        }
    }

    pub fn update_delta(&self, target_side: BookSide, books: Vec<Book>) {
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

    pub fn push(&self, target_side: BookSide, book: Book) {
        match target_side {
            BookSide::Ask => self.push_to_ask(book),
            BookSide::Bid => self.push_to_bid(book),
        }
    }

    pub fn push_to_ask(&self, book: Book) {
        let mut abook = self.ask.write().unwrap();

        if book.is_remove() {
            abook.remove(&OrderedFloat(book.price));
            return;
        }

        abook.insert(OrderedFloat(book.price), book);
    }

    pub fn push_to_bid(&self, book: Book) {
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
                // ロック取得、クローンせずに直接参照でイテレートする
                // クローンすると非効率かつ速度が遅くなる
                // why: ロック取得時間を短くするより、クローンコストが速度に悪影響を及ぼしていた例
                let abook = self.ask.read().unwrap();

                // 検索の該当配列を出力していたが、発見後即時返り値を生成する使用に変更
                // 可読性が向上し、速度も向上する
                for (price, book) in abook.iter() {
                    if is_condition(&(price, book)) {
                        // キーと値が整合しているかチェック
                        if book.is_same(price.0) && !book.is_zero() {
                            return (price.0, true);
                        } else {
                            return (0.0, false);
                        }
                    }
                }
                (0.0, false)
            }
            BookSide::Bid => {
                let bbook = self.bid.read().unwrap();

                // Bid は昇順になっているため、逆方向から探す
                for (price, book) in bbook.iter().rev() {
                    if is_condition(&(price, book)) {
                        if book.is_same(price.0) && !book.is_zero() {
                            return (price.0, true);
                        } else {
                            return (0.0, false);
                        }
                    }
                }
                (0.0, false)
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

    fn is_zero(&self) -> bool {
        self.size == 0.0 || self.price == 0.0
    }

    fn is_same(&self, key_price: f64) -> bool {
        self.price == key_price
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rand::Rng;

    use super::*;

    // 参考: 10000個の板を生成し、検索対象価格を設定し、最小値から検索する
    // create board time: 18.5503ms
    // filterd time: 11.8µs

    fn setup_board(
        price_min: usize,
        price_max: usize,
        target_price: usize,
        side: BookSide,
    ) -> Orderboard {
        let board = Orderboard::new();
        let mut books = Vec::new();
        for i in price_min..=price_max {
            let size = if i % target_price == 0 { 1.5 } else { 1.0 };
            books.push(Book::new(size, i as f64));
        }
        match side {
            BookSide::Ask => board.replace_ask(books),
            BookSide::Bid => board.replace_bid(books),
        }
        board
    }

    fn create_config(side: BookSide, price_max: usize, price_min: usize) -> Config {
        Config {
            side,
            hight: price_max as f64,
            low: price_min as f64,
            size: 1.0,
        }
    }

    #[test]
    fn test_target_book_ask() {
        let price_max = 10000;
        let price_min = 1;

        let wall_price = rand::rng().random_range(7..price_max - 1);
        let excluded_price = None;

        let expected_is_found = true;
        let expected_price = wall_price as f64;
        let expected_best_ask = 1.0;

        let board = setup_board(price_min, price_max, wall_price, BookSide::Ask);
        let config = create_config(BookSide::Ask, price_max + 1, price_min - 1);

        let (price, is_found) = board.target_book(&config, excluded_price);
        assert_eq!(
            is_found, expected_is_found,
            "price: {}, expected: {}",
            price, expected_price,
        );
        // 最良価格の取得
        // 板生成時の最小値となる
        let best_ask = board.best(BookSide::Ask);
        assert_eq!(
            price, expected_price,
            "wall_price: {}, best_ask: {}",
            wall_price, best_ask
        );
        assert_eq!(best_ask, expected_best_ask);
    }

    #[test]
    fn test_target_book_bid() {
        let price_max = 10000;
        let price_min = 1;

        // max値を最大に乱数を生成
        let wall_price = rand::rng().random_range(7..price_max - 1);
        let excluded_price = None;

        let expected_is_found = true;
        let expected_price = (price_max / wall_price * wall_price) as f64;
        let expected_best_bid = price_max as f64;

        let board = setup_board(price_min, price_max, wall_price, BookSide::Bid);
        let config = create_config(BookSide::Bid, price_max + 1, price_min - 1);

        let (price, is_found) = board.target_book(&config, excluded_price);
        assert_eq!(
            is_found, expected_is_found,
            "price: {}, expected: {}",
            price, expected_price,
        );
        // 最良価格の取得
        // 板生成時の最大値となる
        let best_bid = board.best(BookSide::Bid);
        let binding = board.bid();
        let book = binding.get_key_value(&OrderedFloat(best_bid)).unwrap().1;
        assert_eq!(
            price, expected_price,
            "wall_price: {}, best_bid: {}, book: {:?}",
            wall_price, best_bid, book
        );
        assert_eq!(price % wall_price as f64, 0.0);
        assert_eq!(best_bid, expected_best_bid);
    }

    #[test]
    fn test_target_book_ask_with_prev_exclusion() {
        let price_max = 10000;
        let price_min = 1;

        // 小さい乱数を生成
        let divis = rand::rng().random_range(7..99);
        let wall_price = divis;
        let min_wall_price = divis;
        let excluded_price = Some(min_wall_price as f64);

        let expected_is_found = true;
        let expected_price = (min_wall_price * 2) as f64;
        let expected_best_ask = price_min as f64;

        let board = setup_board(price_min, price_max, wall_price, BookSide::Ask);
        let config = create_config(BookSide::Ask, price_max + 1, price_min - 1);

        let (price, is_found) = board.target_book(&config, excluded_price);
        assert_eq!(
            is_found, expected_is_found,
            "price: {}, expected: {}",
            price, expected_price,
        );
        assert_eq!(price, expected_price, "wall_price: {}", wall_price);
        assert_eq!(price % wall_price as f64, 0.0);

        // 最良価格の取得
        // 板生成時の最小値となる
        let best_ask = board.best(BookSide::Ask);
        assert_eq!(best_ask, expected_best_ask);
    }

    #[test]
    fn test_target_book_bid_with_prev_exclusion() {
        let price_max = 10000;
        let price_min = 1;

        // 小さい乱数を生成
        let divis = rand::rng().random_range(7..99);
        let wall_price = divis;
        let max_wall_price = price_max / divis * divis;
        let excluded_price = Some(max_wall_price as f64);

        let expected_is_found = true;
        let expected_price = (max_wall_price - divis) as f64;
        let expected_best_bid = price_max as f64;

        let board = setup_board(price_min, price_max, wall_price, BookSide::Bid);
        let config = create_config(BookSide::Bid, price_max + 1, price_min - 1);

        let (price, is_found) = board.target_book(&config, excluded_price);
        assert_eq!(
            is_found, expected_is_found,
            "price: {}, expected: {}",
            price, expected_price,
        );
        assert_eq!(price, expected_price, "wall_price: {}", wall_price);
        assert_eq!(price % wall_price as f64, 0.0);

        // 最良価格の取得
        // 板生成時の最大値となる
        let best_bid = board.best(BookSide::Bid);
        assert_eq!(best_bid, expected_best_bid);
    }

    #[test]
    fn n_count_try() {
        let start = Instant::now();
        let count = 1000;
        for _ in 0..count {
            test_target_book_ask();
            test_target_book_bid();
            test_target_book_ask_with_prev_exclusion();
            test_target_book_bid_with_prev_exclusion();
        }

        println!("{} times: {}ms", count, start.elapsed().as_millis());
        // 10 times: 555ms
        // 100 times: 5573ms
        // 1000 times: 57298ms
    }
}
