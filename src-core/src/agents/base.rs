use async_trait::async_trait;
use crate::error::Result;

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    async fn think(&self, context: &str) -> Result<String>;
    async fn act(&self, decision: &str) -> Result<String>;
    async fn reflect(&self, action_result: &str) -> Result<String>;
}
