use crate::agents::{Agent, AgentContext, AgentResult};
use crate::llm::{GenerateRequest, OpenAiAdapter, LlmAdapter};
use crate::config::LlmConfig;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 风格模仿 Agent - 分析并模仿指定作者风格
pub struct StyleMimicAgent {
    config: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleAnalysis {
    pub sentence_structure: String,
    pub vocabulary_level: String,
    pub pacing_pattern: String,
    pub dialogue_style: String,
    pub descriptive_technique: String,
    pub tone_qualities: Vec<String>,
    pub distinctive_markers: Vec<String>,
}

impl StyleMimicAgent {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    /// 分析参考文本的风格
    pub fn build_analysis_prompt(&self,
        reference_text: &str,
    ) -> String {
        format!(
            r#"Analyze the writing style of the following text excerpt. Identify specific stylistic characteristics:

Reference Text:
{}

Provide a detailed style analysis including:
1. Sentence structure patterns (short/long, complex/simple, rhythm)
2. Vocabulary level and word choice preferences
3. Pacing and flow patterns
4. Dialogue style (if present)
5. Descriptive techniques and imagery
6. Tone qualities (formal/informal, dark/light, etc.)
7. Distinctive markers that make this style unique

Format as JSON:
{{
  "sentence_structure": "description",
  "vocabulary_level": "description",
  "pacing_pattern": "description",
  "dialogue_style": "description",
  "descriptive_technique": "description",
  "tone_qualities": ["quality1", "quality2"],
  "distinctive_markers": ["marker1", "marker2"]
}}"#,
            reference_text.chars().take(2000).collect::<String>()
        )
    }

    /// 使用分析的风格重写内容
    pub fn build_rewrite_prompt(
        &self,
        style: &StyleAnalysis,
        content: &str,
        target_type: &str, // "chapter", "dialogue", "description"
    ) -> String {
        format!(
            r#"Rewrite the following content in a specific writing style.

Target Style Characteristics:
- Sentence Structure: {}
- Vocabulary: {}
- Pacing: {}
- Dialogue Style: {}
- Descriptive Technique: {}
- Tone Qualities: {}
- Distinctive Markers: {}

Content to Rewrite ({}):
{}

Instructions:
1. Maintain the original plot/events/dialogue meaning
2. Apply the target style characteristics thoroughly
3. Keep the same approximate length
4. Write in Chinese if the original is Chinese, English if English

Rewritten version:"#,
            style.sentence_structure,
            style.vocabulary_level,
            style.pacing_pattern,
            style.dialogue_style,
            style.descriptive_technique,
            style.tone_qualities.join(", "),
            style.distinctive_markers.join(", "),
            target_type,
            content
        )
    }

    pub fn parse_style_analysis(&self,
        json_str: &str,
    ) -> Result<StyleAnalysis, Box<dyn std::error::Error>> {
        let json_content = if json_str.contains("```json") {
            json_str.split("```json").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(json_str)
        } else if json_str.contains("```") {
            json_str.split("```").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(json_str)
        } else {
            json_str
        };

        let analysis: StyleAnalysis = serde_json::from_str(json_content.trim())?;
        Ok(analysis)
    }
}

#[async_trait]
impl Agent for StyleMimicAgent {
    fn name(&self) -> &str {
        "style_mimic"
    }

    fn description(&self) -> &str {
        "Analyzes and mimics specific writing styles"
    }

    async fn execute(
        &self,
        _context: &AgentContext,
        input: &str,
    ) -> Result<AgentResult, Box<dyn std::error::Error>> {
        if self.config.api_key.is_empty() {
            return Err("API Key not configured".into());
        }

        let adapter = OpenAiAdapter::new(
            self.config.api_key.clone(),
            self.config.model.clone(),
            self.config.api_base.clone(),
            self.config.max_tokens,
            0.6,
        );

        let request = GenerateRequest {
            prompt: input.to_string(),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(0.6),
        };

        let response = adapter.generate(request).await?;

        Ok(AgentResult {
            content: response.content,
            score: None,
            suggestions: vec![],
        })
    }
}
