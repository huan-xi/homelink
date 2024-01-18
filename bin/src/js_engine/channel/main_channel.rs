use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use anyhow::anyhow;
use deno_runtime::deno_core::{Resource, v8};
use deno_runtime::deno_core::url::Url;
use serde_aux::prelude::deserialize_number_from_string;
use sea_orm::JsonValue;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::sync::mpsc::Sender;
use crate::js_engine::channel::MsgId;
use futures_util::FutureExt;
use impl_new::New;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use crate::js_engine::channel::params::{BindDeviceModuleParam, ExecuteSideModuleParam, OnCharReadParam, OnCharUpdateParam, OnDeviceEventParam};

pub type ResultSenderPointer = Arc<broadcast::Sender<(MsgId, FromModuleResp)>>;

#[derive(Clone, Serialize, Deserialize, Debug, impl_new::New)]
pub struct V8Value {
    pub value: Option<JsonValue>,
}

/// 执行模块的返回值,返回通道
#[derive(Clone, Serialize, Deserialize, Debug, New)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteHapModuleResult {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub ch_id: i64,
}


/// 从模块过来的返回值
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum FromModuleResp {
    CharReadResp(V8Value),
    CharUpdateResp,
    /// 执行模块的响应
    ExecuteModuleResp(ExecuteHapModuleResult),
    BindDeviceModuleResp,
    /// 错误
    Error(String),
    Success,
    /// 模块退出
    EnginExit(String),
}


/// 发送给模块的事件类型
#[derive(Clone, Serialize)]
#[serde(tag = "type")]
pub enum ToModuleEvent {
    ExecuteSideModule(ExecuteSideModuleParam),
    /// 绑定设备和模块 事件关系
    BindDeviceModule(BindDeviceModuleParam),
    /// 当特征值读取数据
    OnCharRead(OnCharReadParam),
    OnDeviceEvent(OnDeviceEventParam),
    OnCharUpdate(OnCharUpdateParam),
}


#[test]
pub fn test_serde() {
    let a = FromModuleResp::CharReadResp(V8Value::new(JsonValue::from(1)));
    let b = serde_json::to_string(&a).unwrap();
    println!("{}", b);
}


#[test]
pub fn test() {
    let a = FromModuleResp::ExecuteModuleResp(ExecuteHapModuleResult::new(1));
    // let a = ToModuleEvent::ExecuteSideModule(ExecuteSideModuleParam::new(1, Url::parse("http://localhost:8000/03_env.js").unwrap()));
    let b = serde_json::to_string(&a).unwrap();
    println!("{}", b);

    /*let str = r#"{
  "type": "executeSideModule",
  "chId": 1,
  "url": "http://localhost:8000/03_env.js"
}"#;
    let a = serde_json::from_str::<ToModuleEvent>(str);
    println!("{:?}", a);*/
}


/// 发送给模块的发送器
pub struct ToModuleSender {
    /// 发送器
    sender: mpsc::Sender<(MsgId, ToModuleEvent)>,
    id: AtomicU64,
    /// 读取结果接收器,需要从sender 中订阅
    pub read_result_recv: ResultSenderPointer,
}

impl ToModuleSender {
    pub async fn send(&self, event: ToModuleEvent) -> anyhow::Result<FromModuleResp> {
        let id = self.id.fetch_add(1, Ordering::SeqCst);
        self.sender.send((id, event)).await
            .map_err(|_| anyhow!("发送失败"))?;
        // 等待结果
        let res = timeout(std::time::Duration::from_secs(5), async {
            while let Ok((msg_id, resp)) = self.read_result_recv.subscribe().recv().await {
                if msg_id == id {
                    return Ok(resp);
                }
            };
            Err(anyhow!("读取错误"))
        }).await
            .map_err(|_| anyhow!("命令执行超时"))?;

        res
    }
}

impl ToModuleSender {
    pub fn new(sender: mpsc::Sender<(MsgId, ToModuleEvent)>, read_result_recv: ResultSenderPointer) -> Self {
        Self {
            sender,
            id: Default::default(),
            read_result_recv,
        }
    }
}

/// 模块持有的接收器
pub struct ModuleRecv {
    /// 发送结果
    pub result_sender: ResultSenderPointer,
    /// 接受事件
    pub recv: Option<mpsc::Receiver<(MsgId, ToModuleEvent)>>,
}

/*#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverResult {
    pub msg_id: MsgId,
    /// 事件类型
    pub event: String,
    /// 事件参数
    pub param: JsonValue,
}*/
pub type ReceiverResult = (MsgId, ToModuleEvent);

// (pub BoxFuture<'static, Option<ReceiverResult>>);
pub struct ReceiverResource {
    recv: RefCell<mpsc::Receiver<(MsgId, ToModuleEvent)>>,
}

impl ReceiverResource {
    pub async fn recv(&self) -> Option<ReceiverResult> {
        self.recv.borrow_mut().recv().await
    }
}

// pub struct SendResource(pub Arc<ResultSenderPointer>);


impl Resource for ReceiverResource {
    fn name(&self) -> Cow<str> {
        "ReceiverResource".into()
    }
}

impl ModuleRecv {
    pub fn new(result_sender: ResultSenderPointer) -> (Self, Sender<(MsgId, ToModuleEvent)>) {
        let (sender, recv) = mpsc::channel(10);
        (Self {
            result_sender,
            recv: Some(recv),
        }, sender)
    }
    pub fn task_sender_resource(&self) -> anyhow::Result<ToModuleSender> {
        // Ok(ToModuleSender::new(self.recv.as_ref().unwrap().clone(), self.result_sender.clone()))
        todo!();
    }
    pub fn take_receiver_resource(&mut self) -> anyhow::Result<ReceiverResource> {
        let mut recv = self.recv.take()
            .ok_or(anyhow!("该通道接收器已被取走"))?;
        let _ = self.result_sender.send((0, FromModuleResp::Success));

        Ok(ReceiverResource {
            recv: RefCell::new(recv),
        })
    }
}

pub fn channel() -> (Arc<ToModuleSender>, ModuleRecv) {
    let (tx, _) = broadcast::channel(10);
    let module_response_tx = Arc::new(tx);

    let (module_recv, sender) = ModuleRecv::new(module_response_tx.clone());
    let module_sender = ToModuleSender::new(sender, module_response_tx.clone());
    let module_sender = Arc::new(module_sender);
    return (module_sender, module_recv);
}