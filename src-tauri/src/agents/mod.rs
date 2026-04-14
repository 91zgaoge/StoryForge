//! Agent System - 智能代理系统
//!
//! 提供创作辅助的智能Agent框架
//! 
//! ## Agent类型
//! - Writer: 写作助手 - 生成和改写内容
//! - Inspector: 质检员 - 检查内容质量
//! - OutlinePlanner: 大纲规划师 - 设计故事结构
//! - StyleMimic: 风格模仿师 - 分析和模仿文风
//! - PlotAnalyzer: 情节分析师 - 分析情节复杂度

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod service;
pub mod novel_creation;
pub mod memory_compressor;
pub mod commentator;
pub mod distiller;

// ==================== 核心Trait ====================

/// Agent特性 - 所有Agent必须实现
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent名称
    fn name(&self) -> &str;
    
    /// Agent描述
    fn description(&self) -> &str;
    
    /// 执行Agent任务
    async fn execute(
        &self,
        context: &AgentContext,
        input: &str,
    ) -> Result<AgentResult, Box<dyn std::error::Error>>;
}

// ==================== 数据结构 ====================

/// Agent执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub story_id: String,
    pub story_title: String,
    pub genre: String,       // 题材
    pub tone: String,        // 文风
    pub pacing: String,      // 节奏
    pub chapter_number: u32,
    pub characters: Vec<CharacterInfo>,
    pub previous_chapters: Vec<ChapterSummary>,
}

/// 角色信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterInfo {
    pub name: String,
    pub personality: String,
    pub role: String,
}

/// 章节摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterSummary {
    pub title: String,
    pub number: u32,
    pub summary: String,
}

/// Agent执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub content: String,
    pub score: Option<f32>,  // 0.0 - 1.0
    pub suggestions: Vec<String>,
}

// ==================== 辅助函数 ====================

impl AgentContext {
    /// 创建最小上下文（用于测试）
    pub fn minimal(story_id: String, input: String) -> Self {
        Self {
            story_id,
            story_title: "未命名作品".to_string(),
            genre: "小说".to_string(),
            tone: "中性".to_string(),
            pacing: "正常".to_string(),
            chapter_number: 1,
            characters: vec![],
            previous_chapters: vec![],
        }
    }
    
    /// 构建角色描述字符串
    pub fn format_characters(&self) -> String {
        if self.characters.is_empty() {
            "暂无角色信息".to_string()
        } else {
            self.characters
                .iter()
                .map(|c| format!("{}（{}）: {}", c.name, c.role, c.personality))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
    
    /// 构建前文摘要
    pub fn format_previous_chapters(&self) -> String {
        if self.previous_chapters.is_empty() {
            "这是第一章".to_string()
        } else {
            self.previous_chapters
                .iter()
                .map(|c| format!("第{}章 {}: {}", c.number, c.title, c.summary))
                .collect::<Vec<_>>()
                .join("\n\n")
        }
    }
}

impl AgentResult {
    /// 创建简单结果
    pub fn simple(content: String) -> Self {
        Self {
            content,
            score: None,
            suggestions: vec![],
        }
    }
    
    /// 创建带评分的结果
    pub fn with_score(content: String, score: f32) -> Self {
        Self {
            content,
            score: Some(score.clamp(0.0, 1.0)),
            suggestions: vec![],
        }
    }
    
    /// 是否高质量
    pub fn is_high_quality(&self) -> bool {
        self.score.map(|s| s >= 0.8).unwrap_or(true)
    }
}
