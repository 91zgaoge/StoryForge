use super::model::*;
use crate::state::StoryState;
use crate::ComplexityTier;
use std::collections::HashMap;

pub struct ModelRouter {
    models: HashMap<ComplexityTier, Vec<ModelConfig>>,
}

impl ModelRouter {
    pub fn new(models: Vec<ModelConfig>) -> Self {
        let mut grouped: HashMap<ComplexityTier, Vec<ModelConfig>> = HashMap::new();
        for model in models {
            grouped.entry(model.tier.clone()).or_default().push(model);
        }
        for (_, tier_models) in grouped.iter_mut() {
            tier_models.sort_by(|a, b| a.cost_per_1k_tokens.partial_cmp(&b.cost_per_1k_tokens).unwrap());
        }
        Self { models: grouped }
    }
    
    pub fn route(&self, task_type: TaskType, complexity: ComplexityTier) -> RoutingDecision {
        let candidates = self.models.get(&complexity).unwrap();
        let selected = candidates.first().cloned().unwrap();
        RoutingDecision {
            model: selected,
            reason: format!("{:?} complexity", complexity),
            estimated_cost: 0.0,
            temperature: 0.7,
            max_tokens: 2000,
            fallback: candidates.get(1).cloned(),
        }
    }
}
