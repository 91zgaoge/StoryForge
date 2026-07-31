//! 资产落库：把黑板资产区的条目物化到应用资产表（characters/world_buildings/
//! story_outlines/foreshadowing_tracker）。 character 条目 content 须为 JSON
//! {"name","background","personality","goals"}； world/outline 条目 content
//! 为纯文本； foreshadowing 条目 content 为纯文本或 JSON（数组/对象，
//! 经 normalize_foreshadowing 逐条归一化）。item_type 做别名归一化
//! （worldbuilding/world_building→world，story_outline→outline），兼容本地
//! 模型变体。解析失败的条目跳过并 log::warn!。

use rusqlite::params;

use crate::{agency::models::BoardItem, db::DbPool};

fn now() -> String {
    chrono::Local::now().to_rfc3339()
}

pub fn materialize_assets(pool: &DbPool, story_id: &str, items: &[BoardItem]) -> usize {
    let mut count = 0usize;
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("materialize_assets: pool 获取失败: {}", e);
            return 0;
        }
    };
    for item in items.iter().filter(|i| i.status == "active") {
        // item_type 别名归一化：兼容本地模型经 board_write 写入的变体
        let normalized_type = match item.item_type.as_str() {
            "worldbuilding" | "world_building" => "world",
            "story_outline" => "outline",
            other => other,
        };
        match normalized_type {
            "character" => {
                let parsed =
                    crate::agency::coordinator::parse_lenient::<serde_json::Value>(&item.content);
                let (name, background, personality, goals) = match parsed.as_ref() {
                    Some(v) => (
                        v.get("name")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        v.get("background")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        v.get("personality")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        v.get("goals")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                    ),
                    None => {
                        log::warn!("materialize: 角色条目 {} 非 JSON，跳过", item.key);
                        continue;
                    }
                };
                if name.is_empty() {
                    log::warn!("materialize: 角色条目 {} 缺 name，跳过", item.key);
                    continue;
                }
                let id = uuid::Uuid::new_v4().to_string();
                let ts = now();
                // story_id+name upsert：已存在同名角色时刷新字段（创世重跑/资产
                // 补齐场景），否则插入新行。
                let updated = conn.execute(
                    "UPDATE characters SET background = ?3, personality = ?4, goals = ?5, updated_at = ?6
                     WHERE story_id = ?1 AND name = ?2",
                    params![story_id, name, background, personality, goals, ts],
                );
                match updated {
                    Ok(n) if n > 0 => count += 1,
                    Ok(_) => match conn.execute(
                        "INSERT INTO characters (id, story_id, name, background, personality, goals, source, is_auto_generated, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'agency', 1, ?7, ?8)",
                        params![id, story_id, name, background, personality, goals, ts, ts],
                    ) {
                        Ok(n) => count += n,
                        Err(e) => log::warn!("materialize: 插入角色失败: {}", e),
                    },
                    Err(e) => log::warn!("materialize: 更新角色失败: {}", e),
                }
            }
            "world" => {
                let id = uuid::Uuid::new_v4().to_string();
                let ts = now();
                let result = conn.execute(
                    "INSERT INTO world_buildings (id, story_id, concept, rules, source, is_auto_generated, created_at, updated_at)
                     VALUES (?1, ?2, ?3, '[]', 'agency', 1, ?4, ?5)
                     ON CONFLICT(story_id) DO UPDATE SET concept = excluded.concept, updated_at = excluded.updated_at",
                    params![id, story_id, item.content, ts, ts],
                );
                match result {
                    Ok(_) => count += 1,
                    Err(e) => log::warn!("materialize: 写入世界观失败: {}", e),
                }
            }
            "outline" => {
                let id = uuid::Uuid::new_v4().to_string();
                let ts = now();
                let result = conn.execute(
                    "INSERT INTO story_outlines (id, story_id, content, act_count, created_at, updated_at)
                     VALUES (?1, ?2, ?3, 3, ?4, ?5)
                     ON CONFLICT(story_id) DO UPDATE SET content = excluded.content, updated_at = excluded.updated_at",
                    params![id, story_id, item.content, ts, ts],
                );
                match result {
                    Ok(_) => count += 1,
                    Err(e) => log::warn!("materialize: 写入大纲失败: {}", e),
                }
            }
            "foreshadowing" => {
                // content 可能是纯文本（单条），也可能是 JSON 数组/对象
                // （本地模型经 board_write 写入的变体），统一归一化为若干条文本。
                // 先试严格 JSON（覆盖数组形态），再退回 parse_lenient（花括号
                // 截取只覆盖对象形态），最后按纯文本处理。
                let trimmed = item.content.trim();
                let parsed = serde_json::from_str::<serde_json::Value>(trimmed)
                    .ok()
                    .or_else(|| {
                        crate::agency::coordinator::parse_lenient::<serde_json::Value>(trimmed)
                    });
                let texts: Vec<String> = match parsed {
                    Some(serde_json::Value::Array(arr)) => arr
                        .iter()
                        .map(crate::agency::coordinator::normalize_foreshadowing)
                        .collect(),
                    Some(v) => vec![crate::agency::coordinator::normalize_foreshadowing(&v)],
                    None => vec![item.content.clone()],
                };
                for text in texts {
                    let text = text.trim();
                    if text.is_empty() {
                        continue;
                    }
                    let id = uuid::Uuid::new_v4().to_string();
                    let ts = now();
                    // story_id+content 去重：同一伏笔重复物化时跳过
                    match conn.execute(
                        "INSERT INTO foreshadowing_tracker (id, story_id, content, setup_scene_id, status, importance, created_at)
                         SELECT ?1, ?2, ?3, NULL, 'setup', 5, ?4
                         WHERE NOT EXISTS (SELECT 1 FROM foreshadowing_tracker WHERE story_id = ?2 AND content = ?3)",
                        params![id, story_id, text, ts],
                    ) {
                        Ok(n) => count += n,
                        Err(e) => log::warn!("materialize: 写入伏笔失败: {}", e),
                    }
                }
            }
            _ => {}
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agency::models::*,
        db::{create_test_pool, dto::CreateStoryRequest, repositories::StoryRepository},
    };

    fn story(pool: &crate::db::DbPool, id: &str) {
        let s = StoryRepository::new(pool.clone())
            .create(CreateStoryRequest {
                title: "测试书".into(),
                description: None,
                genre: None,
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .unwrap();
        // create 生成自己的 id；测试统一改用其返回值
        let conn = pool.get().unwrap();
        conn.execute(
            "UPDATE stories SET id = ?1 WHERE id = ?2",
            rusqlite::params![id, s.id],
        )
        .unwrap();
    }

    fn item(item_type: &str, key: &str, content: &str) -> BoardItem {
        BoardItem::new(
            "r1",
            "s1",
            BoardZone::Asset,
            item_type,
            key,
            content,
            "摘要",
            AgentRole::Producer,
            "active",
        )
    }

    #[test]
    fn test_materialize_character_json() {
        let pool = create_test_pool().unwrap();
        story(&pool, "s1");
        let items = vec![item(
            "character",
            "主角",
            r#"{"name":"阿苔","background":"拾荒者","personality":"坚韧","goals":"找到星环"}"#,
        )];
        let n = materialize_assets(&pool, "s1", &items);
        assert_eq!(n, 1);
        let conn = pool.get().unwrap();
        let name: String = conn
            .query_row("SELECT name FROM characters WHERE story_id='s1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(name, "阿苔");
        // 重复物化（story_id+name upsert）：刷新字段，仍一行
        assert_eq!(materialize_assets(&pool, "s1", &items), 1);
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM characters WHERE story_id='s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        // upsert 应刷新字段而非跳过
        let items2 = vec![item(
            "character",
            "主角",
            r#"{"name":"阿苔","background":"拾荒者出身·已刷新","personality":"坚韧","goals":"找到星环"}"#,
        )];
        assert_eq!(materialize_assets(&pool, "s1", &items2), 1);
        let bg: String = conn
            .query_row(
                "SELECT background FROM characters WHERE story_id='s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(bg.contains("已刷新"));
    }

    #[test]
    fn test_materialize_world_upsert_idempotent() {
        let pool = create_test_pool().unwrap();
        story(&pool, "s1");
        let items = vec![item("world", "世界观", "双星废土，磁力风暴")];
        assert_eq!(materialize_assets(&pool, "s1", &items), 1);
        // 再次执行不报错（upsert），仍一行
        assert_eq!(materialize_assets(&pool, "s1", &items), 1);
        let conn = pool.get().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM world_buildings WHERE story_id='s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_materialize_skips_non_json_character() {
        let pool = create_test_pool().unwrap();
        story(&pool, "s1");
        let items = vec![item("character", "主角", "自由文本不是 JSON")];
        assert_eq!(materialize_assets(&pool, "s1", &items), 0);
        let conn = pool.get().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM characters WHERE story_id='s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_materialize_outline() {
        let pool = create_test_pool().unwrap();
        story(&pool, "s1");
        let items = vec![item("outline", "第一卷", "第一卷大纲：起承转合……")];
        assert_eq!(materialize_assets(&pool, "s1", &items), 1);
        let conn = pool.get().unwrap();
        let content: String = conn
            .query_row(
                "SELECT content FROM story_outlines WHERE story_id='s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(content.contains("起承转合"));
    }

    #[test]
    fn test_materialize_foreshadowing_text_and_json_array() {
        let pool = create_test_pool().unwrap();
        story(&pool, "s1");
        let items = vec![
            item("foreshadowing", "伏笔1", "妹妹的项链（第三卷回收）"),
            item(
                "foreshadowing",
                "伏笔清单",
                r#"["身世之谜",{"description":"星环秘密"},{"text":"远古文字"}]"#,
            ),
        ];
        assert_eq!(materialize_assets(&pool, "s1", &items), 4);
        let conn = pool.get().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM foreshadowing_tracker WHERE story_id='s1' AND status='setup'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 4);
        // 重复物化按 story_id+content 去重
        assert_eq!(materialize_assets(&pool, "s1", &items), 0);
    }

    #[test]
    fn test_materialize_item_type_alias() {
        let pool = create_test_pool().unwrap();
        story(&pool, "s1");
        let items = vec![
            item("worldbuilding", "世界观", "双星废土"),
            item("story_outline", "大纲", "整本书大纲"),
        ];
        assert_eq!(materialize_assets(&pool, "s1", &items), 2);
        let conn = pool.get().unwrap();
        let world: String = conn
            .query_row(
                "SELECT concept FROM world_buildings WHERE story_id='s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(world, "双星废土");
        let outline: String = conn
            .query_row(
                "SELECT content FROM story_outlines WHERE story_id='s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(outline, "整本书大纲");
    }
}
