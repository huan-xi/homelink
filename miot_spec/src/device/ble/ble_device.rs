use std::mem;
use std::sync::{Arc};
use std::time::Duration;
use futures_util::future::{BoxFuture, join_all};
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotDeviceType, MiotSpecDevice};
use crate::device::wifi_device::ExitCode;
use crate::proto::miio_proto::MiIOProtocol;
use futures_util::FutureExt;
use hex::FromHex;
use log::{debug, error, info, warn};
use packed_struct::derive::PackedStruct;
use packed_struct::{PackedStruct, PackingError};
use tokio::sync::RwLock;
use crate::device::ble::value_types::{BleValue, ContactValue, HumidityValue, TemperatureValue};
use crate::device::gateway::gateway::OpenMiioGatewayDevice;
use crate::device::value::{DataListener, DataEmitter, ListenDateType};
use crate::proto::protocol::RecvMessage;
use crate::proto::transport::{MiioTransport, Transport};

/// https://tasmota.github.io/docs/Bluetooth/
/// 低功耗蓝牙设备
/// 目前低功耗蓝牙设备只支持被动获取数据



pub struct BleDevice {
    pub info: DeviceInfo,
    base: BaseMiotSpecDevice,
    gateway: Arc<OpenMiioGatewayDevice>,
    // 值
    pub values: Arc<RwLock<BleValue>>,

    emitter: Arc<RwLock<DataEmitter<ListenDateType>>>,
}

impl MiotSpecDevice for BleDevice {
    fn get_info(&self) -> &DeviceInfo { &self.info }

    fn get_proto(&self) -> BoxFuture<Result<Arc<MiIOProtocol>, ExitCode>> { self.gateway.get_proto() }

    fn get_device_type(&self) -> MiotDeviceType {
        return MiotDeviceType::Ble(self);
    }

    /// 1.同步网关的状态
    fn run(&self) -> BoxFuture<Result<(), ExitCode>> {
        async move {
            match self.get_proto().await?.get_transport() {
                Transport::OpenMiIOMqtt(mqtt) => {
                    let mut recv = mqtt.recv();
                    while let Ok(msg) = recv.recv().await {
                        let data = msg.get_json_data();
                        if let Some(method) = data.get("method") {
                            //异步蓝牙事件
                            if method.as_str() == Some("_async.ble_event") {
                                if let Some(v) = data.get("params") {
                                    let did = v.as_object()
                                        .and_then(|i| i.get("dev"))
                                        .and_then(|i| i.as_object())
                                        .and_then(|i| i.get("did"))
                                        .and_then(|i| i.as_str());
                                    if Some(self.info.did.as_str()) == did {
                                        //获取evt

                                        let evt_vec = v.as_object()
                                            .and_then(|i| i.get("evt"))
                                            .and_then(|i| i.as_array());
                                        if let Some(data) = evt_vec {
                                            let mut ble_value = BleValue::default();
                                            for val in data {
                                                if let Some(eid) = val.get("eid").and_then(|i| i.as_i64()) {
                                                    if let Some(edata) = val.get("edata")
                                                        .and_then(|i| i.as_str())
                                                        .and_then(|i| <Vec<u8>>::from_hex(i.as_bytes()).ok()) {
                                                        match eid {
                                                            0x1004 => {
                                                                //温度 4119
                                                                let bytes: [u8; 2] = edata.try_into().unwrap();
                                                                let temperature = TemperatureValue::unpack(&bytes).unwrap();
                                                                ble_value.temperature = Some(temperature.clone());
                                                                self.values.clone().write().await.temperature.replace(temperature);
                                                            }
                                                            0x1006 => {
                                                                // 湿度值 4102
                                                                // 这里没看懂
                                                                // if device.model in (903, 1371):
                                                                // # two models has bug, they increase humidity on each msg by 0.1
                                                                // value = int(value)
                                                                let bytes: [u8; 2] = edata.try_into().unwrap();
                                                                match HumidityValue::unpack(&bytes) {
                                                                    Ok(v) => {
                                                                        ble_value.humidity = Some(v.clone());
                                                                        self.values.clone().write().await.humidity.replace(v);
                                                                    }
                                                                    Err(e) => {
                                                                        error!("解码湿度值错误:{:?}",e);
                                                                    }
                                                                };
                                                            }
                                                            0x1017 => {
                                                                // The duration of the unmanned state, in seconds
                                                                //idle_time
                                                                // let bytes: [u8; 4] = edata.try_into().unwrap();
                                                            }

                                                            0x4803 => {
                                                                //电池
                                                                // let bytes: [u8; 1] = edata.try_into().unwrap();
                                                                // self.values.clone().write().await.battery.replace(bytes[0]);
                                                            }
                                                            0x100A => {
                                                                // if device.model == 2691:
                                                                // # this sensor sends some kind of counter once an hour instead
                                                                // # of the battery, so filter out the false values
                                                                // let bytes: [u8; 1] = edata.try_into().unwrap();
                                                                // self.values.clone().write().await.battery.replace(bytes[0]);
                                                            }
                                                            0x1018 => {
                                                                // # Door Sensor 2: 0 - dark, 1 - light light
                                                                // # hass: On means light detected, Off means no light
                                                                // let bytes: [u8; 1] = edata.try_into().unwrap();
                                                            }
                                                            0x1019 => {
                                                                //门窗传感器
                                                                let bytes: [u8; 2] = edata.try_into().unwrap();
                                                                let val= ContactValue::unpack(&bytes).unwrap();
                                                                ble_value.contact = Some(val.clone());
                                                                self.values.clone().write().await.contact.replace(val);
                                                                /*match bytes[0] {
                                                                    0 => {//开}
                                                                    1 => {//关}
                                                                    2=>{timeout}
                                                                    3=>{reset}
                                                                    _ => {
                                                                        error!("未知的门窗传感器事件:{:?}",bytes)
                                                                    }
                                                                }*/
                                                            }
                                                            _ => {
                                                                warn!("未知的蓝牙事件: {:?}", val)
                                                            }
                                                        }
                                                    }

                                                    /*     match eid {
                                                             0x1004 => {
                                                                 //温度
                                                                 if let Some(edata) = val.get("edata")
                                                                     .and_then(|i| i.as_str()) {
                                                                     let bytes = <[u8; 2]>::from_hex(edata.as_bytes()).unwrap();
                                                                     let temperature = TemperatureValue::unpack(&bytes).unwrap();
                                                                     ble_value.temperature = Some(temperature.clone());
                                                                     self.values.clone().write().await.temperature.replace(temperature);
                                                                 }
                                                             }
                                                             0x1006 => {
                                                                 // 湿度值 4102
                                                                 // 这里没看懂
                                                                 // if device.model in (903, 1371):
                                                                 // # two models has bug, they increase humidity on each msg by 0.1
                                                                 // value = int(value)
                                                                 if let Some(edata) = val.get("edata")
                                                                     .and_then(|i| i.as_str()) {
                                                                     let bytes = <[u8; 2]>::from_hex(edata.as_bytes()).unwrap();
                                                                     let temperature = HumidityValue::unpack(&bytes).unwrap();
                                                                     ble_value.humidity = Some(temperature.clone());
                                                                     self.values.clone().write().await.humidity.replace(temperature);
                                                                 }
                                                             }
                                                             _ => {
                                                                 warn!("未知的蓝牙事件: {:?}", val)
                                                             }
                                                         }*/
                                                }
                                            }
                                            //提交事件
                                            self.emit(ble_value).await;
                                        }

                                        /* TemperatureValue::from()
                                         ble.temperature=
                                         info!("蓝牙设备事件: {:?}", v);*/
                                    };


                                    /*if let Some(obj) = v.as_object() {
                                        if let Some(dev) = obj.get("dev") {
                                        }
                                    };*/
                                };
                            }
                        }
                    }
                }
                _ => {
                    return Err(ExitCode::ConnectErr);
                }
            }
            Ok(())
        }.boxed()
    }

    fn add_listener(&self, listener: DataListener<ListenDateType>) -> BoxFuture<()> {
        async move {
            self.emitter.clone().write().await.add_listener(listener).await;
        }.boxed()
    }
}

impl BleDevice {
    pub fn new(info: DeviceInfo, gateway: Arc<OpenMiioGatewayDevice>) -> Self {
        //网关上设置一个监听器,监听属于我的消息
        Self {
            info,
            base: BaseMiotSpecDevice {
                ..BaseMiotSpecDevice::default()
            },
            gateway,
            values: Default::default(),
            emitter: Arc::new(RwLock::new(DataEmitter::new())),
        }
    }
    pub async fn emit(&self, event: BleValue) {
        debug!("emitting event: {:?}", event);
        // let event = ListenDateType::BleData(event);

        self.emitter.read().await.emit(ListenDateType::BleData(event)).await;
    }
}


#[test]
pub fn test() {
    //8C00
    let token_bytes = <[u8; 2]>::from_hex("D002".as_bytes()).unwrap();
    let a = TemperatureValue::unpack(&token_bytes).unwrap();
    println!("{:?}", a);
}