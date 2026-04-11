use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 聊天会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub story_id: String,
    pub title: String,
    pub context: ChatContext,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatContext {
    General,
    StoryWriting { story_id: String },
    CharacterDevelopment { character_id: String },
    PlotPlanning,
    WorldBuilding,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub model: String,
    pub tokens_used: Option<i32>,
    pub suggested_actions: Option<Vec<String>>,
}

/// 写作建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingSuggestion {
    pub id: String,
    pub category: SuggestionCategory,
    pub title: String,
    pub description: String,
    pub confidence: f32,
    pub related_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Plot,
    Character,
    Style,
    Grammar,
    Pacing,
    Dialogue,
}

/// 聊天管理器
pub struct ChatManager {
    sessions: HashMap<String, ChatSession>,
}

impl ChatManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// 创建新会话
    pub fn create_session(
        &mut self,
        story_id: String,
        title: String,
        context: ChatContext,
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

    /// 获取会话
    pub fn get_session(&self, session_id: &str) -> Option<&ChatSession> {
        self.sessions.get(session_id)
    }

    /// 获取故事的所有会话
    pub fn get_story_sessions(&self, story_id: &str) -> Vec<&ChatSession> {
        self.sessions
            .values()
            .filter(|s| s.story_id == story_id)
            .collect()
    }

    /// 添加消息
    pub fn add_message(
        &mut self,
        session_id: &str,
        role: MessageRole,
        content: String,
        metadata: Option<MessageMetadata>,
    ) -> Result<ChatMessage, String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            let message = ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                role,
                content,
                timestamp: Utc::now(),
                metadata,
            };

            session.messages.push(message.clone());
            session.updated_at = Utc::now();

            Ok(message)
        } else {
            Err("Session not found".to_string())
        }
    }

    /// 删除会话
    pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        if self.sessions.remove(session_id).is_some() {
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// 生成系统提示词
    pub fn generate_system_prompt(
        &self,
        context: &ChatContext,
        story: Option<&crate::db::Story>,
        chapters: Option<&[crate::db::Chapter]>,
        characters: Option<&[crate::db::Character]>,
    ) -> String {
        match context {
            ChatContext::General => {
                "你是一位专业的小说写作助手，可以帮助作者进行创意写作、情节构思、角色塑造等方面的工作。请提供专业、有建设性的建议。".to_string()
            }
            ChatContext::StoryWriting { .. } => {
                let mut prompt = "你正在协助作者创作小说。".to_string();
                if let Some(story) = story {
                    prompt.push_str(&format!(
                        "\n当前故事：《{}》\n类型：{}\n基调：{}\n",
                        story.title,
                        story.genre.as_deref().unwrap_or("未指定"),
                        story.tone.as_deref().unwrap_or("未指定")
                    ));
                }
                if let Some(chapters) = chapters {
                    prompt.push_str(&format!("\n已有 {} 个章节\n", chapters.len()));
                }
                prompt.push_str("\n请根据上下文提供写作建议。");
                prompt
            }
            ChatContext::CharacterDevelopment { .. } => {
                let mut prompt = "你正在协助进行角色开发。".to_string();
                if let Some(characters) = characters {
                    prompt.push_str(&format!("\n故事中有 {} 个角色\n", characters.len()));
                    for char in characters.iter().take(3) {
                        prompt.push_str(&format!("- {}\n", char.name));
                    }
                }
                prompt.push_str("\n请帮助完善角色设定、性格特征和发展弧线。");
                prompt
            }
            ChatContext::PlotPlanning => {
                "你是一位专业的小说情节策划师。请帮助作者设计引人入胜的情节结构、转折点和高潮。提供具体、可操作的建议。".to_string()
            }
            ChatContext::WorldBuilding => {
                "你是一位世界观设计专家。请帮助作者构建丰富、连贯的故事世界，包括历史、地理、文化、规则等方面。".to_string()
            }
        }
    }

    /// 生成写作建议
    pub async fn generate_suggestions(
        &self,
        content: &str,
        suggestion_type: SuggestionCategory,
    ) -> Result<Vec<WritingSuggestion>, String> {
        // 这里会调用LLM生成建议
        // 简化版本返回模拟数据
        let suggestions = vec![
            WritingSuggestion {
                id: uuid::Uuid::new_v4().to_string(),
                category: suggestion_type.clone(),
                title: "建议标题".to_string(),
                description: "基于你的内容，建议改进...".to_string(),
                confidence: 0.85,
                related_content: Some(content[..100.min(content.len())].to_string()),
            },
        ];
        Ok(suggestions)
    }
}
