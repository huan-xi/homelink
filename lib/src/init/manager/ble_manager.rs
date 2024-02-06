use std::ops::Deref;
use std::sync::Arc;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures_util::StreamExt;
use log::info;
use tokio::sync::{broadcast, RwLock};
use ble_monitor::parse_advertisement::{parse_advertisement, ServiceDataPacket};

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
}

impl BleManagerInner {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        BleManagerInner {
            status: RwLock::new(Status::EmptyAdapter),
            sender: tx,
        }
    }
    pub fn recv(&self) -> broadcast::Receiver<ServiceDataPacket> {
        self.sender.subscribe()
    }

    pub async fn init(&self) -> anyhow::Result<()> {
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
        let sender = self.sender.clone();
        //接受处理事件
        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                        for (uuid, bytes) in &service_data {
                            match parse_advertisement(uuid, bytes.as_slice()) {
                                Ok(data) => {
                                    // 上报设备事件
                                    // info!("parse_advertisement: {:?}", data);
                                    let _ = sender.send(data);
                                }
                                Err(e) => {
                                    //  info!("parse_advertisement error:{:?}", e);
                                }
                            }
                        }
                        // if let Some(data) = service_data.get(&atc::UUID) {
                        //     info!("test");
                        // }
                        /*    let a = Reading::decode(&service_data);
                            if let Some(a) = a {
                                error!("ServiceDataAdvertisement:  {:?}", a);
                            };*/
                        // println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    //处理蓝牙数据
}