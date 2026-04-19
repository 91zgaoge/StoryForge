//! Book Deconstruction Module
//!
//! 拆书功能：解析小说文件(txt/pdf/epub)，通过LLM智能分析提取
//! 小说类型、人物、世界观、大纲、故事线等结构化信息，保存到参考素材库。

pub mod models;
pub mod parser;
pub mod chunker;
pub mod analyzer;
pub mod repository;
pub mod service;
pub mod commands;

// Module exports
#[allow(unused_imports)]
pub use models::{BookAnalysisResult, BookAnalysisProgressEvent};
#[allow(unused_imports)]
pub use service::AnalysisStatusResponse;
