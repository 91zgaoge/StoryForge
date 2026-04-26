//! Novel Bootstrap Workflow - 小说自动初始化工作流
//!
//! 当用户输入"写一篇XX小说"且无现有故事时，自动完成：
//! 1. 生成故事概念（标题、简介、题材）
//! 2. 生成世界观设定
//! 3. 生成主要角色
//! 4. 生成场景/章节大纲
//! 5. 生成第一章开头

use crate::agents::service::{AgentService, AgentTask, AgentType};
use crate::db::{DbPool, CreateStoryRequest, CreateCharacterRequest};
use crate::db::repositories::{StoryRepository, CharacterRepository};
use crate::db::repositories_v3::{WorldBuildingRepository, SceneRepository};
use crate::db::repositories_v3::SceneUpdate;
use crate::db::models_v3::{WorldRule, RuleType, ConflictType};
use crate::llm::LlmService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

/// 小说初始化会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapSession {
    pub id: String,
    pub status: BootstrapStatus,
    pub current_step: String,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub story_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapStatus {
    InProgress,
    Completed,
    Failed,
}

/// Bootstrap 进度事件（推送到前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapProgressEvent {
    pub session_id: String,
    pub step_name: String,
    pub step_number: usize,
    pub total_steps: usize,
    pub message: String,
}

/// 小说初始化工作流
pub struct NovelBootstrapWorkflow {
    app_handle: AppHandle,
    llm_service: LlmService,
    pool: DbPool,
}

impl NovelBootstrapWorkflow {
    pub fn new(app_handle: AppHandle) -> Self {
        let pool = app_handle.state::<DbPool>().inner().clone();
        let llm_service = LlmService::new(app_handle.clone());
        Self { app_handle, llm_service, pool }
    }

    /// 运行完整的小说初始化工作流
    pub async fn run(&self, user_premise: &str) -> Result<BootstrapSession, String> {
        let session_id = Uuid::new_v4().to_string();
        let total_steps = 5;

        // 创建持久化会话记录
        self.create_session(&session_id, total_steps).map_err(|e| format!("Failed to create session: {}", e))?;

        // Step 1: 生成故事概念
        self.update_session(&session_id, "concept", 0, None).ok();
        self.emit_progress(&session_id, "构思故事", 1, total_steps, "正在生成故事概念...");
        let story_concept = match self.generate_story_concept(user_premise).await {
            Ok(c) => c,
            Err(e) => {
                self.fail_session(&session_id, &format!("故事概念生成失败: {}", e)).ok();
                return Err(e);
            }
        };

        // 创建 Story 记录
        let story_repo = StoryRepository::new(self.pool.clone());
        let story = match story_repo.create(CreateStoryRequest {
            title: story_concept.title.clone(),
            description: Some(story_concept.description.clone()),
            genre: Some(story_concept.genre.clone()),
            style_dna_id: None,
        }) {
            Ok(s) => s,
            Err(e) => {
                self.fail_session(&session_id, &format!("创建故事失败: {}", e)).ok();
                return Err(e.to_string());
            }
        };
        let story_id = story.id.clone();
        self.update_session(&session_id, "world_building", 1, Some(&story_id)).ok();

        self.emit_progress(&session_id, "构思故事", 1, total_steps, &format!("故事《{}》已创建", story.title));

        // Step 2: 生成世界观
        self.emit_progress(&session_id, "构建世界观", 2, total_steps, "正在生成世界观设定...");
        let world_building = match self.generate_world_building(&story_id, &story_concept).await {
            Ok(w) => w,
            Err(e) => {
                self.fail_session(&session_id, &format!("世界观生成失败: {}", e)).ok();
                return Err(e);
            }
        };
        self.update_session(&session_id, "characters", 2, Some(&story_id)).ok();
        self.emit_progress(&session_id, "构建世界观", 2, total_steps, "世界观设定已保存");

        // Step 3: 生成角色
        self.emit_progress(&session_id, "设计角色", 3, total_steps, "正在生成主要角色...");
        let characters = match self.generate_characters(&story_id, &story_concept, &world_building).await {
            Ok(c) => c,
            Err(e) => {
                self.fail_session(&session_id, &format!("角色生成失败: {}", e)).ok();
                return Err(e);
            }
        };
        self.update_session(&session_id, "scenes", 3, Some(&story_id)).ok();
        self.emit_progress(&session_id, "设计角色", 3, total_steps, &format!("已创建 {} 个角色", characters.len()));

        // Step 4: 生成场景大纲
        self.emit_progress(&session_id, "规划场景", 4, total_steps, "正在生成场景大纲...");
        let scenes = match self.generate_scene_outline(&story_id, &story_concept, &characters).await {
            Ok(s) => s,
            Err(e) => {
                self.fail_session(&session_id, &format!("场景大纲生成失败: {}", e)).ok();
                return Err(e);
            }
        };
        self.update_session(&session_id, "first_chapter", 4, Some(&story_id)).ok();
        self.emit_progress(&session_id, "规划场景", 4, total_steps, &format!("已规划 {} 个场景", scenes.len()));

        // Step 5: 生成第一章
        self.emit_progress(&session_id, "撰写开篇", 5, total_steps, "正在撰写第一章...");
        let _first_chapter = match self.generate_first_chapter(&story_id, &story_concept, &world_building, &characters, &scenes).await {
            Ok(c) => c,
            Err(e) => {
                self.fail_session(&session_id, &format!("第一章生成失败: {}", e)).ok();
                return Err(e);
            }
        };
        self.complete_session(&session_id, &story_id).ok();
        self.emit_progress(&session_id, "撰写开篇", 5, total_steps, "第一章已完成");

        // 发送 DataRefresh 事件让前端刷新故事列表
        let _ = crate::window::WindowManager::send_to_frontstage(
            &self.app_handle,
            crate::window::FrontstageEvent::DataRefresh { entity: "stories".to_string() }
        );

        Ok(BootstrapSession {
            id: session_id,
            status: BootstrapStatus::Completed,
            current_step: "completed".to_string(),
            steps_completed: total_steps,
            total_steps,
            story_id: Some(story_id),
            error_message: None,
        })
    }

    // ==================== Step 1: 故事概念 ====================

    async fn generate_story_concept(&self, user_premise: &str) -> Result<StoryConcept, String> {
        let prompt = format!(
            r#"你是一位资深小说编辑。请根据用户的创意，生成一个完整的故事概念。

用户输入："{}"

请用 JSON 格式回复：
{{
  "title": "故事标题（有吸引力的中文标题）",
  "description": "一句话简介（30-50字）",
  "genre": "题材（如：都市玄幻、科幻、悬疑、古言）",
  "tone": "文风基调（如：热血、暗黑、轻松、沉重）",
  "pacing": "叙事节奏（如：快节奏、慢热、跌宕起伏）",
  "themes": ["主题1", "主题2"],
  "target_length": "预计篇幅（如：中篇30万字、长篇100万字）"
}}

要求：
1. 标题要有吸引力，避免俗套
2. 简介要概括核心冲突和卖点
3. 题材要具体，不要笼统"小说"
4. 只输出 JSON，不要其他内容"#,
            user_premise.replace('"', "'")
        );

        let response = self.llm_service.generate(prompt, Some(1024), Some(0.7)).await?;
        let content = response.content.trim();
        let json_str = Self::extract_json(content)?;
        let concept: StoryConcept = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse story concept: {}. JSON: {}", e, json_str))?;

        Ok(concept)
    }

    // ==================== Step 2: 世界观 ====================

    async fn generate_world_building(&self, story_id: &str, concept: &StoryConcept) -> Result<WorldBuildingResult, String> {
        let prompt = format!(
            r#"你是一位世界观架构师。请为以下故事构建完整的世界观设定。

故事：《{}》
题材：{}
简介：{}

请用 JSON 格式回复：
{{
  "concept": "世界观核心概念（50-100字）",
  "rules": [
    {{"name": "规则名称", "description": "规则描述", "rule_type": "physical|magic|social|historical", "importance": 8}}
  ],
  "history": "世界历史背景（200-300字）",
  "key_locations": ["关键地点1", "关键地点2"],
  "power_system": "力量体系概述（如有）"
}}

要求：
1. 规则要有创意，避免陈词滥调
2. 规则之间要有逻辑一致性
3. 重要规则（importance >= 8）不超过5条
4. 只输出 JSON，不要其他内容"#,
            concept.title, concept.genre, concept.description
        );

        let response = self.llm_service.generate(prompt, Some(2048), Some(0.6)).await?;
        let content = response.content.trim();
        let json_str = Self::extract_json(content)?;
        let wb_data: WorldBuildingData = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse world building: {}. JSON: {}", e, json_str))?;

        // 存入数据库 —— WorldBuildingRepository::create 接收 (story_id, concept)
        let repo = WorldBuildingRepository::new(self.pool.clone());
        let world_building = repo.create(story_id, &wb_data.concept).map_err(|e| e.to_string())?;

        // 更新世界观规则和历史文化
        let rules: Vec<WorldRule> = wb_data.rules.iter().map(|r| WorldRule {
            id: Uuid::new_v4().to_string(),
            name: r.name.clone(),
            description: Some(r.description.clone()),
            rule_type: match r.rule_type.as_str() {
                "physical" => RuleType::Physical,
                "magic" => RuleType::Magic,
                "social" => RuleType::Social,
                "historical" => RuleType::Historical,
                "technology" => RuleType::Technology,
                "biological" => RuleType::Biological,
                "cultural" => RuleType::Cultural,
                _ => RuleType::Custom,
            },
            importance: r.importance,
        }).collect();

        let _ = repo.update(
            &world_building.id,
            None,
            Some(&rules),
            Some(&wb_data.history),
            None,
        );

        Ok(WorldBuildingResult {
            concept: wb_data.concept,
            rules: wb_data.rules,
        })
    }

    // ==================== Step 3: 角色 ====================

    async fn generate_characters(&self, story_id: &str, concept: &StoryConcept, world: &WorldBuildingResult) -> Result<Vec<GeneratedCharacter>, String> {
        let prompt = format!(
            r#"你是一位角色设计师。请为以下故事设计 3-5 个主要角色。

故事：《{}》
题材：{}
简介：{}
世界观：{}
核心规则：{}

请用 JSON 格式回复：
{{
  "characters": [
    {{
      "name": "角色姓名",
      "role": "角色定位（主角/反派/导师/盟友/爱情线）",
      "personality": "性格特征（50字）",
      "background": "背景故事（100字）",
      "goals": "核心目标",
      "fears": "深层恐惧",
      "appearance": "外貌特征（50字）",
      "gender": "男/女/其他",
      "age": 25,
      "relationships": [{{"target": "另一个角色名", "nature": "关系性质"}}]
    }}
  ]
}}

要求：
1. 主角要有鲜明的性格弧光空间
2. 角色之间要有冲突和张力
3. 避免刻板印象
4. 只输出 JSON，不要其他内容"#,
            concept.title, concept.genre, concept.description,
            world.concept,
            world.rules.iter().map(|r| format!("{}: {}", r.name, r.description)).collect::<Vec<_>>().join("; ")
        );

        let response = self.llm_service.generate(prompt, Some(3000), Some(0.7)).await?;
        let content = response.content.trim();
        let json_str = Self::extract_json(content)?;
        let char_data: CharacterData = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse characters: {}. JSON: {}", e, json_str))?;

        // 存入数据库
        let repo = CharacterRepository::new(self.pool.clone());
        let mut generated = Vec::new();
        for c in char_data.characters {
            let background = format!("{}", c.background);
            let character = repo.create(CreateCharacterRequest {
                story_id: story_id.to_string(),
                name: c.name.clone(),
                background: Some(background),
            }).map_err(|e| e.to_string())?;

            generated.push(GeneratedCharacter {
                id: character.id,
                name: c.name,
                role: c.role,
                personality: c.personality,
                background: c.background,
                goals: c.goals,
                fears: c.fears,
                appearance: c.appearance,
                gender: c.gender,
                age: c.age,
            });
        }

        Ok(generated)
    }

    // ==================== Step 4: 场景大纲 ====================

    async fn generate_scene_outline(&self, story_id: &str, concept: &StoryConcept, characters: &[GeneratedCharacter]) -> Result<Vec<GeneratedScene>, String> {
        let character_names = characters.iter().map(|c| format!("{}({})", c.name, c.role)).collect::<Vec<_>>().join(", ");

        let prompt = format!(
            r#"你是一位大纲规划师。请为以下故事设计 8-12 个核心场景。

故事：《{}》
题材：{}
简介：{}
角色：{}

请用 JSON 格式回复：
{{
  "scenes": [
    {{
      "sequence_number": 1,
      "title": "场景标题",
      "dramatic_goal": "本场景的戏剧目标（角色想达成什么）",
      "external_pressure": "外部压力/阻碍",
      "conflict_type": "man_vs_man|man_vs_self|man_vs_society|man_vs_nature|man_vs_technology|man_vs_fate|man_vs_supernatural|man_vs_time|man_vs_morality|man_vs_identity|faction_vs_faction",
      "setting_location": "地点",
      "setting_time": "时间",
      "characters_present": ["角色名1", "角色名2"],
      "summary": "场景内容摘要（100字）"
    }}
  ]
}}

要求：
1. 场景之间要有因果关系
2. 每个场景都要推动情节或揭示人物
3. 冲突类型要多样
4. 只输出 JSON，不要其他内容"#,
            concept.title, concept.genre, concept.description, character_names
        );

        let response = self.llm_service.generate(prompt, Some(3000), Some(0.6)).await?;
        let content = response.content.trim();
        let json_str = Self::extract_json(content)?;
        let scene_data: SceneData = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse scenes: {}. JSON: {}", e, json_str))?;

        // 存入数据库 —— SceneRepository::create(story_id, sequence_number, title)
        let repo = SceneRepository::new(self.pool.clone());
        let mut generated = Vec::new();
        for s in scene_data.scenes {
            let scene = repo.create(
                story_id,
                s.sequence_number,
                Some(&s.title),
            ).map_err(|e| e.to_string())?;

            // 使用 SceneUpdate 更新额外字段
            let updates = SceneUpdate {
                title: Some(s.title.clone()),
                dramatic_goal: Some(s.dramatic_goal.clone()),
                external_pressure: Some(s.external_pressure.clone()),
                conflict_type: Some(match s.conflict_type.as_str() {
                    "man_vs_man" => ConflictType::ManVsMan,
                    "man_vs_self" => ConflictType::ManVsSelf,
                    "man_vs_society" => ConflictType::ManVsSociety,
                    "man_vs_nature" => ConflictType::ManVsNature,
                    "man_vs_technology" => ConflictType::ManVsTechnology,
                    "man_vs_fate" => ConflictType::ManVsFate,
                    "man_vs_supernatural" => ConflictType::ManVsSupernatural,
                    "man_vs_time" => ConflictType::ManVsTime,
                    "man_vs_morality" => ConflictType::ManVsMorality,
                    "man_vs_identity" => ConflictType::ManVsIdentity,
                    "faction_vs_faction" => ConflictType::FactionVsFaction,
                    _ => ConflictType::ManVsMan,
                }),
                characters_present: Some(s.characters_present.clone()),
                character_conflicts: None,
                setting_location: Some(s.setting_location.clone()),
                setting_time: Some(s.setting_time.clone()),
                setting_atmosphere: None,
                content: None,
                previous_scene_id: None,
                next_scene_id: None,
                confidence_score: Some(0.8),
                execution_stage: None,
                outline_content: None,
                draft_content: None,
            };
            let _ = repo.update(&scene.id, &updates);

            generated.push(GeneratedScene {
                id: scene.id.clone(),
                sequence_number: s.sequence_number,
                title: s.title,
                summary: s.summary,
            });
        }

        Ok(generated)
    }

    // ==================== Step 5: 第一章 ====================

    async fn generate_first_chapter(&self, story_id: &str, concept: &StoryConcept, world: &WorldBuildingResult, characters: &[GeneratedCharacter], scenes: &[GeneratedScene]) -> Result<String, String> {
        // 构建完整的 AgentContext
        let builder = crate::creative_engine::context_builder::StoryContextBuilder::new(self.pool.clone());
        let agent_context = builder.build(story_id, Some(1), None, None)?;

        let service = AgentService::new(self.app_handle.clone());
        let task = AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: AgentType::Writer,
            context: agent_context,
            input: format!(
                "请撰写《{}》的第一章开头（3000-5000字）。\n\n这是故事的开篇，需要：\n1. 迅速建立世界观和氛围\n2. 引入主角，展示其性格和目标\n3. 埋下至少一个伏笔\n4. 在第一幕结尾制造一个冲突或悬念\n\n世界观核心：{}\n核心角色：{}\n第一章场景：{}",
                concept.title,
                world.concept,
                characters.iter().map(|c| format!("{}({}): {}", c.name, c.role, c.personality)).collect::<Vec<_>>().join("; "),
                scenes.first().map(|s| format!("{} - {}", s.title, s.summary)).unwrap_or_default()
            ),
            parameters: HashMap::new(),
            tier: None,
        };

        let result = service.execute_task(task).await?;

        // 保存到第一个场景的 content 字段
        if let Some(first_scene) = scenes.first() {
            let scene_repo = SceneRepository::new(self.pool.clone());
            let updates = SceneUpdate {
                title: None,
                dramatic_goal: None,
                external_pressure: None,
                conflict_type: None,
                characters_present: None,
                character_conflicts: None,
                setting_location: None,
                setting_time: None,
                setting_atmosphere: None,
                content: Some(result.content.clone()),
                previous_scene_id: None,
                next_scene_id: None,
                confidence_score: None,
                execution_stage: None,
                outline_content: None,
                draft_content: None,
            };
            let _ = scene_repo.update(&first_scene.id, &updates);
        }

        Ok(result.content)
    }

    // ==================== 辅助方法 ====================

    fn emit_progress(&self, session_id: &str, step_name: &str, step_number: usize, total_steps: usize, message: &str) {
        let event = BootstrapProgressEvent {
            session_id: session_id.to_string(),
            step_name: step_name.to_string(),
            step_number,
            total_steps,
            message: message.to_string(),
        };
        let _ = self.app_handle.emit("novel-bootstrap-progress", event);
    }

    // ==================== 会话持久化 ====================

    fn create_session(&self, session_id: &str, total_steps: usize) -> Result<(), rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO novel_bootstrap_sessions (id, status, current_step, steps_completed, total_steps, created_at)
             VALUES (?1, 'in_progress', 'concept', 0, ?2, ?3)",
            rusqlite::params![session_id, total_steps as i32, chrono::Local::now().to_rfc3339()],
        )?;
        Ok(())
    }

    fn update_session(&self, session_id: &str, current_step: &str, steps_completed: usize, story_id: Option<&str>) -> Result<(), rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "UPDATE novel_bootstrap_sessions
             SET current_step = ?2, steps_completed = ?3, story_id = ?4
             WHERE id = ?1",
            rusqlite::params![session_id, current_step, steps_completed as i32, story_id],
        )?;
        Ok(())
    }

    fn complete_session(&self, session_id: &str, story_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "UPDATE novel_bootstrap_sessions
             SET status = 'completed', current_step = 'completed', steps_completed = total_steps,
                 story_id = ?2, completed_at = ?3
             WHERE id = ?1",
            rusqlite::params![session_id, story_id, chrono::Local::now().to_rfc3339()],
        )?;
        Ok(())
    }

    fn fail_session(&self, session_id: &str, error_message: &str) -> Result<(), rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "UPDATE novel_bootstrap_sessions
             SET status = 'failed', error_message = ?2
             WHERE id = ?1",
            rusqlite::params![session_id, error_message],
        )?;
        Ok(())
    }

    fn extract_json(content: &str) -> Result<&str, String> {
        if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
            Ok(&content[start..=end])
        } else {
            Err("No JSON object found in response".to_string())
        }
    }
}

// ==================== 数据结构 ====================

#[derive(Debug, Clone, Deserialize)]
struct StoryConcept {
    title: String,
    description: String,
    genre: String,
    tone: String,
    pacing: String,
    #[serde(default)]
    themes: Vec<String>,
    #[serde(default)]
    target_length: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WorldBuildingData {
    concept: String,
    rules: Vec<WorldRuleData>,
    history: String,
    #[serde(default)]
    key_locations: Vec<String>,
    #[serde(default)]
    power_system: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WorldRuleData {
    name: String,
    description: String,
    rule_type: String,
    importance: i32,
}

#[derive(Debug, Clone)]
struct WorldBuildingResult {
    concept: String,
    rules: Vec<WorldRuleData>,
}

#[derive(Debug, Clone, Deserialize)]
struct CharacterData {
    characters: Vec<CharacterDetail>,
}

#[derive(Debug, Clone, Deserialize)]
struct CharacterDetail {
    name: String,
    role: String,
    personality: String,
    background: String,
    goals: String,
    fears: String,
    appearance: String,
    gender: String,
    age: i32,
    #[serde(default)]
    relationships: Vec<RelationshipData>,
}

#[derive(Debug, Clone, Deserialize)]
struct RelationshipData {
    target: String,
    nature: String,
}

#[derive(Debug, Clone)]
struct GeneratedCharacter {
    id: String,
    name: String,
    role: String,
    personality: String,
    background: String,
    goals: String,
    fears: String,
    appearance: String,
    gender: String,
    age: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct SceneData {
    scenes: Vec<SceneDetail>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SceneDetail {
    sequence_number: i32,
    title: String,
    dramatic_goal: String,
    external_pressure: String,
    conflict_type: String,
    setting_location: String,
    setting_time: String,
    characters_present: Vec<String>,
    summary: String,
}

#[derive(Debug, Clone)]
struct GeneratedScene {
    id: String,
    sequence_number: i32,
    title: String,
    summary: String,
}
