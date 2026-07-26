//! 基础设施 ports（端口）模块
//!
//! 定义 DB、LLM、Vector Store 等基础设施的中性 trait 契约，
//! 供业务模块通过依赖注入使用，消除全局单例。

pub mod db;
pub mod llm;
pub mod vector;

#[cfg(test)]
pub mod testing;

pub use db::*;
pub use llm::*;
#[cfg(test)]
pub use testing::{
    fake_runtime_contract, FakeLlmPort, FakeRuntimeContractProvider, FakeVectorStore,
};
pub use vector::*;
