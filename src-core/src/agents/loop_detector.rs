//! Loop detection for agent conversations (from open-multi-agent)

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct LoopDetectionConfig {
    pub max_repetitions: u32,
    pub window_size: usize,
}

impl Default for LoopDetectionConfig {
    fn default() -> Self {
        Self {
            max_repetitions: 3,
            window_size: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoopDetectionInfo {
    pub kind: LoopKind,
    pub repetitions: u32,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopKind {
    ToolRepetition,
    TextRepetition,
}

pub struct LoopDetector {
    config: LoopDetectionConfig,
    tool_calls: VecDeque<String>,
    text_segments: VecDeque<String>,
}

impl LoopDetector {
    pub fn new(config: LoopDetectionConfig) -> Self {
        Self {
            config,
            tool_calls: VecDeque::with_capacity(config.window_size),
            text_segments: VecDeque::with_capacity(config.window_size),
        }
    }

    pub fn record_tool_call(
        &mut self,
        tool_name: &str,
        input: &str) -> Option<LoopDetectionInfo> {
        let fingerprint = format!("{}:{}", tool_name, input);
        
        self.tool_calls.push_back(fingerprint.clone());
        if self.tool_calls.len() > self.config.window_size {
            self.tool_calls.pop_front();
        }

        let repetitions = self.tool_calls.iter()
            .filter(|f| *f == &fingerprint)
            .count() as u32;

        if repetitions >= self.config.max_repetitions {
            return Some(LoopDetectionInfo {
                kind: LoopKind::ToolRepetition,
                repetitions,
                detail: format!(
                    "Tool {} called {} times with identical input",
                    tool_name, repetitions
                ),
            });
        }

        None
    }

    pub fn record_text(&mut self,
        text: &str) -> Option<LoopDetectionInfo> {
        let normalized = text.trim().to_lowercase();
        
        self.text_segments.push_back(normalized.clone());
        if self.text_segments.len() > self.config.window_size {
            self.text_segments.pop_front();
        }

        let repetitions = self.text_segments.iter()
            .filter(|t| *t == &normalized)
            .count() as u32;

        if repetitions >= self.config.max_repetitions {
            return Some(LoopDetectionInfo {
                kind: LoopKind::TextRepetition,
                repetitions,
                detail: format!(
                    "Similar text generated {} times",
                    repetitions
                ),
            });
        }

        None
    }
}
