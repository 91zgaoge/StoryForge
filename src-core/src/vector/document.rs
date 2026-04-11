use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A document stored in the vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: DocumentMetadata,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    /// Source type (e.g., "chapter", "character", "world_rule")
    pub source_type: String,
    /// Source ID (e.g., chapter number, character ID)
    pub source_id: String,
    /// Chapter number if applicable
    pub chapter: Option<u32>,
    /// Relevance score (0-1)
    pub importance: f32,
    /// Additional custom fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl Document {
    pub fn new(
        id: impl Into<String>,
        content: impl Into<String>,
        embedding: Vec<f32>,
        source_type: impl Into<String>,
        source_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            embedding,
            metadata: DocumentMetadata {
                source_type: source_type.into(),
                source_id: source_id.into(),
                chapter: None,
                importance: 1.0,
                extra: std::collections::HashMap::new(),
            },
            created_at: Utc::now(),
        }
    }

    pub fn with_chapter(mut self, chapter: u32) -> Self {
        self.metadata.chapter = Some(chapter);
        self
    }

    pub fn with_importance(mut self, importance: f32) -> Self {
        self.metadata.importance = importance.clamp(0.0, 1.0);
        self
    }
}

/// Chunk of text with position info
#[derive(Debug, Clone)]
pub struct TextChunk {
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub chunk_index: usize,
}

/// Split text into overlapping chunks for embedding
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<TextChunk> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = vec![];
    let mut start = 0;
    let mut chunk_index = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk_words = &words[start..end];
        let chunk_text = chunk_words.join(" ");

        // Find byte positions
        let start_pos = text.find(chunk_words[0]).unwrap_or(0);
        let end_pos = start_pos + chunk_text.len();

        chunks.push(TextChunk {
            text: chunk_text,
            start_pos,
            end_pos,
            chunk_index,
        });

        // Move forward with overlap
        start += chunk_size - overlap;
        chunk_index += 1;

        if end >= words.len() {
            break;
        }
    }

    chunks
}
