//! Built-in tools inspired by open-multi-agent

use crate::tool::framework::ToolRegistry;
use serde_json::json;
use std::sync::Arc;

pub fn register_built_in_tools(registry: &mut ToolRegistry) {
    // File read tool
    let file_read = super::framework::define_tool(
        "file_read",
        "Read content from a file",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
            "required": ["path"]
        }),
        |input| async move {
            let path = input.get("path").and_then(|p| p.as_str())
                .ok_or("Path required")?;
            let content = tokio::fs::read_to_string(path).await
                .map_err(|e| e.to_string())?;
            Ok(json!({ "content": content }))
        },
    );
    registry.register(file_read);

    // File write tool
    let file_write = super::framework::define_tool(
        "file_write",
        "Write content to a file",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        }),
        |input| async move {
            let path = input.get("path").and_then(|p| p.as_str())
                .ok_or("Path required")?;
            let content = input.get("content").and_then(|c| c.as_str())
                .ok_or("Content required")?;
            tokio::fs::write(path, content).await
                .map_err(|e| e.to_string())?;
            Ok(json!({ "success": true }))
        },
    );
    registry.register(file_write);

    // Grep tool
    let grep = super::framework::define_tool(
        "grep",
        "Search for patterns in files",
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "path": { "type": "string" }
            },
            "required": ["pattern", "path"]
        }),
        |input| async move {
            let pattern = input.get("pattern").and_then(|p| p.as_str())
                .ok_or("Pattern required")?;
            let path = input.get("path").and_then(|p| p.as_str())
                .ok_or("Path required")?;
            
            // Simplified grep implementation
            let mut matches = vec![];
            if let Ok(entries) = tokio::fs::read_dir(path).await {
                // Would implement actual grep here
            }
            Ok(json!({ "matches": matches }))
        },
    );
    registry.register(grep);
}
