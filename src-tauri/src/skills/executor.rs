#![allow(dead_code)]
use super::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct SkillExecutor {
    registry: Arc<Mutex<SkillRegistry>>,
}

impl SkillExecutor {
    pub fn new(registry: Arc<Mutex<SkillRegistry>>) -> Self {
        Self { registry }
    }
    
    /// Execute a skill
    pub fn execute(
        &self,
        skill_id: &str,
        context: &AgentContext,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<SkillResult, String> {
        let start = Instant::now();
        
        let skill = self.registry.lock()
            .unwrap()
            .get(skill_id)
            .ok_or_else(|| "Skill not found".to_string())?;
        
        if !skill.is_enabled {
            return Err("Skill is disabled".to_string());
        }
        
        // Validate parameters
        self.validate_params(&skill.manifest, &params)?;
        
        // Execute based on runtime
        let result = match &skill.runtime {
            SkillRuntime::Prompt(runtime) => {
                self.execute_prompt(runtime, context, params)
            }
            SkillRuntime::Mcp(runtime) => {
                self.execute_mcp(runtime, context, params)
            }
            SkillRuntime::Native(runtime) => {
                runtime.handler.execute(context, params)
                    .map_err(|e| e.to_string())
            }
        };
        
        let execution_time_ms = start.elapsed().as_millis() as u64;
        
        match result {
            Ok(mut r) => {
                r.execution_time_ms = execution_time_ms;
                Ok(r)
            }
            Err(e) => Ok(SkillResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(e),
                execution_time_ms,
            }),
        }
    }
    
    /// Execute hooks for an event
    pub fn execute_hooks(
        &self,
        event: HookEvent,
        context: &AgentContext,
        data: serde_json::Value,
    ) -> Vec<SkillResult> {
        let skills = self.registry.lock()
            .unwrap()
            .get_hook_handlers(&event);
        
        let mut results = Vec::new();
        
        for skill in skills {
            let params = HashMap::from([
                ("event_data".to_string(), data.clone()),
            ]);
            
            match self.execute(&skill.manifest.id,
                context,
                params,
            ) {
                Ok(result) => results.push(result),
                Err(e) => results.push(SkillResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(e),
                    execution_time_ms: 0,
                }),
            }
        }
        
        results
    }
    
    fn validate_params(
        &self,
        manifest: &SkillManifest,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        for param in &manifest.parameters {
            if param.required && !params.contains_key(&param.name) {
                if param.default.is_none() {
                    return Err(format!(
                        "Missing required parameter: {}",
                        param.name
                    ));
                }
            }
        }
        Ok(())
    }
    
    fn execute_prompt(
        &self,
        runtime: &PromptRuntime,
        context: &AgentContext,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<SkillResult, String> {
        // Build user prompt from template
        let mut user_prompt = runtime.user_prompt_template.clone();
        
        // Simple template substitution
        for (key, value) in &params {
            let placeholder = format!("{{{}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            user_prompt = user_prompt.replace(&placeholder, &value_str);
        }
        
        // Add context info
        let context_info = format!(
            "Story: {}\nGenre: {}\nTone: {}\nChapter: {}\n",
            context.story_title,
            context.genre,
            context.tone,
            context.chapter_number
        );
        
        user_prompt = format!("{}\n\n{}", context_info, user_prompt);
        
        // Note: Actual LLM call would happen here
        // For now, return the prompt as result
        Ok(SkillResult {
            success: true,
            data: serde_json::json!({
                "system_prompt": runtime.system_prompt,
                "user_prompt": user_prompt,
            }),
            error: None,
            execution_time_ms: 0,
        })
    }
    
    fn execute_mcp(
        &self,
        runtime: &McpRuntime,
        _context: &AgentContext,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<SkillResult, String> {
        // MCP execution would connect to MCP server
        // For now, return the config as result
        Ok(SkillResult {
            success: true,
            data: serde_json::json!({
                "server_command": runtime.server_config.command,
                "args": runtime.server_config.args,
                "params": params,
            }),
            error: None,
            execution_time_ms: 0,
        })
    }
}
