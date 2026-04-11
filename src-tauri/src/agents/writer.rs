use crate::agents::{Agent, AgentContext, AgentResult};
use crate::llm::{GenerateRequest, OpenAiAdapter, LlmAdapter};
use crate::config::LlmConfig;
use async_trait::async_trait;

pub struct WriterAgent {
    config: LlmConfig,
}

impl WriterAgent {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    fn build_system_prompt(&self,
        context: &AgentContext,
    ) -> String {
        let mut prompt = format!(
            r#"You are an expert Chinese fiction writer specializing in {} genre.
Your writing style should be: {} tone, {} pacing.

Story Context:
- Title: {}
- Genre: {}
- Current Chapter: {}

"#,
            context.genre,
            context.tone,
            context.pacing,
            context.story_title,
            context.genre,
            context.chapter_number
        );

        // Add character information
        if !context.characters.is_empty() {
            prompt.push_str("\nCharacter Information:\n");
            for char in &context.characters {
                prompt.push_str(&format!(
                    "- {}: {} (Current state: {})\n",
                    char.name, char.personality, char.current_state
                ));
            }
        }

        // Add key events summary
        if !context.key_events.is_empty() {
            prompt.push_str("\nKey Events So Far:\n");
            for event in &context.key_events {
                prompt.push_str(&format!("- {}\n", event));
            }
        }

        // Add recent chapter summaries
        if !context.previous_chapters.is_empty() {
            prompt.push_str("\nRecent Chapter Summaries:\n");
            let recent = context.previous_chapters.iter().rev().take(3).rev();
            for ch in recent {
                prompt.push_str(&format!(
                    "Chapter {} - {}: {}\n",
                    ch.chapter_number, ch.title, ch.summary
                ));
            }
        }

        prompt.push_str(r#"

Writing Guidelines:
1. Write in Chinese (简体中文) with rich, descriptive prose
2. Show, don't tell - use vivid sensory details
3. Maintain consistent character voices and personalities
4. Include natural dialogue that advances the plot
5. End with a compelling hook that keeps readers engaged
6. Target length: 1500-2000 Chinese characters

"#);

        prompt
    }

    fn build_user_prompt(&self,
        context: &AgentContext,
        outline: &str,
    ) -> String {
        format!(
            r#"Please write Chapter {} based on the following outline:

Outline:
{}

Requirements:
- Follow the outline closely while adding rich details
- Include both narrative and dialogue
- Advance the plot while developing characters
- Create atmosphere appropriate to the genre
- Write approximately 1500-2000 Chinese characters

Write the chapter now:"#,
            context.chapter_number,
            outline
        )
    }
}

#[async_trait]
impl Agent for WriterAgent {
    fn name(&self) -> &str {
        "writer"
    }

    fn description(&self) -> &str {
        "Generates story chapters with context awareness"
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
            self.config.max_tokens,
            self.config.temperature,
        );

        let system_prompt = self.build_system_prompt(context);
        let user_prompt = self.build_user_prompt(context, input);

        let request = GenerateRequest {
            prompt: format!("{}\n\n{}", system_prompt, user_prompt),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
        };

        let response = adapter.generate(request).await?;

        Ok(AgentResult {
            content: response.content,
            score: None,
            suggestions: vec![],
        })
    }
}