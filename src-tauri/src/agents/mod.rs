use serde::{Deserialize, Serialize};

pub mod writer;
pub mod inspector;
pub mod outline_planner;
pub mod style_mimic;
pub mod plot_analyzer;

// Re-exports for public API
#[allow(unused_imports)]
pub use writer::WriterAgent;
#[allow(unused_imports)]
pub use inspector::InspectorAgent;
#[allow(unused_imports)]
pub use outline_planner::OutlinePlannerAgent;
#[allow(unused_imports)]
pub use style_mimic::StyleMimicAgent;
#[allow(unused_imports)]
pub use plot_analyzer::PlotComplexityAgent;

/// 执行上下文 - 包含所有 Agent 需要的上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub story_id: String,
    pub story_title: String,
    pub genre: String,
    pub tone: String,
    pub pacing: String,
    pub chapter_number: u32,
    pub outline: String,
    pub previous_chapters: Vec<ChapterSummary>,
    pub characters: Vec<CharacterInfo>,
    pub key_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterSummary {
    pub chapter_number: i32,
    pub title: String,
    pub summary: String,
    pub key_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterInfo {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub current_state: String,
}

/// Agent 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub content: String,
    pub score: Option<f32>,
    pub suggestions: Vec<String>,
}

/// Agent trait - 所有 Agent 必须实现
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    async fn execute(
        &self,
        context: &AgentContext,
        input: &str,
    ) -> Result<AgentResult, Box<dyn std::error::Error>>;
}