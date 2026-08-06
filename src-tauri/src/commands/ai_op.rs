//! Ai Op commands

use tauri::{AppHandle, Emitter, State};

use crate::{
    db::{DbPool, SceneRepository, SceneUpdate},
    error::AppError,
};

#[tauri::command(rename_all = "snake_case")]
pub async fn list_ai_operations(
    pool: State<'_, DbPool>,
    story_id: String,
) -> Result<Vec<crate::db::AiOperation>, AppError> {
    let pool = pool.inner().clone();
    let repo = crate::db::AiOperationRepository::new(pool);
    repo.get_by_story(&story_id).map_err(AppError::from)
}

/// 检测内容是否为旧版本缺陷产生的截断预览快照。
///
/// 旧版本 smart_execute 把截断预览（格式 `...(前N字已省略)\n<末尾6000字>`）
/// 误存为 previous_content。rollback 会把 previous_content 原样写回为章节全部
/// 内容，因此必须以开头锚定识别该标记并阻止回滚，避免数据丢失。
/// 仅匹配开头，正文中偶然出现该短语不误判。
pub(crate) fn is_truncated_preview(content: &str) -> bool {
    let Some(rest) = content.strip_prefix("...(前") else {
        return false;
    };
    let digit_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digit_len == 0 {
        return false;
    }
    rest[digit_len..].starts_with("字已省略)\n")
}

/// 回滚核心逻辑（不依赖 Tauri State/AppHandle，便于测试）。
/// 成功时返回 (chapter_id, story_id) 供命令层发射同步事件。
fn rollback_ai_operation_core(
    pool: &DbPool,
    operation_id: &str,
) -> Result<(String, String), AppError> {
    let op_repo = crate::db::AiOperationRepository::new(pool.clone());

    let operation = op_repo
        .get_by_id(operation_id)
        .map_err(AppError::from)?
        .ok_or("Operation not found")?;

    // Only support rollback for chapter content operations that have
    // previous_content
    let prev_content = operation.previous_content.ok_or("此操作不支持回滚")?;

    // 旧版本缺陷：previous_content 可能是截断预览而非全文，原样写回会丢失
    // 章节内容，检测到即拒绝回滚。
    if is_truncated_preview(&prev_content) {
        return Err(AppError::from(
            "此操作的历史快照为截断预览（旧版本缺陷），回滚会丢失内容，已阻止",
        ));
    }

    // Phase 1: 回滚内容恢复走 SceneRepository（Scene 为真相源）
    let chapter_id = operation.chapter_id.ok_or("此操作没有关联章节")?;

    {
        let scene_repo = SceneRepository::new(pool.clone());
        if let Ok(scenes) = scene_repo.get_by_chapter(&chapter_id) {
            if let Some(scene) = scenes.first() {
                scene_repo
                    .update(
                        &scene.id,
                        &SceneUpdate {
                            content: Some(prev_content),
                            ..Default::default()
                        },
                    )
                    .map_err(AppError::from)?;
            }
        }
    }

    // Mark operation as rolled back
    op_repo
        .update_status(operation_id, "rolled_back")
        .map_err(AppError::from)?;

    Ok((chapter_id, operation.story_id))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn rollback_ai_operation(
    pool: State<'_, DbPool>,
    operation_id: String,
    app: AppHandle,
) -> Result<(), AppError> {
    let pool = pool.inner().clone();
    let (chapter_id, story_id) = rollback_ai_operation_core(&pool, &operation_id)?;

    // Emit sync event
    let _ = app.emit(
        "sync-event",
        serde_json::json!({
            "event": "chapterUpdated",
            "chapter_id": chapter_id,
            "story_id": story_id,
        }),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        AiOperationRepository, ChapterRepository, CreateAiOperationRequest, CreateChapterRequest,
        CreateStoryRequest, StoryRepository,
    };

    // ---------- is_truncated_preview 单元测试 ----------

    #[test]
    fn test_is_truncated_preview_matches_real_marker() {
        // 与 orchestrator.rs 实际生成的格式完全一致：...(前{skip}字已省略)\n
        let preview = format!("...(前{}字已省略)\n{}", 12345, "正文末尾片段");
        assert!(is_truncated_preview(&preview));
        assert!(is_truncated_preview("...(前1字已省略)\nx"));
    }

    #[test]
    fn test_is_truncated_preview_rejects_normal_content() {
        assert!(!is_truncated_preview(""));
        assert!(!is_truncated_preview("第一章 风起\n这是一个普通章节。"));
        // 正文中间偶然出现该短语不应误判（必须锚定在开头）
        assert!(!is_truncated_preview(
            "他说：「...(前100字已省略)\n」只是玩笑"
        ));
        assert!(!is_truncated_preview("  ...(前100字已省略)\n前面有空格"));
        // 缺少数字或格式不全不判定为截断标记
        assert!(!is_truncated_preview("...(前字已省略)\n"));
        assert!(!is_truncated_preview("...(100字已省略)\n"));
        assert!(!is_truncated_preview("...(前100字已省略)无换行"));
    }

    // ---------- rollback 集成测试 ----------

    fn setup_story_chapter(pool: &DbPool, content: &str) -> (crate::db::Story, crate::db::Chapter) {
        let story = StoryRepository::new(pool.clone())
            .create(CreateStoryRequest {
                title: "回滚测试故事".to_string(),
                description: None,
                genre: None,
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .expect("create story");
        let chapter = ChapterRepository::new(pool.clone())
            .create(CreateChapterRequest {
                story_id: story.id.clone(),
                chapter_number: 1,
                title: Some("第一章".to_string()),
                outline: None,
                content: Some(content.to_string()),
            })
            .expect("create chapter");
        (story, chapter)
    }

    fn create_op(
        pool: &DbPool,
        story_id: &str,
        chapter_id: &str,
        previous_content: Option<String>,
    ) -> crate::db::AiOperation {
        AiOperationRepository::new(pool.clone())
            .create(CreateAiOperationRequest {
                story_id: story_id.to_string(),
                scene_id: None,
                chapter_id: Some(chapter_id.to_string()),
                operation_type: "smart_execute".to_string(),
                operation_name: "AI 续写".to_string(),
                input_summary: None,
                output_summary: None,
                previous_content,
                new_content: Some("AI 生成的新内容".to_string()),
                metadata: None,
            })
            .expect("create ai operation")
    }

    fn scene_content(pool: &DbPool, chapter_id: &str) -> String {
        SceneRepository::new(pool.clone())
            .get_by_chapter(chapter_id)
            .expect("get scenes")
            .first()
            .expect("scene exists")
            .content
            .clone()
            .expect("scene content")
    }

    #[test]
    fn test_rollback_restores_full_long_previous_content() {
        let pool = crate::db::create_test_pool().expect("test pool");
        // 超过 6000 字的全文快照（修复后 prev_content_for_record 存全文）
        let full_content: String = "文".repeat(7000);
        let (story, chapter) = setup_story_chapter(&pool, "AI 覆盖后的内容");
        let op = create_op(&pool, &story.id, &chapter.id, Some(full_content.clone()));

        let (chapter_id, _) =
            rollback_ai_operation_core(&pool, &op.id).expect("rollback should succeed");

        assert_eq!(chapter_id, chapter.id);
        assert_eq!(scene_content(&pool, &chapter.id), full_content);
        let reloaded = AiOperationRepository::new(pool.clone())
            .get_by_id(&op.id)
            .expect("get op")
            .expect("op exists");
        assert_eq!(reloaded.status, "rolled_back");
    }

    #[test]
    fn test_rollback_blocked_for_legacy_truncated_preview() {
        let pool = crate::db::create_test_pool().expect("test pool");
        let original = "原始章节内容，不应被覆盖";
        let (story, chapter) = setup_story_chapter(&pool, original);
        // 旧版本缺陷记录：previous_content 为截断预览
        let truncated = format!("...(前{}字已省略)\n{}", 1234, "仅剩末尾片段");
        let op = create_op(&pool, &story.id, &chapter.id, Some(truncated));

        let err = rollback_ai_operation_core(&pool, &op.id)
            .expect_err("legacy truncated snapshot must be rejected");
        assert!(err.message().contains("截断预览"));

        // 章节内容保持原样，未被截断预览覆盖
        assert_eq!(scene_content(&pool, &chapter.id), original);
        // 操作状态不应被标记为已回滚
        let reloaded = AiOperationRepository::new(pool.clone())
            .get_by_id(&op.id)
            .expect("get op")
            .expect("op exists");
        assert_eq!(reloaded.status, "success");
    }
}
