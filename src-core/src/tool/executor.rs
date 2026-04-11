use crate::tool::framework::ToolRegistry;
use serde_json::Value;
use std::sync::Arc;

pub struct ToolExecutor {
    registry: Arc<ToolRegistry>,
}

pub struct ToolContext {
    pub agent_name: String,
    pub agent_role: String,
}

pub struct ToolResult {
    pub data: Value,
    pub is_error: bool,
}

impl ToolExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    pub async fn execute(
        &self,
        name: &str,
        input: Value,
        _context: ToolContext,
    ) -> Result<ToolResult, String> {
        let tool = self.registry.get(name)
            .ok_or_else(|| format!("Tool {} not found", name))?;

        match tool.execute(input).await {
            Ok(data) => Ok(ToolResult { data, is_error: false }),
            Err(err) => Ok(ToolResult {
                data: Value::String(err.clone()),
                is_error: true,
            }),
        }
    }
}
