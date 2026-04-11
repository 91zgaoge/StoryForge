//! Text Embedding Module
//!
//! Provides text embedding using local feature extraction.
//! TODO: Upgrade to fastembed or onnxruntime for real embeddings.

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static EMBEDDING_INITIALIZED: OnceCell<bool> = OnceCell::new();
static mut VOCAB: Option<HashMap<String, usize>> = None;

/// Embedding representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: String,
    pub vector: Vec<f32>,
    pub dimensions: usize,
    pub model: String,
}

/// 初始化嵌入模型
pub fn init_embedding_model() -> Result<(), Box<dyn std::error::Error>> {
    EMBEDDING_INITIALIZED.set(true)
        .map_err(|_| "Already initialized")?;

    unsafe {
        VOCAB = Some(HashMap::new());
    }

    log::info!("Embedding module initialized (384-dim feature vectors)");
    Ok(())
}

/// 分词 - 支持中文和英文
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.to_lowercase().chars().collect();

    // 提取单字/单字符
    for ch in &chars {
        if ch.is_alphanumeric() || ch.is_ascii_punctuation() {
            tokens.push(ch.to_string());
        }
    }

    // 提取双字词/bigrams
    for window in chars.windows(2) {
        let bigram: String = window.iter().collect();
        if bigram.chars().any(|c| c.is_alphabetic() || c.is_numeric()) {
            tokens.push(bigram);
        }
    }

    // 提取单词 (英文)
    let word_chars: String = text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '\'' { c } else { ' ' })
        .collect();

    for word in word_chars.split_whitespace() {
        if word.len() > 2 {
            tokens.push(word.to_string());
        }
    }

    tokens
}

/// 基于词频的嵌入 (改进版TF特征)
pub fn embed_text(text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    const DIM: usize = 384;
    let mut features = vec![0.0f32; DIM];

    if text.is_empty() {
        return Ok(features);
    }

    let tokens = tokenize(text);

    if tokens.is_empty() {
        return Ok(features);
    }

    // 统计词频
    let mut token_counts: HashMap<String, usize> = HashMap::new();
    for token in &tokens {
        *token_counts.entry(token.clone()).or_insert(0) += 1;
    }

    // 使用哈希将词映射到固定维度
    for (token, count) in token_counts {
        // 使用FNV-1a哈希
        let hash = fnv1a_hash(&token);
        let idx = (hash % DIM as u64) as usize;
        let tf = 1.0 + (count as f32).ln().max(0.0); // log normalization
        features[idx] += tf;
    }

    // 添加位置编码信息
    let text_len = text.len().min(DIM);
    for i in 0..text_len.min(64) {
        features[DIM - 64 + i] = (text.chars().nth(i).unwrap_or(' ') as u32 as f32) / 65535.0;
    }

    // L2 归一化
    let norm: f32 = features.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-6 {
        for x in &mut features {
            *x /= norm;
        }
    }

    Ok(features)
}

/// FNV-1a 哈希函数
fn fnv1a_hash(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// 批量生成文本嵌入
pub fn embed_texts(texts: Vec<String>) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    texts.iter().map(|t| embed_text(t)).collect()
}

/// 计算余弦相似度
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let min_len = a.len().min(b.len());
    let dot_product: f32 = a[..min_len].iter().zip(&b[..min_len]).map(|(x, y)| x * y).sum();
    dot_product // 向量已归一化
}

/// 获取嵌入维度
pub fn embedding_dim() -> usize {
    384
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_text() {
        let vec1 = embed_text("Hello world").unwrap();
        let vec2 = embed_text("Hello world").unwrap();
        let vec3 = embed_text("Goodbye world").unwrap();

        assert_eq!(vec1.len(), 384);

        // Same text should produce same embedding
        assert!(cosine_similarity(&vec1, &vec2) > 0.99);

        // Different text should have lower similarity
        let sim = cosine_similarity(&vec1, &vec3);
        assert!(sim < 0.9);
    }
}
