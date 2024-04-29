use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures_util::StreamExt;
use log::{error, info};
use tokio::sync::{broadcast, RwLock};
use ble_monitor::parse_advertisement::{parse_advertisement, ServiceDataPacket};
use hap::futures::Stream;

#[derive(Clone)]
pub struct BleManager {
    inner: Arc<BleManagerInner>,
}

impl Deref for BleManager {
    type Target = BleManagerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl BleManager {
    pub fn new() -> Self {
        BleManager {
            inner: Arc::new(BleManagerInner::new()),
        }
    }
}

/// 开启
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum Status {
    /// 关闭
    Off,
    /// 开启
    On,
    Normal,
    /// 无蓝牙适配器
    EmptyAdapter,
}

#[derive(Debug)]
pub struct BleDataEvent {}

pub struct BleManagerInner {
    pub status: RwLock<Status>,
    pub sender: broadcast::Sender<ServiceDataPacket>,
    pub adapter: RwLock<Option<Adapter>>,

}

impl BleManagerInner {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        BleManagerInner {
            status: RwLock::new(Status::EmptyAdapter),
            sender: tx,
            adapter: RwLock::new(None),
        }
    }
    pub fn recv(&self) -> broadcast::Receiver<ServiceDataPacket> {
        self.sender.subscribe()
    }
    pub async fn init(&self) {
        if let Err(e) = self.init0().await {
            error!("init ble error: {:?}", e);
            *self.status.write().await = Status::Off;
        };
    }

    pub async fn init0(&self) -> anyhow::Result<()> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        if adapters.is_empty() {
            *self.status.write().await = Status::EmptyAdapter;
            return Ok(());
        }
        *self.status.write().await = Status::On;
        let adapter = adapters.into_iter().next().unwrap();
        adapter.start_scan(ScanFilter::default()).await?;
        let mut events = adapter.events().await?;
        self.adapter.write().await.replace(adapter);
        let sender = self.sender.clone();
        //接受处理事件
        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                        for (uuid, bytes) in &service_data {
                            match parse_advertisement(uuid, bytes.as_slice()) {
                                Ok(data) => {
                                    if let Some(data) = data {
                                        // 上报设备事件
                                        //尝试解包获取类型,

                                        let _ = sender.send(data);
                                    }
                                }
                                Err(e) => {
                                    //  info!("parse_advertisement error:{:?}", e);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }


    pub async fn adapter_event_listener(&self) -> anyhow::Result<Pin<Box<dyn Stream<Item=CentralEvent> + Send>>> {
        let read = self.adapter.read().await;
        if let Some(adapter) = read.as_ref() {
            let a = adapter.events().await?;
            return Ok(a);
        }
        return Err(anyhow::anyhow!("adapter not found"));
    }
}