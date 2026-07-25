//! LLM Service port

use crate::{
    config::settings::{LlmProfile, ModelRole},
    error::AppError,
    llm::{
        adapter::{GenerateResponse, ResponseFormat},
        service::PipelineContext,
    },
    router::{Complexity, Priority, TaskType},
};

/// LLM 服务端口
///
/// 定义最常用的 LLM 生成能力，供业务模块通过依赖注入使用。
/// 需要完整方法集的场景可直接依赖 `crate::llm::service::LlmService` 具体类型。
#[async_trait::async_trait]
pub trait LlmService: Send + Sync + 'static {
    /// 使用当前活跃 profile 同步生成
    async fn generate(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<GenerateResponse, AppError>;

    /// 使用当前活跃 profile 同步生成，带上下文标签
    async fn generate_with_context(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
        context_label: Option<&str>,
    ) -> Result<GenerateResponse, AppError>;

    /// 使用当前活跃 profile 同步生成，返回 (request_id, Result)
    async fn generate_with_request_id(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
        context_label: Option<&str>,
        pipeline_ctx: Option<PipelineContext>,
        request_id: Option<String>,
    ) -> (String, Result<GenerateResponse, AppError>);

    /// 使用指定 profile 同步生成
    async fn generate_with_profile(
        &self,
        profile_id: &str,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<GenerateResponse, AppError>;

    /// 流式生成
    async fn generate_stream(
        &self,
        request_id: String,
        prompt: String,
        context: Option<String>,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<(), AppError>;

    /// 检查指定 request_id 是否已被取消
    fn is_cancelled(&self, request_id: &str) -> bool;

    /// 测试当前活跃模型连接
    async fn test_connection(&self) -> Result<(bool, u64), AppError>;

    /// 获取当前活跃模型配置
    fn get_active_profile(&self) -> Option<LlmProfile>;
}

/// v0.30.2: 模型网关中性请求 —— 由 `LlmService` 构造后交给 `LlmPort` 执行，
/// 避免 `llm` 模块直接依赖 `model_gateway::types::GatewayRequest`。
#[derive(Debug, Clone)]
pub struct LlmPortRequest {
    /// 原始 prompt
    pub prompt: String,
    /// 发起调用的 Agent 或模块标识
    pub agent_id: String,
    /// 任务类型
    pub task: TaskType,
    /// 复杂度
    pub complexity: Complexity,
    /// 成本优先级
    pub budget_priority: Priority,
    /// 速度优先级
    pub speed_priority: Priority,
    /// 估计输入 token 数
    pub estimated_input_tokens: u32,
    /// 期望最大输出 token 数
    pub max_tokens: Option<i32>,
    /// 温度
    pub temperature: Option<f32>,
    /// 请求 ID（用于日志和取消）
    pub request_id: String,
    /// 上下文标签（可选）
    pub context_label: Option<String>,
    /// 超时覆盖（秒，可选）
    pub timeout_seconds_override: Option<u64>,
    /// 最大重试覆盖（可选）
    pub max_retries_override: Option<u32>,
    /// SING 意图动词
    pub intent_verb: Option<String>,
    /// SING 意图宾语
    pub intent_object: Option<String>,
    /// 意图图发现的资产标签
    pub asset_tags: Vec<String>,
    /// 意图图发现的具体资产 ID 列表
    pub discovered_asset_ids: Vec<String>,
    /// 结构化输出格式
    pub response_format: Option<ResponseFormat>,
    /// 请求级 system_prompt
    pub system_prompt: Option<String>,
    /// 模型角色偏好
    pub model_role: Option<ModelRole>,
    /// 生成链路 trace_id
    pub trace_id: Option<String>,
}

/// v0.30.2: 模型网关端口 —— `GatewayExecutor` 实现此 trait，
/// `llm::service::LlmService` 只依赖 `Arc<dyn LlmPort>` 而非具体类型，
/// 从而打破 `llm` 与 `model_gateway` 之间的循环依赖。
#[async_trait::async_trait]
pub trait LlmPort: Send + Sync + 'static {
    /// 统一生成入口：选择候选链并顺序执行 fallback
    async fn generate(&self, request: LlmPortRequest) -> Result<GenerateResponse, AppError>;

    /// 选取「最快可用模型」profile，用于 TriShot Call 1 路由合成器。
    fn select_fastest_profile(&self) -> Option<LlmProfile>;

    /// 健康数据是否新鲜（<15s 前探测过）。
    fn is_health_fresh(&self, model_id: &str) -> bool;

    /// 标记模型为 Unhealthy。
    fn mark_unhealthy(&self, model_id: &str, model_name: &str, error: Option<String>);

    /// 记录模型成功，重置连续失败计数。
    fn record_success(&self, model_id: &str, model_name: &str);
}

/// 占位端口：在启动时 `GatewayExecutor` 尚未构造前，
/// `LlmService` 使用此端口安全地失败并回退到本地路由。
pub struct NoOpLlmPort;

#[async_trait::async_trait]
impl LlmPort for NoOpLlmPort {
    async fn generate(&self, _request: LlmPortRequest) -> Result<GenerateResponse, AppError> {
        Err(AppError::Internal {
            message: "LlmPort not initialized".to_string(),
        })
    }

    fn select_fastest_profile(&self) -> Option<LlmProfile> {
        None
    }

    fn is_health_fresh(&self, _model_id: &str) -> bool {
        false
    }

    fn mark_unhealthy(&self, _model_id: &str, _model_name: &str, _error: Option<String>) {}

    fn record_success(&self, _model_id: &str, _model_name: &str) {}
}
