//! 自动划分章节（v0.26.57）
//!
//! 设置驱动：
//! - `chapter_split_mode =
//!   "word_count"`：当前章字数超过阈值时，在段落边界切出下一章
//! - `chapter_split_mode = "plot"`：在情节/场景转换点切章（启发式，无额外 LLM）
//!
//! 字数单位：中文「字」（`TextUtils::chinese_word_count`）。
//! 字数上限留空 → 使用 `DEFAULT_CHAPTER_SPLIT_MAX_CHARS`（3000）。
//!
//! 触发：场景内容保存后 30s 空闲（与 auto_commit
//! 同窗口），且仅处理故事最新一章， 避免改写中间章节时重排后续章号。
//!
//! 单次触发循环切分：一次触发内反复切分最新章（切出的新章随即成为最新章），
//! 直到最新章字数 ≤ 阈值或找不到切分点为止——粘贴恢复的大段正文（如
//! 9 万字）可在一次触发内分章完成。安全上限 `MAX_SPLIT_ITERATIONS`（50），
//! 且单轮无进展（新最新章字数不小于上一轮）时中断并告警。

use tauri::AppHandle;

use crate::{
    config::{AppConfig, DEFAULT_CHAPTER_SPLIT_MAX_CHARS},
    db::{
        ChapterRepository, CreateChapterRequest, DbPool, SceneRepository, SceneUpdate,
        StoryContractRepository,
    },
    domain::contracts::ChapterContract,
    state_sync::StateSync,
    utils::text::TextUtils,
};

/// 划分方式（与 AppConfig.chapter_split_mode 对齐）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterSplitMode {
    WordCount,
    Plot,
}

impl ChapterSplitMode {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "plot" | "情节" => Self::Plot,
            _ => Self::WordCount,
        }
    }
}

/// 解析有效字数上限：`None` / `0` / 负数 → 自动默认 3000 字。
pub fn resolve_max_chars(configured: Option<i32>) -> usize {
    match configured {
        Some(n) if n > 0 => n as usize,
        _ => DEFAULT_CHAPTER_SPLIT_MAX_CHARS,
    }
}

/// 纯函数：在内容中找切分点（字节偏移）。找不到则返回 None。
///
/// - WordCount：超过阈值后，优先在最近段落边界切；找不到则在阈值附近的句末切。
/// - Plot：在超过「半阈值」后的情节边界切（空行 /
///   时间地点转换词）；无边界则回退字数切。
pub fn find_split_offset(content: &str, mode: ChapterSplitMode, max_chars: usize) -> Option<usize> {
    let total = TextUtils::chinese_word_count(content);
    if total <= max_chars || content.is_empty() {
        return None;
    }

    match mode {
        ChapterSplitMode::WordCount => find_word_count_split(content, max_chars),
        ChapterSplitMode::Plot => find_plot_split(content, max_chars)
            .or_else(|| find_word_count_split(content, max_chars)),
    }
}

fn char_offset_to_byte(content: &str, char_offset: usize) -> usize {
    content
        .char_indices()
        .nth(char_offset)
        .map(|(i, _)| i)
        .unwrap_or(content.len())
}

/// 将「字」计数映射到近似字符偏移（中文 1 字≈1 char；英文词按字符扫描近似）。
fn approx_char_index_for_word_budget(content: &str, budget: usize) -> usize {
    let mut counted = 0usize;
    let mut last_idx = 0usize;
    let mut in_english = false;

    for (i, ch) in content.char_indices() {
        last_idx = i + ch.len_utf8();
        if matches!(ch, '\u{4e00}'..='\u{9fff}') {
            in_english = false;
            counted += 1;
            if counted >= budget {
                return i + ch.len_utf8();
            }
        } else if ch.is_ascii_alphabetic() {
            if !in_english {
                in_english = true;
                counted += 1;
                if counted >= budget {
                    // 吃完当前英文词
                    let rest = &content[i..];
                    let word_end = rest
                        .find(|c: char| !c.is_ascii_alphabetic())
                        .map(|o| i + o)
                        .unwrap_or(content.len());
                    return word_end;
                }
            }
        } else {
            in_english = false;
        }
    }
    last_idx
}

fn find_word_count_split(content: &str, max_chars: usize) -> Option<usize> {
    let soft_end = approx_char_index_for_word_budget(content, max_chars);
    if soft_end == 0 || soft_end >= content.len() {
        return None;
    }

    // 优先：soft_end 之前最近的双换行 / 单换行段落边界
    let head = &content[..soft_end];
    if let Some(rel) = head.rfind("\n\n") {
        let at = rel + 2;
        if at > 0 && at < content.len() {
            return Some(at);
        }
    }
    if let Some(rel) = head.rfind('\n') {
        let at = rel + 1;
        if at > 0 && at < content.len() {
            return Some(at);
        }
    }

    // 次选：句末标点
    for (i, ch) in head.char_indices().rev() {
        if matches!(ch, '。' | '！' | '？' | '.' | '!' | '?') {
            let at = i + ch.len_utf8();
            if at > 0 && at < content.len() {
                return Some(at);
            }
        }
    }

    // 兜底：soft_end（保证在字符边界）
    let at = char_offset_to_byte(content, content[..soft_end].chars().count());
    if at > 0 && at < content.len() {
        Some(at)
    } else {
        None
    }
}

fn find_plot_split(content: &str, max_chars: usize) -> Option<usize> {
    let min_keep = (max_chars / 2).max(500);
    let min_byte = approx_char_index_for_word_budget(content, min_keep);
    let boundaries = detect_plot_boundaries(content);
    // 选第一个落在 [min_byte, soft_end*1.5] 的边界；否则选 min_byte 之后最近边界
    let soft_end = approx_char_index_for_word_budget(content, max_chars);
    let upper = approx_char_index_for_word_budget(content, max_chars.saturating_mul(3) / 2)
        .max(soft_end + 1);

    let in_window: Vec<usize> = boundaries
        .into_iter()
        .filter(|&b| b >= min_byte && b < content.len() && b > 0)
        .collect();

    in_window
        .iter()
        .copied()
        .find(|&b| b <= upper)
        .or_else(|| in_window.into_iter().next())
}

/// 情节边界启发式（自包含，避免 story_system → book_deconstruction 依赖）。
fn detect_plot_boundaries(content: &str) -> Vec<usize> {
    let mut boundaries = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let time_markers = [
        "三天后",
        "一周后",
        "一个月后",
        "一年后",
        "几年后",
        "数年后",
        "第二天",
        "次日",
        "当晚",
        "翌日",
        "翌晨",
        "与此同时",
        "同一时间",
        "不久",
        "过了一会儿",
        "片刻之后",
        "数日后",
        "几日后",
        "次日清晨",
    ];
    let location_markers = [
        "回到",
        "来到",
        "抵达",
        "进入",
        "离开",
        "走出",
        "走进",
        "另一边",
    ];

    let mut empty_line_count = 0usize;
    let mut last_boundary_line = 0usize;
    let mut byte_pos = 0usize;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_bytes = line.len();
        let trimmed = line.trim();

        if trimmed.is_empty() {
            empty_line_count += 1;
            if empty_line_count >= 2 && line_idx.saturating_sub(last_boundary_line) > 5 {
                let pos = byte_pos;
                if boundaries.last().map(|b| pos > *b + 20).unwrap_or(true) {
                    boundaries.push(pos);
                    last_boundary_line = line_idx;
                }
            }
            // +1 for '\n' except possibly last line — lines() strips newlines;
            // reconstruct: after each line except we track via join length.
            byte_pos += line_bytes;
            if line_idx + 1 < lines.len() {
                byte_pos += 1; // newline
            }
            continue;
        }

        empty_line_count = 0;

        if line_idx.saturating_sub(last_boundary_line) > 10 {
            let is_time = time_markers.iter().any(|m| trimmed.starts_with(m));
            let is_loc = location_markers.iter().any(|m| trimmed.starts_with(m));
            if is_time || is_loc {
                if boundaries
                    .last()
                    .map(|b| byte_pos > *b + 20)
                    .unwrap_or(true)
                {
                    boundaries.push(byte_pos);
                    last_boundary_line = line_idx;
                }
            }
        }

        byte_pos += line_bytes;
        if line_idx + 1 < lines.len() {
            byte_pos += 1;
        }
    }

    boundaries
}

/// 切分结果（纯数据，便于单测）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterSplitPlan {
    pub keep: String,
    pub overflow: String,
    pub split_offset: usize,
}

pub fn plan_split(
    content: &str,
    mode: ChapterSplitMode,
    max_chars: usize,
) -> Option<ChapterSplitPlan> {
    let offset = find_split_offset(content, mode, max_chars)?;
    if offset == 0 || offset >= content.len() {
        return None;
    }
    let keep = content[..offset].to_string();
    let overflow = content[offset..].trim_start().to_string();
    if overflow.is_empty() || TextUtils::chinese_word_count(&keep) == 0 {
        return None;
    }
    Some(ChapterSplitPlan {
        keep,
        overflow,
        split_offset: offset,
    })
}

/// 新章标题：优先取故事最新章节合约（CHAPTER）的 `goal`
/// （截断至 30 个字符、按字符边界切），无合约 / 空 goal 时回退 `第{N}章`。
fn resolve_new_chapter_title(pool: &DbPool, story_id: &str, chapter_number: i32) -> String {
    title_from_goal(
        latest_chapter_contract_goal(pool, story_id).as_deref(),
        chapter_number,
    )
}

/// 纯函数：由合约 goal 推导章节标题（可单测）。
fn title_from_goal(goal: Option<&str>, chapter_number: i32) -> String {
    const MAX_TITLE_CHARS: usize = 30;
    match goal.map(str::trim).filter(|g| !g.is_empty()) {
        Some(g) => g.chars().take(MAX_TITLE_CHARS).collect(),
        None => format!("第{}章", chapter_number),
    }
}

/// 查询故事最新（chapter_number 最大）章节合约的 goal；查询失败时告警并返回
/// None。
fn latest_chapter_contract_goal(pool: &DbPool, story_id: &str) -> Option<String> {
    let repo = StoryContractRepository::new(pool.clone());
    let contracts = match repo.get_by_story(story_id) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[ChapterSplitter] 查询章节合约失败，回退默认命名: {}", e);
            return None;
        }
    };
    contracts
        .iter()
        .filter(|c| c.contract_type == "CHAPTER")
        .filter_map(|c| serde_json::from_str::<ChapterContract>(&c.contract_json).ok())
        .max_by_key(|c| c.chapter_number)
        .map(|c| c.chapter_directive.goal)
}

/// 单次触发内循环切分的安全上限。
const MAX_SPLIT_ITERATIONS: usize = 50;

/// 一次切分的结果（供事件发射与返回值使用）。
struct SplitOutcome {
    /// 被截断的原章 id（切分前的最新章）
    old_chapter_id: String,
    /// 原章第一个场景 id（用于 sceneUpdated 事件）
    scene_id: String,
    /// 新切出的章 id
    new_chapter_id: String,
    /// 新章标题
    new_chapter_title: Option<String>,
}

/// 对指定章（须为故事最新章）执行一次切分；无需/不可切分时返回 `Ok(None)`。
///
/// 不发射事件（便于单测）；事件由调用方按 `SplitOutcome` 补发。
fn split_latest_chapter_once(
    pool: &DbPool,
    story_id: &str,
    chapter_id: &str,
    mode: ChapterSplitMode,
    max_chars: usize,
) -> Result<Option<SplitOutcome>, String> {
    let chapter_repo = ChapterRepository::new(pool.clone());
    let scene_repo = SceneRepository::new(pool.clone());

    let chapters = chapter_repo
        .get_by_story(story_id)
        .map_err(|e| e.to_string())?;
    let Some(latest) = chapters.iter().max_by_key(|c| c.chapter_number) else {
        return Ok(None);
    };
    // 仅最新章可自动切分，避免中间章改写触发重排
    if latest.id != chapter_id {
        return Ok(None);
    }

    let content = chapter_repo
        .get_content(chapter_id)
        .map_err(|e| e.to_string())?;
    let Some(plan) = plan_split(&content, mode, max_chars) else {
        return Ok(None);
    };

    let scenes = scene_repo
        .get_by_chapter(chapter_id)
        .map_err(|e| e.to_string())?;
    let Some(scene) = scenes.first() else {
        return Ok(None);
    };

    let keep_wc = TextUtils::chinese_word_count(&plan.keep) as i32;
    scene_repo
        .update(
            &scene.id,
            &SceneUpdate {
                content: Some(plan.keep.clone()),
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())?;
    chapter_repo
        .update(chapter_id, None, None, Some(keep_wc))
        .map_err(|e| e.to_string())?;

    let next_number = latest.chapter_number + 1;
    // 若下一章号已存在则放弃（并发/手工建章）
    if chapters.iter().any(|c| c.chapter_number == next_number) {
        log::warn!(
            "[ChapterSplitter] next chapter {} already exists for story {}, abort split",
            next_number,
            story_id
        );
        return Ok(None);
    }

    let new_title = resolve_new_chapter_title(pool, story_id, next_number);
    let new_chapter = chapter_repo
        .create(CreateChapterRequest {
            story_id: story_id.to_string(),
            chapter_number: next_number,
            title: Some(new_title),
            outline: None,
            content: Some(plan.overflow),
        })
        .map_err(|e| e.to_string())?;

    log::info!(
        "[ChapterSplitter] split chapter {} → new {} (mode={:?}, max_chars={})",
        chapter_id,
        new_chapter.id,
        mode,
        max_chars
    );

    Ok(Some(SplitOutcome {
        old_chapter_id: chapter_id.to_string(),
        scene_id: scene.id.clone(),
        new_chapter_id: new_chapter.id,
        new_chapter_title: new_chapter.title,
    }))
}

/// 循环切分故事最新章，直到其字数 ≤ 阈值或找不到切分点。
///
/// 每轮重新读取最新章（切分后新章成为最新章）。安全上限
/// `MAX_SPLIT_ITERATIONS`；单轮无进展（新最新章字数不小于上一轮）时中断并告警。
/// 返回各次切分结果（按切分顺序）。
fn split_latest_until_within_threshold(
    pool: &DbPool,
    story_id: &str,
    chapter_id: &str,
    mode: ChapterSplitMode,
    max_chars: usize,
) -> Result<Vec<SplitOutcome>, String> {
    let chapter_repo = ChapterRepository::new(pool.clone());
    let mut outcomes = Vec::new();
    let mut current_id = chapter_id.to_string();
    let mut prev_len: Option<usize> = None;

    for _ in 0..MAX_SPLIT_ITERATIONS {
        let content = chapter_repo
            .get_content(&current_id)
            .map_err(|e| e.to_string())?;
        let len = TextUtils::chinese_word_count(&content);

        // 安全网：新最新章字数不小于上一轮 → 无进展，防死循环
        if let Some(prev) = prev_len {
            if len >= prev {
                log::warn!(
                    "[ChapterSplitter] split loop made no progress (latest chapter {} len {} >= prev {}), stop",
                    current_id,
                    len,
                    prev
                );
                break;
            }
        }
        if len <= max_chars {
            break;
        }
        prev_len = Some(len);

        let Some(outcome) =
            split_latest_chapter_once(pool, story_id, &current_id, mode, max_chars)?
        else {
            break;
        };
        current_id = outcome.new_chapter_id.clone();
        outcomes.push(outcome);
    }

    if outcomes.len() >= MAX_SPLIT_ITERATIONS {
        log::warn!(
            "[ChapterSplitter] split loop reached max iterations ({}), stop",
            MAX_SPLIT_ITERATIONS
        );
    }

    Ok(outcomes)
}

/// 对故事最新一章执行自动划分（若需要）。
///
/// 单次触发内循环切分：每次切出新章后新章随即成为最新章，继续检查并切分，
/// 直到最新章字数 ≤ 阈值或找不到切分点（安全上限 `MAX_SPLIT_ITERATIONS`，
/// 单轮无进展时中断）。每次切分各自发射
/// chapterUpdated/sceneUpdated/chapterCreated 事件。
///
/// 返回 `Ok(Some(last_new_chapter_id))`：多次切分时返回**最后一次**切出的章
/// id（当前唯一调用方 scene_service 仅用于日志）；`Ok(None)` 表示无需切分。
pub fn maybe_split_latest_chapter(
    pool: &DbPool,
    app_handle: &AppHandle,
    story_id: &str,
    chapter_id: &str,
    config: &AppConfig,
) -> Result<Option<String>, String> {
    let mode = ChapterSplitMode::parse(&config.chapter_split_mode);
    let max_chars = resolve_max_chars(config.chapter_split_max_chars);

    let outcomes =
        split_latest_until_within_threshold(pool, story_id, chapter_id, mode, max_chars)?;

    for outcome in &outcomes {
        let _ =
            StateSync::emit_chapter_updated(app_handle, &outcome.old_chapter_id, None, story_id);
        let _ = StateSync::emit_scene_updated(app_handle, story_id, &outcome.scene_id, None, true);
        let _ = StateSync::emit_chapter_created(
            app_handle,
            story_id,
            &outcome.new_chapter_id,
            outcome.new_chapter_title.as_deref(),
            Some(outcome.old_chapter_id.as_str()),
        );
    }

    Ok(outcomes.last().map(|o| o.new_chapter_id.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chinese_repeat(ch: char, n: usize) -> String {
        std::iter::repeat(ch).take(n).collect()
    }

    #[test]
    fn resolve_max_chars_empty_uses_default() {
        assert_eq!(resolve_max_chars(None), DEFAULT_CHAPTER_SPLIT_MAX_CHARS);
        assert_eq!(resolve_max_chars(Some(0)), DEFAULT_CHAPTER_SPLIT_MAX_CHARS);
        assert_eq!(resolve_max_chars(Some(-1)), DEFAULT_CHAPTER_SPLIT_MAX_CHARS);
        assert_eq!(resolve_max_chars(Some(2500)), 2500);
    }

    #[test]
    fn mode_parse_defaults_to_word_count() {
        assert_eq!(
            ChapterSplitMode::parse("word_count"),
            ChapterSplitMode::WordCount
        );
        assert_eq!(ChapterSplitMode::parse("plot"), ChapterSplitMode::Plot);
        assert_eq!(ChapterSplitMode::parse("情节"), ChapterSplitMode::Plot);
        assert_eq!(ChapterSplitMode::parse(""), ChapterSplitMode::WordCount);
        assert_eq!(
            ChapterSplitMode::parse("weird"),
            ChapterSplitMode::WordCount
        );
    }

    #[test]
    fn word_count_below_threshold_no_split() {
        let text = chinese_repeat('测', 100);
        assert!(plan_split(&text, ChapterSplitMode::WordCount, 3000).is_none());
    }

    #[test]
    fn word_count_over_threshold_splits_at_paragraph() {
        let para1 = chinese_repeat('甲', 2000);
        let para2 = chinese_repeat('乙', 2000);
        let text = format!("{}\n\n{}", para1, para2);
        let plan = plan_split(&text, ChapterSplitMode::WordCount, 3000).expect("should split");
        assert!(plan.keep.contains('甲'));
        assert!(plan.overflow.contains('乙'));
        assert!(TextUtils::chinese_word_count(&plan.keep) <= 3000);
        assert!(!plan.overflow.is_empty());
    }

    #[test]
    fn plot_mode_prefers_transition_marker() {
        let before = chinese_repeat('前', 800);
        let after = chinese_repeat('后', 800);
        let text = format!("{}\n\n第二天\n{}", before, after);
        let plan = plan_split(&text, ChapterSplitMode::Plot, 1000).expect("plot split");
        assert!(
            plan.overflow.starts_with("第二天"),
            "overflow should start at plot boundary, got prefix: {:?}",
            plan.overflow.chars().take(12).collect::<String>()
        );
        assert!(plan.keep.contains('前'));
    }

    #[test]
    fn plot_mode_does_not_use_word_threshold_incorrectly_when_under() {
        // 情节模式：总字数未超阈值时不切
        let text = format!(
            "{}\n\n第二天\n{}",
            chinese_repeat('前', 100),
            chinese_repeat('后', 100)
        );
        assert!(plan_split(&text, ChapterSplitMode::Plot, 3000).is_none());
    }

    #[test]
    fn word_count_mode_ignores_plot_markers_when_under_threshold() {
        let text = format!("开头\n\n第二天\n{}", chinese_repeat('续', 50));
        assert!(plan_split(&text, ChapterSplitMode::WordCount, 3000).is_none());
    }

    #[test]
    fn title_from_goal_uses_goal_when_present() {
        assert_eq!(
            title_from_goal(Some("潜入敌营救出同伴"), 3),
            "潜入敌营救出同伴"
        );
    }

    #[test]
    fn title_from_goal_truncates_at_char_boundary() {
        let long_goal = chinese_repeat('长', 40);
        let title = title_from_goal(Some(&long_goal), 3);
        assert_eq!(title.chars().count(), 30);
    }

    #[test]
    fn title_from_goal_falls_back_when_missing_or_blank() {
        assert_eq!(title_from_goal(None, 7), "第7章");
        assert_eq!(title_from_goal(Some("   "), 7), "第7章");
    }

    // ==================== 循环切分（DB 集成） ====================

    use crate::db::{create_test_pool, CreateStoryRequest, StoryRepository};

    /// 种一个故事 + 第一章（内容写入其场景），返回 (story_id, chapter_id)。
    fn seed_story_with_chapter(pool: &DbPool, content: &str) -> (String, String) {
        let story_repo = StoryRepository::new(pool.clone());
        let story = story_repo
            .create(CreateStoryRequest {
                title: "循环分章测试".to_string(),
                description: None,
                genre: None,
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .unwrap();
        let chapter_repo = ChapterRepository::new(pool.clone());
        let chapter = chapter_repo
            .create(CreateChapterRequest {
                story_id: story.id.clone(),
                chapter_number: 1,
                title: Some("第一章".to_string()),
                outline: None,
                content: Some(content.to_string()),
            })
            .unwrap();
        (story.id, chapter.id)
    }

    #[test]
    fn bulk_content_splits_repeatedly_in_one_trigger() {
        let pool = create_test_pool().unwrap();
        // 5 段 × 1800 字 = 9000 字，段落边界保证 word_count 模式每轮可切
        let para = chinese_repeat('文', 1800);
        let content = [para.as_str(); 5].join("\n\n");
        let (story_id, chapter_id) = seed_story_with_chapter(&pool, &content);

        let outcomes = split_latest_until_within_threshold(
            &pool,
            &story_id,
            &chapter_id,
            ChapterSplitMode::WordCount,
            3000,
        )
        .unwrap();

        assert!(
            outcomes.len() >= 2,
            "9000 字应一次触发切出多章, got {}",
            outcomes.len()
        );

        let chapter_repo = ChapterRepository::new(pool.clone());
        let chapters = chapter_repo.get_by_story(&story_id).unwrap();
        assert_eq!(chapters.len(), 1 + outcomes.len());

        // 每章字数都在阈值内（最后一章是剩余部分）
        for ch in &chapters {
            let c = chapter_repo.get_content(&ch.id).unwrap();
            assert!(
                TextUtils::chinese_word_count(&c) <= 3000,
                "chapter {} exceeds threshold: {}",
                ch.chapter_number,
                TextUtils::chinese_word_count(&c)
            );
        }

        // 章号连续 1..=N
        let mut numbers: Vec<i32> = chapters.iter().map(|c| c.chapter_number).collect();
        numbers.sort_unstable();
        assert_eq!(numbers, (1..=chapters.len() as i32).collect::<Vec<_>>());
    }

    #[test]
    fn boundary_less_content_terminates_without_runaway() {
        let pool = create_test_pool().unwrap();
        // 无段落 / 句末边界：word_count 回退到阈值处硬切，每轮新章必缩短，
        // 循环必然终止且章数有界
        let content = chinese_repeat('密', 9000);
        let (story_id, chapter_id) = seed_story_with_chapter(&pool, &content);

        let outcomes = split_latest_until_within_threshold(
            &pool,
            &story_id,
            &chapter_id,
            ChapterSplitMode::WordCount,
            3000,
        )
        .unwrap();

        // 9000 / 3000 → 2 次切分、3 章，远小于安全上限
        assert!(
            outcomes.len() <= 3,
            "boundary-less content should terminate quickly, got {}",
            outcomes.len()
        );
        let chapter_repo = ChapterRepository::new(pool.clone());
        let chapters = chapter_repo.get_by_story(&story_id).unwrap();
        assert_eq!(chapters.len(), 1 + outcomes.len());
        assert!((chapters.len() as usize) < MAX_SPLIT_ITERATIONS);
    }

    #[test]
    fn unsplittable_content_exits_loop_without_new_chapter() {
        let pool = create_test_pool().unwrap();
        // 开头即段落边界 → keep 为 0 字，plan_split 拒绝 → 无切分、循环退出
        let content = format!("\n\n{}", chinese_repeat('孤', 4000));
        let (story_id, chapter_id) = seed_story_with_chapter(&pool, &content);

        let outcomes = split_latest_until_within_threshold(
            &pool,
            &story_id,
            &chapter_id,
            ChapterSplitMode::WordCount,
            3000,
        )
        .unwrap();

        assert!(outcomes.is_empty());
        let chapter_repo = ChapterRepository::new(pool.clone());
        assert_eq!(chapter_repo.get_by_story(&story_id).unwrap().len(), 1);
    }

    #[test]
    fn within_threshold_content_loop_is_noop() {
        let pool = create_test_pool().unwrap();
        let content = chinese_repeat('短', 1000);
        let (story_id, chapter_id) = seed_story_with_chapter(&pool, &content);

        let outcomes = split_latest_until_within_threshold(
            &pool,
            &story_id,
            &chapter_id,
            ChapterSplitMode::WordCount,
            3000,
        )
        .unwrap();

        assert!(outcomes.is_empty());
        let chapter_repo = ChapterRepository::new(pool.clone());
        assert_eq!(chapter_repo.get_by_story(&story_id).unwrap().len(), 1);
    }
}
