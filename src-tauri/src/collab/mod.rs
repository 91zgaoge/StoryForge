pub mod ot;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[allow(unused_imports)]
pub use ot::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabSession {
    pub id: String,
    pub story_id: String,
    pub chapter_id: Option<String>,
    pub participants: Vec<Participant>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub user_id: String,
    pub user_name: String,
    pub cursor_position: Option<CursorPosition>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: i32,
    pub column: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub operation_type: OperationType,
    pub position: CursorPosition,
    pub content: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Insert,
    Delete,
    Replace,
}

pub struct CollabManager {
    sessions: HashMap<String, CollabSession>,
    operations: Vec<EditOperation>,
}

impl CollabManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            operations: Vec::new(),
        }
    }

    pub fn create_session(
        &mut self,
        story_id: String,
        chapter_id: Option<String>,
    ) -> CollabSession {
        let session = CollabSession {
            id: uuid::Uuid::new_v4().to_string(),
            story_id,
            chapter_id,
            participants: Vec::new(),
            created_at: Utc::now(),
        };

        self.sessions.insert(session.id.clone(), session.clone());
        session
    }

    pub fn join_session(
        &mut self,
        session_id: &str,
        user_id: String,
        user_name: String,
    ) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.participants.push(Participant {
                user_id,
                user_name,
                cursor_position: None,
                joined_at: Utc::now(),
            });
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    pub fn leave_session(
        &mut self,
        session_id: &str,
        user_id: &str,
    ) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.participants.retain(|p| p.user_id != user_id);
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    pub fn apply_operation(
        &mut self,
        operation: EditOperation,
    ) -> Result<(), String> {
        self.operations.push(operation);
        Ok(())
    }

    pub fn get_session(&self, session_id: &str
    ) -> Option<&CollabSession> {
        self.sessions.get(session_id)
    }

    pub fn update_cursor(
        &mut self,
        session_id: &str,
        user_id: &str,
        position: CursorPosition,
    ) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if let Some(participant) = session.participants
                .iter_mut()
                .find(|p| p.user_id == user_id) {
                participant.cursor_position = Some(position);
                Ok(())
            } else {
                Err("User not in session".to_string())
            }
        } else {
            Err("Session not found".to_string())
        }
    }
}
