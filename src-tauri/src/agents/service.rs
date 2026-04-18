//! Agent Service - 智能代理服务
//!
//! 协调多个Agent完成复杂的创作任务
//! 支持任务分解、执行、结果整合
#![allow(dead_code)]
#![allow(unused_imports)]

use super::{Agent, AgentContext, AgentResult};
use crate::config::settings::AppConfig;
use crate::llm::service::LlmService;
use crate::subscription::{SubscriptionService, SubscriptionTier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager};

/// Agent类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Writer,           // 写作助手
    Inspector,        // 质检员
    OutlinePlanner,   // 大纲规划师
    StyleMimic,       // 风格模仿师
    PlotAnalyzer,     // 情节分析师
    MemoryCompressor, // 记忆压缩师
    Commentator,      // 古典评点家
    KnowledgeDistiller, // 知识蒸馏师
}

impl AgentType {
    pub fn name(&self) -> &'static str {
        match self {
            AgentType::Writer => "写作助手",
            AgentType::Inspector => "质检员",
            AgentType::OutlinePlanner => "大纲规划师",
            AgentType::StyleMimic => "风格模仿师",
            AgentType::PlotAnalyzer => "情节分析师",
            AgentType::MemoryCompressor => "记忆压缩师",
            AgentType::Commentator => "古典评点家",
            AgentType::KnowledgeDistiller => "知识蒸馏师",
        }
    }

    pub fn agent_id(&self) -> &'static str {
        match self {
            AgentType::Writer => "writer",
            AgentType::Inspector => "inspector",
            AgentType::OutlinePlanner => "outline_planner",
            AgentType::StyleMimic => "style_mimic",
            AgentType::PlotAnalyzer => "plot_analyzer",
            AgentType::MemoryCompressor => "memory_compressor",
            AgentType::Commentator => "commentator",
            AgentType::KnowledgeDistiller => "knowledge_distiller",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AgentType::Writer => "根据上下文生成或改写章节内容",
            AgentType::Inspector => "检查内容质量、逻辑连贯性、人物一致性",
            AgentType::OutlinePlanner => "设计故事大纲、章节结构",
            AgentType::StyleMimic => "分析并模仿特定文风",
            AgentType::PlotAnalyzer => "分析情节复杂度、检测漏洞",
            AgentType::MemoryCompressor => "将详细内容压缩为高层记忆摘要",
            AgentType::Commentator => "以金圣叹风格对小说段落进行实时文学点评",
            AgentType::KnowledgeDistiller => "将知识图谱蒸馏为高层故事摘要与世界观总结",
        }
    }
}

/// Agent任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub agent_type: AgentType,
    pub context: AgentContext,
    pub input: String,
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<SubscriptionTier>,
}

/// Agent执行事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub task_id: String,
    pub agent_type: String,
    pub stage: AgentStage,
    pub message: String,
    pub progress: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStage {
    Started,
    Thinking,
    Generating,
    Reviewing,
    Completed,
    Failed,
}

/// Agent服务
pub struct AgentService {
    app_handle: AppHandle,
    llm_service: LlmService,
}

impl AgentService {
    pub fn new(app_handle: AppHandle) -> Self {
        let llm_service = LlmService::new(app_handle.clone());
        
        Self {
            app_handle,
            llm_service,
        }
    }

    /// 获取 AppHandle 引用（用于上下文构建等场景）
    pub fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }

    /// 执行Agent任务
    pub async fn execute_task(&self, task: AgentTask) -> Result<AgentResult, String> {
        let task_id = task.id.clone();
        let agent_type = task.agent_type;
        
        // 发送开始事件
        self.emit_event(&task_id, agent_type, AgentStage::Started, "开始执行任务", 0.0);
        
        let result = match agent_type {
            AgentType::Writer => self.execute_writer(task).await,
            AgentType::Inspector => self.execute_inspector(task).await,
            AgentType::OutlinePlanner => self.execute_outline_planner(task).await,
            AgentType::StyleMimic => self.execute_style_mimic(task).await,
            AgentType::PlotAnalyzer => self.execute_plot_analyzer(task).await,
            AgentType::MemoryCompressor => self.execute_memory_compressor(task).await,
            AgentType::Commentator => self.execute_commentator(task).await,
            AgentType::KnowledgeDistiller => self.execute_knowledge_distiller(task).await,
        };
        
        match &result {
            Ok(_) => {
                self.emit_event(&task_id, agent_type, AgentStage::Completed, "任务完成", 1.0);
            }
            Err(e) => {
                self.emit_event(&task_id, agent_type, AgentStage::Failed, &format!("执行失败: {}", e), 0.0);
            }
        }
        
        result
    }

    /// 获取Agent对应的聊天模型ID
    fn get_agent_chat_model_id(&self, agent_type: AgentType) -> Option<String> {
        let app_dir = self.app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        
        let config = AppConfig::load(&app_dir).ok()?;
        config.agent_mappings
            .get(agent_type.agent_id())
            .and_then(|m| m.chat_model_id.clone())
    }

    /// 获取当前用户的订阅层级（fallback 查询，优先使用 task.tier）
    fn get_user_tier(&self) -> SubscriptionTier {
        let app_dir = match self.app_handle.path().app_data_dir() {
            Ok(d) => d,
            Err(e) => {
                log::warn!("[AgentService] Failed to get app_data_dir: {}, defaulting to Free", e);
                return SubscriptionTier::Free;
            }
        };
        let machine_id_path = app_dir.join(".machine_id");
        let user_id = if machine_id_path.exists() {
            std::fs::read_to_string(&machine_id_path).unwrap_or_default().trim().to_string()
        } else {
            log::warn!("[AgentService] .machine_id not found, defaulting to Free");
            return SubscriptionTier::Free;
        };

        if user_id.is_empty() {
            log::warn!("[AgentService] user_id is empty, defaulting to Free");
            return SubscriptionTier::Free;
        }

        if let Some(pool) = self.app_handle.try_state::<crate::db::DbPool>() {
            let service = SubscriptionService::new(pool.inner().clone());
            match service.get_or_create_subscription(&user_id) {
                Ok(status) => match status.tier.parse() {
                    Ok(tier) => return tier,
                    Err(e) => log::warn!("[AgentService] Failed to parse tier '{}': {}, defaulting to Free", status.tier, e),
                },
                Err(e) => log::warn!("[AgentService] DB query failed: {}, defaulting to Free", e),
            }
        } else {
            log::warn!("[AgentService] DbPool not available, defaulting to Free");
        }
        SubscriptionTier::Free
    }

    /// 从 task 或 fallback 获取 tier
    fn resolve_tier(&self, task: &AgentTask) -> SubscriptionTier {
        task.tier.unwrap_or_else(|| self.get_user_tier())
    }

    /// 为Agent生成内容，优先使用映射的模型
    /// 免费版限制 max_tokens 以控制成本与质量
    async fn generate_for_agent(
        &self,
        agent_type: AgentType,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
        tier: SubscriptionTier,
    ) -> Result<crate::llm::GenerateResponse, String> {
        let effective_max = match tier {
            SubscriptionTier::Free => max_tokens.map(|m| m.min(1000)).or(Some(1000)),
            _ => max_tokens,
        };
        if let Some(model_id) = self.get_agent_chat_model_id(agent_type) {
            self.llm_service.generate_with_profile(&model_id, prompt, effective_max, temperature).await
        } else {
            self.llm_service.generate(prompt, effective_max, temperature).await
        }
    }

    /// 执行写作助手
    async fn execute_writer(&self, task: AgentTask) -> Result<AgentResult, String> {
        let tier = self.resolve_tier(&task);
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析写作上下文", 0.1);
        
        // 构建写作提示词（根据 tier 决定是否注入高级扩展）
        let prompt = self.build_writer_prompt(&task, tier);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "生成内容", 0.3);
        
        // 调用LLM生成（根据Agent映射选择模型，免费版限制 token）
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2000),
            Some(0.8),
            tier,
        ).await?;
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Reviewing, "检查生成质量", 0.8);
        
        // 简单的质量评分（后续可细化）
        let score = self.calculate_quality_score(&response.content);
        
        let suggestions = if score < 0.7 {
            vec!["建议：内容可能需要进一步润色".to_string()]
        } else {
            vec![]
        };
        
        Ok(AgentResult {
            content: response.content,
            score: Some(score),
            suggestions,
        })
    }

    /// 执行质检员
    async fn execute_inspector(&self, task: AgentTask) -> Result<AgentResult, String> {
        let tier = self.resolve_tier(&task);
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析内容质量", 0.1);
        
        let prompt = self.build_inspector_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "生成质检报告", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(1500),
            Some(0.3), // 低temperature以获得更确定的分析
            tier,
        ).await?;
        
        // 解析质检结果
        let (score, suggestions) = self.parse_inspection_result(&response.content);
        
        Ok(AgentResult {
            content: response.content,
            score: Some(score),
            suggestions,
        })
    }

    /// 执行大纲规划师
    async fn execute_outline_planner(&self, task: AgentTask) -> Result<AgentResult, String> {
        let tier = self.resolve_tier(&task);
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析故事需求", 0.1);
        
        let prompt = self.build_outline_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "设计故事大纲", 0.3);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(3000),
            Some(0.9),
            tier,
        ).await?;
        
        Ok(AgentResult {
            content: response.content,
            score: Some(0.95),
            suggestions: vec![],
        })
    }

    /// 执行风格模仿师
    async fn execute_style_mimic(&self, task: AgentTask) -> Result<AgentResult, String> {
        let tier = self.resolve_tier(&task);
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析文风特征", 0.1);
        
        let prompt = self.build_style_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "模仿指定文风", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2000),
            Some(0.85),
            tier,
        ).await?;
        
        Ok(AgentResult {
            content: response.content,
            score: Some(0.9),
            suggestions: vec![],
        })
    }

    /// 执行情节分析师
    async fn execute_plot_analyzer(&self, task: AgentTask) -> Result<AgentResult, String> {
        let tier = self.resolve_tier(&task);
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析情节结构", 0.1);
        
        let prompt = self.build_plot_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "生成分析报告", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2000),
            Some(0.4),
            tier,
        ).await?;
        
        let (score, suggestions) = self.parse_plot_analysis(&response.content);
        
        Ok(AgentResult {
            content: response.content,
            score: Some(score),
            suggestions,
        })
    }

    /// 执行古典评点家
    async fn execute_commentator(&self, task: AgentTask) -> Result<AgentResult, String> {
        let tier = self.resolve_tier(&task);
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "品读文本", 0.1);
        
        let ctx = &task.context;
        let prompt = format!(
            r#"你是一位中国古典小说评点家，风格类似金圣叹。请对以下小说段落进行简短点评。

【作品信息】
标题: {}
题材: {}

【待评段落】
{}

【点评要求】
1. 用古典文人评点的口吻，简洁有力，每段不超过60字
2. 可点评：文笔、结构、人物、伏笔、情感、节奏
3. 语气可带几分机锋，但不可刻薄伤人
4. 直接输出 JSON 数组，格式：[{{"paragraph_index": 0, "commentary": "...", "tone": "insightful"}}]
5. tone 可选：insightful / witty / emotional / critical
6. 如果没有值得点评之处， commentary 可为空字符串

请直接输出 JSON，不要添加 markdown 代码块标记。"#,
            ctx.story_title,
            ctx.genre,
            task.input
        );
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "生成评点", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2048),
            Some(0.85),
            tier,
        ).await?;
        
        Ok(AgentResult::simple(response.content))
    }

    /// 执行记忆压缩师
    async fn execute_memory_compressor(&self, task: AgentTask) -> Result<AgentResult, String> {
        let tier = self.resolve_tier(&task);
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析待压缩内容", 0.1);
        
        let ctx = &task.context;
        let target_ratio = task.parameters.get("target_ratio")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.25);
        let ratio_pct = (target_ratio * 100.0) as i32;
        
        let prompt = format!(
            r#"你是一位专业的文学记忆压缩师。请将以下小说相关内容压缩为简洁的高层摘要。

【作品信息】
标题: {}
题材: {}
文风: {}
节奏: {}

【待压缩内容】
{}

【压缩要求】
1. 保留核心情节、人物关系、关键伏笔
2. 删除细节描写、重复叙述、过渡段落
3. 输出长度控制在原文的 {}%
4. 使用第三人称客观叙述

请直接输出压缩后的摘要，不要添加解释。"#,
            ctx.story_title,
            ctx.genre,
            ctx.tone,
            ctx.pacing,
            ratio_pct,
            task.input
        );
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "压缩内容", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2048),
            Some(0.3),
            tier,
        ).await?;
        
        let original_len = task.input.chars().count();
        let compressed_len = response.content.chars().count();
        let compression_ratio = if original_len > 0 {
            compressed_len as f32 / original_len as f32
        } else {
            1.0
        };
        let score = (1.0 - compression_ratio).max(0.0).min(1.0);
        
        Ok(AgentResult {
            content: response.content,
            score: Some(score),
            suggestions: vec![format!("压缩率: {:.1}%", compression_ratio * 100.0)],
        })
    }

    /// 执行知识蒸馏师
    async fn execute_knowledge_distiller(&self, task: AgentTask) -> Result<AgentResult, String> {
        let tier = self.resolve_tier(&task);
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析知识图谱结构", 0.1);
        
        let ctx = &task.context;
        let prompt = format!(
            r#"你是一位专业的文学知识蒸馏师。请根据以下小说知识图谱，提炼出高层摘要。

【作品信息】
标题: {}
题材: {}
文风: {}
节奏: {}

【知识图谱】
{}

【蒸馏要求】
1. 世界观概述：提炼故事的宏观设定、核心规则、时代背景
2. 主要势力：总结故事中的重要组织、阵营、群体及其关系
3. 人物关系网：梳理核心角色之间的关系、立场、冲突
4. 核心情节线：提炼当前已展开的主要悬念、伏笔、目标
5. 输出条理清晰，使用Markdown格式，总长度控制在800字以内

请直接输出蒸馏后的摘要。"#,
            ctx.story_title,
            ctx.genre,
            ctx.tone,
            ctx.pacing,
            task.input
        );
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "蒸馏知识图谱", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2048),
            Some(0.4),
            tier,
        ).await?;
        
        Ok(AgentResult::with_score(response.content, 0.9))
    }

    // ==================== 提示词构建（模板化） ====================

    fn build_writer_prompt(&self, task: &AgentTask, tier: SubscriptionTier) -> String {
        use crate::prompts::{TemplateEngine, PromptLibrary};
        use std::collections::HashMap;

        let ctx = &task.context;
        let has_selection = ctx.selected_text.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
        let is_pro = tier != SubscriptionTier::Free;

        // 构建模板变量
        let mut vars = HashMap::new();
        vars.insert("story_title".to_string(), ctx.story_title.clone());
        vars.insert("genre".to_string(), ctx.genre.clone());
        vars.insert("tone".to_string(), ctx.tone.clone());
        vars.insert("pacing".to_string(), ctx.pacing.clone());
        vars.insert("characters".to_string(), ctx.format_characters());
        vars.insert("previous_chapters".to_string(), ctx.format_previous_chapters());
        vars.insert("current_content".to_string(), ctx.current_content.clone().unwrap_or_else(|| "无".to_string()));
        vars.insert("instruction".to_string(), task.input.clone());
        vars.insert("world_rules".to_string(), ctx.world_rules.clone().unwrap_or_default());
        vars.insert("scene_structure".to_string(), ctx.scene_structure.clone().unwrap_or_default());

        let mut system_prompt = TemplateEngine::render_with_conditions(
            PromptLibrary::writer_system_template(),
            &vars
        );

        // 注入创作方法论扩展（仅专业版）
        if is_pro {
            if let Some(ref method_id) = ctx.methodology_id {
                use crate::creative_engine::methodology::{MethodologyConfig, MethodologyType, MethodologyEngine};
                let method_type = match method_id.as_str() {
                    "snowflake" => Some(MethodologyType::Snowflake),
                    "scene_structure" => Some(MethodologyType::SceneStructure),
                    "hero_journey" => Some(MethodologyType::HeroJourney),
                    "character_depth" => Some(MethodologyType::CharacterDepth),
                    _ => None,
                };
                if let Some(mt) = method_type {
                    let config = MethodologyConfig {
                        methodology_type: mt,
                        is_active: true,
                        current_step: ctx.methodology_step.clone(),
                        custom_params: serde_json::json!({}),
                    };
                    let extension = MethodologyEngine::build_prompt_extension(&config);
                    if !extension.is_empty() {
                        system_prompt.push_str("\n\n【创作方法论约束】\n");
                        system_prompt.push_str(&extension);
                    }
                }
            }

            // 注入风格 DNA（仅专业版）
            if let Some(ref style_id) = ctx.style_dna_id {
                use crate::db::DbPool;
                use crate::db::repositories_v3::StyleDnaRepository;
                use crate::creative_engine::style::dna::StyleDNA;
                use tauri::Manager;

                let pool = self.app_handle.state::<DbPool>();
                let repo = StyleDnaRepository::new(pool.inner().clone());
                if let Ok(Some(db_dna)) = repo.get_by_id(style_id) {
                    if let Ok(dna) = serde_json::from_str::<StyleDNA>(&db_dna.dna_json) {
                        let extension = dna.to_prompt_extension();
                        if !extension.is_empty() {
                            system_prompt.push_str("\n\n");
                            system_prompt.push_str(&extension);
                        }
                    }
                }
            }

            // 注入个性化偏好（自适应学习，仅专业版）
            {
                use crate::db::DbPool;
                use crate::creative_engine::adaptive::PromptPersonalizer;
                use tauri::Manager;

                let pool = self.app_handle.state::<DbPool>();
                let personalizer = PromptPersonalizer::new(pool.inner().clone());
                if let Ok(extension) = personalizer.build_prompt_extension(&ctx.story_id) {
                    if !extension.is_empty() {
                        system_prompt.push_str("\n\n");
                        system_prompt.push_str(&extension);
                    }
                }
            }
        }

        let user_prompt = if has_selection {
            vars.insert("selected_text".to_string(), ctx.selected_text.clone().unwrap_or_default());
            TemplateEngine::render_with_conditions(
                PromptLibrary::writer_rewrite_template(),
                &vars
            )
        } else {
            TemplateEngine::render_with_conditions(
                PromptLibrary::writer_continue_template(),
                &vars
            )
        };

        format!("{}\n\n{}", system_prompt, user_prompt)
    }

    fn build_inspector_prompt(&self, task: &AgentTask) -> String {
        use crate::prompts::{TemplateEngine, PromptLibrary};
        use std::collections::HashMap;

        let ctx = &task.context;
        let mut vars = HashMap::new();
        vars.insert("story_title".to_string(), ctx.story_title.clone());
        vars.insert("genre".to_string(), ctx.genre.clone());
        vars.insert("characters".to_string(), ctx.format_characters());
        vars.insert("content".to_string(), task.input.clone());

        let system_prompt = TemplateEngine::render_with_conditions(
            PromptLibrary::inspector_system_template(),
            &vars
        );

        format!("{}\n\n【待检查内容】\n{}", system_prompt, task.input)
    }

    fn build_outline_prompt(&self, task: &AgentTask) -> String {
        use crate::prompts::{TemplateEngine, PromptLibrary};
        use std::collections::HashMap;

        let ctx = &task.context;
        let mut vars = HashMap::new();
        vars.insert("premise".to_string(), task.input.clone());
        vars.insert("characters".to_string(), ctx.format_characters());

        TemplateEngine::render_with_conditions(
            PromptLibrary::outline_planner_template(),
            &vars
        )
    }

    fn build_style_prompt(&self, task: &AgentTask) -> String {
        format!(r#"【参考文风样例】
{}

【需要改写的文本】
{}

请模仿参考文风的语言特点（词汇选择、句式结构、修辞手法等），改写上述文本，保持原意但改变表达方式。"#,
            task.parameters.get("style_sample").and_then(|v| v.as_str()).unwrap_or("无样例"),
            task.input
        )
    }

    fn build_plot_prompt(&self, task: &AgentTask) -> String {
        format!(r#"【故事内容】
{}

【分析要求】
1. 情节复杂度评估（简单/中等/复杂）
2. 主要情节线索梳理
3. 潜在的逻辑漏洞
4. 伏笔和回收情况
5. 高潮设置是否合理
6. 改进建议"#,
            task.input
        )
    }

    // ==================== 辅助方法 ====================

    fn emit_event(&self, task_id: &str, agent_type: AgentType, stage: AgentStage, message: &str, progress: f32) {
        let event = AgentEvent {
            task_id: task_id.to_string(),
            agent_type: agent_type.name().to_string(),
            stage,
            message: message.to_string(),
            progress,
        };
        
        let _ = self.app_handle.emit(&format!("agent-event-{}", task_id), event);
    }

    fn calculate_quality_score(&self, content: &str) -> f32 {
        // 简单的启发式评分
        let length_score = (content.len() as f32 / 500.0).min(1.0); // 长度
        let sentence_count = content.split(['。', '！', '？']).count() as f32;
        let variety_score = (sentence_count / 5.0).min(1.0); // 句子多样性
        
        (length_score * 0.4 + variety_score * 0.6).min(1.0)
    }

    fn parse_inspection_result(&self, content: &str) -> (f32, Vec<String>) {
        // 简单解析，后续可用正则或结构化输出
        let score = if content.contains("90") || content.contains("优秀") {
            0.9
        } else if content.contains("80") || content.contains("良好") {
            0.8
        } else if content.contains("70") {
            0.7
        } else {
            0.6
        };
        
        let suggestions: Vec<String> = content
            .lines()
            .filter(|l| l.contains("建议") || l.contains("改进"))
            .map(|l| l.to_string())
            .collect();
        
        (score, suggestions)
    }

    fn parse_plot_analysis(&self, content: &str) -> (f32, Vec<String>) {
        let score = if content.contains("复杂") || content.contains("优秀") {
            0.85
        } else if content.contains("中等") {
            0.7
        } else {
            0.6
        };
        
        let suggestions = content
            .lines()
            .filter(|l| l.contains("漏洞") || l.contains("建议"))
            .map(|l| l.to_string())
            .collect();
        
        (score, suggestions)
    }
}

impl Clone for AgentService {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            llm_service: LlmService::new(self.app_handle.clone()),
        }
    }
}

/// 获取所有可用的Agent类型
#[tauri::command]
pub fn get_available_agents() -> Vec<(AgentType, String, String)> {
    vec![
        (AgentType::Writer, AgentType::Writer.name().to_string(), AgentType::Writer.description().to_string()),
        (AgentType::Inspector, AgentType::Inspector.name().to_string(), AgentType::Inspector.description().to_string()),
        (AgentType::OutlinePlanner, AgentType::OutlinePlanner.name().to_string(), AgentType::OutlinePlanner.description().to_string()),
        (AgentType::StyleMimic, AgentType::StyleMimic.name().to_string(), AgentType::StyleMimic.description().to_string()),
        (AgentType::PlotAnalyzer, AgentType::PlotAnalyzer.name().to_string(), AgentType::PlotAnalyzer.description().to_string()),
    ]
}
