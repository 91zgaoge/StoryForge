//! 续写落库模式：Append（当前章）与 NextChapter（新章）的纯数据 + Append 写库。

pub use crate::creative_engine::expansion::BeatCounters;
use crate::{db::DbPool, error::AppError};

#[derive(Debug, Clone)]
pub enum PersistMode {
    Append { scene_id: String },
    NextChapter { chapter_number: i32 },
}

#[derive(Debug, Clone)]
pub struct AppendPersistOutcome {
    pub scene_id: String,
    pub chapter_number: i32,
    pub full_content: String,
}

/// 续写落库模式解析。幕前恒传 `explicit_next_chapter=false`（同章 Append）；
/// `true` 仅契约完整（幕后 `agency_continue_chapter` 自己算
/// MAX+1，不走此函数填真实章号）。
pub fn resolve_persist_mode(
    is_continuation: bool,
    scene_id: Option<String>,
    explicit_next_chapter: bool,
) -> Result<PersistMode, AppError> {
    if !is_continuation {
        return Err(AppError::from("resolve_persist_mode 仅用于续写"));
    }
    if explicit_next_chapter {
        return Ok(PersistMode::NextChapter { chapter_number: 0 });
    }
    let sid = scene_id
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::validation_failed("请先打开一个章节", Some("no_scene")))?;
    Ok(PersistMode::Append { scene_id: sid })
}

/// 每次成功 Append/NextChapter 后 +1。失败不阻断落库。
pub fn increment_append_beat(pool: &DbPool, story_id: &str) -> Result<(), AppError> {
    let conn = pool
        .get()
        .map_err(|e| AppError::from(format!("pool: {e}")))?;
    let mut beats = crate::creative_engine::expansion::read_beat_counters(&conn, story_id);
    beats.append_beats = beats.append_beats.saturating_add(1);
    crate::creative_engine::expansion::write_beat_counters(&conn, story_id, beats)
        .map_err(AppError::from)?;
    Ok(())
}

/// 将 current_content + increment 写入已有 scene。禁止 create 新行。
pub fn persist_append(
    pool: &DbPool,
    mode: &PersistMode,
    current_content: &str,
    increment: &str,
) -> Result<AppendPersistOutcome, AppError> {
    let PersistMode::Append { scene_id } = mode else {
        return Err(AppError::from("persist_append 只接受 Append"));
    };
    let repo = crate::db::repositories::SceneRepository::new(pool.clone());
    let scene = repo
        .get_by_id(scene_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::validation_failed("请先打开一个章节", Some("no_scene")))?;
    let cleaned_old = current_content.trim();
    let cleaned_inc = increment.trim();
    if cleaned_inc.chars().count() < 200 {
        return Err(AppError::from("续写增量过短，拒绝落库"));
    }
    let full = join_content(cleaned_old, cleaned_inc);
    repo.update(
        scene_id,
        &crate::db::repositories::SceneUpdate {
            content: Some(full.clone()),
            ..Default::default()
        },
    )
    .map_err(AppError::from)?;
    if let Err(e) = increment_append_beat(pool, &scene.story_id) {
        log::warn!("increment_append_beat 失败: {e}");
    }
    Ok(AppendPersistOutcome {
        scene_id: scene.id,
        chapter_number: scene.sequence_number,
        full_content: full,
    })
}

fn looks_like_html(s: &str) -> bool {
    let t = s.trim();
    t.starts_with('<') && t.contains('>')
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 旧文已是 HTML 时用 `<p>` 包增量，避免把 TipTap 标记压成纯文本；纯文本仍用
/// `\n\n`。
fn join_content(old: &str, increment: &str) -> String {
    if old.is_empty() {
        increment.to_string()
    } else if looks_like_html(old) {
        format!("{old}<p>{}</p>", html_escape(increment))
    } else {
        format!("{old}\n\n{increment}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        connection::create_test_pool,
        repositories::{CreateStoryRequest, SceneRepository, StoryRepository},
    };

    fn long_increment() -> String {
        "续写增量正文。".repeat(30) // 7 * 30 = 210 字，满足 ≥200 落库门槛
    }

    fn seed_story_with_scene(pool: &crate::db::DbPool) -> (String, String) {
        let story = StoryRepository::new(pool.clone())
            .create(CreateStoryRequest {
                title: "追加测试".into(),
                description: None,
                genre: Some("玄幻".into()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .unwrap();
        let scene_repo = SceneRepository::new(pool.clone());
        let mut conn = pool.get().unwrap();
        let tx = conn.transaction().unwrap();
        let scene = scene_repo
            .create_in_tx(&tx, &story.id, 1, Some("第一章"))
            .unwrap();
        scene_repo
            .update_in_tx(
                &tx,
                &scene.id,
                &crate::db::repositories::SceneUpdate {
                    content: Some("旧文开头。".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        tx.commit().unwrap();
        (story.id, scene.id)
    }

    #[test]
    fn append_does_not_create_new_scene_row() {
        let pool = create_test_pool().unwrap();
        let (story_id, scene_id) = seed_story_with_scene(&pool);
        let out = persist_append(
            &pool,
            &PersistMode::Append {
                scene_id: scene_id.clone(),
            },
            "旧文开头。",
            &long_increment(),
        )
        .unwrap();
        assert_eq!(out.scene_id, scene_id);
        let scenes = SceneRepository::new(pool.clone())
            .get_by_story(&story_id)
            .unwrap();
        assert_eq!(scenes.len(), 1);
        let expected = format!("旧文开头。\n\n{}", long_increment());
        assert_eq!(scenes[0].content.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn append_missing_scene_id_is_err() {
        let pool = create_test_pool().unwrap();
        let err = persist_append(
            &pool,
            &PersistMode::Append {
                scene_id: "no-such".into(),
            },
            "旧",
            "新",
        )
        .unwrap_err();
        assert!(err.to_string().contains("请先打开一个章节") || err.to_string().contains("不存在"));
    }

    #[test]
    fn continuation_requires_scene_id_for_append() {
        let err = resolve_persist_mode(true, None, false).unwrap_err();
        assert!(err.to_string().contains("请先打开一个章节"));
        let ok = resolve_persist_mode(true, Some("s1".into()), false).unwrap();
        match ok {
            PersistMode::Append { scene_id } => {
                assert_eq!(scene_id, "s1")
            }
            _ => panic!("expected Append"),
        }
        let next = resolve_persist_mode(true, None, true).unwrap();
        match next {
            PersistMode::NextChapter { chapter_number } => {
                assert_eq!(chapter_number, 0); // 占位；幕后自己算 MAX+1，
                                               // 不走此函数填真实章号
            }
            _ => panic!("expected NextChapter"),
        }
    }

    #[test]
    fn append_html_wraps_increment_in_p() {
        let pool = create_test_pool().unwrap();
        let (story_id, scene_id) = seed_story_with_scene(&pool);
        let old = "<p>旧文开头。</p>";
        let inc = long_increment();
        let out = persist_append(
            &pool,
            &PersistMode::Append {
                scene_id: scene_id.clone(),
            },
            old,
            &inc,
        )
        .unwrap();
        let expected = format!("{old}<p>{inc}</p>");
        assert_eq!(out.full_content, expected);
        let scenes = SceneRepository::new(pool.clone())
            .get_by_story(&story_id)
            .unwrap();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].content.as_deref(), Some(expected.as_str()));
    }
}
