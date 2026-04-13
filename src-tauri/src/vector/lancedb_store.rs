//! Vector Store Module
//!
//! LanceDB-compatible API with JSON file persistence.
//! Records are stored in memory and automatically persisted to disk on every write.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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

/// LanceDB 向量存储 (内存实现 + JSON 持久化)
pub struct LanceVectorStore {
    db_path: PathBuf,
    storage: Arc<Mutex<HashMap<String, Vec<VectorRecord>>>>, // chapter_id -> records
}

impl LanceVectorStore {
    pub fn new(db_path: String) -> Self {
        Self {
            db_path: PathBuf::from(db_path),
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn records_file(&self) -> PathBuf {
        self.db_path.join("records.json")
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure directory exists
        tokio::fs::create_dir_all(&self.db_path).await?;
        self.load().await?;
        log::info!("Vector store initialized with persistence at {:?}", self.db_path);
        Ok(())
    }

    async fn load(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.records_file();
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let data: HashMap<String, Vec<VectorRecord>> = serde_json::from_str(&content)?;
            let mut storage = self.storage.lock().unwrap();
            *storage = data;
            log::info!("Loaded {} chapters from vector store", storage.len());
        }
        Ok(())
    }

    async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.records_file();
        let content = {
            let storage = self.storage.lock().unwrap();
            serde_json::to_string_pretty(&*storage)?
        };
        tokio::fs::write(&path, content).await?;
        Ok(())
    }

    /// Upsert a record (update if exists, insert if not)
    pub async fn upsert(&self, record: VectorRecord) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut storage = self.storage.lock().unwrap();
            let records = storage.entry(record.chapter_id.clone()).or_insert_with(Vec::new);
            records.retain(|r| r.id != record.id);
            records.push(record);
        }
        self.save().await
    }

    pub async fn add_record(&self, record: VectorRecord) -> Result<(), Box<dyn std::error::Error>> {
        self.upsert(record).await
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
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let storage = self.storage.lock().unwrap();
        let mut results = Vec::new();

        for records in storage.values() {
            for record in records {
                if record.story_id == story_id {
                    let score = Self::cosine_similarity(&query_embedding, &record.embedding);
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

    pub async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut storage = self.storage.lock().unwrap();
            for records in storage.values_mut() {
                records.retain(|r| r.id != id);
            }
            // Clean up empty chapters
            storage.retain(|_, records| !records.is_empty());
        }
        self.save().await
    }

    pub async fn delete_chapter(&self, chapter_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut storage = self.storage.lock().unwrap();
            storage.remove(chapter_id);
        }
        self.save().await
    }

    pub async fn count(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let storage = self.storage.lock().unwrap();
        let count: usize = storage.values().map(|v| v.len()).sum();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_record(id: &str, story_id: &str, chapter_id: &str) -> VectorRecord {
        VectorRecord {
            id: id.to_string(),
            story_id: story_id.to_string(),
            chapter_id: chapter_id.to_string(),
            chapter_number: 1,
            text: "测试文本".to_string(),
            record_type: "chapter".to_string(),
            embedding: vec![0.1, 0.2, 0.3, 0.4],
        }
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = env::temp_dir().join(format!("storyforge_vector_test_{}", uuid::Uuid::new_v4()));
        let db_path = temp_dir.to_string_lossy().to_string();

        // Phase 1: Create store, add records, and destroy it
        {
            let mut store = LanceVectorStore::new(db_path.clone());
            store.init().await.unwrap();

            let record = create_test_record("r1", "story_1", "chap_1");
            store.add_record(record).await.unwrap();

            let record2 = create_test_record("r2", "story_1", "chap_2");
            store.add_record(record2).await.unwrap();

            assert_eq!(store.count().await.unwrap(), 2);

            let results = store.search("story_1", vec![0.1, 0.2, 0.3, 0.4], 5).await.unwrap();
            assert_eq!(results.len(), 2);
        }

        // Phase 2: Create a new store instance with the same path
        {
            let mut store = LanceVectorStore::new(db_path.clone());
            store.init().await.unwrap();

            assert_eq!(store.count().await.unwrap(), 2);

            let results = store.search("story_1", vec![0.1, 0.2, 0.3, 0.4], 5).await.unwrap();
            assert_eq!(results.len(), 2);

            // Test delete persists
            store.delete("r1").await.unwrap();
            assert_eq!(store.count().await.unwrap(), 1);
        }

        // Phase 3: Verify delete was persisted
        {
            let mut store = LanceVectorStore::new(db_path.clone());
            store.init().await.unwrap();
            assert_eq!(store.count().await.unwrap(), 1);

            let results = store.search("story_1", vec![0.1, 0.2, 0.3, 0.4], 5).await.unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, "r2");
        }

        // Cleanup
        tokio::fs::remove_dir_all(&temp_dir).await.ok();
    }
}
