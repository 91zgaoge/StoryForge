//! 动态资产菜单：Rust 侧粗筛（轮换排除 + 确定性轮转），零 LLM 成本。
//! beat_planner 从 ≤5 个候选中精选 1-2 个，选中 ID 写入资产历史。

use std::collections::HashSet;

use super::read_asset_history;
use crate::db::connection::DbPool;

/// 近 N 条历史内用过的资产排除出候选
const RECENT_EXCLUDE_ENTRIES: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetMenuItem {
    pub id: String,
    pub kind: &'static str,
    /// 一行摘要：「[kind] name——function/payoff/pressure_source（id）」
    pub line: String,
}

/// 从候选中确定性轮转选取 n 个（排除近期用过的；排除后为空则回退全集）
fn pick_rotating<T: Clone>(
    items: Vec<T>,
    id_of: impl Fn(&T) -> &str,
    recent: &HashSet<String>,
    n: usize,
    offset: usize,
) -> Vec<T> {
    let fresh: Vec<T> = items
        .into_iter()
        .filter(|i| !recent.contains(id_of(i)))
        .collect();
    // 注意：回退全集需要原始列表，调用方保证 builtin 库远大于排除集，此处 fresh
    // 为空时直接返回空 （31/21/13 的库，排除上限 5 条历史，实践中不会空）
    let len = fresh.len();
    if len == 0 {
        return Vec::new();
    }
    let start = offset % len;
    (0..len)
        .map(|i| fresh[(start + i) % len].clone())
        .take(n)
        .collect()
}

pub fn build_asset_menu(pool: &DbPool, story_id: &str, chapter_number: i32) -> Vec<AssetMenuItem> {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let history = read_asset_history(&conn, story_id);
    let recent: HashSet<String> = history
        .iter()
        .rev()
        .take(RECENT_EXCLUDE_ENTRIES)
        .flat_map(|e| e.ids.iter().cloned())
        .collect();
    let offset = chapter_number.max(0) as usize;

    let mut menu = Vec::new();

    let cards = crate::creative_engine::beat_cards::builtin_beat_cards();
    for c in pick_rotating(cards, |c| c.id.as_str(), &recent, 2, offset) {
        menu.push(AssetMenuItem {
            line: format!("[桥段卡] {}——{}（{}）", c.name, c.function, c.id),
            id: c.id,
            kind: "桥段卡",
        });
    }

    let engines = crate::creative_engine::story_engines::builtin_story_engines();
    for e in pick_rotating(engines, |e| e.id.as_str(), &recent, 2, offset + 1) {
        menu.push(AssetMenuItem {
            line: format!("[剧情引擎] {}——{}（{}）", e.name, e.payoff, e.id),
            id: e.id,
            kind: "剧情引擎",
        });
    }

    let rels = crate::creative_engine::pressure_relationships::builtin_pressure_relationships();
    for r in pick_rotating(rels, |r| r.id.as_str(), &recent, 1, offset + 2) {
        menu.push(AssetMenuItem {
            line: format!("[高压关系] {}——{}（{}）", r.name, r.pressure_source, r.id),
            id: r.id,
            kind: "高压关系",
        });
    }

    menu
}

/// 渲染为 prompt 菜单段；空菜单返回 None
pub fn render_asset_menu(items: &[AssetMenuItem]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let lines = items
        .iter()
        .enumerate()
        .map(|(i, m)| format!("{}. {}", i + 1, m.line))
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!(
        "【本章可用创作资产菜单】\n{}\n（从中精选 1-2 个融入本章，将其 id 写入输出的 selected_asset_ids；如均不适用可留空数组）",
        lines
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_story(pool: &DbPool) -> String {
        let repo = crate::db::repositories::StoryRepository::new(pool.clone());
        let req = crate::db::repositories::CreateStoryRequest {
            title: "菜单测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        };
        repo.create(req).unwrap().id
    }

    #[test]
    fn menu_has_expected_shape_and_kinds() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        let menu = build_asset_menu(&pool, &sid, 1);
        assert_eq!(menu.len(), 5);
        assert_eq!(menu.iter().filter(|m| m.kind == "桥段卡").count(), 2);
        assert_eq!(menu.iter().filter(|m| m.kind == "剧情引擎").count(), 2);
        assert_eq!(menu.iter().filter(|m| m.kind == "高压关系").count(), 1);
        // 每项都是一行摘要且带 id
        for item in &menu {
            assert!(!item.id.is_empty());
            assert!(item.line.contains(&item.id));
        }
    }

    #[test]
    fn recent_history_is_excluded() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        // 把当前第 1 章会选中的 5 个全部写进历史，下一章必须全换
        let first = build_asset_menu(&pool, &sid, 1);
        let ids: Vec<String> = first.iter().map(|m| m.id.clone()).collect();
        crate::creative_engine::expansion::append_asset_history(&pool, &sid, 1, &ids).unwrap();
        let second = build_asset_menu(&pool, &sid, 2);
        for item in &second {
            assert!(!ids.contains(&item.id), "{} 应被轮换排除", item.id);
        }
    }

    #[test]
    fn menu_is_deterministic_for_same_inputs() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        let a = build_asset_menu(&pool, &sid, 7);
        let b = build_asset_menu(&pool, &sid, 7);
        assert_eq!(
            a.iter().map(|m| &m.id).collect::<Vec<_>>(),
            b.iter().map(|m| &m.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn render_menu_formats_one_liners() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        let menu = build_asset_menu(&pool, &sid, 1);
        let text = render_asset_menu(&menu).unwrap();
        assert!(text.contains("创作资产菜单"));
        assert!(text.contains("桥段卡"));
        assert!(render_asset_menu(&[]).is_none());
    }
}
