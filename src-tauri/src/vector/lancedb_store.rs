//! Vector Store Module
//!
//! SQLite-backed vector storage with LanceDB-compatible API.
//! Replaces the previous JSON-memory fallback with true persistent storage.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
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

/// SQLite 向量存储 (LanceDB 兼容 API)
pub struct LanceVectorStore {
    db_path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

impl LanceVectorStore {
    pub fn new(db_path: String) -> Self {
        let path = PathBuf::from(db_path);
        // Use a placeholder connection; real connection is established in init()
        let conn = Connection::open_in_memory().expect("Failed to create in-memory connection");
        Self {
            db_path: path,
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    fn db_file(&self) -> PathBuf {
        self.db_path.join("vector_store.db")
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::fs::create_dir_all(&self.db_path).await?;
        let db_file = self.db_file();
        let conn = Connection::open(&db_file)?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS vector_records (
                id TEXT PRIMARY KEY,
                story_id TEXT NOT NULL,
                chapter_id TEXT NOT NULL,
                chapter_number INTEGER NOT NULL,
                text TEXT NOT NULL,
                record_type TEXT NOT NULL,
                embedding TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_vector_records_story ON vector_records(story_id);
            CREATE INDEX IF NOT EXISTS idx_vector_records_chapter ON vector_records(chapter_id);
            "#,
        )?;

        self.conn = Arc::new(Mutex::new(conn));
        log::info!("SQLite vector store initialized at {:?}", db_file);
        Ok(())
    }

    /// Upsert a record (update if exists, insert if not)
    pub async fn upsert(&self, record: VectorRecord) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let embedding_json = serde_json::to_string(&record.embedding)?;

        conn.execute(
            "INSERT INTO vector_records (id, story_id, chapter_id, chapter_number, text, record_type, embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 story_id = excluded.story_id,
                 chapter_id = excluded.chapter_id,
                 chapter_number = excluded.chapter_number,
                 text = excluded.text,
                 record_type = excluded.record_type,
                 embedding = excluded.embedding",
            params![
                &record.id,
                &record.story_id,
                &record.chapter_id,
                record.chapter_number,
                &record.text,
                &record.record_type,
                embedding_json
            ],
        )?;

        Ok(())
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
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, story_id, chapter_id, chapter_number, text, embedding
             FROM vector_records WHERE story_id = ?1"
        )?;

        let rows = stmt.query_map([story_id], |row| {
            let embedding_json: String = row.get(5)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json).unwrap_or_default();
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
                embedding,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (id, sid, cid, cnum, text, embedding) = row?;
            let score = Self::cosine_similarity(&query_embedding, &embedding);
            if score > 0.1 {
                results.push(SearchResult {
                    id,
                    story_id: sid,
                    chapter_id: cid,
                    chapter_number: cnum,
                    text,
                    score,
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(top_k);
        Ok(results)
    }

    pub async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM vector_records WHERE id = ?1", [id])?;
        Ok(())
    }

    pub async fn delete_chapter(&self, chapter_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM vector_records WHERE chapter_id = ?1", [chapter_id])?;
        Ok(())
    }

    pub async fn count(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vector_records",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
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
