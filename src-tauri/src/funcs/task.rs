use core::panic;
use std::env;
use std::future::pending;
use std::sync::Arc;
use std::time::Instant;

use log::{ info, log_enabled, trace};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::task::{spawn, JoinError, JoinHandle};

use crate::board;
use crate::funcs::client::{Controller, Log, Logger};
use crate::funcs::position;
use crate::target::exchanges::models::{
    BookSide, DataType, OrderParams, Orderboard, Position, Ticker, ToExchange
};

pub async fn runner(
    controller: Arc<RwLock<Controller>>,
    logger: Arc<RwLock<Logger>>,
) -> Result<Vec<JoinHandle<()>>, JoinError> {
    // テストの場合注文の処理をスキップする
    let is_test = env::var("IS_TEST").unwrap_or_default() == "true";

    // Runner内の処理を並列に実行するためのハンドル
    let mut handles = vec![];
    // Runner内の処理の一つが終了または失敗したら、他の処理を停止するためのハンドル
    let cancel_handle = tokio_util::sync::CancellationToken::new();

    // 当関数内のみで使用する変数を生成
    // 当関数はControllerが更新されるごとに再生成される
    // Websocketの送受信用のチャネルは都度接続され、停止される
    let (target_symbol, exchange_config, board_config, order_config) = {
        let r = controller.read().await;
        (
            r.order.symbol.clone(),
            Arc::new(r.exchange.clone()),
            Arc::new(r.board.clone()),
            r.order.clone(),
        )
    };

    let exchange_client = ToExchange::create_client(&exchange_config, target_symbol.clone());

    // 直列に実行するためのチャネル
    let (tx_ws_orderboard, mut rx_ws_orderboard) = mpsc::channel::<Orderboard>(32);
    let (tx_ws_ticker, mut rx_ws_ticker) = mpsc::channel::<Ticker>(32);
    #[allow(unused_variables, unused_mut)]
    let (tx_ws_position, mut rx_ws_position) = mpsc::channel::<Vec<Position>>(32);
    let (tx_order, mut rx_order) = mpsc::channel::<f64>(32);

    // RestRequest依頼の送受信用のチャネル
    #[allow(unused_variables)]
    let (fetch_rest_ticker, recive_rest_ticker) = mpsc::channel::<()>(32);
    #[allow(unused_variables)]
    let (fetch_rest_orderboard, recive_rest_orderboard) = mpsc::channel::<()>(32);
    #[allow(unused_variables)]
    let (fetch_rest_position, recive_rest_position) = mpsc::channel::<()>(32);

    // RestRequestのデータ送受信用のチャネル
    #[allow(unused_variables)]
    let (tx_rest_ticker, _) = broadcast::channel::<Ticker>(32);
    #[allow(unused_variables)]
    let (tx_rest_orderboard, _) = broadcast::channel::<Orderboard>(32);
    let (tx_rest_position, _) = broadcast::channel::<Vec<Position>>(32);

    //更新データ群
    // - スレッド間共有使用データ
    let order_manage = Arc::new(Mutex::new(order_config.to_order_info()));
    // 更新データ群
    // - 外部データ
    let ticker = Arc::new(RwLock::new(Ticker::default()));
    let positions = Arc::new(RwLock::new(vec![]));

    let cloned_ticker = ticker.clone();
    handles.push(spawn(async move {
        // WebSocketの受信
        loop {
            tokio::select! {
                Some(t) = rx_ws_ticker.recv() => {
                    let mut w = cloned_ticker.write().await;
                    *w = t.clone();

                    trace!("ticker: {:?}", t);
                }
                _ = pending::<()>() => {
                    // handle.abort()を待つ
                }
            }
        }
    }));

    let cloned_positions = positions.clone();
    handles.push(spawn(async move {
        // WebSocketの受信
        loop {
            tokio::select! {
                Some(pos) = rx_ws_position.recv() => {
                    {
                        let mut w = cloned_positions.write().await;
                        *w = pos.clone();
                    }

                    info!("position: {:?}", pos);
                }
                _ = pending::<()>() => {
                    // handle.abort()を待つ
                }
            }
        }
    }));

    let ( cloned_order_manage, cloned_board_config, cloned_logger) = {
        ( order_manage.clone(),  board_config.clone(), logger.clone())
    };
    handles.push(spawn(async move {
        let board = Arc::new(board::book::Orderboard::new());
        // WebSocketの送信
        loop {
            tokio::select! {
                Some(books) = rx_ws_orderboard.recv() => {
                    // Orderboardの更新
                    match books.data_type {
                        DataType::Snapshot => {
                            // Arc, Lockの粒度は親とする
                            // 板を差し替える
                            board.replace(BookSide::Bid, books.b);
                            board.replace(BookSide::Ask, books.a);
                        }
                        DataType::UpdateDelta => {
                            // size: 0の場合は削除
                            // 同priceは上書き
                            board.update_delta(BookSide::Bid, books.b);
                            board.update_delta(BookSide::Ask, books.a);
                        }
                    }

                    // env_logger traceであれば表示
                    if log_enabled!(log::Level::Info)  {
                        let (best_ask, best_bid) = {
                            board.best_prices()
                        };

                        info!("mid: {}", (best_ask + best_bid)/ 2.0);
                    };

                    // 対象の板を検出する
                    // - 指定価格内
                    // - 指定サイズ以上
                    // - 自己注文価格以外
                    let start = Instant::now();   
                    let (target_price, is_there) = {
                        let prev_order_price = {
                            let r = cloned_order_manage.lock().await;
                            r.price
                        };
                        board.target_book(&cloned_board_config, prev_order_price)
                    };
                    info!("search target price elapsed: {:?}", start.elapsed());

                    if !is_there {
                        continue;
                    }

                    trace!("target_price before: {:?}", target_price);
                    match tx_order.send(target_price).await {
                        Ok(_) => {
                            continue;
                        }
                        Err(e) => {
                                let mut w = cloned_logger.write().await;
                                w.add(Log {
                                    level: "error".to_string(),
                                    message: format!("board send error: {:?}", e),
                                    timestamp: chrono::Local::now().to_string(),
                                });

                            continue;
                        }
                    };
                }
                _ = pending::<()>() => {
                    // handle.abort()を待つ
                },
            }
        }
    }));

    // 設定情報
    let (cloned_cancel_handle, cloned_exchange, cloned_target_symbol, cloned_order_manage, cloned_positions, cloned_logger, cloned_fetch_rest_position, rx_rest_position) = (
        cancel_handle.clone(),
        exchange_config.name.clone(),
        target_symbol.clone(),
        order_manage.clone(),
        positions.clone(),
        logger.clone(),
        fetch_rest_position.clone(),
        tx_rest_position.subscribe(),
    );
    handles.push(spawn(async move {
        let set_order_link_id = format!("{}_{}_board4rs", cloned_exchange.clone().as_str(), cloned_target_symbol.clone());

        loop {
            tokio::select! {
                Some(target_price) = rx_order.recv() => {
                    trace!("target_price after: {:?}",  target_price);
                    // 条件を満たす対象の情報を受信する
                    // - is_allowed: interval_sec以上経過しているか
                    // why: あまりにも頻繁な注文を回避する
                    let order_id = {
                        let r = cloned_order_manage.lock().await;
                        if !r.is_allowed() {
                            continue;
                        }
                        r.order_id.clone()
                    };

                    // - cancel: order_idがある場合、キャンセルする
                    if let Some(order_id) = order_id.clone() {
                        trace!("cancel by order id: {:?}", order_id);
                        if !is_test {
                            exchange_client.cancel_order(order_id.clone()).await.unwrap();
                        }
                    }



                    // 建玉を取得し、残りの数量を計算する
                    // 部分約定があれば、その分を差し引き、再注文する
                    let ramaining_qty_as_order_qty = {
                        let prev_order = {
                            let r = cloned_order_manage.lock().await;
                            r.clone()
                        };

                        // 自己注文の価格と同値であれば、注文しない
                        // Boardでもチェックして、二重チェック
                        if prev_order.price.unwrap_or_default() == target_price {
                            info!("order and target_price are same price: {}", target_price);
                            continue;   
                        }

                        // Websocket非実装取引所の場合、ポジションは空であるため
                        // REST APIで取得する
                        let resubscribed_positions = rx_rest_position.resubscribe();
                        let temp = position::get_positions(cloned_positions.clone(), cloned_fetch_rest_position.clone(),resubscribed_positions).await;

                        // 先注文があれば、部分約定の可能性がある
                        // 建玉の確認を行い、指定枚数以上の約定を確認する
                        // 建玉がなければ、注文数量をそのまま使用する
                        // 建玉があれば、部分約定の数量を差し引いた数量を使用する
                        if let Some(order_id) = order_id.clone() {
                            let has_position = position::aggrigate_position(order_id.clone(), temp);
                            let remain = prev_order.qty - has_position.qty;
                            if remain <= 0.0 {
                                // すべて約定している場合はログを追加
                                let mut w = cloned_logger.write().await;
                                w.add(Log {
                                    level: "success".to_string(),
                                    message: format!("[completed] close runner by latest order id: {:?}, order size: {}, executed size: {}", order_id, remain, has_position.qty),
                                    timestamp: chrono::Local::now().to_string(),
                                });

                                // 終了フラグを立てる
                                cloned_cancel_handle.cancel();
                                // 終了フラグはRunner.handlesが管理するspawn処理の.awaitに対して伝播し、全てのspawnが終了する
                                break;
                            } 
                            
                            remain
                            
                        } else {
                            prev_order.qty
                        }
                    };

                    // - add_tick_size: 対象価格に対してtick_sizeを加算する
                    // why: 取引所の指定する最小価格値を加算または減算し、約定有利な価格を設定する
                    // 設定型にはticker baseのティックサイズは入っている
                    let target_price = order_config.add_tick_size(target_price);

                    // - order: 新規注文または再注文を行う
                    // 約定が指定サイズ以上であれば、再注文前にほか全ての処理を終了する
                    let order_params = OrderParams {
                        // 同じ注文IDを使用する
                        order_id: Some(set_order_link_id.clone()),
                        side: order_config.side.clone(),
                        price: target_price,
                        qty: ramaining_qty_as_order_qty,
                        is_post_only: order_config.is_post_only,
                    };
                    if is_test {
                        info!("[test] order created, params: {:?}", order_params.clone());
                        let mut w = cloned_order_manage.lock().await;
                        w.set_order(order_params.clone().order_id.unwrap().clone());

                        let mut w = cloned_logger.write().await;
                        w.add(Log {
                            level: "info".to_string(),
                            message: format!("[test] order created, params: {:?}", order_params.clone()),
                            timestamp: chrono::Local::now().to_string(),
                        });
                        continue;
                    }

                    match exchange_client.place_order(&order_params.clone()).await {
                        Ok(latest_order_id) => {
                            // - set_order: 注文ID及び最終注文時間を更新する
                            {
                                let mut w = cloned_order_manage.lock().await;
                                w.set_order(latest_order_id.clone());
                            }

                            let mut w = cloned_logger.write().await;
                            w.add(Log {
                                level: "info".to_string(),
                                message: format!("order created, params: {:?}", order_params),
                                timestamp: chrono::Local::now().to_string(),
                            });
                        }
                        Err(e) => {
                                {
                                    let mut w = cloned_order_manage.lock().await;
                                    w.set_error_order();
                                }

                                let mut w = cloned_logger.write().await;
                                w.add(Log {
                                    level: "error".to_string(),
                                    message: format!("order error: {:?}", e),
                                    timestamp: chrono::Local::now().to_string(),
                                });
                            continue;
                        }
                    };
                }
                _ = pending::<()>() => {
                    // handle.abort()を待つ
                }
            }
        }
    }));

    let handle_orderboard = exchange_config
        .orderboard(
            target_symbol.clone(),
            tx_ws_orderboard,
            recive_rest_orderboard,
            tx_rest_orderboard,
        )
        .await
        .unwrap();
    handles.push(handle_orderboard);

    let handle_ticker = exchange_config
        .ticker(
            target_symbol.clone(),
            tx_ws_ticker,
            recive_rest_ticker,
            tx_rest_ticker,
        )
        .await
        .unwrap();
    handles.push(handle_ticker);

    let handle_position = exchange_config
        .position(
            target_symbol.clone(),
            tx_ws_position,
            recive_rest_position,
            tx_rest_position,
        )
        .await
        .unwrap();
    handles.push(handle_position);

    Ok(handles)
}

