use tokio::sync::oneshot;

pub struct MainChannel {
    pub tx: oneshot::Sender<i64>,
    pub rx: oneshot::Receiver<i64>,
}