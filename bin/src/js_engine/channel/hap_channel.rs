use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use anyhow::anyhow;
use axum::body::HttpBody;
use deno_runtime::deno_core::Resource;
use futures_util::future::BoxFuture;
use sea_orm::JsonValue;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::sync::mpsc::Sender;
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use crate::js_engine::init_js_engine::MsgId;

/// 结果接收器
pub type ResultRecvPointer = Arc<broadcast::Sender<(MsgId, FromModuleResp)>>;

#[derive(Clone,Serialize,Deserialize)]
pub struct ReadPropertyParam {
    pub service_tag: String,
    pub ch_tag: String,
}

impl ReadPropertyParam {
    pub fn new(service_tag: String, ch_tag: String) -> Self {
        Self {
            service_tag,
            ch_tag,
        }
    }
}

/// 发送给模块的时间类型
#[derive(Clone,Serialize,Deserialize)]
pub enum ToModuleEvent {
    /// 读取结果 <-
    ReadProperty(ReadPropertyParam),
    UpdateCmd,
}

/// 从模块过来的返回值
#[derive(Clone)]
pub enum FromModuleResp {
    ReadPropertyResult(JsonValue),
}

/// 发送给模块的发送器
pub struct ToHapModuleSender {
    /// 发送器
    pub sender: mpsc::Sender<(MsgId, ToModuleEvent)>,
    pub id: AtomicU64,
    /// 读取结果接收器,需要从sender 中订阅
    pub read_result_recv: ResultRecvPointer,
}

impl ToHapModuleSender {
    pub fn new(sender: mpsc::Sender<(MsgId, ToModuleEvent)>, read_result_recv: ResultRecvPointer) -> Self {
        Self {
            sender,
            id: Default::default(),
            read_result_recv,
        }
    }

    pub async fn read_property(&self, param: ReadPropertyParam) -> anyhow::Result<JsonValue> {
        //发送读取指令
        let id = self.id.fetch_add(1, Ordering::SeqCst);
        self.sender.send((id, ToModuleEvent::ReadProperty(param))).await?;
        // 等待读取结果
        let mut recv = self.read_result_recv.subscribe();

        let resp = timeout(Duration::from_secs(2), async move {
            while let Ok((resp_id, resp)) = recv.recv().await {
                if resp_id == id {
                    /* if let FromModuleResp::ReadPropertyResult(value) = resp {
                         info!("js engine recv resp:{:?}", value);
                         return Ok(value);
                     }*/
                }
            }
            return Err(anyhow!("读取通道已经关闭关闭"));
        }).await.map_err(|f| anyhow!("读取响应超时"))?;
        resp
    }
}

// pub type Receiver = broadcast::Receiver<(MsgId, FromModuleResp)>;

pub struct ReceiverResource(pub BoxFuture<'static, Option<(MsgId, ToModuleEvent)>>);

impl Resource for ReceiverResource {
    fn name(&self) -> Cow<str> {
        Cow::from("ReceiverResource")
    }
}

/// 特征的接收器
/// 用于js 接受
pub struct HapAccessoryModuleRecv {
    pub result_sender: ResultRecvPointer,
    /// 读取结果
    pub recv: Option<mpsc::Receiver<(MsgId, ToModuleEvent)>>,
    // 等待
    pub exit_recv: Option<oneshot::Receiver<u8>>,
}


impl HapAccessoryModuleRecv {
    pub fn new(result_sender: ResultRecvPointer) -> (HapAccessoryModuleRecv, oneshot::Sender<u8>, Sender<(MsgId, ToModuleEvent)>) {
        let (tx, exit_recv) = oneshot::channel();
        let (sender, recv) = mpsc::channel(10);
        (Self {
            result_sender,
            recv: Some(recv),
            exit_recv: Some(exit_recv),
        }, tx, sender)
    }
    pub fn take_receiver_resource(&mut self) -> anyhow::Result<ReceiverResource> {
        let mut recv = self.recv.take()
            .ok_or(anyhow!("改通道接收器已被取走"))?;
        Ok(ReceiverResource(async move {
            recv.recv().await
        }.boxed()))
    }
}


pub fn channel() -> (Arc<ToHapModuleSender>, HapAccessoryModuleRecv, oneshot::Sender<u8>) {
    let (tx, _) = broadcast::channel(10);
    let module_response_tx = Arc::new(tx);
    let (module_recv, exit, sender) = HapAccessoryModuleRecv::new(module_response_tx.clone());
    let module_sender = ToHapModuleSender::new(sender, module_response_tx.clone());
    let module_sender = Arc::new(module_sender);
    return (module_sender, module_recv, exit);
}