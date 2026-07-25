//! 导出章节装配：以 scenes.content 为叙事真相源，按章节序输出。
//!
//! 契约：
//! - 章节按 `chapter_number` 升序
//! - 章内场景按 `sequence_number` 升序，非空 content 以 `\n\n` 拼接
//! - 所有内容均来自 Scene；chapters 表不再保存 content
//! - `chapter_id` 为空的孤儿场景按 sequence 追加为合成章节

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::db::{Chapter, Scene};

/// 导出用章节：包含数据库 Chapter 的元数据以及从 Scene 聚合后的正文。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportChapter {
    pub id: String,
    pub story_id: String,
    pub chapter_number: i32,
    pub title: Option<String>,
    pub outline: Option<String>,
    pub content: Option<String>,
    pub word_count: Option<i32>,
    pub model_used: Option<String>,
    pub cost: Option<f64>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl From<&Chapter> for ExportChapter {
    fn from(c: &Chapter) -> Self {
        Self {
            id: c.id.clone(),
            story_id: c.story_id.clone(),
            chapter_number: c.chapter_number,
            title: c.title.clone(),
            outline: c.outline.clone(),
            content: None,
            word_count: c.word_count,
            model_used: c.model_used.clone(),
            cost: c.cost,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

/// 用于 `chapter_display_title` 的最小抽象。
pub trait ChapterDisplay {
    fn chapter_title(&self) -> Option<&str>;
    fn chapter_number(&self) -> i32;
}

impl ChapterDisplay for Chapter {
    fn chapter_title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    fn chapter_number(&self) -> i32 {
        self.chapter_number
    }
}

impl ChapterDisplay for ExportChapter {
    fn chapter_title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    fn chapter_number(&self) -> i32 {
        self.chapter_number
    }
}

/// Markdown / 纯文本标题行：有标题用标题，否则「第N章」。
pub fn chapter_display_title<T: ChapterDisplay>(chapter: &T) -> String {
    chapter
        .chapter_title()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .unwrap_or_else(|| format!("第{}章", chapter.chapter_number()))
}

/// 将章节与场景装配为导出用章节列表（已填好 content）。
pub fn assemble_export_chapters(chapters: &[Chapter], scenes: &[Scene]) -> Vec<ExportChapter> {
    let mut ordered: Vec<ExportChapter> = chapters.iter().map(ExportChapter::from).collect();
    ordered.sort_by_key(|c| c.chapter_number);

    for chapter in &mut ordered {
        let aggregated = aggregate_scenes_for_chapter(scenes, &chapter.id);
        chapter.content = Some(if aggregated.is_empty() {
            String::new()
        } else {
            aggregated
        });
    }

    let mut orphan_chapters = assemble_orphan_scene_chapters(scenes, ordered.len() as i32);
    ordered.append(&mut orphan_chapters);
    ordered
}

/// 聚合某章下全部非空场景正文（按 sequence_number）。
pub fn aggregate_scenes_for_chapter(scenes: &[Scene], chapter_id: &str) -> String {
    let mut chapter_scenes: Vec<&Scene> = scenes
        .iter()
        .filter(|s| s.chapter_id.as_deref() == Some(chapter_id))
        .collect();
    chapter_scenes.sort_by_key(|s| s.sequence_number);
    chapter_scenes
        .iter()
        .filter_map(|s| s.content.as_deref())
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// 无 chapter_id 的场景 → 合成章节，编号接在已有章节之后。
fn assemble_orphan_scene_chapters(scenes: &[Scene], chapter_count: i32) -> Vec<ExportChapter> {
    let mut orphans: Vec<&Scene> = scenes
        .iter()
        .filter(|s| s.chapter_id.as_ref().map_or(true, |id| id.is_empty()))
        .filter(|s| {
            s.content
                .as_ref()
                .map(|c| !c.trim().is_empty())
                .unwrap_or(false)
        })
        .collect();
    orphans.sort_by_key(|s| s.sequence_number);

    orphans
        .into_iter()
        .enumerate()
        .map(|(i, scene)| {
            let n = chapter_count + i as i32 + 1;
            let title = scene
                .title
                .clone()
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| format!("未分章场景 {}", n));
            ExportChapter {
                id: format!("orphan-{}", scene.id),
                story_id: scene.story_id.clone(),
                chapter_number: n,
                title: Some(title),
                outline: None,
                content: scene.content.clone(),
                word_count: scene.content.as_ref().map(|c| c.chars().count() as i32),
                model_used: None,
                cost: None,
                created_at: scene.created_at,
                updated_at: scene.updated_at,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;

    fn chapter(id: &str, num: i32, title: &str) -> Chapter {
        Chapter {
            id: id.to_string(),
            story_id: "s1".to_string(),
            chapter_number: num,
            title: Some(title.to_string()),
            outline: Some(format!("{}大纲", title)),
            word_count: None,
            model_used: None,
            cost: None,
            created_at: Local::now(),
            updated_at: Local::now(),
        }
    }

    fn export_chapter(id: &str, num: i32, title: &str, content: Option<&str>) -> ExportChapter {
        ExportChapter {
            id: id.to_string(),
            story_id: "s1".to_string(),
            chapter_number: num,
            title: Some(title.to_string()),
            outline: Some(format!("{}大纲", title)),
            content: content.map(|c| c.to_string()),
            word_count: content.map(|c| c.len() as i32),
            model_used: None,
            cost: None,
            created_at: Local::now(),
            updated_at: Local::now(),
        }
    }

    fn scene(
        id: &str,
        seq: i32,
        chapter_id: Option<&str>,
        content: &str,
        title: Option<&str>,
    ) -> Scene {
        Scene {
            id: id.to_string(),
            story_id: "s1".to_string(),
            sequence_number: seq,
            title: title.map(|t| t.to_string()),
            dramatic_goal: None,
            external_pressure: None,
            conflict_type: None,
            characters_present: vec![],
            character_conflicts: vec![],
            content: Some(content.to_string()),
            setting_location: None,
            setting_time: None,
            setting_atmosphere: None,
            previous_scene_id: None,
            next_scene_id: None,
            execution_stage: None,
            outline_content: None,
            draft_content: None,
            model_used: None,
            cost: None,
            source: None,
            is_auto_generated: None,
            created_at: Local::now(),
            updated_at: Local::now(),
            confidence_score: None,
            style_blend_override: None,
            foreshadowing_ids: None,
            chapter_id: chapter_id.map(|c| c.to_string()),
            narrative_intensity: None,
            narrative_sentiment: None,
            narrative_event_types: None,
            narrative_preceding_scene_id: None,
            narrative_following_scene_id: None,
            act_number: None,
            position_in_act: None,
        }
    }

    #[test]
    fn empty_story_yields_empty_chapters() {
        let out = assemble_export_chapters(&[], &[]);
        assert!(out.is_empty());
    }

    #[test]
    fn chapters_ordered_by_chapter_number() {
        let chapters = vec![chapter("c2", 2, "第二章"), chapter("c1", 1, "第一章")];
        let scenes = vec![
            scene("s1", 1, Some("c1"), "A", None),
            scene("s2", 1, Some("c2"), "B", None),
        ];
        let out = assemble_export_chapters(&chapters, &scenes);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].chapter_number, 1);
        assert_eq!(out[1].chapter_number, 2);
        assert_eq!(out[0].content.as_deref(), Some("A"));
        assert_eq!(out[1].content.as_deref(), Some("B"));
    }

    #[test]
    fn scenes_are_truth_source() {
        let chapters = vec![chapter("c1", 1, "第一章")];
        let scenes = vec![
            scene("s2", 2, Some("c1"), "场景二", None),
            scene("s1", 1, Some("c1"), "场景一", None),
        ];
        let out = assemble_export_chapters(&chapters, &scenes);
        assert_eq!(out[0].content.as_deref(), Some("场景一\n\n场景二"));
    }

    #[test]
    fn empty_chapter_content_with_no_scenes_becomes_empty_string() {
        let chapters = vec![chapter("c1", 1, "空章")];
        let out = assemble_export_chapters(&chapters, &[]);
        assert_eq!(out[0].content.as_deref(), Some(""));
    }

    #[test]
    fn orphan_scenes_appended_as_synthetic_chapters() {
        let chapters = vec![chapter("c1", 1, "第一章")];
        let scenes = vec![
            scene("s1", 1, Some("c1"), "章内", None),
            scene("o1", 5, None, "孤儿正文", Some("插曲")),
        ];
        let out = assemble_export_chapters(&chapters, &scenes);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].content.as_deref(), Some("章内"));
        assert_eq!(out[1].title.as_deref(), Some("插曲"));
        assert_eq!(out[1].content.as_deref(), Some("孤儿正文"));
        assert_eq!(out[1].chapter_number, 2);
    }

    #[test]
    fn chapter_display_title_falls_back_to_number() {
        let mut ch = export_chapter("c1", 3, "有标题", None);
        assert_eq!(chapter_display_title(&ch), "有标题");
        ch.title = Some("  ".to_string());
        assert_eq!(chapter_display_title(&ch), "第3章");
        ch.title = None;
        assert_eq!(chapter_display_title(&ch), "第3章");
    }

    #[test]
    fn markdown_headers_use_display_titles() {
        // 契约：导出正文标题行使用 chapter_display_title
        let chapters = vec![
            chapter("c1", 1, "开端"),
            Chapter {
                title: None,
                ..chapter("c2", 2, "x")
            },
        ];
        let scenes = vec![
            scene("s1", 1, Some("c1"), "a", None),
            scene("s2", 1, Some("c2"), "b", None),
        ];
        let assembled = assemble_export_chapters(&chapters, &scenes);
        let titles: Vec<String> = assembled.iter().map(chapter_display_title).collect();
        assert_eq!(titles, vec!["开端".to_string(), "第2章".to_string()]);
    }
}
