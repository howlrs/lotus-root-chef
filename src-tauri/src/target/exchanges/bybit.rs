use crypto_botters::{
    bybit::{BybitHttpAuth, BybitOption},
    Client,
};
use futures_util::future::pending;
use log::{error, trace};

use serde_json::json;

use crate::{
    board::book::Book,
    target::exchanges::{
        bybit_models::{
            ApiDefaultResponse, ApiOrderResponse, ApiOrderbook, InstrumentInfo, TickerInfo,
        },
        models::{
            DataType, Instrument, OrderClient, OrderParams, OrderSide, Orderboard, Position, Ticker,
        },
    },
};

use tokio::sync::{
    broadcast,
    mpsc::{Receiver, Sender},
};
use tokio::{spawn, task::JoinHandle};

pub struct BybitClient {
    client: Client,
    category: String,
    symbol: String,
}

impl OrderClient for BybitClient {
    fn new_for_order_client(
        key: String,
        secret: String,
        _passphrase: Option<String>,
        category: Option<String>,
        symbol: String,
    ) -> Self {
        let mut client = Client::new();
        client.update_default_option(BybitOption::Key(key));
        client.update_default_option(BybitOption::Secret(secret));

        BybitClient {
            client: client.clone(),
            category: category.unwrap_or("spot".to_string()),
            symbol,
        }
    }

    async fn cancel(&self, order_id: String) -> Result<(), String> {
        let res: ApiOrderResponse = match self
            .client
            .post(
                "/v5/order/cancel",
                Some(json!({
                    "category": self.category.clone(),
                    "symbol": self.symbol.clone(),
                    "orderLinkId": order_id
                })),
                [BybitOption::HttpAuth(BybitHttpAuth::V3AndAbove)],
            )
            .await
        {
            Ok(res) => res,
            Err(e) => return Err(e.to_string()),
        };
        if res.ret_code != 0 {
            return Err(res.ret_msg);
        }

        trace!("cancel order: {}, response: {:?}", order_id, res);

        Ok(())
    }

    async fn order(&self, params: &OrderParams) -> Result<String, String> {
        let order_id = params.order_id.as_ref().map(|s| s.as_str()).unwrap_or("");
        let oside = match params.side {
            OrderSide::Buy => "Buy",
            OrderSide::Sell => "Sell",
        };
        let tif = if params.is_post_only {
            "PostOnly"
        } else {
            "GTC"
        };

        let res: ApiOrderResponse = match self
            .client
            .post(
                "/v5/order/create",
                Some(json!({
                    "category": self.category.clone(),
                    "symbol": self.symbol.clone(),
                    "orderLinkId": order_id,
                    "side": oside,
                    "price": params.price,
                    "qty": params.qty,
                    "timeInForce": tif,
                })),
                [BybitOption::HttpAuth(BybitHttpAuth::V3AndAbove)],
            )
            .await
        {
            Ok(res) => res,
            Err(e) => return Err(e.to_string()),
        };
        if res.ret_code != 0 {
            return Err(res.ret_msg);
        }

        trace!("place order: {}, response: {:?}", order_id, res);

        Ok(res.result.order_link_id)
    }
}

impl BybitClient {
    pub fn new(
        key: Option<String>,
        secret: Option<String>,
        category: String,
        symbol: String,
    ) -> Self {
        let mut client = Client::new();
        if let Some(key) = key {
            client.update_default_option(BybitOption::Key(key));
        }
        if let Some(secret) = secret {
            client.update_default_option(BybitOption::Secret(secret));
        }
        let client = client.clone();

        BybitClient {
            client,
            category,
            symbol,
        }
    }

    pub async fn public_ticker(
        &self,
        // websocket用
        tx_ws_ticker: Sender<Ticker>,
        // rest用取得依頼
        // why: websocket非実装の場合、必要に応じてRestRequestを実行する
        // API Limitの観点から必要に応じてのみ実行する
        mut rx_rest_ticker: Receiver<()>,
        // rest用取得結果通知用
        // why: websocket非実装の場合、必要に応じてRestRequest結果を送信する
        tx_rest_ticker: broadcast::Sender<Ticker>,
    ) -> Result<JoinHandle<()>, String> {
        let client = self.client.clone();
        let category = self.category.clone();
        let symbol = self.symbol.clone();

        let handler = spawn(async move {
            let url_string = format!("/v5/public/{}", category.clone());
            let url = url_string.as_str();
            let set_symbol = symbol.clone();

            let _connection = client
                .websocket(
                    url,
                    move |message| {
                        let data = message.clone()["data"].take();

                        trace!("ticker raw data: {}", data);

                        let ltp = match data["lastPrice"].as_str() {
                            Some(v) => v.parse::<f64>().unwrap_or_default(),
                            None => return,
                        };
                        let v24 = match data["volume24h"].as_str() {
                            Some(v) => v.parse::<f64>().unwrap_or_default(),
                            None => return,
                        };
                        let bid = match data["bid1Price"].as_str() {
                            Some(v) => v.parse::<f64>().unwrap_or_default(),
                            None => return,
                        };
                        let ask = match data["ask1Price"].as_str() {
                            Some(v) => v.parse::<f64>().unwrap_or_default(),
                            None => return,
                        };

                        match tx_ws_ticker.try_send(Ticker::new(
                            set_symbol.clone(),
                            ltp,
                            v24,
                            ask,
                            bid,
                        )) {
                            Ok(()) => (),
                            Err(e) => {
                                error!("ticker send error: {}", e);
                            }
                        };
                    },
                    [
                        BybitOption::WebSocketTopics(vec![
                            format!("tickers.{}", symbol.clone()).to_owned()
                        ]),
                        BybitOption::WebSocketAuth(true),
                    ],
                )
                .await
                .expect("Failed to connect to websocket");

            loop {
                tokio::select! {
                    result = rx_rest_ticker.recv() => {
                        if result.is_some() {
                            // rest用取得依頼
                            // 実取得
                            let ltp = 0.0;
                            let v24 = 0.0;
                            let bid = 0.0;
                            let ask = 0.0;


                            // rest用取得結果通知
                            tx_rest_ticker.send(Ticker::new(symbol.clone(),ltp, v24, ask,bid)).unwrap();
                        }
                    }
                    _ = pending::<()>() => {},
                }
            }
        });

        Ok(handler)
    }

    pub async fn public_orderboard(
        &self,
        depth: Option<i64>,
        // websocket用
        tx_ws_orderboard: Sender<Orderboard>,
        // rest用取得依頼
        // why: websocket非実装の場合、必要に応じてRestRequestを実行する
        // API Limitの観点から必要に応じてのみ実行する
        mut rx_rest_orderboard: Receiver<()>,
        // rest用取得結果通知用
        // why: websocket非実装の場合、必要に応じてRestRequest結果を送信する
        tx_rest_orderboard: broadcast::Sender<Orderboard>,
    ) -> Result<JoinHandle<()>, String> {
        let client = self.client.clone();
        let category = self.category.clone();
        let symbol = self.symbol.clone();
        let set_depth = depth.unwrap_or(200);

        let handler = spawn(async move {
            let url_string = format!("/v5/public/{}", category.clone());
            let url = url_string.as_str();
            let set_symbol = symbol.clone();

            let _connection = client
                .websocket(
                    url,
                    move |message| {
                        let data = message.clone()["data"].take();

                        trace!("orderboard raw data: {}", data);

                        let data_type = match message.clone()["type"].as_str() {
                            Some(v) => match v {
                                "snapshot" => DataType::Snapshot,
                                "delta" => DataType::UpdateDelta,
                                _ => DataType::Snapshot,
                            },
                            None => return,
                        };
                        let get_orderboards: ApiOrderbook = match serde_json::from_value(data) {
                            Ok(v) => v,
                            Err(e) => {
                                trace!("error: {}", e);
                                ApiOrderbook {
                                    s: "".to_owned(),
                                    b: vec![], // Bids [price, size]
                                    a: vec![], // Asks [price, size]
                                    u: 0,      // Update ID
                                    seq: 0,    // Sequence number
                                }
                            }
                        };

                        // create generic orderboard
                        let mut a = vec![];
                        let mut b = vec![];
                        for book in get_orderboards.a {
                            a.push(Book {
                                price: book[0].parse().unwrap_or_default(),
                                size: book[1].parse().unwrap_or_default(),
                            });
                        }
                        for book in get_orderboards.b {
                            b.push(Book {
                                price: book[0].parse().unwrap_or_default(),
                                size: book[1].parse().unwrap_or_default(),
                            });
                        }

                        match tx_ws_orderboard.try_send(Orderboard::new(
                            data_type.clone(),
                            set_symbol.clone(),
                            a,
                            b,
                            None,
                            Some(get_orderboards.u),
                        )) {
                            Ok(()) => (),
                            Err(e) => {
                                error!("orderboard send error: {}", e);
                            }
                        };
                    },
                    [
                        BybitOption::WebSocketTopics(vec![format!(
                            "orderbook.{}.{}",
                            set_depth,
                            symbol.clone()
                        )
                        .to_owned()]),
                        BybitOption::WebSocketAuth(false),
                    ],
                )
                .await
                .expect("Failed to connect to websocket");

            loop {
                tokio::select! {
                    result = rx_rest_orderboard.recv() => {
                        if result.is_some() {
                            // rest用取得依頼
                            // 実取得

                            let o = Orderboard::new(DataType::Snapshot,symbol.clone(), vec![], vec![], None, None);

                            // rest用取得結果通知
                            tx_rest_orderboard.send(o).unwrap();
                        }
                    }
                    _ = pending::<()>() => {},
                }
            }
        });

        Ok(handler)
    }

    pub async fn private_position(
        &self,
        // websocket用
        tx_ws_position: Sender<Vec<Position>>,
        // rest用取得依頼
        // why: websocket非実装の場合、必要に応じてRestRequestを実行する
        // API Limitの観点から必要に応じてのみ実行する
        mut rx_rest_position: Receiver<()>,
        // rest用取得結果通知用
        // why: websocket非実装の場合、必要に応じてRestRequest結果を送信する
        tx_rest_position: broadcast::Sender<Vec<Position>>,
    ) -> Result<JoinHandle<()>, String> {
        let client = self.client.clone();
        let set_symbol = self.symbol.clone();

        let handler = spawn(async move {
            let url = "/v5/private";
            let _connection = client
                .websocket(
                    url,
                    move |message| {
                        let data = message.clone()["data"].take();

                        trace!("position raw data: {}", data);

                        let get_positions: Vec<Position> = match serde_json::from_value(data) {
                            Ok(v) => v,
                            Err(e) => {
                                println!("error: {}", e);
                                vec![]
                            }
                        };

                        let use_positions = get_positions
                            .into_iter()
                            .filter(|p| p.symbol == set_symbol)
                            .collect::<Vec<Position>>();

                        if use_positions.is_empty() {
                            return;
                        }

                        match tx_ws_position.try_send(use_positions) {
                            Ok(()) => (),
                            Err(e) => {
                                error!("position send error: {}", e);
                            }
                        };
                    },
                    [
                        BybitOption::WebSocketTopics(vec!["position".to_owned()]),
                        BybitOption::WebSocketAuth(true),
                    ],
                )
                .await
                .expect("Failed to connect to websocket");

            loop {
                tokio::select! {
                    result = rx_rest_position.recv() => {
                        if result.is_some() {
                            // rest用取得依頼
                            // 実取得

                            // rest用取得結果通知
                            tx_rest_position.send(vec![]).unwrap();
                        }
                    }
                    _ = pending::<()>() => {},
                }
            }
        });

        Ok(handler)
    }
}

pub async fn instruments(category: String) -> Result<Vec<Instrument>, String> {
    let client = Client::new();

    // public GET
    let res: ApiDefaultResponse = match client
        .get(
            "/v5/market/instruments-info",
            Some(&[("category", category)]),
            [BybitOption::Default],
        )
        .await
    {
        Ok(res) => res,
        Err(e) => return Err(e.to_string()),
    };
    if res.ret_code != 0 {
        return Err(res.ret_msg);
    }

    let list: Vec<InstrumentInfo> = serde_json::from_value(res.result.list).unwrap();

    Ok(list
        .iter()
        .map(|item| Instrument {
            symbol: item.symbol.clone(),
            ltp: 0.0,
            volume24h: 0.0,
            price_tick: item.price_filter.tick_size.parse().unwrap(),
            size_tick: item.lot_size_filter.qty_step.parse().unwrap(),
            size_min: item.lot_size_filter.min_order_qty.parse().unwrap(),
        })
        .collect())
}

pub async fn ticker(category: String, symbol: String) -> Result<Ticker, String> {
    let client = Client::new();
    // public GET
    let res: ApiDefaultResponse = match client
        .get(
            "/v5/market/tickers",
            Some(&[("category", category), ("symbol", symbol.clone())]),
            [BybitOption::Default],
        )
        .await
    {
        Ok(res) => res,
        Err(e) => return Err(e.to_string()),
    };
    if res.ret_code != 0 {
        return Err(res.ret_msg);
    }

    let tickers = match serde_json::from_value::<Vec<TickerInfo>>(res.result.list) {
        Ok(v) => v,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    let target_ticker = match tickers.iter().find(|t| t.symbol == symbol) {
        Some(v) => v,
        None => {
            return Err(format!("ticker is not match for {}", symbol).to_string());
        }
    };
    let ticker = Ticker {
        symbol: target_ticker.symbol.clone(),
        ltp: target_ticker.last_price.parse().unwrap_or_default(),
        volume24h: target_ticker.volume_24h.parse().unwrap_or_default(),
        best_ask: target_ticker.ask1_price.parse().unwrap_or_default(),
        best_bid: target_ticker.bid1_price.parse().unwrap_or_default(),
    };

    Ok(ticker)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_instruments() {
        let category = "linear".to_string();
        let instruments = instruments(category).await.unwrap();
        println!("{:?}", instruments);
    }

    #[tokio::test]
    async fn test_ticker() {
        let category = "linear".to_string();
        let symbol = "BTCUSDT".to_string();
        let ticker = match ticker(category, symbol).await {
            Ok(v) => v,
            Err(e) => {
                println!("error: {}", e);
                Ticker::default()
            }
        };

        println!("{:?}", ticker);
    }
}
