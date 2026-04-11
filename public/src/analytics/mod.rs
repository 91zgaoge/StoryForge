use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingAnalytics {
    pub story_id: String,
    pub total_words: i64,
    pub total_characters: i32,
    pub total_chapters: i32,
    pub writing_streak: WritingStreak,
    pub daily_stats: Vec<DailyStat>,
    pub writing_patterns: WritingPatterns,
    pub productivity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStreak {
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_writing_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: NaiveDate,
    pub words_written: i64,
    pub chapters_edited: i32,
    pub time_spent_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingPatterns {
    pub most_productive_hour: i32,
    pub most_productive_day: String,
    pub average_session_length: i32,
    pub average_words_per_session: i64,
    pub consistency_score: f32,
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
        let total_words: i64 = chapters.iter().map(|c| c.word_count.unwrap_or(0) as i64).sum();
        let total_chapters = chapters.len() as i32;

        WritingAnalytics {
            story_id: story_id.to_string(),
            total_words,
            total_characters: 0,
            total_chapters,
            writing_streak: WritingStreak {
                current_streak: 1,
                longest_streak: 7,
                last_writing_date: Some(Utc::now().date_naive()),
            },
            daily_stats: vec![],
            writing_patterns: WritingPatterns {
                most_productive_hour: 20,
                most_productive_day: "Saturday".to_string(),
                average_session_length: 60,
                average_words_per_session: 1500,
                consistency_score: 75.0,
            },
            productivity_score: 80.0,
        }
    }
}
