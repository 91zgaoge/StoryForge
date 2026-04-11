use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub story_id: String,
    pub title: String,
    pub context: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

pub struct ChatManager {
    sessions: HashMap<String, ChatSession>,
}

impl ChatManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(
        &mut self,
        story_id: String,
        title: String,
        context: String,
    ) -> ChatSession {
        let session = ChatSession {
            id: uuid::Uuid::new_v4().to_string(),
            story_id,
            title,
            context,
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.sessions.insert(session.id.clone(), session.clone());
        session
    }

    pub fn get_session(&self, session_id: &str
    ) -> Option<&ChatSession> {
        self.sessions.get(session_id)
    }

    pub fn get_story_sessions(
        &self, story_id: &str
    ) -> Vec<&ChatSession> {
        self.sessions
            .values()
            .filter(|s| s.story_id == story_id)
            .collect()
    }

    pub fn add_message(
        &mut self,
        session_id: &str,
        role: String,
        content: String,
    ) -> Result<ChatMessage, String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            let message = ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                role,
                content,
                timestamp: Utc::now(),
            };

            session.messages.push(message.clone());
            session.updated_at = Utc::now();
            Ok(message)
        } else {
            Err("Session not found".to_string())
        }
    }

    pub fn delete_session(
        &mut self, session_id: &str
    ) -> Result<(), String> {
        if self.sessions.remove(session_id).is_some() {
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }
}
