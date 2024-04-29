use std::sync::Arc;
use dashmap::DashMap;
use socketioxide::socket::Sid;
use crate::api::state::AppState;

pub type SocketContext = Arc<SocketContextInner>;


pub trait Task: Send + Sync {
    fn name(&self) -> String;
}

pub struct SocketContextInner {
    pub app: AppState,
    //任务
    pub socket_tasks: DashMap<Sid, Vec<Box<dyn Task>>>,
    //开
}

impl SocketContextInner {
    pub fn new(app: AppState) -> Self {
        Self {
            app,
            socket_tasks: DashMap::new(),
        }
    }
    pub fn remove_task(&self, sid: Sid, name: &str) {
        if let Some(mut tasks) = self.socket_tasks.get_mut(&sid) {
            tasks.retain(|task| task.name() != name);
        }
    }
    pub fn push_task(&self, sid: Sid, task: Box<dyn Task>) {
        let mut list = self.socket_tasks.entry(sid).or_insert_with(Vec::new);
        //判断是否存在
        if !list.iter().any(|t| t.name() == task.name()) {
            list.push(task);
        }
    }
}