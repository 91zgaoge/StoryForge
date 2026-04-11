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
