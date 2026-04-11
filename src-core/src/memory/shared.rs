//! Shared memory for teams (from open-multi-agent)

use crate::team::Message;
use std::collections::VecDeque;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct SharedMemory {
    messages: Arc<RwLock<VecDeque<Message>>>,
    max_messages: usize,
}

impl SharedMemory {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(VecDeque::new())),
            max_messages: 100,
        }
    }

    pub fn add_message(&self,
        msg: Message) {
        let mut msgs = self.messages.write();
        msgs.push_back(msg);
        
        while msgs.len() > self.max_messages {
            msgs.pop_front();
        }
    }

    pub fn get_recent(&self,
        count: usize) -> Vec<Message> {
        let msgs = self.messages.read();
        msgs.iter().rev().take(count).cloned().collect()
    }

    pub fn search(&self,
        _query: &str) -> Vec<Message> {
        // Simplified - would use vector search in production
        self.get_recent(10)
    }
}
