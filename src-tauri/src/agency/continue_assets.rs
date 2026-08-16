//! 续写资产按拍筛选。0 LLM、0 I/O。
//! 设计：docs/plans/2026-08-14-continue-prompt-asset-selection-design.md
//! 不改 `WriteTimeBundle::to_prompt()` 全局语义。

use crate::{
    creative_engine::write_time_bundle::extract_redline_text,
    domain::write_time_bundle::{CoreCharacter, WriteTimeBundle},
};

pub const ADMITTED_CAP: usize = 8;
pub const OUTLINE_CHAR_CAP: usize = 1200;
pub const WORLD_CHAR_CAP: usize = 800;
pub const WORLD_CONCEPT_CHAR_CAP: usize = 400;
pub const WORLD_RULE_CAP: usize = 5;
pub const ROSTER_NAME_CAP: usize = 40;
pub const ASSET_CHAR_BUDGET: usize = 6000;
/// 长章双窗：开篇窗口。与 `PRIOR_TAIL_CHAR_CAP` 合计为注入上限。
pub const PRIOR_HEAD_CHAR_CAP: usize = 600;
/// 长章双窗：近文窗口（衔接用）。预算收缩时也优先保住这段。
pub const PRIOR_TAIL_CHAR_CAP: usize = 1800;
/// 节拍卡「谁必须还在场」只看这一拍镜头，不看整段近文。
/// 1800 字会把章前半的人算进本拍，探针再要求增量点齐全员 → 人丢了仍落库。
pub const PRIOR_CAST_CHAR_CAP: usize = 500;
/// 短章整段可进时的上限（开篇+近文）。
pub const PRIOR_PROSE_CHAR_CAP: usize = PRIOR_HEAD_CHAR_CAP + PRIOR_TAIL_CHAR_CAP;
pub const CHAPTER_OUTLINE_CHAR_CAP: usize = 800;
pub const TENSION_ARC_CHAR_CAP: usize = 400;
pub const ROSTER_PREFIX: &str = "本拍未上场（禁止新编下列姓名，亦不得当主角使用）";

pub fn names_in_text(character_names: &[impl AsRef<str>], text: &str) -> Vec<String> {
    match_character_names(character_names, text)
}

pub const CONFLICT_VERBS: &[&str] = &["对峙", "反转", "代价", "冲突", "对打", "逼迫", "加压"];

pub fn has_conflict_verb(text: &str) -> bool {
    CONFLICT_VERBS.iter().any(|v| text.contains(v))
}

/// 2 字名 → 全名 + 阿末字；≥3 字再加末两字。禁止单字。
pub fn aliases_for(name: &str) -> Vec<String> {
    let name = name.trim();
    if name.is_empty() {
        return vec![];
    }
    let chars: Vec<char> = name.chars().collect();
    let mut out = vec![name.to_string()];
    if chars.len() >= 2 {
        if let Some(last) = chars.last() {
            let nick = format!("阿{last}");
            if nick != name {
                out.push(nick);
            }
        }
    }
    if chars.len() >= 3 {
        let last2: String = chars[chars.len() - 2..].iter().collect();
        if last2 != name {
            out.push(last2);
        }
    }
    out.retain(|s| s.chars().count() >= 2);
    out.sort_by_key(|s| std::cmp::Reverse(s.chars().count()));
    out.dedup();
    out
}

/// 最长别名优先。别名若等于另一角色全名，只归属全名角色。
pub fn match_character_names(names: &[impl AsRef<str>], text: &str) -> Vec<String> {
    let canonical: Vec<String> = names
        .iter()
        .map(|n| n.as_ref().trim().to_string())
        .filter(|n| !n.is_empty())
        .collect();
    let exact: std::collections::HashSet<&str> = canonical.iter().map(|s| s.as_str()).collect();
    let mut pairs: Vec<(usize, String, String)> = Vec::new();
    for canon in &canonical {
        for alias in aliases_for(canon) {
            if alias != *canon && exact.contains(alias.as_str()) {
                continue;
            }
            pairs.push((alias.chars().count(), alias, canon.clone()));
        }
    }
    pairs.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut hit = Vec::new();
    for (_, alias, canon) in pairs {
        if text.contains(&alias) && !hit.iter().any(|n| n == &canon) {
            hit.push(canon);
        }
    }
    hit
}

/// 增量中最后出现的已知地点；与 prev 相同则 None。
pub fn detect_location_shift(
    known: &[String],
    prev: Option<&str>,
    increment: &str,
) -> Option<String> {
    let mut last: Option<(usize, String)> = None;
    for loc in known {
        let loc = loc.trim();
        if loc.is_empty() {
            continue;
        }
        if let Some(idx) = increment.rfind(loc) {
            if last.as_ref().map(|(i, _)| idx >= *i).unwrap_or(true) {
                last = Some((idx, loc.to_string()));
            }
        }
    }
    let n = last.map(|(_, s)| s)?;
    match prev.map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) if p == n => None,
        _ => Some(n),
    }
}

pub fn merge_admitted(
    present: &[String],
    parties: &[String],
    mentioned: &[String],
    rest: &[String],
) -> Vec<String> {
    let mut out = Vec::new();
    for src in [present, parties, mentioned, rest] {
        for n in src {
            if n.is_empty() {
                continue;
            }
            if !out.iter().any(|e| e == n) {
                out.push(n.clone());
            }
            if out.len() >= ADMITTED_CAP {
                return out;
            }
        }
    }
    out
}

pub fn truncate_chars(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        s.to_string()
    } else {
        format!("{}…（已截断）", s.chars().take(max).collect::<String>())
    }
}

/// 幕前编辑器传 HTML。提示词必须用纯正文，否则 800/2400 字预算会被标签吃掉。
pub fn strip_editor_markup(text: &str) -> String {
    let raw = text.trim();
    if raw.is_empty() {
        return String::new();
    }
    if !raw.contains('<') && !raw.contains('&') {
        return raw.to_string();
    }
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for c in raw.chars() {
        if c == '<' {
            in_tag = true;
            tag.clear();
            continue;
        }
        if in_tag {
            if c == '>' {
                in_tag = false;
                let t = tag.trim().trim_start_matches('/').to_ascii_lowercase();
                let name = t.split(|ch: char| ch.is_whitespace()).next().unwrap_or("");
                if matches!(
                    name,
                    "p" | "div" | "br" | "h1" | "h2" | "h3" | "li" | "tr" | "blockquote"
                ) {
                    out.push('\n');
                }
            } else {
                tag.push(c);
            }
            continue;
        }
        out.push(c);
    }
    let decoded = out
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");
    let mut collapsed = decoded;
    while collapsed.contains("\n\n\n") {
        collapsed = collapsed.replace("\n\n\n", "\n\n");
    }
    collapsed.trim().to_string()
}

/// 节拍卡「末段已在场」只看本拍镜头，避免开篇/章中人物被当成还在场。
pub fn prior_tail_for_cast(text: &str) -> String {
    let plain = strip_editor_markup(text);
    let n = plain.chars().count();
    if n <= PRIOR_CAST_CHAR_CAP {
        plain
    } else {
        plain.chars().skip(n - PRIOR_CAST_CHAR_CAP).collect()
    }
}

/// 当前章（Append）或最近一场（NextChapter）的前文：短章全文；
/// 长章保留开篇 + 近文，中间省略。不叠更早场次的正文。
pub fn slice_prior_prose(text: &str) -> String {
    let plain = strip_editor_markup(text);
    let n = plain.chars().count();
    if n <= PRIOR_PROSE_CHAR_CAP {
        return plain;
    }
    let head: String = plain.chars().take(PRIOR_HEAD_CHAR_CAP).collect();
    let tail: String = plain.chars().skip(n - PRIOR_TAIL_CHAR_CAP).collect();
    format!("{head}\n…（本章中间已省略）…\n{tail}")
}

pub fn condense_story_outline(raw: &str, next_node: &str) -> String {
    let raw = raw.trim();
    let mut blocks: Vec<String> = Vec::new();
    if !raw.is_empty() {
        let mut current = String::new();
        for line in raw.lines() {
            let t = line.trim();
            if t.starts_with('【') && t.contains('】') && !current.is_empty() {
                push_unique_block(&mut blocks, current.trim().to_string());
                current.clear();
            }
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
        if !current.trim().is_empty() {
            push_unique_block(&mut blocks, current.trim().to_string());
        }
        if blocks.is_empty() {
            push_unique_block(&mut blocks, raw.to_string());
        }
    }

    let mut core: Option<String> = None;
    let mut turning: Vec<String> = Vec::new();
    let mut other: Vec<String> = Vec::new();
    for b in blocks {
        if b.contains("【核心冲突】") && core.is_none() {
            core = Some(b);
        } else if b.contains("【转折点】") || b.contains("【关键转折点】") {
            if turning.len() < 3 {
                turning.push(b);
            }
        } else if !b.contains("【核心冲突】") && other.len() < 2 {
            other.push(b);
        }
    }

    if turning.len() > 1 && !next_node.trim().is_empty() {
        let overlapped: Vec<String> = turning
            .iter()
            .filter(|t| overlaps(t, next_node))
            .cloned()
            .collect();
        if !overlapped.is_empty() {
            turning = overlapped.into_iter().take(3).collect();
        }
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = core {
        parts.push(c);
    }
    parts.extend(turning);
    parts.extend(other);
    let next = next_node.trim();
    if !next.is_empty() && !parts.iter().any(|p| p.contains(next)) {
        parts.push(format!("【下一节点】{next}"));
    }
    if parts.is_empty() && !next.is_empty() {
        parts.push(next.to_string());
    }
    truncate_chars(&parts.join("\n"), OUTLINE_CHAR_CAP)
}

fn push_unique_block(blocks: &mut Vec<String>, block: String) {
    if !blocks.iter().any(|b| b == &block) {
        blocks.push(block);
    }
}

fn overlaps(a: &str, b: &str) -> bool {
    let needle: String = b.chars().filter(|c| !c.is_whitespace()).take(8).collect();
    (!needle.is_empty() && a.contains(&needle)) || (b.chars().count() >= 2 && a.contains(b.trim()))
}

pub fn build_roster(
    table_names: &[impl AsRef<str>],
    admitted: &[impl AsRef<str>],
    evidence: &str,
) -> Vec<String> {
    table_names
        .iter()
        .map(|n| n.as_ref())
        .filter(|n| !n.is_empty())
        .filter(|n| !admitted.iter().any(|a| a.as_ref() == *n))
        .filter(|n| evidence.contains(*n))
        .map(|n| n.to_string())
        .collect()
}

pub fn render_roster_line(roster: &[String]) -> String {
    if roster.is_empty() {
        return String::new();
    }
    let overflow = roster.len() > ROSTER_NAME_CAP;
    let shown = &roster[..roster.len().min(ROSTER_NAME_CAP)];
    let mut line = format!("{ROSTER_PREFIX}{}", shown.join("、"));
    if overflow {
        line.push_str("等");
    }
    line
}

pub fn filter_relationship_lines(lines: &[String], admitted: &[impl AsRef<str>]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| {
            let Some((src, tgt)) = parse_relationship_ends(line) else {
                return false;
            };
            let src_ok = admitted.iter().any(|n| n.as_ref() == src);
            let tgt_ok = admitted.iter().any(|n| n.as_ref() == tgt);
            src_ok && tgt_ok
        })
        .cloned()
        .collect()
}

fn parse_relationship_ends(line: &str) -> Option<(String, String)> {
    let rest = line.trim().trim_start_matches('■').trim();
    let (left, right) = rest.split_once(" -> ")?;
    let src = left.trim().to_string();
    let tgt = right
        .split(['：', ':'])
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if src.is_empty() || tgt.is_empty() {
        None
    } else {
        Some((src, tgt))
    }
}

pub fn render_admitted_cards(chars: &[CoreCharacter], admitted: &[impl AsRef<str>]) -> String {
    let mut lines = Vec::new();
    for name in admitted {
        let Some(c) = chars.iter().find(|c| c.name == name.as_ref()) else {
            continue;
        };
        lines.push(render_one_card(c));
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("【本拍角色（须遵循当前状态）】\n{}", lines.join("\n"))
}

fn render_one_card(c: &CoreCharacter) -> String {
    let mut parts = vec![format!("姓名：{}", c.name)];
    if let Some(ref id) = c.identity {
        parts.push(format!("身份：{}", id));
    }
    let mut state_parts = vec![];
    if let Some(ref s) = c.physical_state {
        state_parts.push(format!("身体：{}", s));
    }
    if let Some(ref s) = c.mental_state {
        state_parts.push(format!("精神：{}", s));
    }
    if let Some(ref s) = c.location {
        state_parts.push(format!("位置：{}", s));
    }
    if !state_parts.is_empty() {
        parts.push(format!("当前状态：{}", state_parts.join("，")));
    }
    if let Some(ref p) = c.personality {
        parts.push(format!("性格：{}", p));
    }
    if let Some(ref v) = c.emotional_core {
        parts.push(format!("情感内核：{}", v));
    }
    if let Some(ref v) = c.emotional_trigger {
        parts.push(format!("情感触发：{}", v));
    }
    if let Some(ref v) = c.emotional_wound {
        parts.push(format!("情感创伤：{}", v));
    }
    if let Some(ref v) = c.emotional_need {
        parts.push(format!("情感需求：{}", v));
    }
    format!("- {}", parts.join(" | "))
}

pub fn condense_world_setting(raw: &str, location: Option<&str>, admitted: &[String]) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }
    let mut concept = String::new();
    let mut rules: Vec<String> = Vec::new();
    let mut section = "";
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with("世界概念：") {
            section = "concept";
            concept = t.trim_start_matches("世界概念：").trim().to_string();
            continue;
        }
        if t.starts_with("核心规则：") {
            section = "rules";
            continue;
        }
        if t.starts_with("历史背景：") || t.starts_with("历史：") {
            section = "skip";
            continue;
        }
        if t.starts_with("文化与势力：") {
            section = "skip";
            continue;
        }
        match section {
            "concept" => {
                if !concept.is_empty() {
                    concept.push('\n');
                }
                concept.push_str(t);
            }
            "rules" => {
                let rule = t.trim_start_matches('-').trim();
                if !rule.is_empty() {
                    rules.push(rule.to_string());
                }
            }
            _ => {}
        }
    }
    if concept.is_empty() && rules.is_empty() {
        // 无标题时只留概念预算，避免把历史整段灌进去
        return truncate_chars(raw.trim(), WORLD_CONCEPT_CHAR_CAP);
    }
    let mut preferred: Vec<String> = Vec::new();
    let mut rest: Vec<String> = Vec::new();
    for r in rules {
        let hit_loc = location
            .map(|l| !l.is_empty() && r.contains(l))
            .unwrap_or(false);
        let hit_name = admitted.iter().any(|n| !n.is_empty() && r.contains(n));
        if hit_loc || hit_name {
            preferred.push(r);
        } else {
            rest.push(r);
        }
    }
    preferred.extend(rest);
    preferred.truncate(WORLD_RULE_CAP);

    let mut parts = Vec::new();
    if !concept.is_empty() {
        parts.push(format!(
            "世界概念：{}",
            truncate_chars(&concept, WORLD_CONCEPT_CHAR_CAP)
        ));
    }
    if !preferred.is_empty() {
        let rules_text = preferred
            .iter()
            .map(|r| format!("- {r}"))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("核心规则：\n{rules_text}"));
    }
    truncate_chars(&parts.join("\n\n"), WORLD_CHAR_CAP)
}

fn take_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

/// 预算收缩：普通段保标题（从头截）；【前文】保近文（从尾截，
/// 并留下「【前文】」前缀）。
fn shrink_part(s: &str, new_len: usize) -> String {
    let n = s.chars().count();
    if n <= new_len {
        return s.to_string();
    }
    const PRIOR_PREFIX: &str = "【前文】";
    if s.starts_with(PRIOR_PREFIX) {
        let prefix_n = PRIOR_PREFIX.chars().count();
        if new_len <= prefix_n {
            return PRIOR_PREFIX.chars().take(new_len).collect();
        }
        let body: String = s.chars().skip(prefix_n).collect();
        let keep = new_len - prefix_n;
        let bn = body.chars().count();
        let tail = if bn <= keep {
            body
        } else {
            body.chars().skip(bn - keep).collect()
        };
        format!("{PRIOR_PREFIX}{tail}")
    } else {
        s.chars().take(new_len).collect()
    }
}

/// `protected=true` 的段不得删光；超预算时从非保护段（通常是前文）往前截。
/// 预算收缩禁止使用带「已截断」后缀的
/// `truncate_chars`（后缀会让长度不降反升）。
pub fn apply_asset_budget(parts: &[(String, bool)]) -> String {
    let mut items: Vec<(String, bool)> = parts.to_vec();
    for _ in 0..32 {
        let joined = join_nonempty(&items);
        let n = joined.chars().count();
        if n <= ASSET_CHAR_BUDGET {
            return joined;
        }
        let Some(i) = items
            .iter()
            .rposition(|(t, protected)| !*protected && !t.is_empty())
        else {
            return take_chars(&joined, ASSET_CHAR_BUDGET);
        };
        let over = n - ASSET_CHAR_BUDGET;
        let cur = items[i].0.chars().count();
        if cur <= over {
            items[i].0.clear();
        } else {
            let next = shrink_part(&items[i].0, cur - over);
            if next.chars().count() >= cur {
                items[i].0.clear();
            } else {
                items[i].0 = next;
            }
        }
    }
    take_chars(&join_nonempty(&items), ASSET_CHAR_BUDGET)
}

fn join_nonempty(items: &[(String, bool)]) -> String {
    items
        .iter()
        .map(|(t, _)| t.as_str())
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub struct ContinueAssetsInput<'a> {
    pub bundle: &'a WriteTimeBundle,
    pub admitted: &'a [String],
    pub roster: &'a [String],
    pub location: Option<&'a str>,
    pub next_node: &'a str,
    pub chapter_outline: &'a str,
    pub progress_lines: &'a [String],
    pub prior_prose: &'a str,
    pub tension_lines: &'a [String],
    pub arc_lines: &'a [String],
    pub logline: Option<&'a str>,
}

pub fn render_continue_assets(input: &ContinueAssetsInput<'_>) -> String {
    let redline = input
        .bundle
        .contract_redlines
        .as_deref()
        .map(|raw| {
            format!(
                "【⚠️ 世界观红线（绝不可违背，违反即判定为严重错误）】\n{}",
                extract_redline_text(raw)
            )
        })
        .unwrap_or_default();

    let outline_raw = input.bundle.story_outline.as_deref().unwrap_or("");
    let condensed = condense_story_outline(outline_raw, input.next_node);
    let outline = if condensed.is_empty() {
        String::new()
    } else {
        format!(
            "【故事大纲（本场景必须围绕此大纲展开，禁止偏离）】\n{}",
            condensed
        )
    };

    let world = input
        .bundle
        .world_setting
        .as_deref()
        .map(|w| {
            let c = condense_world_setting(w, input.location, input.admitted);
            if c.is_empty() {
                String::new()
            } else {
                format!("【世界观设定（须遵循其规则与约束，违反即判定为严重错误）】\n{c}")
            }
        })
        .unwrap_or_default();

    let cards = render_admitted_cards(&input.bundle.core_characters, input.admitted);
    let roster = render_roster_line(input.roster);
    let rels = filter_relationship_lines(&input.bundle.relationship_lines, input.admitted);
    let relationships = if rels.is_empty() {
        String::new()
    } else {
        format!(
            "【角色情感关系（真实情感，可与表面关系不一致）】\n{}\n要求：言行须与情感关系一致。",
            rels.join("\n")
        )
    };

    let conflicts = input
        .bundle
        .active_conflicts
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            format!(
                "【活跃冲突（本拍须承接，不得当已解决除非正文写明）】\n{}",
                truncate_chars(s, 400)
            )
        })
        .unwrap_or_default();
    let goals = {
        let raw = input.bundle.character_goals.as_deref().unwrap_or("");
        if raw.trim().is_empty() {
            String::new()
        } else {
            let kept: Vec<&str> = raw
                .lines()
                .filter(|line| {
                    input
                        .admitted
                        .iter()
                        .any(|n| !n.is_empty() && line.contains(n.as_str()))
                })
                .collect();
            if kept.is_empty() {
                String::new()
            } else {
                format!(
                    "【角色当前目标】\n{}",
                    truncate_chars(&kept.join("\n"), 400)
                )
            }
        }
    };

    let chapter = if input.chapter_outline.trim().is_empty() {
        input
            .bundle
            .scene_outline
            .as_ref()
            .and_then(|o| o.outline_content.as_deref())
            .filter(|s| !s.trim().is_empty())
            .map(|s| {
                format!(
                    "【本章大纲（必须遵循的章节方向）】\n{}",
                    truncate_chars(s, CHAPTER_OUTLINE_CHAR_CAP)
                )
            })
            .unwrap_or_default()
    } else {
        format!(
            "【本章大纲（必须遵循的章节方向）】\n{}",
            truncate_chars(input.chapter_outline, CHAPTER_OUTLINE_CHAR_CAP)
        )
    };

    let progress = if input.progress_lines.is_empty() {
        String::new()
    } else {
        format!(
            "【已推进进度（承接此处，推进到下一节点，不得原地踏步）】\n{}",
            input.progress_lines.join("\n")
        )
    };

    let prior = if input.prior_prose.trim().is_empty() {
        String::new()
    } else {
        format!("【前文】{}", input.prior_prose)
    };

    let tensions = if input.tension_lines.is_empty() {
        String::new()
    } else {
        truncate_chars(&input.tension_lines.join("\n"), TENSION_ARC_CHAR_CAP)
    };
    let arcs = if input.arc_lines.is_empty() {
        String::new()
    } else {
        truncate_chars(&input.arc_lines.join("\n"), TENSION_ARC_CHAR_CAP)
    };
    let logline = input
        .logline
        .filter(|s| !s.is_empty())
        .map(|s| format!("【故事Logline】{s}"))
        .unwrap_or_default();

    let mut foreshadow = String::new();
    if !input.bundle.pending_foreshadowings.is_empty() {
        let lines: Vec<String> = input
            .bundle
            .pending_foreshadowings
            .iter()
            .take(3)
            .map(|f| format!("  - {f}"))
            .collect();
        foreshadow.push_str(&format!(
            "【待回收伏笔（请在续写中适时推进）】\n{}",
            lines.join("\n")
        ));
    }
    if !input.bundle.overdue_foreshadowings.is_empty() {
        if !foreshadow.is_empty() {
            foreshadow.push_str("\n\n");
        }
        let lines: Vec<String> = input
            .bundle
            .overdue_foreshadowings
            .iter()
            .take(1)
            .map(|f| format!("  ⚠️ {f}"))
            .collect();
        foreshadow.push_str(&format!(
            "【⚠️ 逾期伏笔——请在续写中优先回收】\n{}",
            lines.join("\n")
        ));
    }

    let parts = [
        (redline, true),
        (outline, false),
        (world, false),
        (cards, true),
        (roster, true),
        (relationships, false),
        (conflicts, false),
        (goals, false),
        (chapter, false),
        (progress, false),
        (prior, false),
        (tensions, false),
        (arcs, false),
        (logline, false),
        (foreshadow, false),
    ];
    let before = join_nonempty(&parts);
    let out = apply_asset_budget(&parts);
    let truncated = out.chars().count() < before.chars().count();
    log::info!(
        "continue_assets: admitted={} roster={} chars={} truncated={}",
        input.admitted.len(),
        input.roster.len(),
        out.chars().count(),
        truncated
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::write_time_bundle::{GenreCategory, StoryMeta, WriteTimeBundle};

    fn core(name: &str) -> CoreCharacter {
        CoreCharacter {
            name: name.into(),
            identity: Some("身份".into()),
            physical_state: None,
            mental_state: None,
            location: None,
            personality: Some("性格".into()),
            emotional_core: Some(format!("{name}的情感内核")),
            emotional_trigger: None,
            emotional_wound: None,
            emotional_need: None,
        }
    }

    fn empty_bundle() -> WriteTimeBundle {
        WriteTimeBundle {
            contract_redlines: Some(r#"{"world_rules":"禁止时间旅行"}"#.into()),
            core_characters: vec![],
            relationship_lines: vec![],
            scene_outline: None,
            story_outline: Some("【核心冲突】皇权裂痕\n【核心冲突】皇权裂痕".into()),
            world_setting: Some("世界概念：阴阳两界。".into()),
            genre_antipatterns: vec![],
            style_slice: None,
            story_meta: StoryMeta {
                title: "镇魂".into(),
                genre: None,
                tone: None,
                pacing: None,
                description: None,
            },
            genre_category: GenreCategory::Speculative,
            narrative_phase_guidance: None,
            pending_foreshadowings: vec!["旧剑将出鞘".into()],
            overdue_foreshadowings: vec![],
            style_dna_summary: None,
            narrative_quartet: None,
            style_dna_extension: None,
            methodology_extension: None,
            genre_profile_strategy: Some("不得进本拍".into()),
            secondary_genre_profile_strategy: None,
            writing_strategy_constraints: None,
            runtime_contract: None,
            reference_scene_fewshots: vec![],
            related_entity_summaries: vec![],
            active_conflicts: None,
            character_goals: None,
            chase_debt_text: None,
            genre_reference: Some("题材表不得进".into()),
            style_blend_text: None,
            rotation_ledger_text: None,
        }
    }

    #[test]
    fn condense_outline_dedupes_repeated_core_conflict() {
        let stacked = "【核心冲突】皇权裂痕\n【转折点】密诏\n".repeat(10);
        let out = condense_story_outline(&stacked, "推进到密诏败露");
        let count = out.matches("【核心冲突】皇权裂痕").count();
        assert_eq!(count, 1, "got: {out}");
        assert!(out.chars().count() <= 1200);
        assert!(out.contains("密诏败露") || out.contains("密诏"));
    }

    #[test]
    fn condense_outline_empty_falls_back_to_next_node() {
        let out = condense_story_outline("   ", "下一节点：入宫");
        assert!(out.contains("入宫"), "got: {out}");
        assert!(!out.contains("【核心冲突】【核心冲突】"));
    }

    #[test]
    fn merge_admitted_priority_and_cap_eight() {
        let present = vec!["甲".into(), "乙".into(), "丙".into()];
        let parties = vec!["丁".into()];
        let mentioned = vec!["戊".into(), "己".into()];
        let rest = vec!["庚".into(), "辛".into(), "壬".into(), "癸".into()];
        let got = merge_admitted(&present, &parties, &mentioned, &rest);
        assert_eq!(got, vec!["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛"]);
        assert_eq!(got.len(), 8);
        assert!(!got.iter().any(|n| n == "壬" || n == "癸"));
    }

    #[test]
    fn names_in_text_intersects_character_table() {
        let names = ["苏亦铁", "周奕辰", "何双"];
        let text = "苏亦铁推开殿门，何双跟在后面。";
        let got = names_in_text(&names, text);
        assert_eq!(got, vec!["苏亦铁", "何双"]);
        assert!(!got.iter().any(|n| n == "周奕辰"));
    }

    #[test]
    fn roster_excludes_admitted_and_orphans() {
        let table = ["苏亦铁", "何双", "周奕辰", "赵奎"];
        let admitted = ["苏亦铁"];
        let evidence = "何双在茶馆坐下。赵奎守门。";
        let got = build_roster(&table, &admitted, evidence);
        assert!(!got.iter().any(|n| n == "苏亦铁"));
        assert!(got.contains(&"何双".to_string()));
        assert!(got.contains(&"赵奎".to_string()));
        assert!(
            !got.iter().any(|n| n == "周奕辰"),
            "脏名不得进名单: {got:?}"
        );
    }

    #[test]
    fn roster_line_format_and_cap() {
        let names: Vec<String> = (0..45).map(|i| format!("角{i:02}")).collect();
        let line = render_roster_line(&names);
        assert!(line.starts_with("本拍未上场（禁止新编下列姓名，亦不得当主角使用）"));
        assert!(line.ends_with("等"));
        assert!(!line.contains("情感内核"));
    }

    #[test]
    fn relationships_require_both_ends_admitted() {
        let lines = vec![
            "■ 甲 -> 乙：社会关系=同僚 ｜ 情感=恨[0.9]".into(),
            "■ 甲 -> 丙：社会关系=路人 ｜ 情感=无[0.1]".into(),
        ];
        let admitted = ["甲", "乙"];
        let got = filter_relationship_lines(&lines, &admitted);
        assert_eq!(got.len(), 1);
        assert!(got[0].contains("甲 -> 乙"));
        assert!(!got.iter().any(|l| l.contains("丙")));
    }

    #[test]
    fn render_cards_only_admitted_and_renames_heading() {
        let chars = vec![core("甲"), core("乙"), core("丙")];
        let admitted = ["甲", "乙"];
        let text = render_admitted_cards(&chars, &admitted);
        assert!(text.contains("【本拍角色（须遵循当前状态）】"));
        assert!(!text.contains("【登场角色（必须严格遵循其当前状态）】"));
        assert!(text.contains("情感内核：甲的情感内核"));
        assert!(text.contains("情感内核：乙的情感内核"));
        assert!(!text.contains("情感内核：丙的情感内核"));
    }

    #[test]
    fn condense_world_drops_history_and_cultures() {
        let raw = "世界概念：阴阳两界裂开了一道缝，缝里是旧朝的祭法。\n\n核心规则：\n- 夜行：入夜后阳气不可过界\n- 血契：以血换名\n\n历史背景：三千年前的巫祭。\n\n文化与势力：\n- 阴司：收魂";
        let out = condense_world_setting(raw, Some("茶馆"), &["苏亦铁".into()]);
        assert!(out.contains("世界概念"));
        assert!(out.contains("夜行") || out.contains("核心规则"));
        assert!(!out.contains("三千年前"));
        assert!(!out.contains("阴司"));
        assert!(out.chars().count() <= 800);
    }

    #[test]
    fn budget_keeps_redline_cards_roster_truncates_prior() {
        let redline = "【⚠️ 世界观红线（绝不可违背，违反即判定为严重错误）】\n禁止时间旅行";
        let cards = "【本拍角色（须遵循当前状态）】\n- 姓名：甲 | 情感内核：核";
        let roster = format!("{ROSTER_PREFIX}乙、丙");
        let prior = format!("【前文】{}", "哈".repeat(8000));
        let out = apply_asset_budget(&[
            (redline.to_string(), true),
            (cards.to_string(), true),
            (roster.clone(), true),
            (prior, false),
        ]);
        assert!(out.chars().count() <= ASSET_CHAR_BUDGET);
        assert!(out.contains("禁止时间旅行"));
        assert!(out.contains("情感内核：核"));
        assert!(out.contains(&roster));
    }

    #[test]
    fn twenty_chars_three_present_full_cards_rest_roster() {
        let mut bundle = empty_bundle();
        let names: Vec<String> = (0..20).map(|i| format!("角色{:02}", i)).collect();
        bundle.core_characters = names.iter().map(|n| core(n)).collect();
        bundle.relationship_lines = vec![
            "■ 角色00 -> 角色01：社会关系=同僚 ｜ 情感=恨[0.9]".into(),
            "■ 角色00 -> 角色19：社会关系=路人 ｜ 情感=无[0.1]".into(),
        ];
        let present = vec!["角色00".into(), "角色01".into(), "角色02".into()];
        let evidence: String = names.join("、");
        let roster = build_roster(&names, &present, &evidence);
        let input = ContinueAssetsInput {
            bundle: &bundle,
            admitted: &present,
            roster: &roster,
            location: None,
            next_node: "下一节点",
            chapter_outline: "本章：角色00对质",
            progress_lines: &[],
            prior_prose: "前文末段。",
            tension_lines: &[],
            arc_lines: &[],
            logline: Some("一句话"),
        };
        let out = render_continue_assets(&input);
        assert!(out.contains("情感内核：角色00的情感内核"));
        assert!(out.contains("情感内核：角色01的情感内核"));
        assert!(out.contains("情感内核：角色02的情感内核"));
        assert!(!out.contains("情感内核：角色19的情感内核"));
        assert!(!out.contains("必须严格遵循其当前状态"));
        assert!(out.contains(ROSTER_PREFIX));
        assert!(out.contains("角色19"));
        assert!(out.contains("角色00 -> 角色01"));
        assert!(!out.contains("角色00 -> 角色19"));
        assert_eq!(out.matches("【核心冲突】皇权裂痕").count(), 1);
        assert!(!out.contains("题材表不得进"));
        assert!(!out.contains("不得进本拍"));
        assert!(out.contains("旧剑将出鞘"));
        assert!(out.chars().count() <= ASSET_CHAR_BUDGET);
    }

    #[test]
    fn slice_prior_prose_keeps_opening_and_ending_of_long_chapter() {
        let opening = "开篇标记青梧镇雨夜。".repeat(60);
        let middle = "中间标记不该出现。".repeat(200);
        let ending = "章末标记他扣上匣子。".repeat(180);
        let out = slice_prior_prose(&format!("{opening}{middle}{ending}"));
        assert!(out.contains("开篇标记青梧镇雨夜"), "out={out}");
        assert!(out.contains("章末标记他扣上匣子"), "out={out}");
        assert!(out.contains("本章中间已省略"), "out={out}");
        assert!(
            !out.contains("中间标记不该出现"),
            "中间不得整段灌入 out={}",
            out.chars().take(200).collect::<String>()
        );
    }

    #[test]
    fn slice_prior_prose_strips_html_and_keeps_last_paragraph() {
        let html = format!("<p>{}</p><p>乙收刀，转身离开客栈。</p>", "甲".repeat(3000));
        let out = slice_prior_prose(&html);
        assert!(!out.contains("<p>"), "out={out}");
        assert!(!out.contains("</p>"), "out={out}");
        assert!(out.contains("乙收刀，转身离开客栈"), "out={out}");
        assert!(out.contains("甲"), "开篇窗口应仍有甲 out={out}");
    }

    #[test]
    fn slice_prior_prose_short_chapter_kept_in_full() {
        let text = "短章全文都要进。他推开门。";
        assert_eq!(slice_prior_prose(text), text);
    }

    #[test]
    fn budget_shrinks_prior_from_the_tail() {
        let redline = "【⚠️ 世界观红线（绝不可违背，违反即判定为严重错误）】\n禁止时间旅行";
        let cards = "【本拍角色（须遵循当前状态）】\n- 姓名：甲 | 情感内核：核";
        let roster = format!("{ROSTER_PREFIX}乙、丙");
        let prior = format!("【前文】{}章末必须留下", "哈".repeat(8000));
        let out = apply_asset_budget(&[
            (redline.to_string(), true),
            (cards.to_string(), true),
            (roster.clone(), true),
            (prior, false),
        ]);
        assert!(out.chars().count() <= ASSET_CHAR_BUDGET);
        assert!(out.contains("禁止时间旅行"));
        assert!(out.contains("章末必须留下"), "预算截前文须保近文 out={out}");
    }

    #[test]
    fn prior_tail_for_cast_is_shot_window() {
        let long = format!("{}客栈乙扣上匣子。", "闲笔。".repeat(400));
        let tail = prior_tail_for_cast(&long);
        assert!(tail.chars().count() <= PRIOR_CAST_CHAR_CAP);
        assert!(tail.contains("客栈乙"));
        assert!(!tail.contains("青梧甲"));
    }

    #[test]
    fn aliases_for_two_char_name_includes_a_prefix() {
        let a = aliases_for("沈砚");
        assert!(a.contains(&"沈砚".into()));
        assert!(a.contains(&"阿砚".into()));
        assert!(!a.iter().any(|s| s.chars().count() == 1));
    }

    #[test]
    fn match_character_names_alias_and_no_single_char() {
        let names: Vec<String> = vec!["沈砚".into(), "白芷".into()];
        let hit = match_character_names(&names, "阿砚握着罗盘，白芷在侧。");
        assert!(hit.contains(&"沈砚".into()));
        assert!(hit.contains(&"白芷".into()));
        let miss = match_character_names(&names, "白雪落在石阶上。");
        assert!(!miss.contains(&"白芷".into()));
    }

    #[test]
    fn match_character_names_exact_name_wins_over_alias() {
        let names: Vec<String> = vec!["沈砚".into(), "阿砚".into()];
        let hit = match_character_names(&names, "阿砚先开口。");
        assert_eq!(hit, vec!["阿砚".to_string()]);
    }

    #[test]
    fn detect_location_shift_picks_last_known_place_in_increment() {
        let known = vec!["雨巷".into(), "钟楼".into()];
        let got = detect_location_shift(&known, Some("雨巷"), "他们离开雨巷，潜入钟楼底层。");
        assert_eq!(got.as_deref(), Some("钟楼"));
        assert!(detect_location_shift(&known, Some("雨巷"), "继续对话。").is_none());
    }

    #[test]
    fn render_includes_filtered_conflicts_and_goals() {
        let mut bundle = empty_bundle();
        bundle.active_conflicts = Some("皇权裂痕".into());
        bundle.character_goals = Some("阿岩：讨回公道\n路人：无关目标".into());
        let admitted = vec!["阿岩".into()];
        let input = ContinueAssetsInput {
            bundle: &bundle,
            admitted: &admitted,
            roster: &[],
            location: None,
            next_node: "下一节点",
            chapter_outline: "",
            progress_lines: &[],
            prior_prose: "",
            tension_lines: &[],
            arc_lines: &[],
            logline: None,
        };
        let out = render_continue_assets(&input);
        assert!(out.contains("皇权裂痕"), "{out}");
        assert!(out.contains("讨回公道"), "{out}");
        assert!(!out.contains("无关目标"), "{out}");
        assert!(out.chars().count() <= ASSET_CHAR_BUDGET);
    }
}
