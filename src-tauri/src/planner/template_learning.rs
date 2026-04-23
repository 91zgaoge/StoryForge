//! Plan Template Learning - 计划模板学习
//!
//! Records successful execution plans and reuses them for similar requests.

use serde::{Deserialize, Serialize};
use super::ExecutionPlan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTemplate {
    pub id: String,
    pub trigger_patterns: Vec<String>,
    pub plan: ExecutionPlan,
    pub success_count: u32,
    pub failure_count: u32,
}

pub struct PlanTemplateLibrary {
    templates: Vec<PlanTemplate>,
}

impl PlanTemplateLibrary {
    pub fn new() -> Self {
        Self { templates: Vec::new() }
    }

    pub fn find_match(&self, user_input: &str) -> Option<&PlanTemplate> {
        self.templates.iter()
            .find(|t| t.trigger_patterns.iter().any(|p| user_input.contains(p)))
    }

    pub fn record_success(&mut self, user_input: &str, plan: ExecutionPlan) {
        let patterns: Vec<String> = user_input.split_whitespace()
            .filter(|w| w.len() > 2)
            .map(|w| w.to_string())
            .collect();

        if !patterns.is_empty() {
            self.templates.push(PlanTemplate {
                id: uuid::Uuid::new_v4().to_string(),
                trigger_patterns: patterns,
                plan,
                success_count: 1,
                failure_count: 0,
            });
        }
    }
}
