use super::{WorkflowInstance, WorkflowStatus, NodeExecutionStatus};
use std::collections::HashMap;

/// Workflow scheduler - manages task execution
pub struct WorkflowScheduler;

impl WorkflowScheduler {
    pub fn new() -> Self {
        Self
    }

    pub async fn schedule_execution(
        &self,
        instance_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("[WorkflowScheduler] Queuing workflow instance {} for execution", instance_id);
        // In production, this would enqueue the instance to a task queue
        // and let a worker pool pick it up. For now, we just log the request.
        Ok(())
    }

    /// Get next executable nodes based on current state
    pub fn get_next_nodes(
        &self,
        instance: &WorkflowInstance,
        workflow_nodes: &[super::WorkflowNode],
        workflow_edges: &[super::WorkflowEdge],
    ) -> Vec<String> {
        let mut next_nodes = Vec::new();
        let completed: std::collections::HashSet<String> = instance.context.completed_nodes.iter().cloned().collect();

        for node in workflow_nodes {
            // Skip already processed nodes
            if let Some(state) = instance.node_states.get(&node.id) {
                if state.status != NodeExecutionStatus::Pending {
                    continue;
                }
            }

            // Check if all dependencies are completed
            let dependencies: Vec<String> = workflow_edges
                .iter()
                .filter(|e| e.to_node == node.id)
                .map(|e| e.from_node.clone())
                .collect();

            let all_deps_completed = dependencies.iter().all(|dep| completed.contains(dep));

            if all_deps_completed || dependencies.is_empty() {
                next_nodes.push(node.id.clone());
            }
        }

        next_nodes
    }

    /// Update node execution status
    pub fn update_node_status(
        &self,
        instance: &mut WorkflowInstance,
        node_id: &str,
        status: NodeExecutionStatus,
        output: Option<serde_json::Value>,
        error: Option<String>,
    ) {
        if let Some(state) = instance.node_states.get_mut(node_id) {
            state.status = status.clone();
            state.output = output;
            state.error = error;

            match status {
                NodeExecutionStatus::Running => {
                    state.started_at = Some(chrono::Utc::now());
                }
                NodeExecutionStatus::Completed => {
                    state.completed_at = Some(chrono::Utc::now());
                    if !instance.context.completed_nodes.contains(&node_id.to_string()) {
                        instance.context.completed_nodes.push(node_id.to_string());
                    }
                }
                NodeExecutionStatus::Failed => {
                    instance.context.failed_nodes.push(node_id.to_string());
                }
                _ => {}
            }
        }
    }

    /// Check if workflow is complete
    pub fn is_workflow_complete(
        &self,
        instance: &WorkflowInstance,
        workflow_nodes: &[super::WorkflowNode],
    ) -> bool {
        let end_nodes: Vec<String> = workflow_nodes
            .iter()
            .filter(|n| matches!(n.node_type, super::NodeType::End))
            .map(|n| n.id.clone())
            .collect();

        end_nodes.iter().all(|end_id| {
            instance.node_states.get(end_id)
                .map(|s| s.status == NodeExecutionStatus::Completed)
                .unwrap_or(false)
        })
    }
}

impl Default for WorkflowScheduler {
    fn default() -> Self {
        Self::new()
    }
}
