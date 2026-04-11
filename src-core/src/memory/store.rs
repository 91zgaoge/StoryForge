//! In-memory store for agent state

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct MemoryStore {
    entries: HashMap<String, MemoryEntry>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn set(&mut self,
        key: &str,
        value: serde_json::Value) {
        self.entries.insert(key.to_string(), MemoryEntry {
            key: key.to_string(),
            value,
            timestamp: chrono::Utc::now(),
        });
    }

    pub fn get(&self,
        key: &str) -> Option<&MemoryEntry> {
        self.entries.get(key)
    }
}
