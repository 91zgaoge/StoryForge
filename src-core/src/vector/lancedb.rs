use crate::error::{CinemaError, Result};
use crate::vector::document::{Document, DocumentMetadata};
use async_trait::async_trait;
use std::sync::Arc;

/// Vector store trait
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Add a document
    async fn add(&self,
        document: Document) -> Result<()>;

    /// Add multiple documents
    async fn add_batch(
        &self,
        documents: Vec<Document>,
    ) -> Result<()>;

    /// Search for similar documents
    async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
        filter: Option<DocumentFilter>,
    ) -> Result<Vec<SearchResult>>;

    /// Search by text (embeds query first)
    async fn search_text(
        &self,
        query: &str,
        limit: usize,
        filter: Option<DocumentFilter>,
    ) -> Result<Vec<SearchResult>>;

    /// Delete documents by filter
    async fn delete(&self,
        filter: DocumentFilter) -> Result<u64>;

    /// Get document by ID
    async fn get(&self,
        id: &str) -> Result<Option<Document>>;

    /// Count documents
    async fn count(&self) -> Result<u64>;
}

/// Filter for document queries
#[derive(Debug, Clone, Default)]
pub struct DocumentFilter {
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub chapter_range: Option<(u32, u32)>,
    pub min_importance: Option<f32>,
}

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
}

/// LanceDB implementation
pub struct LanceDbStore {
    /// Embedding dimension
    dimension: usize,
    /// In-memory storage (simplified - would use actual LanceDB in production)
    documents: Arc<tokio::sync::RwLock<Vec<Document>>>,
}

impl LanceDbStore {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            documents: Arc::new(tokio::sync::RwLock::new(vec![])),
        }
    }

    /// Cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot_product / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    /// Check if document matches filter
    fn matches_filter(doc: &Document, filter: &DocumentFilter) -> bool {
        if let Some(source_type) = &filter.source_type {
            if &doc.metadata.source_type != source_type {
                return false;
            }
        }

        if let Some(source_id) = &filter.source_id {
            if &doc.metadata.source_id != source_id {
                return false;
            }
        }

        if let Some((min_ch, max_ch)) = filter.chapter_range {
            if let Some(ch) = doc.metadata.chapter {
                if ch < min_ch || ch > max_ch {
                    return false;
                }
            }
        }

        if let Some(min_imp) = filter.min_importance {
            if doc.metadata.importance < min_imp {
                return false;
            }
        }

        true
    }
}

#[async_trait]
impl VectorStore for LanceDbStore {
    async fn add(&self,
        document: Document) -> Result<()> {
        if document.embedding.len() != self.dimension {
            return Err(CinemaError::Memory(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension, document.embedding.len()
            )));
        }

        let mut docs = self.documents.write().await;
        docs.push(document);
        Ok(())
    }

    async fn add_batch(&self,
        documents: Vec<Document>,
    ) -> Result<()> {
        for doc in &documents {
            if doc.embedding.len() != self.dimension {
                return Err(CinemaError::Memory(format!(
                    "Embedding dimension mismatch: expected {}, got {}",
                    self.dimension, doc.embedding.len()
                )));
            }
        }

        let mut docs = self.documents.write().await;
        docs.extend(documents);
        Ok(())
    }

    async fn search(&self,
        query_embedding: &[f32],
        limit: usize,
        filter: Option<DocumentFilter>,
    ) -> Result<Vec<SearchResult>> {
        let docs = self.documents.read().await;

        let mut results: Vec<SearchResult> = docs.iter()
            .filter(|doc| {
                filter.as_ref()
                    .map(|f| Self::matches_filter(doc, f))
                    .unwrap_or(true)
            })
            .map(|doc| {
                let score = Self::cosine_similarity(query_embedding, &doc.embedding);
                SearchResult {
                    document: doc.clone(),
                    score,
                }
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    async fn search_text(&self,
        _query: &str,
        _limit: usize,
        _filter: Option<DocumentFilter>,
    ) -> Result<Vec<SearchResult>> {
        // This would embed the query first using an embedding model
        // For now, return empty results
        Ok(vec![])
    }

    async fn delete(&self,
        filter: DocumentFilter) -> Result<u64> {
        let mut docs = self.documents.write().await;
        let original_len = docs.len();
        docs.retain(|doc| !Self::matches_filter(doc, &filter));
        let deleted = (original_len - docs.len()) as u64;
        Ok(deleted)
    }

    async fn get(&self,
        id: &str) -> Result<Option<Document>> {
        let docs = self.documents.read().await;
        Ok(docs.iter().find(|d| d.id == id).cloned())
    }

    async fn count(&self) -> Result<u64> {
        let docs = self.documents.read().await;
        Ok(docs.len() as u64)
    }
}
