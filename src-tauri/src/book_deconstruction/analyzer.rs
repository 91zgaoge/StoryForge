//! Book Analyzer - LLM 分析编排器
//!
//! 5步分析 Pipeline：元信息 → 世界观 → 人物 → 章节概要 → 故事线

use super::models::*;
use super::chunker::extract_sample;
use chrono::Local;
use crate::db::DbPool;
use crate::llm::LlmService;
use serde_json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;

/// LLM 并发限制
const MAX_CONCURRENT_LLM_CALLS: usize = 3;

pub struct BookAnalyzer {
    llm_service: LlmService,
    app_handle: AppHandle,
    pool: DbPool,
    semaphore: Arc<Semaphore>,
}

impl BookAnalyzer {
    pub fn new(llm_service: LlmService, app_handle: AppHandle, pool: DbPool) -> Self {
        Self {
            llm_service,
            app_handle,
            pool,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_LLM_CALLS)),
        }
    }

    /// 执行完整分析 Pipeline
    /// 
    /// `heartbeat_callback`: 可选的心跳回调，每完成一个主要步骤调用一次
    pub async fn analyze(
        &self,
        book_id: &str,
        chunks: &[TextChunk],
        total_word_count: usize,
        heartbeat_callback: Option<Box<dyn Fn() + Send + Sync>>,
    ) -> Result<BookAnalysisResult, AnalysisError> {
        log::info!("[BookAnalyzer] Starting analysis for book {}", book_id);

        // Step 1: 元信息识别 (10%)
        self.emit_progress(book_id, "extracting", 5, "正在提取文本信息...").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }
        let sample_text = extract_sample(&chunks.first().map(|c| c.content.clone()).unwrap_or_default(), 3000);
        let metadata = self.extract_metadata(&sample_text).await?;
        self.emit_progress(book_id, "analyzing", 10, "正在识别小说类型...").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }

        // Step 2: 世界观提取 (25%)
        self.emit_progress(book_id, "analyzing", 15, "正在分析世界观设定...").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }
        let world_setting = self.extract_world_setting(chunks, total_word_count).await?;
        self.emit_progress(book_id, "analyzing", 25, "世界观分析完成").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }

        // Step 3: 人物拆解 (50%)
        self.emit_progress(book_id, "analyzing", 30, "正在拆解人物角色...").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }
        let characters = self.extract_characters(book_id, chunks).await?;
        self.emit_progress(book_id, "analyzing", 50, "人物拆解完成").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }

        // Step 4: 章节概要 (75%)
        self.emit_progress(book_id, "analyzing", 55, "正在生成章节概要...").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }
        let scenes = self.extract_scene_summaries(book_id, chunks).await?;
        self.emit_progress(book_id, "analyzing", 75, "章节概要完成").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }

        // Step 5: 故事线生成 (90%)
        self.emit_progress(book_id, "analyzing", 80, "正在生成故事线...").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }
        let story_arc = self.extract_story_arc(&scenes).await?;
        self.emit_progress(book_id, "analyzing", 90, "故事线生成完成").await;
        if let Some(ref cb) = heartbeat_callback { cb(); }

        // 构建结果
        let book = self.build_book_result(
            book_id,
            metadata,
            world_setting,
            plot_summary_from_scenes(&scenes),
            story_arc,
        )?;

        self.emit_progress(book_id, "analyzing", 100, "分析完成").await;

        Ok(BookAnalysisResult {
            book,
            characters,
            scenes,
        })
    }

    // ==================== Step 1: 元信息识别 ====================

    async fn extract_metadata(&self, sample_text: &str) -> Result<ExtractedMetadata, AnalysisError> {
        let prompt = format!(
            r#"请分析以下小说开头，提取基本信息。只输出 JSON，不要有任何其他文字。

要求：
1. title: 小说标题（如无法确定则为null）
2. author: 作者名（如无法确定则为null）
3. genre: 主要类型（如：玄幻、都市、穿越、志怪、言情、科幻、武侠、历史等）
4. genre_tags: 类型标签数组（如：["系统流","无敌流","爽文"]）
5. estimated_word_count: 估计总字数（仅数字）

文本样本：
{}

JSON格式：
{{"title":"...","author":"...","genre":"...","genre_tags":["..."],"estimated_word_count":12345}}"#,
            sample_text
        );

        let response = self.call_llm(prompt, Some(500), Some(0.3)).await?;
        let metadata: LlmMetadataResponse = parse_json_response(&response)?;

        Ok(ExtractedMetadata {
            title: metadata.title,
            author: metadata.author,
            genre: metadata.genre,
            genre_tags: metadata.genre_tags.unwrap_or_default(),
            estimated_word_count: metadata.estimated_word_count,
        })
    }

    // ==================== Step 2: 世界观提取 ====================

    async fn extract_world_setting(
        &self,
        chunks: &[TextChunk],
        total_word_count: usize,
    ) -> Result<String, AnalysisError> {
        let sample = if total_word_count <= 100_000 {
            // 短篇：取全文的前30%作为样本
            let combined: String = chunks.iter().map(|c| c.content.clone()).collect::<Vec<_>>().join("\n\n");
            extract_sample(&combined, 15000)
        } else {
            // 中长篇：均匀采样
            let sample_size = chunks.len().min(10);
            let step = chunks.len() / sample_size.max(1);
            let mut samples = Vec::new();
            for i in 0..sample_size {
                let idx = i * step;
                if idx < chunks.len() {
                    samples.push(extract_sample(&chunks[idx].content, 1500));
                }
            }
            samples.join("\n\n---\n\n")
        };

        let prompt = format!(
            r#"请基于以下小说文本片段，提取世界观设定。用中文简洁描述，不超过500字。

需要包含：
1. 世界背景（古代/现代/未来/异世界等）
2. 力量体系（如有：修炼体系、魔法系统、科技水平等）
3. 社会结构（国家、门派、势力分布等）
4. 地理环境（主要场景设定）

文本片段：
{}

请直接输出世界观描述文本，不要输出 JSON。"#,
            sample
        );

        let response = self.call_llm(prompt, Some(800), Some(0.5)).await?;
        Ok(response.trim().to_string())
    }

    // ==================== Step 3: 人物拆解 ====================

    async fn extract_characters(
        &self,
        book_id: &str,
        chunks: &[TextChunk],
    ) -> Result<Vec<ReferenceCharacter>, AnalysisError> {
        // 对每个 chunk 并行提取人物
        let mut character_results: Vec<Vec<ExtractedCharacter>> = Vec::new();
        let total = chunks.len();

        for (i, chunk) in chunks.iter().enumerate() {
            let prompt = format!(
                r#"请分析以下小说章节，提取所有出现的人物角色。只输出 JSON，不要有任何其他文字。

要求：
1. name: 人物姓名
2. role_type: 角色定位（主角/反派/配角/龙套）
3. personality: 性格特征（简要描述）
4. appearance: 外貌描写（如有）
5. relationships: 与其他人物的关系数组 [{{"target":"姓名","type":"关系类型","description":"描述"}}]

注意：只提取本章出现的人物。如某人仅被提及但未出场，标记为"提及"。

章节内容：
{}

JSON格式：
{{"characters":[{{"name":"...","role_type":"...","personality":"...","appearance":"...","relationships":[]}}]}}"#,
                extract_sample(&chunk.content, 4000)
            );

            let response = self.call_llm_with_semaphore(prompt, Some(1000), Some(0.3)).await?;
            let parsed: Result<LlmCharacterResponse, _> = parse_json_response(&response);
            
            match parsed {
                Ok(result) => {
                    let extracted: Vec<ExtractedCharacter> = result.characters.into_iter().map(|c| {
                        ExtractedCharacter {
                            name: c.name,
                            role_type: c.role_type,
                            personality: c.personality,
                            appearance: c.appearance,
                            relationships: c.relationships.unwrap_or_default().into_iter().map(|r| {
                                CharacterRelationship {
                                    target_name: r.target,
                                    relation_type: r.relation_type,
                                    description: r.description,
                                }
                            }).collect(),
                        }
                    }).collect();
                    character_results.push(extracted);
                }
                Err(e) => {
                    log::warn!("[BookAnalyzer] Failed to parse characters for chunk {}: {}", i, e);
                    character_results.push(Vec::new());
                }
            }

            // 发送进度
            let progress = 30 + ((i + 1) * 20 / total.max(1)) as i32;
            self.emit_progress(book_id, "analyzing", progress, &format!("正在拆解人物 ({}/{})", i + 1, total)).await;
        }

        // 合并去重（按姓名）
        let merged = merge_characters(character_results);
        
        // 转换为 ReferenceCharacter
        let now = Local::now();
        let characters: Vec<ReferenceCharacter> = merged.into_iter().enumerate().map(|(i, c)| {
            let importance = calculate_importance(&c);
            ReferenceCharacter {
                id: format!("{}_char_{}", book_id, i),
                book_id: book_id.to_string(),
                name: c.name,
                role_type: c.role_type,
                personality: c.personality,
                appearance: c.appearance,
                relationships: Some(serde_json::to_string(&c.relationships).unwrap_or_default()),
                key_scenes: None,
                importance_score: Some(importance),
                created_at: now,
            }
        }).collect();

        Ok(characters)
    }

    // ==================== Step 4: 章节概要 ====================

    async fn extract_scene_summaries(
        &self,
        book_id: &str,
        chunks: &[TextChunk],
    ) -> Result<Vec<ReferenceScene>, AnalysisError> {
        let mut scenes = Vec::new();
        let total = chunks.len();
        let now = Local::now();

        for (i, chunk) in chunks.iter().enumerate() {
            let prompt = format!(
                r#"请总结以下小说章节的内容。只输出 JSON，不要有任何其他文字。

要求：
1. summary: 章节内容概要（100-200字）
2. characters_present: 出场人物名称数组
3. key_events: 关键事件数组
4. conflict_type: 冲突类型（如有：人与人/人与自我/人与社会/人与自然/人与命运）
5. emotional_tone: 情感基调（如：紧张/温馨/悲伤/激昂/悬疑）

章节内容：
{}

JSON格式：
{{"summary":"...","characters_present":["..."],"key_events":["..."],"conflict_type":"...","emotional_tone":"..."}}"#,
                extract_sample(&chunk.content, 5000)
            );

            let response = self.call_llm_with_semaphore(prompt, Some(800), Some(0.3)).await?;
            let parsed: Result<LlmSceneSummaryResponse, _> = parse_json_response(&response);

            let scene = match parsed {
                Ok(result) => ReferenceScene {
                    id: format!("{}_scene_{}", book_id, i),
                    book_id: book_id.to_string(),
                    sequence_number: (i + 1) as i32,
                    title: chunk.title.clone(),
                    summary: Some(result.summary),
                    characters_present: Some(serde_json::to_string(&result.characters_present).unwrap_or_default()),
                    key_events: Some(serde_json::to_string(&result.key_events).unwrap_or_default()),
                    conflict_type: result.conflict_type,
                    emotional_tone: result.emotional_tone,
                    created_at: now,
                },
                Err(e) => {
                    log::warn!("[BookAnalyzer] Failed to parse scene summary for chunk {}: {}", i, e);
                    ReferenceScene {
                        id: format!("{}_scene_{}", book_id, i),
                        book_id: book_id.to_string(),
                        sequence_number: (i + 1) as i32,
                        title: chunk.title.clone(),
                        summary: Some("（解析失败）".to_string()),
                        characters_present: None,
                        key_events: None,
                        conflict_type: None,
                        emotional_tone: None,
                        created_at: now,
                    }
                }
            };

            scenes.push(scene);

            // 发送进度
            let progress = 55 + ((i + 1) * 20 / total.max(1)) as i32;
            self.emit_progress(book_id, "analyzing", progress, &format!("正在生成章节概要 ({}/{})", i + 1, total)).await;
        }

        Ok(scenes)
    }

    // ==================== Step 5: 故事线生成 ====================

    async fn extract_story_arc(
        &self,
        scenes: &[ReferenceScene],
    ) -> Result<ExtractedStoryArc, AnalysisError> {
        // 构建章节概要文本
        let summaries: Vec<String> = scenes
            .iter()
            .map(|s| {
                format!(
                    "第{}章 {}: {}",
                    s.sequence_number,
                    s.title.as_deref().unwrap_or(""),
                    s.summary.as_deref().unwrap_or("无概要")
                )
            })
            .collect();

        let combined = summaries.join("\n");
        let sample = extract_sample(&combined, 8000);

        let prompt = format!(
            r#"请基于以下章节概要，生成完整的故事线。只输出 JSON，不要有任何其他文字。

要求：
1. main_arc: 主线故事（简要概括）
2. sub_arcs: 重要支线数组
3. climaxes: 高潮点数组
4. turning_points: 转折点数组

章节概要：
{}

JSON格式：
{{"main_arc":"...","sub_arcs":["..."],"climaxes":["..."],"turning_points":["..."]}}"#,
            sample
        );

        let response = self.call_llm(prompt, Some(1000), Some(0.5)).await?;
        let arc: LlmStoryArcResponse = parse_json_response(&response)?;

        Ok(ExtractedStoryArc {
            main_arc: arc.main_arc,
            sub_arcs: arc.sub_arcs,
            climaxes: arc.climaxes,
            turning_points: arc.turning_points,
        })
    }

    // ==================== 辅助方法 ====================

    async fn call_llm(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<String, AnalysisError> {
        match self.llm_service.generate(prompt, max_tokens, temperature).await {
            Ok(response) => Ok(response.content),
            Err(e) => Err(AnalysisError::LlmError(e)),
        }
    }

    async fn call_llm_with_semaphore(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<String, AnalysisError> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            AnalysisError::LlmError(format!("Semaphore error: {}", e))
        });
        self.call_llm(prompt, max_tokens, temperature).await
    }

    async fn emit_progress(&self, book_id: &str, status: &str, progress: i32, message: &str) {
        let event = BookAnalysisProgressEvent {
            book_id: book_id.to_string(),
            status: status.to_string(),
            progress,
            current_step: message.to_string(),
            message: Some(message.to_string()),
        };
        let _ = self.app_handle.emit("book-analysis-progress", event);
    }

    fn build_book_result(
        &self,
        book_id: &str,
        metadata: ExtractedMetadata,
        world_setting: String,
        plot_summary: String,
        story_arc: ExtractedStoryArc,
    ) -> Result<ReferenceBook, AnalysisError> {
        let story_arc_json = serde_json::to_string(&story_arc)
            .map_err(|e| AnalysisError::StorageError(format!("JSON serialize error: {}", e)))?;

        Ok(ReferenceBook {
            id: book_id.to_string(),
            title: metadata.title.unwrap_or_else(|| "未命名".to_string()),
            author: metadata.author,
            genre: metadata.genre,
            word_count: metadata.estimated_word_count,
            file_format: None,
            file_hash: None,
            file_path: None,
            world_setting: Some(world_setting),
            plot_summary: Some(plot_summary),
            story_arc: Some(story_arc_json),
            analysis_status: AnalysisStatus::Completed,
            analysis_progress: 100,
            analysis_error: None,
            created_at: Local::now(),
            updated_at: Local::now(),
        })
    }
}

// ==================== 工具函数 ====================

/// 解析 LLM 返回的 JSON（带容错）
fn parse_json_response<T: serde::de::DeserializeOwned>(response: &str) -> Result<T, AnalysisError> {
    let trimmed = response.trim();
    
    // 尝试直接解析
    if let Ok(result) = serde_json::from_str::<T>(trimmed) {
        return Ok(result);
    }
    
    // 尝试提取 markdown 代码块
    if let Some(start) = trimmed.find("```json") {
        let after_start = &trimmed[start + 7..];
        if let Some(end) = after_start.find("```") {
            let json_str = after_start[..end].trim();
            if let Ok(result) = serde_json::from_str::<T>(json_str) {
                return Ok(result);
            }
        }
    }
    
    // 尝试提取任意 ``` 代码块
    if let Some(start) = trimmed.find("```") {
        let after_start = &trimmed[start + 3..];
        if let Some(end) = after_start.find("```") {
            let json_str = after_start[..end].trim();
            if let Ok(result) = serde_json::from_str::<T>(json_str) {
                return Ok(result);
            }
        }
    }
    
    // 尝试提取第一个 { 到最后一个 }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            let json_str = &trimmed[start..=end];
            if let Ok(result) = serde_json::from_str::<T>(json_str) {
                return Ok(result);
            }
        }
    }
    
    Err(AnalysisError::ParseError(format!(
        "Failed to parse JSON from LLM response: {}",
        &trimmed[..trimmed.len().min(200)]
    )))
}

/// 合并去重人物
fn merge_characters(all_results: Vec<Vec<ExtractedCharacter>>) -> Vec<ExtractedCharacter> {
    use std::collections::HashMap;
    
    let mut merged: HashMap<String, ExtractedCharacter> = HashMap::new();
    
    for batch in all_results {
        for character in batch {
            let name = character.name.clone();
            if let Some(existing) = merged.get_mut(&name) {
                // 合并信息：优先保留非空字段
                if existing.role_type.is_none() && character.role_type.is_some() {
                    existing.role_type = character.role_type;
                }
                if existing.personality.is_none() && character.personality.is_some() {
                    existing.personality = character.personality;
                }
                if existing.appearance.is_none() && character.appearance.is_some() {
                    existing.appearance = character.appearance;
                }
                // 合并关系
                for rel in character.relationships {
                    if !existing.relationships.iter().any(|r| r.target_name == rel.target_name) {
                        existing.relationships.push(rel);
                    }
                }
            } else {
                merged.insert(name, character);
            }
        }
    }
    
    merged.into_values().collect()
}

/// 从场景生成剧情概要
fn plot_summary_from_scenes(scenes: &[ReferenceScene]) -> String {
    let summaries: Vec<String> = scenes
        .iter()
        .take(20)
        .map(|s| s.summary.clone().unwrap_or_default())
        .collect();
    summaries.join("；")
}

/// 计算人物重要度分数
fn calculate_importance(character: &ExtractedCharacter) -> f32 {
    let mut score = 0.0;
    
    // 角色定位权重
    if let Some(ref role) = character.role_type {
        score += match role.as_str() {
            "主角" | "主人公" => 1.0,
            "反派" | "反派角色" => 0.8,
            "配角" | "重要配角" => 0.6,
            "龙套" | "路人" => 0.2,
            _ => 0.4,
        };
    }
    
    // 性格描述越详细越重要
    if character.personality.is_some() {
        score += 0.1;
    }
    
    // 外貌描述
    if character.appearance.is_some() {
        score += 0.05;
    }
    
    // 关系越多越重要
    score += (character.relationships.len() as f32 * 0.05).min(0.3);
    
    score.min(1.0)
}
