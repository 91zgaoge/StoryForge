#![allow(dead_code)]
//! 指导书提炼器：LLM 状态机
//!
//! 流程：元信息（→10%）→ 分块提炼（10→70%，并发）→ 合并去重（→85%）→
//! 结构化方法论（→100%）。方法论 JSON 解析失败重试一次，仍失败则整体 Failed。

use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;

use super::models::*;
use crate::{
    book_deconstruction::{
        analyzer::parse_json_response,
        chunker::extract_sample,
        models::{AnalysisError, TextChunk},
    },
    db::DbPool,
    llm::LlmService,
    router::{Complexity, Priority, RoutingRequest, TaskType},
};

pub struct GuidebookDistiller {
    llm_service: LlmService,
    app_handle: AppHandle,
    pool: DbPool,
    semaphore: Arc<Semaphore>,
    active_requests: Arc<AtomicI32>,
}

impl GuidebookDistiller {
    pub fn new(
        llm_service: LlmService,
        app_handle: AppHandle,
        pool: DbPool,
        concurrency: usize,
    ) -> Self {
        Self {
            llm_service,
            app_handle,
            pool,
            semaphore: Arc::new(Semaphore::new(concurrency.max(1).min(100))),
            active_requests: Arc::new(AtomicI32::new(0)),
        }
    }

    pub async fn distill(
        &self,
        guidebook_id: &str,
        chunks: &[TextChunk],
        heartbeat_callback: Option<Box<dyn Fn() + Send + Sync>>,
        cancel_check: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<DistillationOutput, AnalysisError> {
        let check_cancel = || -> Result<(), AnalysisError> {
            if let Some(ref cb) = cancel_check {
                if cb() {
                    return Err(AnalysisError::Cancelled("用户取消提炼".to_string()));
                }
            }
            Ok(())
        };
        let heartbeat = || {
            if let Some(ref cb) = heartbeat_callback {
                cb();
            }
        };

        // Step 1: 元信息（→10%）
        self.emit_progress(guidebook_id, "extracting", 5, "正在识别指导书元信息...")
            .await;
        heartbeat();
        check_cancel()?;
        let sample = extract_sample(
            &chunks
                .first()
                .map(|c| c.content.clone())
                .unwrap_or_default(),
            3000,
        );
        let metadata = self.extract_metadata(&sample).await?;
        let book_title = metadata
            .title
            .clone()
            .unwrap_or_else(|| "未命名".to_string());
        self.emit_progress(
            guidebook_id,
            "distilling",
            10,
            &format!("识别完成：《{}》", book_title),
        )
        .await;
        heartbeat();
        check_cancel()?;

        // Step 2: 分块提炼（10→70%，并发）
        let total = chunks.len();
        self.emit_progress(
            guidebook_id,
            "distilling",
            12,
            &format!("正在分块提炼创作要点（共 {} 块）...", total),
        )
        .await;
        let points = self
            .distill_chunks(guidebook_id, chunks, &cancel_check)
            .await?;
        heartbeat();
        check_cancel()?;

        // Step 3: 合并去重（→85%）
        self.emit_progress(guidebook_id, "merging", 72, "正在合并去重创作要点...")
            .await;
        let principles = self.merge_points(&points).await?;
        heartbeat();
        check_cancel()?;

        // Step 4: 结构化方法论（→100%），JSON 失败重试一次
        self.emit_progress(guidebook_id, "merging", 88, "正在生成创作方法论...")
            .await;
        let methodology = match self.generate_methodology(&principles, &book_title).await {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "[GuidebookDistiller] methodology 首次生成失败，重试一次: {}",
                    e
                );
                self.generate_methodology(&principles, &book_title).await?
            }
        };
        self.emit_progress(guidebook_id, "merging", 100, "提炼完成")
            .await;
        heartbeat();

        Ok(DistillationOutput {
            metadata,
            methodology,
        })
    }

    // ==================== 各步骤实现 ====================

    fn render_prompt(&self, id: &str, vars: &[(&str, String)]) -> Option<String> {
        let tpl = crate::prompts::registry::resolve_prompt(&self.pool, id)
            .ok()
            .or_else(|| crate::prompts::registry::resolve_prompt_default(id))?;
        let mut map = std::collections::HashMap::new();
        for (k, v) in vars {
            map.insert(k.to_string(), v.clone());
        }
        Some(crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &map))
    }

    async fn extract_metadata(
        &self,
        sample_text: &str,
    ) -> Result<LlmGuidebookMetadataResponse, AnalysisError> {
        let prompt = self
            .render_prompt("distill_metadata", &[("text", sample_text.to_string())])
            .ok_or_else(|| AnalysisError::LlmError("prompt distill_metadata 未注册".into()))?;
        let resp = call_llm(
            &self.llm_service,
            "guidebook_metadata",
            prompt,
            Some(500),
            Some(0.3),
        )
        .await?;
        parse_json_response(&resp)
    }

    async fn distill_chunks(
        &self,
        guidebook_id: &str,
        chunks: &[TextChunk],
        cancel_check: &Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<Vec<String>, AnalysisError> {
        let total = chunks.len();
        let processed = Arc::new(AtomicI32::new(0));
        let mut set = tokio::task::JoinSet::new();

        for chunk in chunks {
            if let Some(cb) = cancel_check {
                if cb() {
                    return Err(AnalysisError::Cancelled("用户取消提炼".to_string()));
                }
            }
            let sem = self.semaphore.clone();
            let llm = self.llm_service.clone();
            let pool = self.pool.clone();
            let active = self.active_requests.clone();
            let processed = processed.clone();
            let app = self.app_handle.clone();
            let gid = guidebook_id.to_string();
            let content = chunk.content.clone();
            let total = total;

            set.spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| AnalysisError::LlmError(format!("Semaphore error: {}", e)))?;
                active.fetch_add(1, Ordering::Relaxed);
                let result = async {
                    let tpl = crate::prompts::registry::resolve_prompt(&pool, "distill_chunk")
                        .ok()
                        .or_else(|| {
                            crate::prompts::registry::resolve_prompt_default("distill_chunk")
                        })
                        .ok_or_else(|| {
                            AnalysisError::LlmError("prompt distill_chunk 未注册".into())
                        })?;
                    let mut vars = std::collections::HashMap::new();
                    vars.insert("text".to_string(), content);
                    let prompt =
                        crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &vars);
                    let resp =
                        call_llm(&llm, "guidebook_chunk", prompt, Some(2000), Some(0.3)).await?;
                    let parsed: LlmDistillChunkResponse = parse_json_response(&resp)?;
                    Ok::<Vec<String>, AnalysisError>(parsed.points)
                }
                .await;
                active.fetch_sub(1, Ordering::Relaxed);
                let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
                // 10→70 按分块进度线性推进
                let progress = 10 + (60 * done / total.max(1) as i32);
                let _ = app.emit(
                    "guidebook-distillation-progress",
                    DistillationProgressEvent {
                        guidebook_id: gid,
                        status: "distilling".to_string(),
                        progress,
                        current_step: format!("分块提炼中 {}/{}", done, total),
                        message: None,
                        active_threads: active.load(Ordering::Relaxed),
                    },
                );
                result
            });
        }

        let mut all_points = Vec::new();
        while let Some(res) = set.join_next().await {
            match res {
                Ok(Ok(points)) => all_points.extend(points),
                Ok(Err(e)) => {
                    // 单块失败不致命：记录并继续（剩余块仍可提供要点）
                    log::warn!("[GuidebookDistiller] 单块提炼失败，跳过: {}", e);
                }
                Err(e) => {
                    return Err(AnalysisError::LlmError(format!("Join error: {}", e)));
                }
            }
        }
        Ok(all_points)
    }

    async fn merge_points(&self, points: &[String]) -> Result<Vec<String>, AnalysisError> {
        if points.is_empty() {
            return Err(AnalysisError::LlmError(
                "全书未提炼出任何创作要点".to_string(),
            ));
        }
        // 截断防爆 token：每条 200 字、总量 12000 字
        let joined = points
            .iter()
            .map(|p| {
                let s = p.trim();
                if s.chars().count() > 200 {
                    s.chars().take(200).collect::<String>()
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let joined = if joined.chars().count() > 12000 {
            joined.chars().take(12000).collect::<String>()
        } else {
            joined
        };
        let prompt = self
            .render_prompt("distill_merge", &[("points", joined)])
            .ok_or_else(|| AnalysisError::LlmError("prompt distill_merge 未注册".into()))?;
        let resp = call_llm(
            &self.llm_service,
            "guidebook_merge",
            prompt,
            Some(2000),
            Some(0.3),
        )
        .await?;
        let parsed: LlmDistillMergeResponse = parse_json_response(&resp)?;
        if parsed.principles.is_empty() {
            return Err(AnalysisError::LlmError("合并后原则为空".to_string()));
        }
        Ok(parsed.principles)
    }

    async fn generate_methodology(
        &self,
        principles: &[String],
        book_title: &str,
    ) -> Result<LlmMethodologyResponse, AnalysisError> {
        let prompt = self
            .render_prompt(
                "distill_methodology",
                &[
                    ("principles", principles.join("\n")),
                    ("book_title", book_title.to_string()),
                ],
            )
            .ok_or_else(|| AnalysisError::LlmError("prompt distill_methodology 未注册".into()))?;
        let resp = call_llm(
            &self.llm_service,
            "guidebook_methodology",
            prompt,
            Some(4000),
            Some(0.5),
        )
        .await?;
        let parsed: LlmMethodologyResponse = parse_json_response(&resp)?;
        validate_methodology(parsed)
    }

    async fn emit_progress(&self, guidebook_id: &str, status: &str, progress: i32, message: &str) {
        let _ = self.app_handle.emit(
            "guidebook-distillation-progress",
            DistillationProgressEvent {
                guidebook_id: guidebook_id.to_string(),
                status: status.to_string(),
                progress,
                current_step: message.to_string(),
                message: Some(message.to_string()),
                active_threads: self.active_requests.load(Ordering::Relaxed),
            },
        );
    }
}

/// 校验提炼产物：名称非空、至少一个步骤、每个步骤有 instruction
fn validate_methodology(
    m: LlmMethodologyResponse,
) -> Result<LlmMethodologyResponse, AnalysisError> {
    if m.name.trim().is_empty() {
        return Err(AnalysisError::LlmError("方法论名称为空".to_string()));
    }
    if m.steps.is_empty() {
        return Err(AnalysisError::LlmError("方法论步骤为空".to_string()));
    }
    if m.steps.iter().any(|s| s.instruction.trim().is_empty()) {
        return Err(AnalysisError::LlmError(
            "存在空 instruction 的步骤".to_string(),
        ));
    }
    Ok(m)
}

/// LLM 调用（与 book_deconstruction/analyzer.rs call_llm 相同的路由方式，
/// RoutingRequest/Complexity/Priority/TaskType 来自 crate::router）
async fn call_llm(
    llm_service: &LlmService,
    context_label: &str,
    prompt: String,
    max_tokens: Option<i32>,
    temperature: Option<f32>,
) -> Result<String, AnalysisError> {
    let request = RoutingRequest {
        task: TaskType::Analysis,
        complexity: Complexity::Medium,
        budget_priority: Priority::Low,
        speed_priority: Priority::Low,
        estimated_input_tokens: 0,
        constraints: vec![],
    };
    llm_service
        .generate_for_request(
            request,
            prompt,
            max_tokens,
            temperature,
            Some(context_label),
        )
        .await
        .map(|r| r.content)
        .map_err(|e| AnalysisError::LlmError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_methodology_accepts_valid() {
        let m = LlmMethodologyResponse {
            name: "三幕冲突法".into(),
            description: Some("d".into()),
            steps: vec![LlmMethodologyStepResponse {
                title: "建冲突".into(),
                instruction: "先确立核心冲突".into(),
                checklist: vec!["冲突是否明确？".into()],
            }],
        };
        assert!(validate_methodology(m).is_ok());
    }

    #[test]
    fn validate_methodology_rejects_empty() {
        let no_name = LlmMethodologyResponse {
            name: "  ".into(),
            description: None,
            steps: vec![LlmMethodologyStepResponse {
                title: "t".into(),
                instruction: "i".into(),
                checklist: vec![],
            }],
        };
        assert!(validate_methodology(no_name).is_err());

        let no_steps = LlmMethodologyResponse {
            name: "n".into(),
            description: None,
            steps: vec![],
        };
        assert!(validate_methodology(no_steps).is_err());

        let empty_instruction = LlmMethodologyResponse {
            name: "n".into(),
            description: None,
            steps: vec![LlmMethodologyStepResponse {
                title: "t".into(),
                instruction: " ".into(),
                checklist: vec![],
            }],
        };
        assert!(validate_methodology(empty_instruction).is_err());
    }
}
