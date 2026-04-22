#![allow(dead_code)]
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

        // Calculate writing streak from chapter creation dates
        let mut dates: Vec<NaiveDate> = chapters
            .iter()
            .map(|c| c.created_at.date_naive())
            .collect();
        dates.sort_unstable();
        dates.dedup();
        dates.reverse();

        let (current_streak, longest_streak, last_writing_date) = if dates.is_empty() {
            (0, 0, None)
        } else {
            let today = Utc::now().date_naive();
            let mut longest = 0;
            let mut streak = 0;
            let mut prev_date = today.succ_opt().unwrap_or(today);

            for &date in &dates {
                if prev_date.succ_opt() == Some(date) || prev_date == date {
                    streak += 1;
                } else {
                    longest = longest.max(streak);
                    streak = 1;
                }
                prev_date = date;
            }
            longest = longest.max(streak);

            // Current streak: count backwards from today
            let mut check_date = today;
            let mut curr_streak = 0;
            let date_set: std::collections::HashSet<_> = dates.iter().cloned().collect();
            while date_set.contains(&check_date) {
                curr_streak += 1;
                check_date = check_date.pred_opt().unwrap_or(check_date);
            }
            // If no writing today, check yesterday
            if curr_streak == 0 {
                check_date = today.pred_opt().unwrap_or(today);
                while date_set.contains(&check_date) {
                    curr_streak += 1;
                    check_date = check_date.pred_opt().unwrap_or(check_date);
                }
            }

            (curr_streak, longest, Some(dates[0]))
        };

        let writing_days = dates.len().max(1) as i32;
        let avg_words_per_day = if writing_days > 0 {
            (total_words as f32) / (writing_days as f32)
        } else {
            0.0
        };

        // Productivity score: combination of consistency and output
        let consistency_factor = (current_streak as f32).min(30.0) / 30.0;
        let output_factor = (avg_words_per_day / 2000.0).min(1.0);
        let productivity_score = (consistency_factor * 50.0 + output_factor * 50.0).min(100.0);

        WritingAnalytics {
            story_id: story_id.to_string(),
            total_words,
            total_chapters,
            writing_streak: WritingStreak {
                current_streak,
                longest_streak,
                last_writing_date,
            },
            productivity_score,
            avg_words_per_day,
        }
    }
}
