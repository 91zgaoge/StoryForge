//! Task-to-agent assignment strategies (from open-multi-agent)

use crate::agents::AgentConfig;
use crate::task::Task;

/// Scheduling strategy for task assignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingStrategy {
    /// Round-robin assignment
    RoundRobin,
    /// Assign to least loaded agent
    LeastLoaded,
    /// Assign based on capability matching
    CapabilityMatch,
    /// Random assignment
    Random,
}

/// Task scheduler
pub struct Scheduler {
    strategy: SchedulingStrategy,
    round_robin_index: std::sync::atomic::AtomicUsize,
}

impl Scheduler {
    pub fn new(strategy: SchedulingStrategy) -> Self {
        Self {
            strategy,
            round_robin_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Select the best agent for a task
    pub fn select_agent(
        &self,
        _task: &Task,
        agents: &[AgentConfig],
    ) -> Option<&AgentConfig> {
        if agents.is_empty() {
            return None;
        }

        match self.strategy {
            SchedulingStrategy::RoundRobin => {
                let idx = self.round_robin_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Some(&agents[idx % agents.len()])
            }
            SchedulingStrategy::LeastLoaded => {
                // Simplified - would track load in production
                agents.first()
            }
            SchedulingStrategy::CapabilityMatch => {
                // Would match task requirements to agent capabilities
                agents.first()
            }
            SchedulingStrategy::Random => {
                use rand::Rng;
                let idx = rand::thread_rng().gen_range(0..agents.len());
                Some(&agents[idx])
            }
        }
    }
}
