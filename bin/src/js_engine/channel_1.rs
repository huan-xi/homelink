use tokio::sync::mpsc;

/// 特折
pub struct MappingCharacteristicChannel {
    // pub sender: tokio::sync::mpsc::Sender<MappingChannelMsg>,

    // 读取结果
    pub read_result_recv: mpsc::Receiver<u8>,
    pub js_recv: mpsc::Receiver<u8>,

}
// pub struct read_result_recv

impl MappingCharacteristicChannel {
    pub fn new() -> Self {
        // let (sender, recv) = tokio::sync::mpsc::channel(100);
       /* Self {
            // sender,
            // recv,
        }*/
        todo!();
    }
    /// hap 设备读取值
    pub fn hap_read() {}
    pub fn split(self) {}
}


/// 设备事件
pub struct DeviceEventChannel {}