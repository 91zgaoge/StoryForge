use serde::{Deserialize, Serialize};
use chrono::{Utc, NaiveDate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingAnalytics {
    pub story_id: String,
    pub total_words: i64,
    pub total_chapters: i32,
    pub writing_streak: WritingStreak,
    pub productivity_score: f32,
    pub avg_words_per_day: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStreak {
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_writing_date: Option<NaiveDate>,
}

pub struct AnalyticsEngine;

impl AnalyticsEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_writing_data(
        &self,
        story_id: &str,
        chapters: &[crate::db::Chapter],
    ) -> WritingAnalytics {
        let total_words: i64 = chapters
            .iter()
            .map(|c| c.word_count.unwrap_or(0) as i64)
            .sum();
        let total_chapters = chapters.len() as i32;

        WritingAnalytics {
            story_id: story_id.to_string(),
            total_words,
            total_chapters,
            writing_streak: WritingStreak {
                current_streak: 1,
                longest_streak: 7,
                last_writing_date: Some(Utc::now().date_naive()),
            },
            productivity_score: 80.0,
            avg_words_per_day: 1500.0,
        }
    }
}
