use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 章节版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterVersion {
    pub id: String,
    pub chapter_id: String,
    pub version_number: i32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub outline: Option<String>,
    pub word_count: i32,
    pub created_by: String, // user_id or agent_name
    pub created_at: DateTime<Utc>,
    pub change_summary: String,
    pub diff_stats: DiffStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub added_chars: usize,
    pub removed_chars: usize,
    pub modified_percentage: f32,
}

/// 版本管理器
pub struct VersionManager {
    versions: HashMap<String, Vec<ChapterVersion>>, // chapter_id -> versions
}

impl VersionManager {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    /// 创建新版本
    pub fn create_version(
        &mut self,
        chapter_id: String,
        title: Option<String>,
        content: Option<String>,
        outline: Option<String>,
        word_count: i32,
        created_by: String,
        change_summary: String,
    ) -> ChapterVersion {
        let versions = self.versions.entry(chapter_id.clone()).or_default();

        let version_number = versions.len() as i32 + 1;

        // 计算diff统计
        let diff_stats = if let Some(prev) = versions.last() {
            self.calculate_diff(prev, content.as_deref().unwrap_or(""))
        } else {
            DiffStats {
                added_chars: content.as_ref().map(|c| c.len()).unwrap_or(0),
                removed_chars: 0,
                modified_percentage: 100.0,
            }
        };

        let version = ChapterVersion {
            id: uuid::Uuid::new_v4().to_string(),
            chapter_id,
            version_number,
            title,
            content,
            outline,
            word_count,
            created_by,
            created_at: Utc::now(),
            change_summary,
            diff_stats,
        };

        versions.push(version.clone());
        version
    }

    /// 获取章节的所有版本
    pub fn get_versions(&self, chapter_id: &str) -> Vec<&ChapterVersion> {
        self.versions
            .get(chapter_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// 获取特定版本
    pub fn get_version(&self, chapter_id: &str, version_id: &str) -> Option<&ChapterVersion> {
        self.versions
            .get(chapter_id)
            .and_then(|versions| versions.iter().find(|v| v.id == version_id))
    }

    /// 回滚到指定版本
    pub fn rollback_to_version(
        &mut self,
        chapter_id: &str,
        version_id: &str,
    ) -> Result<ChapterVersion, String> {
        let target = self
            .get_version(chapter_id, version_id)
            .cloned()
            .ok_or("Version not found")?;

        // 创建一个新的回滚版本
        let rollback = self.create_version(
            target.chapter_id,
            target.title.clone(),
            target.content.clone(),
            target.outline.clone(),
            target.word_count,
            "system".to_string(),
            format!("Rollback to version {}", target.version_number),
        );

        Ok(rollback)
    }

    /// 比较两个版本
    pub fn compare_versions(
        &self,
        chapter_id: &str,
        version1_id: &str,
        version2_id: &str,
    ) -> Result<VersionComparison, String> {
        let v1 = self
            .get_version(chapter_id, version1_id)
            .ok_or("Version 1 not found")?;
        let v2 = self
            .get_version(chapter_id, version2_id)
            .ok_or("Version 2 not found")?;

        let diff = self.generate_diff(
            v1.content.as_deref().unwrap_or(""),
            v2.content.as_deref().unwrap_or(""),
        );

        Ok(VersionComparison {
            version1: v1.clone(),
            version2: v2.clone(),
            diff,
        })
    }

    /// 计算diff统计
    fn calculate_diff(&self,
        prev: &ChapterVersion,
        current: &str,
    ) -> DiffStats {
        let prev_content = prev.content.as_deref().unwrap_or("");

        // 简单字符级别的diff计算
        let added = current.len().saturating_sub(prev_content.len());
        let removed = if current.len() < prev_content.len() {
            prev_content.len() - current.len()
        } else {
            0
        };

        let max_len = std::cmp::max(current.len(), prev_content.len());
        let modified = if max_len > 0 {
            ((added + removed) as f32 / max_len as f32) * 100.0
        } else {
            0.0
        };

        DiffStats {
            added_chars: added,
            removed_chars: removed,
            modified_percentage: modified.min(100.0),
        }
    }

    /// 生成文本diff
    fn generate_diff(&self,
        old_text: &str,
        new_text: &str,
    ) -> Vec<DiffLine> {
        let old_lines: Vec<&str> = old_text.lines().collect();
        let new_lines: Vec<&str> = new_text.lines().collect();

        let mut diff = Vec::new();
        let max_lines = std::cmp::max(old_lines.len(), new_lines.len());

        for i in 0..max_lines {
            let old_line = old_lines.get(i);
            let new_line = new_lines.get(i);

            match (old_line, new_line) {
                (Some(old), Some(new)) if old == new => {
                    diff.push(DiffLine {
                        line_number: i + 1,
                        change_type: ChangeType::Unchanged,
                        old_content: Some(old.to_string()),
                        new_content: Some(new.to_string()),
                    });
                }
                (Some(old), Some(new)) => {
                    diff.push(DiffLine {
                        line_number: i + 1,
                        change_type: ChangeType::Modified,
                        old_content: Some(old.to_string()),
                        new_content: Some(new.to_string()),
                    });
                }
                (Some(old), None) => {
                    diff.push(DiffLine {
                        line_number: i + 1,
                        change_type: ChangeType::Removed,
                        old_content: Some(old.to_string()),
                        new_content: None,
                    });
                }
                (None, Some(new)) => {
                    diff.push(DiffLine {
                        line_number: i + 1,
                        change_type: ChangeType::Added,
                        old_content: None,
                        new_content: Some(new.to_string()),
                    });
                }
                (None, None) => break,
            }
        }

        diff
    }

    /// 删除章节的所有版本
    pub fn delete_chapter_versions(&mut self, chapter_id: &str) {
        self.versions.remove(chapter_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComparison {
    pub version1: ChapterVersion,
    pub version2: ChapterVersion,
    pub diff: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_number: usize,
    pub change_type: ChangeType,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Unchanged,
    Added,
    Removed,
    Modified,
}
