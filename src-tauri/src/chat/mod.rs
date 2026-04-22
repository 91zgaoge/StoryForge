use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::db::DbPool;
use rusqlite::params;

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
    pool: DbPool,
}

impl ChatManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_session(
        &self,
        story_id: String,
        title: String,
        context: String,
    ) -> Result<ChatSession, String> {
        let session = ChatSession {
            id: uuid::Uuid::new_v4().to_string(),
            story_id: story_id.clone(),
            title: title.clone(),
            context: context.clone(),
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO chat_sessions (id, story_id, title, context, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &session.id,
                &story_id,
                &title,
                &context,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| e.to_string())?;

        Ok(session)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<ChatSession>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, title, context, created_at, updated_at
             FROM chat_sessions WHERE id = ?1"
        ).map_err(|e| e.to_string())?;

        let session_result = stmt.query_row([session_id], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            Ok(ChatSession {
                id: row.get(0)?,
                story_id: row.get(1)?,
                title: row.get(2)?,
                context: row.get(3)?,
                messages: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        });

        let mut session = match session_result {
            Ok(s) => s,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };

        // Load messages
        let mut msg_stmt = conn.prepare(
            "SELECT id, role, content, timestamp FROM chat_messages WHERE session_id = ?1 ORDER BY timestamp"
        ).map_err(|e| e.to_string())?;

        let messages = msg_stmt.query_map([session_id], |row| {
            let ts_str: String = row.get(3)?;
            Ok(ChatMessage {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                timestamp: DateTime::parse_from_rfc3339(&ts_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        session.messages = messages;
        Ok(Some(session))
    }

    pub fn get_story_sessions(
        &self,
        story_id: &str,
    ) -> Result<Vec<ChatSession>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, title, context, created_at, updated_at
             FROM chat_sessions WHERE story_id = ?1 ORDER BY updated_at DESC"
        ).map_err(|e| e.to_string())?;

        let sessions = stmt.query_map([story_id], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            Ok(ChatSession {
                id: row.get(0)?,
                story_id: row.get(1)?,
                title: row.get(2)?,
                context: row.get(3)?,
                messages: Vec::new(),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(sessions)
    }

    pub fn add_message(
        &self,
        session_id: &str,
        role: String,
        content: String,
    ) -> Result<ChatMessage, String> {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: role.clone(),
            content: content.clone(),
            timestamp: Utc::now(),
        };

        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO chat_messages (id, session_id, role, content, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &message.id,
                session_id,
                &role,
                &content,
                message.timestamp.to_rfc3339(),
            ],
        ).map_err(|e| e.to_string())?;

        // Update session updated_at
        let _ = conn.execute(
            "UPDATE chat_sessions SET updated_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), session_id],
        );

        Ok(message)
    }

    pub fn delete_session(&self, session_id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM chat_sessions WHERE id = ?1",
            [session_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}
