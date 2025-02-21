use std::sync::Arc;

use log::error;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::target::exchanges::models::Position;

// ポジションを取得する
// Websocket非実装の場合ポジションは空であるため
// ポジションがない場合はRestRequestを送信し、取得する
pub async fn get_positions(
    positions: Arc<RwLock<Vec<Position>>>,
    tx: mpsc::Sender<()>,
    mut rx: broadcast::Receiver<Vec<Position>>,
) -> Vec<Position> {
    let r = positions.read().await;

    if !r.is_empty() {
        r.clone()
    } else {
        // RestRequestを送信
        tx.send(()).await.unwrap();

        // RestRequestの受信を待機
        match rx.recv().await {
            Ok(pos) => pos,
            Err(e) => {
                error!("get_positions error: {:?}", e);
                vec![]
            }
        }
    }
}

// 指定した注文IDのポジションを集計する
pub fn aggrigate_position(order_id: String, positions: Vec<Position>) -> Position {
    let mut pos = Position::default();
    if positions.is_empty() {
        return pos;
    }

    for p in positions {
        if order_id != p.order_id {
            continue;
        }

        pos.qty += p.qty;
        pos.price += p.price;
    }

    pos
}
