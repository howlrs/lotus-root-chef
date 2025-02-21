use log::error;
use tokio::task::{JoinError, JoinHandle};

pub struct Workers {
    // 返り値を持たない非同期タスクのハンドル
    // .abort() でキャンセル可能
    // .await で終了待ち
    // spawn内で.awaitに対してキャンセル命令を送ることで終了させる
    pub handles: Vec<JoinHandle<()>>,
}

impl Workers {
    pub fn new() -> Self {
        Workers {
            handles: Vec::new(),
        }
    }

    pub fn extend(&mut self, handles: Vec<JoinHandle<()>>) {
        self.handles.extend(handles);
    }

    pub async fn abort_all(&mut self) -> Result<(), JoinError> {
        for handle in self.handles.drain(..) {
            handle.abort();
            let _ = match handle.await {
                Ok(_) => Ok(()),
                // JoinError::Cancelledはabort()による正常な終了
                Err(e) => {
                    if e.is_cancelled() {
                        Ok(())
                    } else {
                        error!("abort error: {:?}", e);
                        Err(e)
                    }
                }
            };
        }

        Ok(())
    }
}
