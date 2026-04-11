use crate::agents::{Agent, AgentContext, AgentResult};
use crate::llm::{GenerateRequest, OpenAiAdapter, LlmAdapter};
use crate::config::LlmConfig;
use async_trait::async_trait;

pub struct InspectorAgent {
    config: LlmConfig,
}

impl InspectorAgent {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    fn build_prompt(
        &self,
        context: &AgentContext,
        content: &str,
    ) -> String {
        format!(
            r#"You are a professional fiction editor. Please analyze the following chapter and provide a detailed quality assessment.

Story Context:
- Genre: {}
- Tone: {}
- Pacing: {}
- Chapter: {}

Characters in this story:
{}

Previous key events:
{}

Chapter Content to Analyze:
{}

Please evaluate on a scale of 0-100 for each category:
1. Writing Quality (prose style, descriptions, dialogue)
2. Character Consistency (alignment with established traits)
3. Plot Coherence (logical progression from previous events)
4. Genre Alignment (appropriate tone and conventions)
5. Engagement (hook, pacing, reader interest)

Format your response as:
OVERALL_SCORE: [0-100]

STRENGTHS:
- [List 2-3 strengths]

WEAKNESSES:
- [List 2-3 areas for improvement]

SUGGESTIONS:
- [List 2-3 specific suggestions with examples]

CONSISTENCY_ISSUES:
- [List any contradictions with established facts, or "None found"]
"#,
            context.genre,
            context.tone,
            context.pacing,
            context.chapter_number,
            context.characters.iter()
                .map(|c| format!("- {}: {}", c.name, c.personality))
                .collect::<Vec<_>>()
                .join("\n"),
            context.key_events.join("\n"),
            content
        )
    }

    fn parse_result(
        &self,
        raw_response: &str,
    ) -> AgentResult {
        let mut score = None;
        let mut suggestions = vec![];

        for line in raw_response.lines() {
            if line.starts_with("OVERALL_SCORE:") {
                if let Some(num_str) = line.split(':').nth(1) {
                    score = num_str.trim().parse::<f32>().ok();
                }
            } else if line.starts_with("- ") {
                suggestions.push(line.trim_start_matches("- ").to_string());
            }
        }

        AgentResult {
            content: raw_response.to_string(),
            score,
            suggestions,
        }
    }
}

#[async_trait]
impl Agent for InspectorAgent {
    fn name(&self) -> &str {
        "inspector"
    }

    fn description(&self) -> &str {
        "Quality checks chapters and provides improvement suggestions"
    }

    async fn execute(
        &self,
        context: &AgentContext,
        input: &str,
    ) -> Result<AgentResult, Box<dyn std::error::Error>> {
        if self.config.api_key.is_empty() {
            return Err("API Key not configured".into());
        }

        let adapter = OpenAiAdapter::new(
            self.config.api_key.clone(),
            self.config.model.clone(),
            self.config.api_base.clone(),
            1500,
            0.2,
        );

        let prompt = self.build_prompt(context, input);

        let request = GenerateRequest {
            prompt,
            max_tokens: Some(1500),
            temperature: Some(0.2),
        };

        let response = adapter.generate(request).await?;
        let result = self.parse_result(&response.content);

        Ok(result)
    }
}