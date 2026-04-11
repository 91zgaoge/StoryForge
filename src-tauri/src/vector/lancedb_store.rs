//! Vector Store Module
//!
//! LanceDB implementation for persistent vector storage.
//! Note: Using memory-based storage until Rust is upgraded to 1.88+

// use lancedb::{connect, Table, TableRef};
// use arrow_array::{Float32Array, Int32Array, StringArray, RecordBatch};
// use arrow_schema::{DataType, Field, Schema};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;

/// 向量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_number: i32,
    pub text: String,
    pub record_type: String,
    pub embedding: Vec<f32>,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_number: i32,
    pub text: String,
    pub score: f32,
}

/// LanceDB 向量存储 (内存实现，API与LanceDB兼容)
pub struct LanceVectorStore {
    _db_path: String,
    storage: Arc<Mutex<HashMap<String, Vec<VectorRecord>>>>,
}

impl LanceVectorStore {
    pub fn new(db_path: String) -> Self {
        Self {
            _db_path: db_path,
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Vector store initialized (memory mode, LanceDB compatible)");
        Ok(())
    }

    pub async fn add_record(
        &self,
        record: VectorRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage = self.storage.lock().unwrap();
        storage.entry(record.chapter_id.clone()).or_insert_with(Vec::new).push(record);
        Ok(())
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let min_len = a.len().min(b.len());
        let dot_product: f32 = a[..min_len].iter().zip(&b[..min_len]).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a > 0.0 && norm_b > 0.0 {
            dot_product / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    pub async fn search(
        &self,
        story_id: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let query_vec = crate::embeddings::embed_text(query)?;
        let storage = self.storage.lock().unwrap();
        let mut results = Vec::new();

        for records in storage.values() {
            for record in records {
                if record.story_id == story_id {
                    let score = Self::cosine_similarity(&query_vec, &record.embedding);
                    if score > 0.1 {
                        results.push(SearchResult {
                            id: record.id.clone(),
                            story_id: record.story_id.clone(),
                            chapter_id: record.chapter_id.clone(),
                            chapter_number: record.chapter_number,
                            text: record.text.clone(),
                            score,
                        });
                    }
                }
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(top_k);
        Ok(results)
    }

    pub async fn delete_chapter(
        &self,
        chapter_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage = self.storage.lock().unwrap();
        storage.remove(chapter_id);
        Ok(())
    }
}
