//! Writer 资产段落组装共享函数
//!
//! 从 Full 路径（`agents/service.rs` 的 `build_writer_prompt`）下沉的段落组装
//! 逻辑，供 Full 路径与 TimeSliced（WriteTimeBundle）双路复用。
//! `format_active_conflicts` / `format_character_goals`
//! 为纯格式化函数：规范状态
//! 快照由调用方一次性加载（`CanonicalStateManager::get_snapshot_sync`）后传入，
//! 避免同一次 prompt 构建内重复聚合快照。

/// 按字符数截断；`usize::MAX` 等价不截断（保持 Full 路径原行为）。
fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}…（已截断）", truncated)
    }
}

/// 当前活跃冲突段落（原 service.rs:2345-2357 内联逻辑下沉）。
/// 快照由调用方一次性加载传入；无活跃冲突返回 None。
pub(crate) fn format_active_conflicts(
    snapshot: &crate::canonical_state::CanonicalStateSnapshot,
    budget_chars: usize,
) -> Option<String> {
    let conflicts = &snapshot.story_context.active_conflicts;
    if conflicts.is_empty() {
        return None;
    }
    let mut lines = vec!["【当前活跃冲突】".to_string()];
    for conflict in conflicts {
        lines.push(format!(
            "- {}: 涉及 {}, 赌注: {}",
            conflict.conflict_type,
            conflict.parties.join(", "),
            conflict.stakes
        ));
    }
    Some(truncate_chars(&lines.join("\n"), budget_chars))
}

/// 角色当前状态（目标/弧光/秘密）段落（原 service.rs:2386-2411 内联逻辑下沉）。
/// 快照由调用方一次性加载传入；每个角色行按 per_char_budget
/// 截断；无角色状态返回 None。
pub(crate) fn format_character_goals(
    snapshot: &crate::canonical_state::CanonicalStateSnapshot,
    per_char_budget: usize,
) -> Option<String> {
    if snapshot.character_states.is_empty() {
        return None;
    }
    let mut lines = vec!["【角色当前状态】".to_string()];
    for cs in &snapshot.character_states {
        let mut parts = vec![format!("{}:", cs.name)];
        if let Some(ref loc) = cs.current_location {
            parts.push(format!("位置: {}", loc));
        }
        if let Some(ref emo) = cs.current_emotion {
            parts.push(format!("情绪: {}", emo));
        }
        if let Some(ref goal) = cs.active_goal {
            parts.push(format!("目标: {}", goal));
        }
        if !cs.secrets_known.is_empty() {
            parts.push(format!("已知秘密: {}", cs.secrets_known.join(", ")));
        }
        if !cs.secrets_unknown.is_empty() {
            parts.push(format!("未知秘密: {}", cs.secrets_unknown.join(", ")));
        }
        parts.push(format!("弧光进度: {:.0}%", cs.arc_progress * 100.0));
        lines.push(format!(
            "- {}",
            truncate_chars(&parts.join(" "), per_char_budget)
        ));
    }
    Some(lines.join("\n"))
}

/// 体裁元素参考表 + 典型结构段落（原 service.rs:1966-1971 内联逻辑下沉）。
/// 两者皆空返回 None。
pub(crate) fn format_genre_reference_tables(
    profile: &crate::db::GenreProfile,
    budget_chars: usize,
) -> Option<String> {
    let mut lines = Vec::new();
    if let Some(reference_tables) = &profile.reference_tables_json {
        if !reference_tables.trim().is_empty() {
            lines.push(format!("元素参考表：\n{}", reference_tables));
        }
    }
    if let Some(typical_structure) = &profile.typical_structure_json {
        if !typical_structure.trim().is_empty() {
            lines.push(format!("典型结构参考：\n{}", typical_structure));
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(truncate_chars(&lines.join("\n"), budget_chars))
    }
}

/// `WritingStrategy.pace` 字符串 → 节奏因子（fast=1.5 / slow=0.5 / 其他=1.0）。
pub(crate) fn pace_to_factor(pace: &str) -> f64 {
    match pace {
        "fast" => 1.5,
        "slow" => 0.5,
        _ => 1.0,
    }
}

/// 冲突强度 + 叙事节奏的分档语义文案
/// （原 service.rs:1888-1899 冲突五档 + 1901-1908 节奏三档下沉）。
///
/// `conflict_intensity` 沿用 `WritingStrategy.conflict_level` 的 0-100 刻度；
/// `pacing_factor` 由 `pace_to_factor` 映射，>=1.2 判快、<=0.8 判慢。
pub(crate) fn writing_constraints_semantic_text(
    conflict_intensity: f64,
    pacing_factor: f64,
) -> String {
    let conflict_line = if conflict_intensity >= 80.0 {
        "冲突强度：极高。每 500 字至少设置一次冲突或张力，保持高度紧张感。"
    } else if conflict_intensity >= 60.0 {
        "冲突强度：高。保持频繁的冲突和对抗，推动情节快速展开。"
    } else if conflict_intensity >= 40.0 {
        "冲突强度：中等。适度安排冲突，兼顾人物发展和情节推进。"
    } else if conflict_intensity >= 20.0 {
        "冲突强度：低。以人物内心和情感为主，减少外部冲突。"
    } else {
        "冲突强度：极低。以平和、抒情、描写为主，避免剧烈冲突。"
    };
    let pacing_line = if pacing_factor >= 1.2 {
        "叙事节奏：快。减少环境描写和冗余叙述，增加动作和对话，快速推进情节。"
    } else if pacing_factor <= 0.8 {
        "叙事节奏：慢。允许细腻的环境描写和心理刻画，注重氛围营造。"
    } else {
        "叙事节奏：均衡。动作与描写交替，保持适度的推进速度。"
    };
    format!("{}\n{}", conflict_line, pacing_line)
}

/// 渲染追读力债务 + 本章追读力目标段落，供 TimeSliced 路径消费 executor
/// 注入的追读力参数（原仅 Full 路径消费，service.rs:2041-2111）。
///
/// 有资产才渲染：`chase_debt_count` 解析为 0 或缺失时跳过债务段；
/// `reading_power_hook_type` 缺失时跳过目标段；两段皆跳过返回 None。
pub(crate) fn render_chase_debt_and_reading_goal(
    pool: &crate::db::DbPool,
    params: &std::collections::HashMap<String, serde_json::Value>,
) -> Option<String> {
    let mut sections = Vec::new();

    // 追读力债务（模板 writer_chase_debt，对齐 Full 路径 service.rs:2041-2067
    // 的变量）
    let debt_count = params
        .get("chase_debt_count")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    if debt_count > 0 {
        let mut debt_vars = std::collections::HashMap::new();
        debt_vars.insert("debt_count".to_string(), debt_count.to_string());
        debt_vars.insert(
            "debts".to_string(),
            params
                .get("chase_debts")
                .and_then(|v| v.as_str())
                .unwrap_or("无")
                .to_string(),
        );
        let tpl = crate::prompts::registry::resolve_prompt(pool, "writer_chase_debt")
            .ok()
            .or_else(|| crate::prompts::registry::resolve_prompt_default("writer_chase_debt"));
        if let Some(tpl) = tpl {
            let rendered =
                crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &debt_vars);
            if !rendered.trim().is_empty() {
                sections.push(rendered);
            }
        }
    }

    // 本章追读力目标（模板 writer_reading_power_goal，对齐 service.rs:2069-2111
    // 的变量）
    if let Some(hook_type) = params
        .get("reading_power_hook_type")
        .and_then(|v| v.as_str())
    {
        let mut goal_vars = std::collections::HashMap::new();
        goal_vars.insert("hook_type".to_string(), hook_type.to_string());
        goal_vars.insert(
            "hook_strength".to_string(),
            params
                .get("reading_power_hook_strength")
                .and_then(|v| v.as_str())
                .unwrap_or("medium")
                .to_string(),
        );
        goal_vars.insert(
            "foreshadowing_list".to_string(),
            params
                .get("reading_power_foreshadowing_list")
                .and_then(|v| v.as_str())
                .unwrap_or("无")
                .to_string(),
        );
        goal_vars.insert(
            "micropayoff_count".to_string(),
            params
                .get("reading_power_micropayoff_count")
                .and_then(|v| v.as_str())
                .unwrap_or("1-2")
                .to_string(),
        );
        let tpl = crate::prompts::registry::resolve_prompt(pool, "writer_reading_power_goal")
            .ok()
            .or_else(|| {
                crate::prompts::registry::resolve_prompt_default("writer_reading_power_goal")
            });
        if let Some(tpl) = tpl {
            let rendered =
                crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &goal_vars);
            if !rendered.trim().is_empty() {
                sections.push(rendered);
            }
        }
    }

    if sections.is_empty() {
        None
    } else {
        Some(sections.join("\n\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        connection::create_test_pool,
        repositories::{CharacterRepository, SceneRepository, SceneUpdate, StoryRepository},
        CharacterConflict, CreateCharacterRequest, CreateStoryRequest,
    };

    fn block_on<F>(f: F) -> F::Output
    where
        F: std::future::Future,
    {
        tokio::runtime::Runtime::new().unwrap().block_on(f)
    }

    fn seed_story(pool: &crate::db::DbPool) -> crate::db::Story {
        StoryRepository::new(pool.clone())
            .create(CreateStoryRequest {
                title: "测试故事".to_string(),
                description: None,
                genre: Some("奇幻".to_string()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .unwrap()
    }

    /// 测试辅助：调用方一次性加载快照的等价操作。
    fn load_snapshot(
        pool: &crate::db::DbPool,
        story_id: &str,
    ) -> crate::canonical_state::CanonicalStateSnapshot {
        crate::canonical_state::CanonicalStateManager::new(pool.clone())
            .get_snapshot_sync(story_id)
            .expect("测试快照应加载成功")
    }

    // ---- format_active_conflicts ----

    #[test]
    fn format_active_conflicts_with_conflicts() {
        let pool = create_test_pool().unwrap();
        let story = seed_story(&pool);
        let scene_repo = SceneRepository::new(pool.clone());
        let scene = scene_repo.create(&story.id, 1, Some("第一章")).unwrap();
        scene_repo
            .update(
                &scene.id,
                &SceneUpdate {
                    character_conflicts: Some(vec![CharacterConflict {
                        character_a_id: "张三".to_string(),
                        character_b_id: "李四".to_string(),
                        conflict_nature: "杀父之仇".to_string(),
                        stakes: "家族存亡".to_string(),
                    }]),
                    ..Default::default()
                },
            )
            .unwrap();
        let snapshot = load_snapshot(&pool, &story.id);
        let text = format_active_conflicts(&snapshot, 600).expect("有资产应输出段落");
        assert!(text.contains("【当前活跃冲突】"));
        assert!(text.contains("杀父之仇"));
        assert!(text.contains("家族存亡"));
        assert!(text.contains("张三"));
    }

    #[test]
    fn format_active_conflicts_empty_returns_none() {
        let pool = create_test_pool().unwrap();
        let story = seed_story(&pool);
        let snapshot = load_snapshot(&pool, &story.id);
        assert!(format_active_conflicts(&snapshot, 600).is_none());
        // 故事不存在时快照聚合失败——加载失败语义已上移到调用方
        assert!(
            crate::canonical_state::CanonicalStateManager::new(pool.clone())
                .get_snapshot_sync("no-such-story")
                .is_err()
        );
    }

    // ---- format_character_goals ----

    #[test]
    fn format_character_goals_with_states() {
        let pool = create_test_pool().unwrap();
        let story = seed_story(&pool);
        let char_repo = CharacterRepository::new(pool.clone());
        let character = char_repo
            .create(CreateCharacterRequest {
                story_id: story.id.clone(),
                name: "张三".to_string(),
                background: Some("主角".to_string()),
                personality: None,
                goals: None,
                appearance: None,
                gender: None,
                age: None,
                source: None,
                is_auto_generated: None,
            })
            .unwrap();
        let manager = crate::canonical_state::CanonicalStateManager::new(pool.clone());
        block_on(manager.update_character_state(
            &story.id,
            &character.id,
            crate::canonical_state::CharacterStateSnapshot {
                character_id: character.id.clone(),
                name: "张三".to_string(),
                current_location: Some("京城".to_string()),
                current_emotion: None,
                active_goal: Some("复仇".to_string()),
                secrets_known: vec![],
                secrets_unknown: vec!["身世之谜".to_string()],
                arc_progress: 0.5,
            },
        ))
        .unwrap();
        let snapshot = load_snapshot(&pool, &story.id);
        let text = format_character_goals(&snapshot, 200).expect("有资产应输出段落");
        assert!(text.contains("【角色当前状态】"));
        assert!(text.contains("目标: 复仇"));
        assert!(text.contains("未知秘密: 身世之谜"));
        assert!(text.contains("弧光进度: 50%"));
    }

    #[test]
    fn format_character_goals_empty_returns_none() {
        let pool = create_test_pool().unwrap();
        let story = seed_story(&pool);
        // 故事无角色 → character_states 为空 → None
        let snapshot = load_snapshot(&pool, &story.id);
        assert!(format_character_goals(&snapshot, 200).is_none());
    }

    // ---- format_genre_reference_tables ----

    fn test_profile(
        reference_tables: Option<String>,
        typical_structure: Option<String>,
    ) -> crate::db::GenreProfile {
        crate::db::GenreProfile {
            id: "gp1".to_string(),
            genre_name: "玄幻".to_string(),
            canonical_name: "Fantasy".to_string(),
            aliases_json: None,
            core_tone: None,
            pacing_strategy: None,
            anti_patterns_json: None,
            reference_tables_json: reference_tables,
            typical_structure_json: typical_structure,
            reader_promise: None,
            recommended_style_dna_ids: None,
            recommended_methodology_id: None,
            recommended_skill_ids: None,
            min_quality_tier: None,
            is_builtin: true,
            created_at: chrono::Local::now(),
        }
    }

    #[test]
    fn format_genre_reference_tables_with_assets() {
        let profile = test_profile(
            Some("境界体系：炼气-筑基-金丹".to_string()),
            Some("起承转合四幕".to_string()),
        );
        let text = format_genre_reference_tables(&profile, 800).expect("有资产应输出段落");
        assert!(text.contains("元素参考表："));
        assert!(text.contains("炼气-筑基-金丹"));
        assert!(text.contains("典型结构参考："));
        assert!(text.contains("起承转合四幕"));
    }

    #[test]
    fn format_genre_reference_tables_empty_returns_none() {
        let profile = test_profile(None, None);
        assert!(format_genre_reference_tables(&profile, 800).is_none());
        let blank = test_profile(Some("  ".to_string()), None);
        assert!(format_genre_reference_tables(&blank, 800).is_none());
    }

    // ---- writing_constraints_semantic_text ----

    #[test]
    fn writing_constraints_semantic_conflict_tiers() {
        assert!(writing_constraints_semantic_text(90.0, 1.0).contains("冲突强度：极高"));
        assert!(writing_constraints_semantic_text(70.0, 1.0).contains("冲突强度：高。"));
        assert!(writing_constraints_semantic_text(50.0, 1.0).contains("冲突强度：中等"));
        assert!(writing_constraints_semantic_text(30.0, 1.0).contains("冲突强度：低。"));
        assert!(writing_constraints_semantic_text(10.0, 1.0).contains("冲突强度：极低"));
    }

    #[test]
    fn writing_constraints_semantic_pacing_tiers() {
        assert!(writing_constraints_semantic_text(50.0, 1.5).contains("叙事节奏：快"));
        assert!(writing_constraints_semantic_text(50.0, 0.5).contains("叙事节奏：慢"));
        assert!(writing_constraints_semantic_text(50.0, 1.0).contains("叙事节奏：均衡"));
    }

    #[test]
    fn pace_to_factor_mapping() {
        assert_eq!(pace_to_factor("fast"), 1.5);
        assert_eq!(pace_to_factor("slow"), 0.5);
        assert_eq!(pace_to_factor("normal"), 1.0);
    }

    // ---- render_chase_debt_and_reading_goal ----

    #[test]
    fn render_chase_debt_and_goal_with_assets() {
        let pool = create_test_pool().unwrap();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "chase_debt_count".to_string(),
            serde_json::Value::String("2".to_string()),
        );
        params.insert(
            "chase_debts".to_string(),
            serde_json::Value::String(
                "1. 类型：钩子，当前金额：3.0，到期章节：5，来源章节：2".to_string(),
            ),
        );
        params.insert(
            "reading_power_hook_type".to_string(),
            serde_json::Value::String("身份悬念".to_string()),
        );
        params.insert(
            "reading_power_hook_strength".to_string(),
            serde_json::Value::String("high".to_string()),
        );
        params.insert(
            "reading_power_foreshadowing_list".to_string(),
            serde_json::Value::String("身世之谜".to_string()),
        );
        params.insert(
            "reading_power_micropayoff_count".to_string(),
            serde_json::Value::String("2".to_string()),
        );
        let text = render_chase_debt_and_reading_goal(&pool, &params).expect("有资产应渲染");
        assert!(text.contains("【追读力债务】"));
        assert!(text.contains("当前有 2 条待偿还的追读力债务"));
        assert!(text.contains("到期章节：5"));
        assert!(text.contains("【本章追读力目标】"));
        assert!(text.contains("身份悬念"));
        assert!(text.contains("身世之谜"));
    }

    #[test]
    fn render_chase_debt_and_goal_empty_params_returns_none() {
        let pool = create_test_pool().unwrap();
        let params = std::collections::HashMap::new();
        assert!(render_chase_debt_and_reading_goal(&pool, &params).is_none());
        // debt_count 为 "0"（executor 缺省注入值）时跳过债务段
        let mut zero = std::collections::HashMap::new();
        zero.insert(
            "chase_debt_count".to_string(),
            serde_json::Value::String("0".to_string()),
        );
        assert!(render_chase_debt_and_reading_goal(&pool, &zero).is_none());
    }

    #[test]
    fn render_chase_debt_and_goal_only_hook_type() {
        // 仅 hook_type 无债务：只渲染目标段
        let pool = create_test_pool().unwrap();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "reading_power_hook_type".to_string(),
            serde_json::Value::String("（延续）".to_string()),
        );
        let text = render_chase_debt_and_reading_goal(&pool, &params).expect("应有目标段");
        assert!(!text.contains("【追读力债务】"));
        assert!(text.contains("【本章追读力目标】"));
        assert!(text.contains("（延续）"));
    }
}
