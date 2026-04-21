//! Repository 单元测试
//!
//! 覆盖 Story / Character / Chapter 的完整 CRUD 流程。

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::db::connection::create_test_pool;

    // ==================== StoryRepository ====================

    #[test]
    fn test_story_create_and_get() {
        let pool = create_test_pool().unwrap();
        let repo = StoryRepository::new(pool);

        let req = CreateStoryRequest {
            title: "测试小说".to_string(),
            description: Some("描述".to_string()),
            genre: Some("科幻".to_string()),
            style_dna_id: None,
        };

        let story = repo.create(req).unwrap();
        assert_eq!(story.title, "测试小说");
        assert_eq!(story.genre, Some("科幻".to_string()));

        let fetched = repo.get_by_id(&story.id).unwrap().unwrap();
        assert_eq!(fetched.title, story.title);
        assert_eq!(fetched.genre, story.genre);
    }

    #[test]
    fn test_story_get_all() {
        let pool = create_test_pool().unwrap();
        let repo = StoryRepository::new(pool);

        let req1 = CreateStoryRequest {
            title: "小说A".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        };
        let req2 = CreateStoryRequest {
            title: "小说B".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        };

        repo.create(req1).unwrap();
        repo.create(req2).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_story_update() {
        let pool = create_test_pool().unwrap();
        let repo = StoryRepository::new(pool);

        let req = CreateStoryRequest {
            title: "原标题".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        };
        let story = repo.create(req).unwrap();

        let update_req = UpdateStoryRequest {
            title: Some("新标题".to_string()),
            description: Some("新描述".to_string()),
            tone: Some("轻快".to_string()),
            pacing: Some("快速".to_string()),
            style_dna_id: None,
        };

        let count = repo.update(&story.id, &update_req).unwrap();
        assert_eq!(count, 1);

        let updated = repo.get_by_id(&story.id).unwrap().unwrap();
        assert_eq!(updated.title, "新标题");
        assert_eq!(updated.tone, Some("轻快".to_string()));
    }

    #[test]
    fn test_story_delete() {
        let pool = create_test_pool().unwrap();
        let repo = StoryRepository::new(pool);

        let req = CreateStoryRequest {
            title: "待删除".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        };
        let story = repo.create(req).unwrap();

        let count = repo.delete(&story.id).unwrap();
        assert_eq!(count, 1);

        let deleted = repo.get_by_id(&story.id).unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_story_get_by_id_not_found() {
        let pool = create_test_pool().unwrap();
        let repo = StoryRepository::new(pool);

        let result = repo.get_by_id("non-existent-id").unwrap();
        assert!(result.is_none());
    }

    // ==================== CharacterRepository ====================

    #[test]
    fn test_character_create_and_get_by_story() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let char_repo = CharacterRepository::new(pool);

        let story_req = CreateStoryRequest {
            title: "角色测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        };
        let story = story_repo.create(story_req).unwrap();

        let char_req = CreateCharacterRequest {
            story_id: story.id.clone(),
            name: "张三".to_string(),
            background: Some("主角".to_string()),
        };
        let character = char_repo.create(char_req).unwrap();
        assert_eq!(character.name, "张三");
        assert_eq!(character.story_id, story.id);

        let chars = char_repo.get_by_story(&story.id).unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].name, "张三");
    }

    #[test]
    fn test_character_get_by_id() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let char_repo = CharacterRepository::new(pool);

        let story = story_repo.create(CreateStoryRequest {
            title: "测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        }).unwrap();

        let char_req = CreateCharacterRequest {
            story_id: story.id.clone(),
            name: "李四".to_string(),
            background: None,
        };
        let character = char_repo.create(char_req).unwrap();

        let fetched = char_repo.get_by_id(&character.id).unwrap().unwrap();
        assert_eq!(fetched.name, "李四");
    }

    #[test]
    fn test_character_update() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let char_repo = CharacterRepository::new(pool);

        let story = story_repo.create(CreateStoryRequest {
            title: "测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        }).unwrap();

        let char_req = CreateCharacterRequest {
            story_id: story.id.clone(),
            name: "原名".to_string(),
            background: Some("背景".to_string()),
        };
        let character = char_repo.create(char_req).unwrap();

        let count = char_repo.update(
            &character.id,
            Some("新名".to_string()),
            Some("新背景".to_string()),
            Some("开朗".to_string()),
            Some("成为英雄".to_string()),
        ).unwrap();
        assert_eq!(count, 1);

        let updated = char_repo.get_by_id(&character.id).unwrap().unwrap();
        assert_eq!(updated.name, "新名");
        assert_eq!(updated.background, Some("新背景".to_string()));
        assert_eq!(updated.personality, Some("开朗".to_string()));
        assert_eq!(updated.goals, Some("成为英雄".to_string()));
    }

    #[test]
    fn test_character_delete() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let char_repo = CharacterRepository::new(pool);

        let story = story_repo.create(CreateStoryRequest {
            title: "测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        }).unwrap();

        let char_req = CreateCharacterRequest {
            story_id: story.id.clone(),
            name: "待删除".to_string(),
            background: None,
        };
        let character = char_repo.create(char_req).unwrap();

        let count = char_repo.delete(&character.id).unwrap();
        assert_eq!(count, 1);

        let chars = char_repo.get_by_story(&story.id).unwrap();
        assert_eq!(chars.len(), 0);
    }

    // ==================== ChapterRepository ====================

    #[test]
    fn test_chapter_create_and_get_by_story() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let chapter_repo = ChapterRepository::new(pool);

        let story = story_repo.create(CreateStoryRequest {
            title: "章节测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        }).unwrap();

        let chapter_req = CreateChapterRequest {
            story_id: story.id.clone(),
            chapter_number: 1,
            title: Some("第一章".to_string()),
            outline: Some("大纲".to_string()),
            content: Some("正文内容".to_string()),
        };
        let chapter = chapter_repo.create(chapter_req).unwrap();
        assert_eq!(chapter.chapter_number, 1);
        assert_eq!(chapter.title, Some("第一章".to_string()));

        let chapters = chapter_repo.get_by_story(&story.id).unwrap();
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, Some("第一章".to_string()));
    }

    #[test]
    fn test_chapter_get_by_id() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let chapter_repo = ChapterRepository::new(pool);

        let story = story_repo.create(CreateStoryRequest {
            title: "测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        }).unwrap();

        let chapter_req = CreateChapterRequest {
            story_id: story.id.clone(),
            chapter_number: 1,
            title: Some("标题".to_string()),
            outline: None,
            content: None,
        };
        let chapter = chapter_repo.create(chapter_req).unwrap();

        let fetched = chapter_repo.get_by_id(&chapter.id).unwrap().unwrap();
        assert_eq!(fetched.chapter_number, 1);
    }

    #[test]
    fn test_chapter_update() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let chapter_repo = ChapterRepository::new(pool);

        let story = story_repo.create(CreateStoryRequest {
            title: "测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        }).unwrap();

        let chapter_req = CreateChapterRequest {
            story_id: story.id.clone(),
            chapter_number: 1,
            title: Some("原标题".to_string()),
            outline: Some("原大纲".to_string()),
            content: Some("原内容".to_string()),
        };
        let chapter = chapter_repo.create(chapter_req).unwrap();

        let count = chapter_repo.update(
            &chapter.id,
            Some("新标题".to_string()),
            Some("新大纲".to_string()),
            Some("新内容，更长一些".to_string()),
            None, // word_count 应该从 content 自动计算
        ).unwrap();
        assert_eq!(count, 1);

        let updated = chapter_repo.get_by_id(&chapter.id).unwrap().unwrap();
        assert_eq!(updated.title, Some("新标题".to_string()));
        assert_eq!(updated.content, Some("新内容，更长一些".to_string()));
    }

    #[test]
    fn test_chapter_delete() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let chapter_repo = ChapterRepository::new(pool);

        let story = story_repo.create(CreateStoryRequest {
            title: "测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        }).unwrap();

        let chapter_req = CreateChapterRequest {
            story_id: story.id.clone(),
            chapter_number: 1,
            title: None,
            outline: None,
            content: None,
        };
        let chapter = chapter_repo.create(chapter_req).unwrap();

        let count = chapter_repo.delete(&chapter.id).unwrap();
        assert_eq!(count, 1);

        let deleted = chapter_repo.get_by_id(&chapter.id).unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_chapter_order_by_number() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let chapter_repo = ChapterRepository::new(pool);

        let story = story_repo.create(CreateStoryRequest {
            title: "排序测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
        }).unwrap();

        let req1 = CreateChapterRequest {
            story_id: story.id.clone(),
            chapter_number: 3,
            title: Some("第三章".to_string()),
            outline: None,
            content: None,
        };
        let req2 = CreateChapterRequest {
            story_id: story.id.clone(),
            chapter_number: 1,
            title: Some("第一章".to_string()),
            outline: None,
            content: None,
        };
        chapter_repo.create(req1).unwrap();
        chapter_repo.create(req2).unwrap();

        let chapters = chapter_repo.get_by_story(&story.id).unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].chapter_number, 1); // 按 chapter_number 排序
        assert_eq!(chapters[1].chapter_number, 3);
    }
}
