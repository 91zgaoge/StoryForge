//! 指导书提炼模块：上传故事创作指导书 → LLM 提炼为自定义创作方法论资产

pub mod models;
pub mod repository;

pub use models::*;
pub use repository::{CustomMethodologyRepository, GuidebookRepository};
