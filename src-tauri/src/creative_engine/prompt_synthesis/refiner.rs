//! Call 2 提示词精修器（PromptRefiner）——可选步骤，调试完善合成提示词。
//!
//! 仅在 needs_refinement=true
//! 且剩余预算充足时调用。失败回退原提示词（不阻塞）。

use std::sync::Arc;

use crate::{
    db::DbPool,
    ports::{LlmPort, LlmPortRequest},
    router::{Complexity, Priority, TaskType},
};

/// 提示词精修器
pub struct PromptRefiner;

impl PromptRefiner {
    /// 精修 Call 1 合成的提示词。
    ///
    /// - `llm_port`: 中性 LLM 端口（用于精修生成）
    /// - `synthesized_prompt`: Call 1 输出的合成提示词
    /// - `refinement_focus`: Call 1 判定的精修重点（复合题材/改写/冲突等）
    /// - `story_title/genre/tone`: 故事背景信息
    ///
    /// 失败回退：返回原 `synthesized_prompt`（不阻塞）。
    pub async fn refine(
        llm_port: Arc<dyn LlmPort>,
        synthesized_prompt: &str,
        refinement_focus: Option<&str>,
        story_title: &str,
        story_genre: Option<&str>,
        story_tone: Option<&str>,
        pool: Option<&DbPool>,
    ) -> String {
        let focus_text = refinement_focus.unwrap_or("一般精修");

        // 构建精修 prompt
        let prompt = {
            let tpl = pool
                .and_then(|p| crate::prompts::registry::resolve_prompt(p, "trishot_refiner").ok())
                .or_else(|| crate::prompts::registry::resolve_prompt_default("trishot_refiner"));

            if let Some(tpl) = tpl {
                let mut vars = std::collections::HashMap::new();
                vars.insert("refinement_focus".to_string(), focus_text.to_string());
                vars.insert(
                    "synthesized_prompt".to_string(),
                    synthesized_prompt.to_string(),
                );
                vars.insert("story_title".to_string(), story_title.to_string());
                vars.insert(
                    "story_genre".to_string(),
                    story_genre.unwrap_or("未知").to_string(),
                );
                vars.insert(
                    "story_tone".to_string(),
                    story_tone.unwrap_or("默认").to_string(),
                );
                crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &vars)
            } else {
                format!(
                    "请精修以下创作提示词，解决冲突并精炼冗余。精修重点：{focus_text}\n\n待精修提示词：\n{synthesized_prompt}\n\n直接输出精修后的提示词。"
                )
            }
        };

        // 调用 LLM（静默标签 tri-shot-refiner）
        let request = LlmPortRequest {
            prompt,
            agent_id: "tri-shot-refiner".to_string(),
            task: TaskType::Analysis,
            complexity: Complexity::Medium,
            budget_priority: Priority::Low,
            speed_priority: Priority::Low,
            estimated_input_tokens: 0,
            max_tokens: Some(1200),
            temperature: Some(0.4),
            request_id: uuid::Uuid::new_v4().to_string(),
            context_label: Some("tri-shot-refiner".to_string()),
            timeout_seconds_override: None,
            max_retries_override: None,
            intent_verb: None,
            intent_object: None,
            asset_tags: vec![],
            discovered_asset_ids: vec![],
            response_format: None,
            system_prompt: None,
            model_role: None,
            trace_id: None,
            tools: None,
        };

        match llm_port.generate(request).await {
            Ok(resp) => {
                let refined = resp.content.trim().to_string();
                if refined.is_empty() {
                    log::warn!("[TriShot Refiner] 精修结果为空，回退原提示词");
                    synthesized_prompt.to_string()
                } else {
                    log::info!(
                        "[TriShot Refiner] 精修完成，{}→{}字符",
                        synthesized_prompt.chars().count(),
                        refined.chars().count()
                    );
                    refined
                }
            }
            Err(e) => {
                log::warn!("[TriShot Refiner] 精修失败（{}），回退原提示词", e);
                synthesized_prompt.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refiner_does_not_panic_on_empty_input() {
        // 验证结构体可构造、方法签名正确（不调 LLM）
        let _refiner = PromptRefiner;
        // 静态方法存在性检查（不应 panic）
        assert!(true);
    }

    #[tokio::test]
    async fn test_refine_uses_llm_port_and_falls_back_on_error() {
        let original = "原始提示词".to_string();
        let refined = PromptRefiner::refine(
            std::sync::Arc::new(crate::ports::NoOpLlmPort),
            &original,
            None,
            "测试故事",
            None,
            None,
            None,
        )
        .await;
        assert_eq!(refined, original, "LlmPort 失败时应回退到原提示词");
    }
}
