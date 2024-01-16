use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use anyhow::anyhow;
use log::info;
use sea_orm::JsonValue;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::timeout;

pub type MsgId = u64;

/// 结果发送器
pub type ResultSendPointer = Arc<broadcast::Sender<(MsgId, EventResultType)>>;
// pub type ResultRecvPointer = Arc<broadcast::Sender<(MsgId, EventResultType)>>;

pub enum EventType {
    /// 读取结果 <-
    ReadProperty,
    UpdateCmd,
}

#[derive(Clone)]
pub enum EventResultType {
    ReadPropertyResult(JsonValue),
}


pub struct MappingCharacteristicSender {
    pub sender: mpsc::Sender<(MsgId, EventType)>,
    pub id: AtomicU64,
    /// 读取结果接收器,需要从sender 中订阅
    pub read_result_recv: ResultSendPointer,
}


impl MappingCharacteristicSender {
    pub fn new(sender: mpsc::Sender<(MsgId, EventType)>, read_result_recv: Arc<broadcast::Sender<(MsgId, EventResultType)>>) -> Self {
        Self {
            sender,
            id: Default::default(),
            read_result_recv,
        }
    }

    pub async fn read_property(&self) -> anyhow::Result<JsonValue> {
        //发送读取指令
        let id = self.id.fetch_add(1, Ordering::SeqCst);
        self.sender.send((id, EventType::ReadProperty)).await?;
        // 等待读取结果
        let mut recv = self.read_result_recv.subscribe();

        let resp = timeout(Duration::from_secs(2), async move {
            /* while let Ok((resp_id, resp)) = recv.recv().await {
                 if resp_id == id {
                     if let EventResultType::ReadPropertyResult(value) = resp {
                         info!("js engine recv resp:{:?}", value);
                         return Ok(value);
                     }
                 }
             }*/
            return Err(anyhow!("读取通道已经关闭关闭"));
        }).await.map_err(|f| anyhow!("读取响应超时"))?;
        resp
    }
}


/// 特征的接收器
/// 用于js 接受
pub struct MappingCharacteristicRecv {
    pub result_sender: ResultSendPointer,
    /// 读取结果
    pub js_recv: Option<mpsc::Receiver<(MsgId, EventType)>>,
    /// 等待
    pub exit_recv: Option<oneshot::Receiver<u8>>,
}


/// 特征映射接收器
impl MappingCharacteristicRecv {
    pub fn new(result_sender: ResultSendPointer) -> (Self, oneshot::Sender<u8>, mpsc::Sender<(MsgId, EventType)>) {
        let (tx, rx) = oneshot::channel();
        let (sender, read_result_recv) = mpsc::channel(10);
        return (Self {
            result_sender,
            js_recv: Some(read_result_recv),
            exit_recv: Some(rx),
        }, tx, sender);
    }

    /// hap 设备读取值
    pub fn hap_read() {}
    pub fn split(self) {}
}