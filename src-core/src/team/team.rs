use crate::agents::{Agent, AgentConfig, AgentRunResult};
use crate::tool::{ToolRegistry, ToolExecutor};
use crate::router::ModelRouter;
use crate::memory::SharedMemory;

use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TeamConfig {
    pub agents: Vec<AgentConfig>,
    pub shared_memory: bool,
}

pub struct Team {
    name: String,
    config: TeamConfig,
    agents: HashMap<String, Agent>,
    shared_memory: Option<SharedMemory>,
    tool_registry: Arc<ToolRegistry>,
    tool_executor: Arc<ToolExecutor>,
    model_router: Arc<ModelRouter>,
}

#[derive(Debug, Clone)]
pub struct TeamRunResult {
    pub goal: String,
    pub tasks_completed: usize,
    pub final_output: String,
    pub agent_results: Vec<AgentRunResult>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: Option<String>,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Team {
    pub fn new(
        name: String,
        config: TeamConfig,
        tool_registry: Arc<ToolRegistry>,
        tool_executor: Arc<ToolExecutor>,
        model_router: Arc<ModelRouter>,
        enable_shared_memory: bool,
    ) -> Self {
        let mut agents = HashMap::new();
        
        for agent_config in &config.agents {
            let agent = Agent::new(
                agent_config.clone(),
                tool_registry.clone(),
                tool_executor.clone(),
                model_router.clone(),
            );
            agents.insert(agent_config.name.clone(), agent);
        }

        let shared_memory = if enable_shared_memory {
            Some(SharedMemory::new())
        } else {
            None
        };

        Self {
            name,
            config,
            agents,
            shared_memory,
            tool_registry,
            tool_executor,
            model_router,
        }
    }

    pub fn get_agent(&self,
        name: &str) -> Option<&Agent> {
        self.agents.get(name)
    }

    pub fn get_agent_names(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    pub fn broadcast_message(&self,
        from: &str,
        content: &str) {
        let msg = Message {
            from: from.to_string(),
            to: None,
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        if let Some(ref memory) = self.shared_memory {
            memory.add_message(msg);
        }
    }

    pub fn send_direct_message(
        &self,
        from: &str,
        to: &str,
        content: &str) {
        let msg = Message {
            from: from.to_string(),
            to: Some(to.to_string()),
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        if let Some(ref memory) = self.shared_memory {
            memory.add_message(msg);
        }
    }
}
