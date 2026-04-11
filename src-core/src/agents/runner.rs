//! AgentRunner - Core conversation loop engine (ported from open-multi-agent)
//!
//! Handles:
//! - Sending messages to LLM adapter
//! - Extracting tool-use blocks from response
//! - Executing tool calls in parallel
//! - Appending tool results and looping back
//! - Accumulating token usage across turns

use crate::agents::{ToolCallRecord, TokenUsage};
use crate::tool::{ToolRegistry, ToolExecutor, ToolContext};
use crate::error::{CinemaError, Result};

use serde_json::Value;
use std::sync::Arc;

/// Runner configuration (from open-multi-agent RunnerOptions)
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub model: String,
    pub system_prompt: Option<String>,
    pub max_turns: u32,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub max_token_budget: Option<u32>,
    pub agent_name: Option<String>,
    pub agent_role: Option<String>,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            model: "claude-3-sonnet".to_string(),
            system_prompt: None,
            max_turns: 10,
            max_tokens: None,
            temperature: Some(0.7),
            max_token_budget: None,
            agent_name: Some("runner".to_string()),
            agent_role: Some("assistant".to_string()),
        }
    }
}

/// The core conversation loop engine
pub struct AgentRunner {
    config: RunnerConfig,
    tool_registry: Arc<ToolRegistry>,
    tool_executor: Arc<ToolExecutor>,
}

/// A single turn in the conversation
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub assistant_message: String,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: Value,
}

impl AgentRunner {
    pub fn new(
        config: RunnerConfig,
        tool_registry: Arc<ToolRegistry>,
        tool_executor: Arc<ToolExecutor>,
    ) -> Self {
        Self {
            config,
            tool_registry,
            tool_executor,
        }
    }

    /// Run a complete conversation
    pub async fn run(
        &self,
        initial_prompt: &str,
    ) -> Result<RunResult> {
        let mut conversation = vec![initial_prompt.to_string()];
        let mut total_usage = TokenUsage::default();
        let mut all_tool_calls: Vec<ToolCallRecord> = vec![];
        let mut turns = 0;

        loop {
            if turns >= self.config.max_turns {
                break;
            }
            turns += 1;

            // Step 1: Call LLM (simplified)
            let response = self.call_llm(&conversation).await?;
            
            // Step 2: Check for tool calls
            if response.tool_calls.is_empty() {
                // No tools - conversation complete
                return Ok(RunResult {
                    output: response.assistant_message,
                    tool_calls: all_tool_calls,
                    token_usage: total_usage,
                    turns,
                });
            }

            // Step 3: Execute tool calls in PARALLEL (open-multi-agent pattern)
            let tool_results = self.execute_tools_parallel(&response.tool_calls).await?;
            
            // Step 4: Append results to conversation
            conversation.push(response.assistant_message.clone());
            for result in &tool_results {
                conversation.push(format!("Tool result: {:?}", result));
            }

            all_tool_calls.extend(tool_results);
        }

        Ok(RunResult {
            output: conversation.last().cloned().unwrap_or_default(),
            tool_calls: all_tool_calls,
            token_usage: total_usage,
            turns,
        })
    }

    async fn call_llm(
        &self,
        _conversation: &[String],
    ) -> Result<ConversationTurn> {
        // Placeholder - would integrate with actual LLM
        Ok(ConversationTurn {
            assistant_message: "Processing complete.".to_string(),
            tool_calls: vec![],
        })
    }

    async fn execute_tools_parallel(
        &self,
        tool_calls: &[ToolCall],
    ) -> Result<Vec<ToolCallRecord>> {
        let context = ToolContext {
            agent_name: self.config.agent_name.clone().unwrap_or_default(),
            agent_role: self.config.agent_role.clone().unwrap_or_default(),
        };

        let mut handles = vec![];

        for call in tool_calls {
            let executor = self.tool_executor.clone();
            let call = call.clone();
            let ctx = context.clone();

            let handle = tokio::spawn(async move {
                let start = std::time::Instant::now();
                let result = executor.execute(&call.name, call.input.clone(), ctx).await;
                let duration = start.elapsed().as_millis() as u64;

                match result {
                    Ok(tool_result) => ToolCallRecord {
                        tool_name: call.name,
                        input: call.input,
                        output: tool_result.data,
                        duration_ms: duration,
                    },
                    Err(e) => ToolCallRecord {
                        tool_name: call.name,
                        input: call.input,
                        output: Value::String(e),
                        duration_ms: duration,
                    },
                }
            });

            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.map_err(|e| 
                CinemaError::Agent(e.to_string()))?);
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub output: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub token_usage: TokenUsage,
    pub turns: u32,
}
