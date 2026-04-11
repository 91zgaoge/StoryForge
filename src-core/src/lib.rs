// Minimal CINEMA-AI Core
pub mod error;
pub use error::{CinemaError, Result};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityTier {
    Low, Medium, High, Critical
}

impl ComplexityTier {
    pub fn as_str(&self) -> &str {
        match self {
            ComplexityTier::Low => "low",
            ComplexityTier::Medium => "medium",
            ComplexityTier::High => "high",
            ComplexityTier::Critical => "critical",
        }
    }

    /// Classify complexity based on prompt content
    pub fn classify_from_prompt(prompt: &str) -> Self {
        let prompt_lower = prompt.to_lowercase();
        
        // Critical complexity indicators
        if prompt_lower.contains("analyze")
            || prompt_lower.contains("complex")
            || prompt_lower.contains("revelation")
            || prompt_lower.contains("climax")
        {
            ComplexityTier::Critical
        }
        // High complexity indicators
        else if prompt_lower.contains("evaluate")
            || prompt_lower.contains("compare")
            || prompt_lower.contains("detailed")
        {
            ComplexityTier::High
        }
        // Medium complexity indicators
        else if prompt_lower.contains("describe")
            || prompt_lower.contains("explain")
            || prompt_lower.contains("develop")
        {
            ComplexityTier::Medium
        }
        // Default to Low
        else {
            ComplexityTier::Low
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterOutput {
    pub chapter_number: u32,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryConfig {
    pub title: String,
    pub total_chapters: u32,
}
