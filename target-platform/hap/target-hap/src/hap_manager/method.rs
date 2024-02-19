use anyhow::anyhow;
use hap::server::Server;
use crate::hap_manager::HapManageInner;

impl HapManageInner {

    pub async fn close(&self) {}
    pub async fn stop_server(&self, bid: i64) -> anyhow::Result<()> {
        if let Some((_, task)) = self.server_map.remove(&bid) {
            task.sender
                .send(true)
                .map_err(|e| anyhow!("发送退出指令失败:{:?}", e))?;
        }
        Ok(())
    }

    pub async fn refresh_accessory_config(&self, aid: u64) {
        if let Some(bid) = self.accessory_map.get(&aid) {
            if let Some(server) = self.server_map.get(&bid.bid) {
                server.server.configuration_number_incr().await;
            }
        }
    }
}