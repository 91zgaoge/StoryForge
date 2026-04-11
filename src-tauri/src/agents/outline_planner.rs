use crate::agents::{Agent, AgentContext, AgentResult};
use crate::llm::{GenerateRequest, OpenAiAdapter, LlmAdapter};
use crate::config::LlmConfig;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 大纲规划 Agent - 生成完整的故事大纲
pub struct OutlinePlannerAgent {
    config: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryOutline {
    pub title: String,
    pub total_chapters: i32,
    pub acts: Vec<Act>,
    pub character_arcs: Vec<CharacterArc>,
    pub key_themes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Act {
    pub act_number: i32,
    pub name: String,
    pub description: String,
    pub chapters: Vec<ChapterOutline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterOutline {
    pub chapter_number: i32,
    pub title: String,
    pub summary: String,
    pub key_events: Vec<String>,
    pub characters_involved: Vec<String>,
    pub emotional_arc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterArc {
    pub character_name: String,
    pub initial_state: String,
    pub goal: String,
    pub transformation: String,
    pub key_moments: Vec<i32>, // chapter numbers
}

impl OutlinePlannerAgent {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    pub fn build_prompt(
        &self,
        story_title: &str,
        genre: &str,
        tone: &str,
        target_chapters: i32,
        premise: &str,
        characters: &[crate::db::Character],
    ) -> String {
        let character_desc = characters
            .iter()
            .map(|c| format!("- {}: {}", c.name, c.background.clone().unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"You are an expert story structure consultant. Create a detailed story outline based on the following information.

Story Information:
- Title: {}
- Genre: {}
- Tone: {}
- Target Chapters: {}
- Premise: {}

Characters:
{}

Create a comprehensive outline following the three-act structure:
1. Act 1: Setup (25% of chapters)
2. Act 2: Confrontation (50% of chapters)
3. Act 3: Resolution (25% of chapters)

For each chapter provide:
- A compelling title
- Brief summary (2-3 sentences)
- Key events
- Characters involved
- Emotional arc

Also outline character arcs showing how each major character transforms throughout the story.

Format your response as JSON:
{{
  "acts": [
    {{
      "act_number": 1,
      "name": "Act Name",
      "description": "What happens in this act",
      "chapters": [
        {{
          "chapter_number": 1,
          "title": "Chapter Title",
          "summary": "What happens",
          "key_events": ["event1", "event2"],
          "characters_involved": ["Character1"],
          "emotional_arc": "hopeful -> anxious"
        }}
      ]
    }}
  ],
  "character_arcs": [
    {{
      "character_name": "Name",
      "initial_state": "starting point",
      "goal": "what they want",
      "transformation": "how they change",
      "key_moments": [1, 5, 10]
    }}
  ],
  "key_themes": ["theme1", "theme2"]
}}

Provide only the JSON output, no additional text."#,
            story_title, genre, tone, target_chapters, premise, character_desc
        )
    }

    pub fn parse_outline(&self, json_str: &str) -> Result<StoryOutline, Box<dyn std::error::Error>> {
        // Try to find JSON content between code blocks or braces
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

        let outline: StoryOutline = serde_json::from_str(json_content.trim())?;
        Ok(outline)
    }
}

#[async_trait]
impl Agent for OutlinePlannerAgent {
    fn name(&self) -> &str {
        "outline_planner"
    }

    fn description(&self) -> &str {
        "Creates comprehensive story outlines with three-act structure"
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
            3000,
            0.7,
        );

        let request = GenerateRequest {
            prompt: input.to_string(),
            max_tokens: Some(3000),
            temperature: Some(0.7),
        };

        let response = adapter.generate(request).await?;

        Ok(AgentResult {
            content: response.content.clone(),
            score: None,
            suggestions: vec![],
        })
    }
}
