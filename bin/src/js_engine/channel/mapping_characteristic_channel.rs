pub enum EventType {
    /// 读取结果 <-
    ReadResult,
    UpdateCmd,
}

pub struct MappingCharacteristicChannel {
    // pub sender: tokio::sync::mpsc::Sender<MappingChannelMsg>,

    // 读取结果
    // pub read_result_recv: mpsc::Receiver<u8>,
}