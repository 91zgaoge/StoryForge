//! 两步思维链Ingest流程
//! 
//! 基于llm_wiki方法论：
//! Step 1: 分析 - 使用LLM深入分析内容
//! Step 2: 生成 - 基于分析结果生成结构化知识

use crate::llm::LlmService;
use crate::db::models_v3::{Entity, EntityType, Relation, RelationType};
use serde::{Deserialize, Serialize};
use serde_json;
use chrono::Local;

/// Ingest管道
pub struct IngestPipeline {
    llm_service: LlmService,
}

/// 待Ingest的内容
#[derive(Debug, Clone)]
pub struct IngestContent {
    pub text: String,
    pub source: String,
    pub story_id: String,
    pub scene_id: Option<String>,
}

/// Step 1: 内容分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    /// 识别出的实体
    pub entities: Vec<AnalyzedEntity>,
    /// 实体间的关系
    pub relationships: Vec<AnalyzedRelation>,
    /// 关键事件
    pub events: Vec<AnalyzedEvent>,
    /// 情感分析
    pub sentiment: SentimentAnalysis,
    /// 伏笔和照应
    pub foreshadowing: Vec<Foreshadowing>,
    /// 主题标签
    pub themes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedEntity {
    pub name: String,
    pub entity_type: String,
    pub mentions: Vec<usize>, // 在文本中的位置
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedRelation {
    pub source: String,
    pub target: String,
    pub relation_type: String,
    pub evidence: String,
    pub strength: f32, // 0-1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedEvent {
    pub description: String,
    pub participants: Vec<String>,
    pub importance: i32, // 1-10
    pub trigger: String,
    pub consequence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    pub overall: String, // positive/negative/neutral
    pub intensity: f32, // 0-1
    pub arc: Vec<SentimentPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentPoint {
    pub position: f32, // 0-1 文本位置
    pub sentiment: String,
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foreshadowing {
    pub content: String,
    pub type_: String, // setup/payoff
    pub related_to: Vec<String>,
}

/// Step 2: 生成的知识
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedKnowledge {
    pub entities: Vec<EntityProfile>,
    pub relations: Vec<RelationProfile>,
    pub events: Vec<EventProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityProfile {
    pub name: String,
    pub entity_type: EntityType,
    pub description: String,
    pub attributes: serde_json::Value,
    pub importance: i32, // 1-10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationProfile {
    pub source: String,
    pub target: String,
    pub relation_type: RelationType,
    pub description: String,
    pub strength: f32,
    pub evolution: String, // 关系如何发展
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventProfile {
    pub title: String,
    pub description: String,
    pub importance: i32,
    pub impact: String,
}

impl IngestPipeline {
    pub fn new(llm_service: LlmService) -> Self {
        Self { llm_service }
    }

    /// 执行两步思维链Ingest
    pub async fn ingest(&self, content: &IngestContent) -> Result<IngestResult, Box<dyn std::error::Error>> {
        // Step 1: 分析阶段
        let analysis = self.analyze_content(content).await?;
        
        // Step 2: 生成阶段
        let knowledge = self.generate_knowledge(&analysis).await?;
        
        // 转换为数据库模型
        let entities = self.convert_entities(&knowledge.entities, content);
        let relations = self.convert_relations(&knowledge.relations, content);
        
        Ok(IngestResult {
            analysis,
            knowledge,
            entities,
            relations,
        })
    }

    /// Step 1: 使用LLM分析内容
    async fn analyze_content(&self, content: &IngestContent) -> Result<ContentAnalysis, Box<dyn std::error::Error>> {
        let prompt = format!(
            r#"请深入分析以下小说内容，提取结构化信息：

【内容】
{}

【来源】
{}

请按以下JSON格式输出分析结果：
{{
  "entities": [
    {{
      "name": "实体名称",
      "entity_type": "类型(Character/Location/Item/Organization/Concept/Event)",
      "mentions": [位置索引],
      "attributes": {{}}
    }}
  ],
  "relationships": [
    {{
      "source": "源实体",
      "target": "目标实体", 
      "relation_type": "关系类型",
      "evidence": "证据文本",
      "strength": 0.8
    }}
  ],
  "events": [
    {{
      "description": "事件描述",
      "participants": ["参与者"],
      "importance": 8,
      "trigger": "触发原因",
      "consequence": "后果影响"
    }}
  ],
  "sentiment": {{
    "overall": "positive/negative/neutral",
    "intensity": 0.7,
    "arc": [{{"position": 0.5, "sentiment": "tense", "intensity": 0.8}}]
  }},
  "foreshadowing": [
    {{
      "content": "伏笔内容",
      "type_": "setup/payoff",
      "related_to": ["相关内容"]
    }}
  ],
  "themes": ["主题1", "主题2"]
}}

注意：
1. 识别所有重要实体（人物、地点、物品、组织、概念、事件）
2. 分析实体间的关系和互动
3. 提取关键情节转折点
4. 标注伏笔和照应
5. 确保JSON格式正确"#,
            content.text,
            content.source
        );

        let response = self.llm_service.generate(prompt, None, None).await?;
        let analysis: ContentAnalysis = serde_json::from_str(&response.content)?;
        
        Ok(analysis)
    }

    /// Step 2: 基于分析生成结构化知识
    async fn generate_knowledge(&self, analysis: &ContentAnalysis) -> Result<GeneratedKnowledge, Box<dyn std::error::Error>> {
        let analysis_json = serde_json::to_string_pretty(analysis)?;
        
        let prompt = format!(
            r#"基于以下内容分析结果，生成知识库条目：

【分析结果】
{}

请生成详细的知识档案，按以下JSON格式输出：
{{
  "entities": [
    {{
      "name": "实体名称",
      "entity_type": "Character/Location/Item/Organization/Concept/Event",
      "description": "详细描述",
      "attributes": {{}},
      "importance": 8
    }}
  ],
  "relations": [
    {{
      "source": "源实体",
      "target": "目标实体",
      "relation_type": "Friend/Enemy/Family/...",
      "description": "关系描述",
      "strength": 0.8,
      "evolution": "关系发展趋势"
    }}
  ],
  "events": [
    {{
      "title": "事件标题",
      "description": "详细描述",
      "importance": 9,
      "impact": "对故事的影响"
    }}
  ]
}}

注意：
1. 为每个实体生成完整档案
2. 计算关系强度（0-1）
3. 评估事件重要性（1-10）
4. 确保JSON格式正确"#,
            analysis_json
        );

        let response = self.llm_service.generate(prompt, None, None).await?;
        let knowledge: GeneratedKnowledge = serde_json::from_str(&response.content)?;
        
        Ok(knowledge)
    }

    /// 转换为数据库实体模型
    fn convert_entities(&self, profiles: &[EntityProfile], content: &IngestContent) -> Vec<Entity> {
        profiles.iter().map(|profile| {
            Entity {
                id: uuid::Uuid::new_v4().to_string(),
                story_id: content.story_id.clone(),
                name: profile.name.clone(),
                entity_type: profile.entity_type.clone(),
                attributes: profile.attributes.clone(),
                embedding: None, // 后续通过Embedding服务生成
                first_seen: Local::now(),
                last_updated: Local::now(),
            }
        }).collect()
    }

    /// 转换为数据库关系模型
    fn convert_relations(&self, profiles: &[RelationProfile], content: &IngestContent) -> Vec<Relation> {
        profiles.iter().map(|profile| {
            Relation {
                id: uuid::Uuid::new_v4().to_string(),
                story_id: content.story_id.clone(),
                source_id: profile.source.clone(),
                target_id: profile.target.clone(),
                relation_type: profile.relation_type.clone(),
                strength: profile.strength,
                evidence: vec![content.source.clone()],
                first_seen: Local::now(),
            }
        }).collect()
    }
}

/// Ingest结果
#[derive(Debug, Clone)]
pub struct IngestResult {
    pub analysis: ContentAnalysis,
    pub knowledge: GeneratedKnowledge,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

/// Ingest批次（用于批量处理）
pub struct IngestBatch {
    pub contents: Vec<IngestContent>,
}

impl IngestBatch {
    pub fn new() -> Self {
        Self { contents: vec![] }
    }

    pub fn add(&mut self, content: IngestContent) {
        self.contents.push(content);
    }

    pub async fn process(&self, pipeline: &IngestPipeline) -> Vec<Result<IngestResult, Box<dyn std::error::Error>>> {
        let mut results = vec![];
        
        for content in &self.contents {
            results.push(pipeline.ingest(content).await);
        }
        
        results
    }
}

impl Default for IngestBatch {
    fn default() -> Self {
        Self::new()
    }
}
