#![allow(dead_code)]
pub mod lancedb_store;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// 重新导出 SQLite 向量存储类型（LanceDB 兼容 API）
pub use lancedb_store::{LanceVectorStore, VectorRecord, SearchResult};

/// 纯Rust实现的向量存储 - 使用词频向量（保留作为fallback）
pub struct VectorStore {
    storage: Arc<Mutex<HashMap<String, Vec<EmbeddingRecord>>>>,
    vocab: Arc<Mutex<HashMap<String, usize>>>, // 词到索引的映射
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    pub id: String,
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_number: i32,
    pub text: String,
    pub embedding: Vec<f32>,
    pub record_type: RecordType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordType {
    ChapterSummary,
    KeyEvent,
    CharacterTrait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub record: EmbeddingRecord,
    pub score: f32,
}

impl VectorStore {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            vocab: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 提取中文词语（简化版：按字切分+双字词）
    fn extract_tokens(&self, text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut tokens = Vec::new();

        // 单字
        for ch in &chars {
            if !ch.is_ascii_punctuation() && !ch.is_whitespace() {
                tokens.push(ch.to_string());
            }
        }

        // 双字词
        for window in chars.windows(2) {
            let bigram: String = window.iter().collect();
            tokens.push(bigram);
        }

        tokens
    }

    /// 构建词频向量
    fn compute_embedding(
        &self,
        text: &str,
        vocab: &mut HashMap<String, usize>,
    ) -> Vec<f32> {
        let tokens = self.extract_tokens(text);
        let mut token_counts: HashMap<String, usize> = HashMap::new();

        // 统计词频
        for token in &tokens {
            *token_counts.entry(token.clone()).or_insert(0) += 1;
        }

        // 更新词汇表
        for token in token_counts.keys() {
            if !vocab.contains_key(token) {
                let idx = vocab.len();
                vocab.insert(token.clone(), idx);
            }
        }

        // 构建向量
        let mut vec = vec![0.0; vocab.len()];
        for (token, count) in token_counts {
            if let Some(&idx) = vocab.get(&token) {
                // TF-IDF 简化版：词频 * log(文档数/出现文档数)
                // 这里使用 sqrt 作为平滑
                vec[idx] = (count as f32).sqrt();
            }
        }

        // L2 归一化
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut vec {
                *x /= norm;
            }
        }

        vec
    }

    /// 计算余弦相似度
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let min_len = a.len().min(b.len());
        let dot_product: f32 = a[..min_len].iter().zip(&b[..min_len]).map(|(x, y)| x * y).sum();
        dot_product // 向量已归一化，所以只需要点积
    }

    /// 为章节创建嵌入向量
    pub fn embed_chapter(
        &self,
        story_id: String,
        chapter_id: String,
        chapter_number: i32,
        content: &str,
    ) -> Result<Vec<EmbeddingRecord>, Box<dyn std::error::Error>> {
        let chunks = self.chunk_text(content, 300); // 每段300字
        let mut records = Vec::new();
        let mut vocab = self.vocab.lock().unwrap();

        for (i, chunk) in chunks.iter().enumerate() {
            let embedding = self.compute_embedding(chunk, &mut vocab);

            records.push(EmbeddingRecord {
                id: format!("{}-{}", chapter_id, i),
                story_id: story_id.clone(),
                chapter_id: chapter_id.clone(),
                chapter_number,
                text: chunk.clone(),
                embedding,
                record_type: RecordType::ChapterSummary,
            });
        }

        let mut storage = self.storage.lock().unwrap();
        storage.insert(chapter_id, records.clone());

        Ok(records)
    }

    /// 相似度搜索
    pub fn search_similar(
        &self,
        story_id: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SimilarityResult>, Box<dyn std::error::Error>> {
        let vocab = self.vocab.lock().unwrap();
        let query_vec = self.compute_embedding_with_vocab(query, &vocab);
        drop(vocab); // 释放锁

        let storage = self.storage.lock().unwrap();
        let mut results = Vec::new();

        for records in storage.values() {
            for record in records {
                if record.story_id == story_id {
                    let score = Self::cosine_similarity(&query_vec, &record.embedding);
                    if score > 0.1 { // 过滤低相似度
                        results.push(SimilarityResult {
                            record: record.clone(),
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

    /// 使用固定词汇表计算嵌入
    fn compute_embedding_with_vocab(
        &self,
        text: &str,
        vocab: &HashMap<String, usize>,
    ) -> Vec<f32> {
        let tokens = self.extract_tokens(text);
        let mut token_counts: HashMap<String, usize> = HashMap::new();

        for token in &tokens {
            *token_counts.entry(token.clone()).or_insert(0) += 1;
        }

        let mut vec = vec![0.0; vocab.len()];
        for (token, count) in token_counts {
            if let Some(&idx) = vocab.get(&token) {
                vec[idx] = (count as f32).sqrt();
            }
        }

        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut vec {
                *x /= norm;
            }
        }

        vec
    }

    /// 文本分块
    fn chunk_text(&self, text: &str, chunk_size: usize) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();

        for chunk in chars.chunks(chunk_size) {
            chunks.push(chunk.iter().collect());
        }

        chunks
    }

    /// 删除章节的所有嵌入
    pub fn delete_chapter_embeddings(
        &self,
        chapter_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage = self.storage.lock().unwrap();
        storage.remove(chapter_id);
        Ok(())
    }

    /// 获取故事的所有向量记录
    pub fn get_story_embeddings(
        &self,
        story_id: &str,
    ) -> Vec<EmbeddingRecord> {
        let storage = self.storage.lock().unwrap();
        let mut results = Vec::new();

        for records in storage.values() {
            for record in records {
                if record.story_id == story_id {
                    results.push(record.clone());
                }
            }
        }

        results
    }
}