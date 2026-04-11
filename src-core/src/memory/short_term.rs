use std::collections::VecDeque;

pub struct ContextWindow {
    max_chapters: usize,
    buffer: VecDeque<ChapterContext>,
    max_tokens: usize,
    current_tokens: usize,
}

pub struct ChapterContext {
    pub chapter_number: u32,
    pub content: String,
    pub summary: String,
    pub token_count: usize,
}

impl ContextWindow {
    pub fn new(max_chapters: usize, max_tokens: usize) -> Self {
        Self {
            max_chapters,
            buffer: VecDeque::with_capacity(max_chapters),
            max_tokens,
            current_tokens: 0,
        }
    }
    
    pub fn add_chapter(&mut self,
        chapter: u32,
        content: String,
        summary: String,
    ) {
        let tokens = self.estimate_tokens(&content);
        
        while self.buffer.len() >= self.max_chapters 
        || self.current_tokens + tokens > self.max_tokens {
            if let Some(removed) = self.buffer.pop_front() {
                self.current_tokens -= removed.token_count;
            }
        }
        
        self.buffer.push_back(ChapterContext {
            chapter_number: chapter,
            content: content.clone(),
            summary,
            token_count: tokens,
        });
        self.current_tokens += tokens;
    }
    
    pub fn get_context(&self, query: &str) -> String {
        // Simple implementation - returns recent chapters
        self.buffer.iter()
            .map(|c| format!("Chapter {}: {}", c.chapter_number, c.summary))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    
    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 2
    }
}
