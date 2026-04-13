//! V3 架构数据模型
//! 
//! 包含场景化叙事、知识图谱、工作室配置等新模型

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};

// ==================== 场景模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub story_id: String,
    pub sequence_number: i32,
    pub title: Option<String>,
    
    // 戏剧结构
    pub dramatic_goal: Option<String>,
    pub external_pressure: Option<String>,
    pub conflict_type: Option<ConflictType>,
    
    // 角色参与
    pub characters_present: Vec<String>,
    pub character_conflicts: Vec<CharacterConflict>,
    
    // 内容
    pub content: Option<String>,
    
    // 场景设置
    pub setting_location: Option<String>,
    pub setting_time: Option<String>,
    pub setting_atmosphere: Option<String>,
    
    // 关联
    pub previous_scene_id: Option<String>,
    pub next_scene_id: Option<String>,
    
    // 元数据
    pub model_used: Option<String>,
    pub cost: Option<f64>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    
    // 置信度评分 (0-1)
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    ManVsMan,        // 人与人
    ManVsSelf,       // 人与自我
    ManVsSociety,    // 人与社会
    ManVsNature,     // 人与自然
    ManVsTechnology, // 人与科技
    ManVsFate,       // 人与命运
    ManVsSupernatural, // 人与超自然
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ConflictType::ManVsMan => "人与人",
            ConflictType::ManVsSelf => "人与自我",
            ConflictType::ManVsSociety => "人与社会",
            ConflictType::ManVsNature => "人与自然",
            ConflictType::ManVsTechnology => "人与科技",
            ConflictType::ManVsFate => "人与命运",
            ConflictType::ManVsSupernatural => "人与超自然",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for ConflictType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "人与人" | "ManVsMan" => Ok(ConflictType::ManVsMan),
            "人与自我" | "ManVsSelf" => Ok(ConflictType::ManVsSelf),
            "人与社会" | "ManVsSociety" => Ok(ConflictType::ManVsSociety),
            "人与自然" | "ManVsNature" => Ok(ConflictType::ManVsNature),
            "人与科技" | "ManVsTechnology" => Ok(ConflictType::ManVsTechnology),
            "人与命运" | "ManVsFate" => Ok(ConflictType::ManVsFate),
            "人与超自然" | "ManVsSupernatural" => Ok(ConflictType::ManVsSupernatural),
            _ => Err(format!("Unknown conflict type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterConflict {
    pub character_a_id: String,
    pub character_b_id: String,
    pub conflict_nature: String,
    pub stakes: String,
}

// ==================== 保留配置模型 (Phase 1.4) ====================

/// 艾宾浩斯遗忘曲线配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// 衰减率 λ (0.01 架构级, 0.05 默认, 0.1 瞬态)
    pub lambda: f64,
    /// 每次访问的强化奖励
    pub reinforcement_bonus: f64,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            lambda: 0.05,
            reinforcement_bonus: 0.1,
        }
    }
}

/// 保留优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPriority {
    Critical,    // 关键记忆（世界观、主要角色）
    High,        // 重要记忆（次要角色、关键事件）
    Medium,      // 普通记忆
    Low,         // 可压缩记忆
    Forgotten,   // 已遗忘/可归档
}

// ==================== 场景版本模型 (新增) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneVersion {
    pub id: String,
    pub scene_id: String,
    pub version_number: i32,
    
    // 版本内容快照
    pub title: Option<String>,
    pub content: Option<String>,
    pub dramatic_goal: Option<String>,
    pub external_pressure: Option<String>,
    pub conflict_type: Option<ConflictType>,
    pub characters_present: Vec<String>,
    pub character_conflicts: Vec<CharacterConflict>,
    pub setting_location: Option<String>,
    pub setting_time: Option<String>,
    pub setting_atmosphere: Option<String>,
    
    // 版本元数据
    pub word_count: i32,
    pub change_summary: String,
    pub created_by: CreatorType,  // user/ai/system
    pub model_used: Option<String>, // AI生成时使用的模型
    pub confidence_score: Option<f32>, // AI生成置信度
    
    // 版本链 (Supersession)
    pub previous_version_id: Option<String>,
    pub superseded_by: Option<String>, // 被哪个版本取代
    
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreatorType {
    User,
    Ai,
    System,
}

impl std::fmt::Display for CreatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CreatorType::User => "user",
            CreatorType::Ai => "ai",
            CreatorType::System => "system",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for CreatorType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(CreatorType::User),
            "ai" => Ok(CreatorType::Ai),
            "system" => Ok(CreatorType::System),
            _ => Err(format!("Unknown creator type: {}", s)),
        }
    }
}

// ==================== 世界观模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBuilding {
    pub id: String,
    pub story_id: String,
    pub concept: String,
    pub rules: Vec<WorldRule>,
    pub history: Option<String>,
    pub cultures: Vec<Culture>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: RuleType,
    pub importance: i32, // 1-10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Magic,       // 魔法规则
    Technology,  // 科技规则
    Social,      // 社会规则
    Physical,    // 物理规则
    Biological,  // 生物规则
    Historical,  // 历史规则
    Cultural,    // 文化规则
    Custom,      // 自定义
}

impl std::fmt::Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RuleType::Magic => "魔法",
            RuleType::Technology => "科技",
            RuleType::Social => "社会",
            RuleType::Physical => "物理",
            RuleType::Biological => "生物",
            RuleType::Historical => "历史",
            RuleType::Cultural => "文化",
            RuleType::Custom => "自定义",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Culture {
    pub name: String,
    pub description: String,
    pub customs: Vec<String>,
    pub values: Vec<String>,
}

// ==================== 场景设置模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub id: String,
    pub story_id: String,
    pub name: String,
    pub description: Option<String>,
    pub location_type: LocationType,
    pub sensory_details: SensoryDetails,
    pub significance: Option<String>,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    City,
    Building,
    Nature,
    Underground,
    Underwater,
    Space,
    Dream,
    Virtual,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensoryDetails {
    pub visual: Vec<String>,
    pub auditory: Vec<String>,
    pub olfactory: Vec<String>,
    pub tactile: Vec<String>,
    pub gustatory: Vec<String>,
}

// ==================== 文字风格模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStyle {
    pub id: String,
    pub story_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tone: Option<String>,
    pub pacing: Option<String>,
    pub vocabulary_level: Option<String>,
    pub sentence_structure: Option<String>,
    pub custom_rules: Vec<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// ==================== 知识图谱模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub story_id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub attributes: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub first_seen: DateTime<Local>,
    pub last_updated: DateTime<Local>,
    
    // 置信度评分 (0-1)
    pub confidence_score: Option<f32>,
    // 访问计数（用于遗忘曲线）
    pub access_count: i32,
    // 最后访问时间
    pub last_accessed: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Character,
    Location,
    Item,
    Organization,
    Concept,
    Event,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EntityType::Character => "Character",
            EntityType::Location => "Location",
            EntityType::Item => "Item",
            EntityType::Organization => "Organization",
            EntityType::Concept => "Concept",
            EntityType::Event => "Event",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for EntityType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Character" | "角色" => Ok(EntityType::Character),
            "Location" | "地点" => Ok(EntityType::Location),
            "Item" | "物品" => Ok(EntityType::Item),
            "Organization" | "组织" => Ok(EntityType::Organization),
            "Concept" | "概念" => Ok(EntityType::Concept),
            "Event" | "事件" => Ok(EntityType::Event),
            _ => Err(format!("Unknown entity type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub story_id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: RelationType,
    pub strength: f32,
    pub evidence: Vec<String>,
    pub first_seen: DateTime<Local>,
    
    // 置信度评分
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    // 人际关系
    Friend,
    Enemy,
    Family,
    Lover,
    Mentor,
    Rival,
    Ally,
    
    // 物品关系
    LocatedAt,
    BelongsTo,
    Uses,
    Owns,
    Created,
    Destroyed,
    
    // 组织关系
    PartOf,
    Leads,
    MemberOf,
    FounderOf,
    
    // 因果关系
    Causes,
    Enables,
    Prevents,
    ResultsIn,
    
    // 语义关系
    SimilarTo,
    OppositeOf,
    RelatedTo,
    EvolvesInto,
    
    // 动态关系
    Supersedes,   // 取代
    Contradicts,  // 矛盾
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RelationType::Friend => "Friend",
            RelationType::Enemy => "Enemy",
            RelationType::Family => "Family",
            RelationType::Lover => "Lover",
            RelationType::Mentor => "Mentor",
            RelationType::Rival => "Rival",
            RelationType::Ally => "Ally",
            RelationType::LocatedAt => "LocatedAt",
            RelationType::BelongsTo => "BelongsTo",
            RelationType::Uses => "Uses",
            RelationType::Owns => "Owns",
            RelationType::Created => "Created",
            RelationType::Destroyed => "Destroyed",
            RelationType::PartOf => "PartOf",
            RelationType::Leads => "Leads",
            RelationType::MemberOf => "MemberOf",
            RelationType::FounderOf => "FounderOf",
            RelationType::Causes => "Causes",
            RelationType::Enables => "Enables",
            RelationType::Prevents => "Prevents",
            RelationType::ResultsIn => "ResultsIn",
            RelationType::SimilarTo => "SimilarTo",
            RelationType::OppositeOf => "OppositeOf",
            RelationType::RelatedTo => "RelatedTo",
            RelationType::EvolvesInto => "EvolvesInto",
            RelationType::Supersedes => "Supersedes",
            RelationType::Contradicts => "Contradicts",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for RelationType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Friend" => Ok(RelationType::Friend),
            "Enemy" => Ok(RelationType::Enemy),
            "Family" => Ok(RelationType::Family),
            "Lover" => Ok(RelationType::Lover),
            "Mentor" => Ok(RelationType::Mentor),
            "Rival" => Ok(RelationType::Rival),
            "Ally" => Ok(RelationType::Ally),
            "LocatedAt" => Ok(RelationType::LocatedAt),
            "BelongsTo" => Ok(RelationType::BelongsTo),
            "Uses" => Ok(RelationType::Uses),
            "Owns" => Ok(RelationType::Owns),
            "Created" => Ok(RelationType::Created),
            "Destroyed" => Ok(RelationType::Destroyed),
            "PartOf" => Ok(RelationType::PartOf),
            "Leads" => Ok(RelationType::Leads),
            "MemberOf" => Ok(RelationType::MemberOf),
            "FounderOf" => Ok(RelationType::FounderOf),
            "Causes" => Ok(RelationType::Causes),
            "Enables" => Ok(RelationType::Enables),
            "Prevents" => Ok(RelationType::Prevents),
            "ResultsIn" => Ok(RelationType::ResultsIn),
            "SimilarTo" => Ok(RelationType::SimilarTo),
            "OppositeOf" => Ok(RelationType::OppositeOf),
            "RelatedTo" => Ok(RelationType::RelatedTo),
            "EvolvesInto" => Ok(RelationType::EvolvesInto),
            "Supersedes" => Ok(RelationType::Supersedes),
            "Contradicts" => Ok(RelationType::Contradicts),
            _ => Err(format!("Unknown relation type: {}", s)),
        }
    }
}

// ==================== 工作室配置模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioConfig {
    pub id: String,
    pub story_id: String,
    pub pen_name: Option<String>,
    pub llm_config: LlmStudioConfig,
    pub ui_config: UiStudioConfig,
    pub agent_bots: Vec<AgentBotConfig>,
    pub frontstage_theme: Option<String>,
    pub backstage_theme: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmStudioConfig {
    pub default_provider: String,
    pub default_model: String,
    pub generation_temperature: f32,
    pub max_tokens: i32,
    pub profiles: Vec<LlmProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProfile {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiStudioConfig {
    pub frontstage_font_size: i32,
    pub frontstage_font_family: String,
    pub frontstage_line_height: f32,
    pub frontstage_paper_color: String,
    pub frontstage_text_color: String,
    pub backstage_theme: String,
    pub backstage_accent_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBotConfig {
    pub id: String,
    pub name: String,
    pub agent_type: AgentBotType,
    pub enabled: bool,
    pub llm_profile_id: String,
    pub system_prompt: String,
    pub custom_settings: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentBotType {
    WorldBuilding,  // 世界观助手
    Character,      // 人物助手
    WritingStyle,   // 文风助手
    Plot,           // 情节助手
    Scene,          // 场景助手
    Memory,         // 记忆助手
}

// ==================== Request/Response 模型 ====================

#[derive(Debug, Deserialize)]
pub struct CreateSceneRequest {
    pub story_id: String,
    pub sequence_number: i32,
    pub title: Option<String>,
    pub dramatic_goal: Option<String>,
    pub external_pressure: Option<String>,
    pub conflict_type: Option<String>,
    pub characters_present: Vec<String>,
    pub setting_location: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSceneRequest {
    pub title: Option<String>,
    pub dramatic_goal: Option<String>,
    pub external_pressure: Option<String>,
    pub conflict_type: Option<String>,
    pub characters_present: Option<Vec<String>>,
    pub character_conflicts: Option<Vec<CharacterConflict>>,
    pub content: Option<String>,
    pub setting_location: Option<String>,
    pub setting_time: Option<String>,
    pub setting_atmosphere: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorldBuildingRequest {
    pub story_id: String,
    pub concept: String,
    pub rules: Option<Vec<WorldRule>>,
    pub history: Option<String>,
    pub cultures: Option<Vec<Culture>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWritingStyleRequest {
    pub story_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tone: Option<String>,
    pub pacing: Option<String>,
    pub vocabulary_level: Option<String>,
    pub sentence_structure: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StudioExportRequest {
    pub story_id: String,
    pub include_world_building: bool,
    pub include_characters: bool,
    pub include_writing_style: bool,
    pub include_scenes: bool,
    pub include_llm_config: bool,
    pub include_ui_config: bool,
    pub include_agent_bots: bool,
}

#[derive(Debug, Serialize)]
pub struct StudioExportData {
    pub manifest: ExportManifest,
    pub story: crate::db::models::Story,
    pub world_building: Option<WorldBuilding>,
    pub characters: Vec<crate::db::models::Character>,
    pub writing_style: Option<WritingStyle>,
    pub scenes: Vec<Scene>,
    pub studio_config: Option<StudioConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportManifest {
    pub version: String,
    pub exported_at: DateTime<Local>,
    pub story_id: String,
    pub story_title: String,
}

// ==================== 场景版本相关请求/响应 (新增) ====================

#[derive(Debug, Deserialize)]
pub struct CreateSceneVersionRequest {
    pub scene_id: String,
    pub change_summary: String,
    pub created_by: String, // "user" | "ai" | "system"
    pub model_used: Option<String>,
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct SceneVersionDiff {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompareVersionsResponse {
    pub version_a: SceneVersion,
    pub version_b: SceneVersion,
    pub differences: Vec<SceneVersionDiff>,
}
