use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> Value;
    async fn execute(&self, input: Value) -> Result<Value, String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn to_tool_defs(&self) -> Vec<ToolDefinition> {
        self.tools.values()
            .map(|t| ToolDefinition {
                name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.schema(),
            })
            .collect()
    }
}

pub fn define_tool<F, Fut>(
    name: &str,
    description: &str,
    schema: Value,
    handler: F,
) -> Arc<dyn Tool>
where
    F: Fn(Value) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Value, String>> + Send,
{
    struct DefinedTool {
        name: String,
        description: String,
        schema: Value,
        handler: Box<dyn Fn(Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>> + Send + Sync>,
    }

    #[async_trait]
    impl Tool for DefinedTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn schema(&self) -> Value {
            self.schema.clone()
        }

        async fn execute(&self, input: Value) -> Result<Value, String> {
            (self.handler)(input).await
        }
    }

    Arc::new(DefinedTool {
        name: name.to_string(),
        description: description.to_string(),
        schema,
        handler: Box::new(move |input| Box::new(handler(input))),
    })
}
