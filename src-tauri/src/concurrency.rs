use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::Semaphore;

/// 全局后台 LLM 任务信号量，用于避免后台 ingest/audit/insight
/// 任务并发压垮模型。
pub static BACKGROUND_LLM_SEMAPHORE: Lazy<Arc<Semaphore>> =
    Lazy::new(|| Arc::new(Semaphore::new(1)));
