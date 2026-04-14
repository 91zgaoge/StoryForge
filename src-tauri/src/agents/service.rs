//! Agent Service - 智能代理服务
//!
//! 协调多个Agent完成复杂的创作任务
//! 支持任务分解、执行、结果整合

use super::{Agent, AgentContext, AgentResult};
use crate::config::settings::{AppConfig, LlmProvider, AgentMapping};
use crate::llm::service::LlmService;
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

    /// 为Agent生成内容，优先使用映射的模型
    async fn generate_for_agent(
        &self,
        agent_type: AgentType,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<crate::llm::GenerateResponse, String> {
        if let Some(model_id) = self.get_agent_chat_model_id(agent_type) {
            self.llm_service.generate_with_profile(&model_id, prompt, max_tokens, temperature).await
        } else {
            self.llm_service.generate(prompt, max_tokens, temperature).await
        }
    }

    /// 执行写作助手
    async fn execute_writer(&self, task: AgentTask) -> Result<AgentResult, String> {
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析写作上下文", 0.1);
        
        // 构建写作提示词
        let prompt = self.build_writer_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "生成内容", 0.3);
        
        // 调用LLM生成（根据Agent映射选择模型）
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2000),
            Some(0.8),
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
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析内容质量", 0.1);
        
        let prompt = self.build_inspector_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "生成质检报告", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(1500),
            Some(0.3), // 低temperature以获得更确定的分析
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
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析故事需求", 0.1);
        
        let prompt = self.build_outline_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "设计故事大纲", 0.3);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(3000),
            Some(0.9),
        ).await?;
        
        Ok(AgentResult {
            content: response.content,
            score: Some(0.95),
            suggestions: vec![],
        })
    }

    /// 执行风格模仿师
    async fn execute_style_mimic(&self, task: AgentTask) -> Result<AgentResult, String> {
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析文风特征", 0.1);
        
        let prompt = self.build_style_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "模仿指定文风", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2000),
            Some(0.85),
        ).await?;
        
        Ok(AgentResult {
            content: response.content,
            score: Some(0.9),
            suggestions: vec![],
        })
    }

    /// 执行情节分析师
    async fn execute_plot_analyzer(&self, task: AgentTask) -> Result<AgentResult, String> {
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析情节结构", 0.1);
        
        let prompt = self.build_plot_prompt(&task);
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "生成分析报告", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2000),
            Some(0.4),
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
        ).await?;
        
        Ok(AgentResult::simple(response.content))
    }

    /// 执行记忆压缩师
    async fn execute_memory_compressor(&self, task: AgentTask) -> Result<AgentResult, String> {
        self.emit_event(&task.id, task.agent_type, AgentStage::Thinking, "分析待压缩内容", 0.1);
        
        let ctx = &task.context;
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
3. 输出长度控制在原文的 20%-30%
4. 使用第三人称客观叙述

请直接输出压缩后的摘要，不要添加解释。"#,
            ctx.story_title,
            ctx.genre,
            ctx.tone,
            ctx.pacing,
            task.input
        );
        
        self.emit_event(&task.id, task.agent_type, AgentStage::Generating, "压缩内容", 0.4);
        
        let response = self.generate_for_agent(
            task.agent_type,
            prompt,
            Some(2048),
            Some(0.3),
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

    // ==================== 提示词构建 ====================

    fn build_writer_prompt(&self, task: &AgentTask) -> String {
        let ctx = &task.context;
        format!(r#"【故事信息】
标题: {}
类型: {}
风格: {} / 节奏: {}

【前文内容】
{}

【写作要求】
{}

【角色信息】
{}

请根据以上上下文，续写接下来的内容。要求：
1. 保持文风一致
2. 情节连贯自然
3. 人物行为符合性格设定
4. 适当加入环境描写和对话

直接输出续写内容，不要添加解释。"#,
            ctx.story_title,
            ctx.genre,
            ctx.tone,
            ctx.pacing,
            ctx.previous_chapters.last().map(|c| &c.summary).unwrap_or(&"无".to_string()),
            task.input,
            ctx.characters.iter().map(|c| format!("- {}: {}", c.name, c.personality)).collect::<Vec<_>>().join("\n")
        )
    }

    fn build_inspector_prompt(&self, task: &AgentTask) -> String {
        format!(r#"【待检查内容】
{}

【检查维度】
1. 逻辑连贯性 - 情节是否通顺，有无矛盾
2. 人物一致性 - 角色行为是否符合设定
3. 文笔质量 - 语言是否流畅，描写是否生动
4. 节奏把控 - 快慢是否得当，有无冗余

请提供：
1. 总体评分（0-100）
2. 各维度评分
3. 具体问题指出
4. 改进建议"#,
            task.input
        )
    }

    fn build_outline_prompt(&self, task: &AgentTask) -> String {
        format!(r#"【故事创意】
{}

【要求】
设计一个完整的故事大纲，包括：
1. 故事主线（起承转合）
2. 主要章节划分（建议10-20章）
3. 每章核心情节
4. 关键情节点
5. 角色成长弧线

请用清晰的层次结构输出。"#,
            task.input
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
