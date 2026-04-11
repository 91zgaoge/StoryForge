//! StoryState schema definitions
//! 
//! 基于 Zod Schema 的 Rust 实现
//! 使用 garde 进行验证

use garde::Validate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 全局故事状态
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StoryState {
    #[garde(skip)]
    pub metadata: Metadata,
    
    #[garde(skip)]
    pub world: WorldState,
    
    #[garde(skip)]
    pub characters: HashMap<String, Character>,
    
    #[garde(skip)]
    pub writing_style: WritingStyle,
    
    #[garde(skip)]
    pub plot_threads: Vec<PlotThread>,
    
    #[garde(skip)]
    pub chapter_complexity: HashMap<u32, String>,
    
    #[garde(skip)]
    pub quality_metrics: QualityState,
}

impl Default for StoryState {
    fn default() -> Self {
        Self {
            metadata: Metadata {
                title: "Untitled Story".to_string(),
                current_chapter: 0,
                total_chapters: None,
                last_updated: Utc::now(),
                version: "1.0.0".to_string(),
            },
            world: WorldState {
                rules: HashMap::new(),
                timeline: Vec::new(),
                locations: HashMap::new(),
                current_time: "Day 1".to_string(),
            },
            characters: HashMap::new(),
            writing_style: WritingStyle {
                tone: "neutral".to_string(),
                pacing: Pacing::Medium,
                vocabulary_density: 0.5,
                sentence_complexity: SentenceComplexity::Moderate,
                dialogue_ratio: 0.3,
                show_dont_tell: 0.5,
                evolution_history: Vec::new(),
            },
            plot_threads: Vec::new(),
            chapter_complexity: HashMap::new(),
            quality_metrics: QualityState {
                consistency_score: 1.0,
                last_inspection: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub current_chapter: u32,
    pub total_chapters: Option<u32>,
    pub last_updated: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub rules: HashMap<String, serde_json::Value>,
    pub timeline: Vec<TimelineEvent>,
    pub locations: HashMap<String, Location>,
    pub current_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub chapter: u32,
    pub key_events: Vec<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub description: String,
    pub current_state: String,
    pub important_events: Vec<u32>,
}

/// 角色定义 - 核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub base_profile: BaseProfile,
    /// 动态特质 - 核心进化机制
    pub dynamic_traits: Vec<DynamicTrait>,
    pub current_mood: Mood,
    pub relationships: HashMap<String, Relationship>,
    pub arc_status: ArcStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseProfile {
    pub age: Option<u32>,
    pub appearance: Option<String>,
    pub background: String,
    pub core_desire: String,
    pub fear: Option<String>,
}

/// 动态特质 - "越写越懂"的核心
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTrait {
    pub trait: String,
    pub source_chapter: u32,
    pub confidence: f32, // 0.0 - 1.0
    pub evidence: String,
    pub status: TraitStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraitStatus {
    Active,
    Dormant,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mood {
    Happy,
    Sad,
    Angry,
    Neutral,
    Anxious,
    Excited,
    Determined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub target_id: String,
    pub relation_type: RelationType,
    pub level: i8, // -10 (深仇) to 10 (至交)
    pub dynamics: String,
    pub history: Vec<RelationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Friend,
    Enemy,
    Lover,
    Family,
    Neutral,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationEvent {
    pub chapter: u32,
    pub event: String,
    pub impact: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArcStatus {
    Rising,
    Stable,
    Falling,
    Transforming,
}

/// 写作风格画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStyle {
    pub tone: String,
    pub pacing: Pacing,
    pub vocabulary_density: f32, // 0.0 - 1.0
    pub sentence_complexity: SentenceComplexity,
    pub dialogue_ratio: f32,
    pub show_dont_tell: f32,
    pub evolution_history: Vec<StyleAdjustment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pacing {
    Slow,
    Medium,
    Fast,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentenceComplexity {
    Simple,
    Moderate,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleAdjustment {
    pub chapter: u32,
    pub parameter: String,
    pub old_value: String,
    pub new_value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotThread {
    pub id: String,
    pub description: String,
    pub status: ThreadStatus,
    pub start_chapter: u32,
    pub target_resolution_chapter: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadStatus {
    Active,
    Resolved,
    Dropped,
    Foreshadowing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityState {
    pub consistency_score: f32,
    pub last_inspection: Option<InspectionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionRecord {
    pub chapter: u32,
    pub score: f32,
    pub issues: Vec<String>,
}
