//! Creative Engine - 智能化创作引擎
//!
//! 提供创作上下文构建、故事连续性管理、方法论驱动等核心能力。
//! 所有模块在幕后运行，为 Agent 提供真实、完整的创作上下文。

pub mod context_builder;
pub mod continuity;
pub mod foreshadowing;
pub mod methodology;
pub mod style;
pub mod adaptive;
pub mod workflow;

pub use context_builder::StoryContextBuilder;
