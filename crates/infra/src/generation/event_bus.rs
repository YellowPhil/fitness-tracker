use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use domain::types::UserId;
use tokio::sync::broadcast;

use crate::repos::generation_jobs::GenerationJob;

#[derive(Clone, Default)]
pub struct GenerationEventBus {
    channels: Arc<Mutex<HashMap<i64, broadcast::Sender<GenerationJob>>>>,
}

impl GenerationEventBus {
    pub fn subscribe(&self, user_id: UserId) -> broadcast::Receiver<GenerationJob> {
        let mut channels = self.channels.lock().expect("event bus lock poisoned");
        let sender = channels
            .entry(user_id.as_i64())
            .or_insert_with(|| broadcast::channel(128).0)
            .clone();
        sender.subscribe()
    }

    pub fn publish(&self, job: GenerationJob) {
        let mut channels = self.channels.lock().expect("event bus lock poisoned");
        let sender = channels
            .entry(job.user_id.as_i64())
            .or_insert_with(|| broadcast::channel(128).0)
            .clone();
        let _ = sender.send(job);
    }
}
