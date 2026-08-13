//! 续写落库模式：Append（当前章）与 NextChapter（新章）的纯数据 + Append 写库。

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
    let full = if cleaned_old.is_empty() {
        cleaned_inc.to_string()
    } else {
        format!("{cleaned_old}\n\n{cleaned_inc}")
    };
    repo.update(
        scene_id,
        &crate::db::repositories::SceneUpdate {
            content: Some(full.clone()),
            ..Default::default()
        },
    )
    .map_err(AppError::from)?;
    Ok(AppendPersistOutcome {
        scene_id: scene.id,
        chapter_number: scene.sequence_number,
        full_content: full,
    })
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
}
