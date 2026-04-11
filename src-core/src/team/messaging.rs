//! Inter-agent messaging system (from open-multi-agent)

use crate::team::Message;
use std::collections::VecDeque;
use parking_lot::Mutex;

pub struct MessageBus {
    messages: Mutex<VecDeque<Message>>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(VecDeque::new()),
        }
    }

    pub fn publish(&self,
        msg: Message) {
        self.messages.lock().push_back(msg);
    }

    pub fn subscribe(&self,
        _agent_name: &str) -> Vec<Message> {
        // Return messages addressed to this agent or broadcast
        self.messages.lock()
            .iter()
            .filter(|m| m.to.is_none() || m.to.as_deref() == Some(_agent_name))
            .cloned()
            .collect()
    }
}
