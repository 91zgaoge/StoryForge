use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Markdown,
    PlainText,
    Json,
    Html,
    Pdf,
    Epub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub include_outline: bool,
    pub include_metadata: bool,
    pub chapter_separator: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Markdown,
            include_outline: true,
            include_metadata: true,
            chapter_separator: "\n\n---\n\n".to_string(),
        }
    }
}

pub struct StoryExporter;

impl StoryExporter {
    pub fn new() -> Self {
        Self
    }

    pub fn export_to_file(
        &self,
        _story: &crate::db::Story,
        _chapters: &[crate::db::Chapter],
        _characters: &[crate::db::Character],
        config: &ExportConfig,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match config.format {
            ExportFormat::Pdf => {
                pdf::generate_pdf(_story, _chapters, _characters, config, output_path)?;
                Ok(())
            }
            ExportFormat::Epub => {
                epub::generate_epub(_story, _chapters, _characters, config, output_path)?;
                Ok(())
            }
            _ => {
                let content = "Export content placeholder".to_string();
                fs::write(output_path, content)?;
                Ok(())
            }
        }
    }
}

pub struct StoryImporter;

impl StoryImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn import_from_text(
        &self,
        _content: &str,
        story_title: &str,
    ) -> Result<(crate::db::CreateStoryRequest, Vec<ImportChapter>), Box<dyn std::error::Error>> {
        let story_req = crate::db::CreateStoryRequest {
            title: story_title.to_string(),
            description: None,
            genre: None,
        };
        Ok((story_req, vec![]))
    }
}

#[derive(Debug, Clone)]
pub struct ImportChapter {
    pub chapter_number: i32,
    pub title: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportResult {
    pub file_path: String,
    pub content: String,
    pub format: String,
}

pub mod pdf;
pub mod epub;