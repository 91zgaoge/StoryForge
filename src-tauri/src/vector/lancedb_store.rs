//! Vector Store Module
//!
//! SQLite-backed vector storage with LanceDB-compatible API.
//! Replaces the previous JSON-memory fallback with true persistent storage.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 搜索缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct SearchCacheKey {
    story_id: String,
    query_hash: u64,
    top_k: usize,
}

/// 搜索缓存项
struct SearchCacheEntry {
    results: Vec<SearchResult>,
    inserted_at: Instant,
}

const CACHE_TTL: Duration = Duration::from_secs(300); // 5分钟
const MAX_CACHE_SIZE: usize = 100;

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
    search_cache: Arc<Mutex<HashMap<SearchCacheKey, SearchCacheEntry>>>,
}

impl LanceVectorStore {
    pub fn new(db_path: String) -> Self {
        let path = PathBuf::from(db_path);
        // Use a placeholder connection; real connection is established in init()
        let conn = Connection::open_in_memory().expect("Failed to create in-memory connection");
        Self {
            db_path: path,
            conn: Arc::new(Mutex::new(conn)),
            search_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn compute_query_hash(query_embedding: &[f32]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        // 采样前16个维度进行哈希，平衡精度与性能
        let sample: Vec<i32> = query_embedding.iter().take(16).map(|&f| (f * 1000.0) as i32).collect();
        sample.hash(&mut hasher);
        hasher.finish()
    }

    fn invalidate_cache(&self, story_id: &str) {
        let mut cache = self.search_cache.lock().unwrap();
        cache.retain(|key, _| key.story_id != story_id);
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

            -- FTS5 全文索引用于语义搜索优化
            CREATE VIRTUAL TABLE IF NOT EXISTS vector_records_fts USING fts5(
                text,
                content='vector_records',
                content_rowid='rowid'
            );

            -- 同步触发器：插入
            CREATE TRIGGER IF NOT EXISTS vector_records_fts_insert
            AFTER INSERT ON vector_records BEGIN
                INSERT INTO vector_records_fts(rowid, text) VALUES (new.rowid, new.text);
            END;

            -- 同步触发器：删除
            CREATE TRIGGER IF NOT EXISTS vector_records_fts_delete
            AFTER DELETE ON vector_records BEGIN
                INSERT INTO vector_records_fts(vector_records_fts, rowid, text)
                VALUES ('delete', old.rowid, old.text);
            END;

            -- 同步触发器：更新
            CREATE TRIGGER IF NOT EXISTS vector_records_fts_update
            AFTER UPDATE ON vector_records BEGIN
                INSERT INTO vector_records_fts(vector_records_fts, rowid, text)
                VALUES ('delete', old.rowid, old.text);
                INSERT INTO vector_records_fts(rowid, text) VALUES (new.rowid, new.text);
            END;
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

        drop(conn);
        self.invalidate_cache(&record.story_id);
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
        // 1. 检查缓存
        let cache_key = SearchCacheKey {
            story_id: story_id.to_string(),
            query_hash: Self::compute_query_hash(&query_embedding),
            top_k,
        };
        {
            let cache = self.search_cache.lock().unwrap();
            if let Some(entry) = cache.get(&cache_key) {
                if entry.inserted_at.elapsed() < CACHE_TTL {
                    return Ok(entry.results.clone());
                }
            }
        }

        // 2. 执行搜索（语义搜索优化：维度预过滤 + 提前终止低分）
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

        let query_dim = query_embedding.len();
        let mut results = Vec::new();
        for row in rows {
            let (id, sid, cid, cnum, text, embedding) = row?;
            // 维度预过滤：跳过维度不匹配的向量
            if embedding.len() != query_dim {
                continue;
            }
            let score = Self::cosine_similarity(&query_embedding, &embedding);
            // 提前终止阈值
            if score > 0.05 {
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

        // 3. 写入缓存
        {
            let mut cache = self.search_cache.lock().unwrap();
            if cache.len() >= MAX_CACHE_SIZE {
                // 简单LRU：删除最旧的20%条目
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| a.1.inserted_at.cmp(&b.1.inserted_at));
                let to_remove = entries.len() / 5;
                let keys_to_remove: Vec<_> = entries.into_iter().take(to_remove).map(|(k, _)| k.clone()).collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
            cache.insert(cache_key, SearchCacheEntry {
                results: results.clone(),
                inserted_at: Instant::now(),
            });
        }

        Ok(results)
    }

    /// 基于 FTS5 的全文关键词搜索（BM25 排序）
    pub async fn text_search(
        &self,
        story_id: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT v.id, v.story_id, v.chapter_id, v.chapter_number, v.text, rank
             FROM vector_records_fts f
             JOIN vector_records v ON v.rowid = f.rowid
             WHERE f.text MATCH ?1 AND v.story_id = ?2
             ORDER BY rank ASC
             LIMIT ?3"
        )?;

        let mut results = Vec::new();
        let rows = stmt.query_map(params![query, story_id, top_k as i32], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, f64>(5)?,
            ))
        })?;

        for row in rows {
            let (id, sid, cid, cnum, text, rank) = row?;
            results.push(SearchResult {
                id,
                story_id: sid,
                chapter_id: cid,
                chapter_number: cnum,
                text,
                score: rank as f32,
            });
        }

        // 将 rank 归一化为 0-1 分数（越低越好 -> 越高越好）
        if !results.is_empty() {
            let min_rank = results.iter().map(|r| r.score).fold(f32::INFINITY, |a, b| a.min(b));
            let max_rank = results.iter().map(|r| r.score).fold(f32::NEG_INFINITY, |a, b| a.max(b));
            for r in &mut results {
                r.score = if max_rank > min_rank {
                    (max_rank - r.score) / (max_rank - min_rank)
                } else {
                    1.0
                };
            }
        }

        Ok(results)
    }

    /// 混合搜索：向量相似度 + FTS5 全文搜索，使用 RRF 融合
    pub async fn hybrid_search(
        &self,
        story_id: &str,
        query_text: &str,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        const RRF_K: f32 = 60.0;

        let vector_results = self.search(story_id, query_embedding, top_k * 2).await?;
        let text_results = self.text_search(story_id, query_text, top_k * 2).await?;

        use std::collections::HashMap;
        let mut scores: HashMap<String, f32> = HashMap::new();

        for (rank, r) in vector_results.iter().enumerate() {
            let score = 1.0 / (RRF_K + rank as f32 + 1.0);
            *scores.entry(r.id.clone()).or_insert(0.0) += score;
        }

        for (rank, r) in text_results.iter().enumerate() {
            let score = 1.0 / (RRF_K + rank as f32 + 1.0);
            *scores.entry(r.id.clone()).or_insert(0.0) += score;
        }

        let mut all_results: HashMap<String, SearchResult> = HashMap::new();
        for r in vector_results.into_iter().chain(text_results.into_iter()) {
            all_results.entry(r.id.clone()).or_insert(r);
        }

        let mut fused: Vec<SearchResult> = all_results.into_iter().map(|(id, mut r)| {
            r.score = scores.get(&id).copied().unwrap_or(0.0);
            r
        }).collect();

        fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        fused.truncate(top_k);

        Ok(fused)
    }

    pub async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        // 获取 story_id 以清空对应缓存
        let story_id: Option<String> = conn.query_row(
            "SELECT story_id FROM vector_records WHERE id = ?1",
            [id],
            |row| row.get(0),
        ).optional().ok().flatten();
        conn.execute("DELETE FROM vector_records WHERE id = ?1", [id])?;
        drop(conn);
        if let Some(sid) = story_id {
            self.invalidate_cache(&sid);
        }
        Ok(())
    }

    pub async fn delete_chapter(&self, chapter_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let story_ids: Vec<String> = conn.prepare(
            "SELECT DISTINCT story_id FROM vector_records WHERE chapter_id = ?1"
        )?.query_map([chapter_id], |row| row.get(0))?
         .collect::<Result<Vec<_>, _>>()?;
        conn.execute("DELETE FROM vector_records WHERE chapter_id = ?1", [chapter_id])?;
        drop(conn);
        for sid in story_ids {
            self.invalidate_cache(&sid);
        }
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
