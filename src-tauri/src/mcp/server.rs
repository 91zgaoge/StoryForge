use super::types::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

pub trait McpToolHandler: Send + Sync {
    fn handle(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

/// Built-in tool: File System Operations
pub struct FileSystemTool;

impl McpToolHandler for FileSystemTool {
    fn handle(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let operation = arguments.get("operation").and_then(|v| v.as_str()).unwrap_or("read");
        let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("");

        match operation {
            "read" => {
                let content = std::fs::read_to_string(path)?;
                Ok(serde_json::json!({ "content": content }))
            }
            "write" => {
                let content = arguments.get("content").and_then(|v| v.as_str()).unwrap_or("");
                std::fs::write(path, content)?;
                Ok(serde_json::json!({ "success": true }))
            }
            "list" => {
                let entries: Vec<String> = std::fs::read_dir(path)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                Ok(serde_json::json!({ "entries": entries }))
            }
            _ => Err("Unknown operation".into()),
        }
    }
}

/// Built-in tool: Text Processing
pub struct TextProcessingTool;

impl McpToolHandler for TextProcessingTool {
    fn handle(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let operation = arguments.get("operation").and_then(|v| v.as_str()).unwrap_or("count");
        let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("");

        match operation {
            "count" => {
                let chars = text.chars().count();
                let words = text.split_whitespace().count();
                let lines = text.lines().count();
                Ok(serde_json::json!({
                    "characters": chars,
                    "words": words,
                    "lines": lines
                }))
            }
            "split" => {
                let delimiter = arguments.get("delimiter").and_then(|v| v.as_str()).unwrap_or("\n");
                let parts: Vec<String> = text.split(delimiter).map(|s| s.to_string()).collect();
                Ok(serde_json::json!({ "parts": parts }))
            }
            "replace" => {
                let from = arguments.get("from").and_then(|v| v.as_str()).unwrap_or("");
                let to = arguments.get("to").and_then(|v| v.as_str()).unwrap_or("");
                let result = text.replace(from, to);
                Ok(serde_json::json!({ "result": result }))
            }
            _ => Err("Unknown operation".into()),
        }
    }
}

/// Built-in tool: Web Search (Simulated)
pub struct WebSearchTool;

impl McpToolHandler for WebSearchTool {
    fn handle(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");

        // Simulate search results
        Ok(serde_json::json!({
            "query": query,
            "results": [
                {
                    "title": format!("Search result for: {}", query),
                    "snippet": "This is a simulated search result...",
                    "url": "https://example.com/result1"
                }
            ],
            "note": "This is a simulated search. Connect to real search API for actual results."
        }))
    }
}

pub struct McpServer {
    config: McpServerConfig,
    tools: Arc<Mutex<HashMap<String, (McpTool, Box<dyn McpToolHandler>)>>>,
    child_process: Arc<Mutex<Option<Child>>>,
}

impl McpServer {
    pub fn new(config: McpServerConfig) -> Self {
        let server = Self {
            config,
            tools: Arc::new(Mutex::new(HashMap::new())),
            child_process: Arc::new(Mutex::new(None)),
        };

        // Register built-in tools
        server.register_built_in_tools();
        server
    }

    fn register_built_in_tools(&self) {
        // File System Tool
        self.register_tool(
            McpTool {
                name: "filesystem".to_string(),
                description: "File system operations (read, write, list)".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "operation": { "type": "string", "enum": ["read", "write", "list"] },
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["operation", "path"]
                }),
            },
            Box::new(FileSystemTool),
        );

        // Text Processing Tool
        self.register_tool(
            McpTool {
                name: "text_processing".to_string(),
                description: "Text processing operations (count, split, replace)".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "operation": { "type": "string", "enum": ["count", "split", "replace"] },
                        "text": { "type": "string" },
                        "delimiter": { "type": "string" },
                        "from": { "type": "string" },
                        "to": { "type": "string" }
                    },
                    "required": ["operation", "text"]
                }),
            },
            Box::new(TextProcessingTool),
        );

        // Web Search Tool
        self.register_tool(
            McpTool {
                name: "web_search".to_string(),
                description: "Search the web for information".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
            },
            Box::new(WebSearchTool),
        );
    }

    pub fn register_tool(
        &self,
        tool: McpTool,
        handler: Box<dyn McpToolHandler>,
    ) {
        self.tools.lock().unwrap().insert(tool.name.clone(), (tool, handler));
    }

    pub fn get_tools(&self) -> Vec<McpTool> {
        self.tools.lock().unwrap().values().map(|(t, _)| t.clone()).collect()
    }

    pub fn handle_tool_call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let tools = self.tools.lock().unwrap();
        if let Some((_, handler)) = tools.get(tool_name) {
            handler
                .handle(arguments)
                .map_err(|e| McpError::RpcError(e.to_string()))
        } else {
            Err(McpError::RpcError(format!("Tool not found: {}", tool_name)))
        }
    }

    pub async fn start(&self) -> Result<(), McpError> {
        // Start external MCP server process if configured
        if !self.config.command.is_empty() {
            let mut cmd = Command::new(&self.config.command);
            cmd.args(&self.config.args)
                .envs(&self.config.env)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let child = cmd.spawn().map_err(|e| McpError::TransportError(e.to_string()))?;

            *self.child_process.lock().unwrap() = Some(child);
        }

        log::info!("MCP Server started with {} tools", self.get_tools().len());
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), McpError> {
        if let Some(mut child) = self.child_process.lock().unwrap().take() {
            let _ = child.kill().await;
        }
        Ok(())
    }

    /// Execute tool with timeout
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let timeout = tokio::time::Duration::from_secs(self.config.timeout_seconds.max(30));

        match tokio::time::timeout(timeout, async {
            self.handle_tool_call(tool_name, arguments)
        }).await {
            Ok(result) => result,
            Err(_) => Err(McpError::Timeout),
        }
    }
}
