//! SceneBeatCard：本拍硬任务纯 Rust 编译（0 LLM）。
//! 设计：docs/plans/2026-08-13-agency-only-continuation-design.md §5

use crate::{
    creative_engine::expansion::{debt::QuotaItem, ExpansionDebt, RotationLedger},
    db::{
        repositories::{
            CharacterRelationshipRepository, CharacterRepository, StoryOutlineRepository,
        },
        Character, DbPool, SceneRepository,
    },
    error::AppError,
};

#[derive(Debug, Clone)]
pub struct CastMember {
    pub name: String,
    pub purpose: String,
}

#[derive(Debug, Clone)]
pub struct ConflictMove {
    pub action: String,
    pub parties: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EmotionBeat {
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct SceneBeatCard {
    pub cast: Vec<CastMember>,
    pub conflict_move: ConflictMove,
    pub emotion_beat: EmotionBeat,
    pub next_outline_node: String,
    pub expansion_quota: Vec<QuotaItem>,
    pub expansion_quota_text: Option<String>,
    pub setting_location: Option<String>,
}

pub fn quota_text_for_beats(
    debt: &crate::creative_engine::expansion::ExpansionDebt,
) -> Option<String> {
    debt.quota_text().map(|s| {
        s.replace(
            "【本章扩张任务（硬性要求，必须落实）】",
            "【本拍扩张任务（硬性要求，必须落实）】",
        )
        .replace(" 章——本章", " 拍——本拍")
        .replace(" 章无更新——本章", " 拍无更新——本拍")
        .replace(" 章无动静——本章", " 拍无动静——本拍")
    })
}

impl SceneBeatCard {
    pub fn render_full(&self) -> String {
        let mut lines = vec!["【本章节拍任务】".to_string()];
        let cast = self
            .cast
            .iter()
            .map(|c| format!("{}（{}）", c.name, c.purpose))
            .collect::<Vec<_>>()
            .join("、");
        lines.push(format!(
            "阵容：{}",
            if cast.is_empty() {
                "待定".into()
            } else {
                cast
            }
        ));
        lines.push(format!("冲突：{}", self.conflict_move.action));
        lines.push(format!("情感：{}", self.emotion_beat.summary));
        lines.push(format!("推进：{}", self.next_outline_node));
        if let Some(ref loc) = self.setting_location {
            if !loc.is_empty() {
                lines.push(format!("地点：{}", loc));
            }
        }
        if let Some(ref q) = self.expansion_quota_text {
            if !q.is_empty() {
                lines.push(q.clone());
            }
        }
        lines.join("\n")
    }

    pub fn render_tail_summary(&self) -> String {
        format!(
            "【节拍摘要】上场：{}｜冲突：{}｜情感：{}｜推进：{}",
            self.cast
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join("、"),
            self.conflict_move
                .action
                .chars()
                .take(40)
                .collect::<String>(),
            self.emotion_beat
                .summary
                .chars()
                .take(40)
                .collect::<String>(),
            self.next_outline_node.chars().take(40).collect::<String>(),
        )
    }
}

/// 从 DB + 当前章末段编译本拍硬任务卡。缺槽用降级句，永不空卡。
pub fn compile_beat_card(
    pool: &DbPool,
    story_id: &str,
    current_content: &str,
) -> Result<SceneBeatCard, AppError> {
    compile_beat_card_located(pool, story_id, current_content, None)
}

pub fn compile_beat_card_located(
    pool: &DbPool,
    story_id: &str,
    current_content: &str,
    current_scene_location: Option<&str>,
) -> Result<SceneBeatCard, AppError> {
    let chars = CharacterRepository::new(pool.clone())
        .get_by_story(story_id)
        .map_err(AppError::from)?;
    let protagonist = chars.first().map(|c| c.name.as_str()).unwrap_or("主角");

    let tail = crate::agency::continue_assets::prior_tail_for_cast(current_content);
    let mut cast = present_in_text(&chars, &tail);
    let ledger = RotationLedger::load_sync(pool, story_id).unwrap_or_default();
    let debt = ExpansionDebt::compute(pool, story_id, &ledger).unwrap_or_default();
    let expansion_quota = debt.triggered();
    let allow_character_move = expansion_quota.contains(&QuotaItem::CharacterMove);

    if chars.len() >= 3 && allow_character_move {
        if let Some(silent) = ledger
            .character_silence
            .iter()
            .find(|s| !cast.iter().any(|c| c.name == s.name))
        {
            cast.push(CastMember {
                name: silent.name.clone(),
                purpose: "沉寂回归，推进其弧光".into(),
            });
        }
    }
    let tensions = crate::agency::emotional_ledger::load_tensions(pool, story_id);
    if let Some(t) = tensions.iter().max_by(|a, b| {
        a.pressure
            .partial_cmp(&b.pressure)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        let src_in = !t.source_name.is_empty() && cast.iter().any(|c| c.name == t.source_name);
        let tgt_in = !t.target_name.is_empty() && cast.iter().any(|c| c.name == t.target_name);
        if src_in ^ tgt_in {
            let name = if src_in {
                &t.target_name
            } else {
                &t.source_name
            };
            if !name.is_empty() && !cast.iter().any(|c| c.name == *name) {
                cast.push(CastMember {
                    name: name.clone(),
                    purpose: format!("张力对手（{}）", t.tension_type),
                });
            }
        }
    }
    if cast.is_empty() {
        cast.push(CastMember {
            name: protagonist.to_string(),
            purpose: "本拍行动主体".into(),
        });
    }
    cast.truncate(8);
    if chars.len() >= 3 && cast.len() < 3 && allow_character_move {
        for c in &chars {
            if cast.len() >= 3 {
                break;
            }
            if !cast.iter().any(|m| m.name == c.name) {
                cast.push(CastMember {
                    name: c.name.clone(),
                    purpose: "补位上场".into(),
                });
            }
        }
    }

    let conflict_move = compile_conflict(&chars, &cast, pool, story_id, protagonist);
    let emotion_beat = compile_emotion(&chars, &cast, pool, story_id, protagonist);
    let next_outline_node = compile_next_node(pool, story_id);
    let expansion_quota_text = quota_text_for_beats(&debt);
    let setting_location = current_scene_location
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    Ok(SceneBeatCard {
        cast,
        conflict_move,
        emotion_beat,
        next_outline_node,
        expansion_quota,
        expansion_quota_text,
        setting_location,
    })
}

fn present_in_text(chars: &[Character], text: &str) -> Vec<CastMember> {
    let names: Vec<String> = chars.iter().map(|c| c.name.clone()).collect();
    crate::agency::continue_assets::match_character_names(&names, text)
        .into_iter()
        .map(|name| CastMember {
            name,
            purpose: "末段已在场，承接行动".into(),
        })
        .collect()
}

pub(crate) fn compile_conflict(
    chars: &[Character],
    cast: &[CastMember],
    pool: &DbPool,
    story_id: &str,
    protagonist: &str,
) -> ConflictMove {
    let cast_names: Vec<&str> = cast.iter().map(|c| c.name.as_str()).collect();
    let rels = CharacterRelationshipRepository::new(pool.clone())
        .get_by_story(story_id)
        .unwrap_or_default();
    const HOSTILE: &[&str] = &[
        "仇", "敌", "对立", "背叛", "欺骗", "复仇", "恨", "enemy", "rival", "conflict",
    ];
    for r in &rels {
        let ty = r.relationship_type.to_lowercase();
        let bond = r.emotional_bond.as_deref().unwrap_or("");
        let hostile = HOSTILE.iter().any(|k| ty.contains(k) || bond.contains(k));
        if !hostile {
            continue;
        }
        let src = chars
            .iter()
            .find(|c| c.id == r.source_character_id)
            .map(|c| c.name.as_str())
            .unwrap_or(protagonist);
        let tgt = r.target_character_name.as_deref().unwrap_or("对方");
        if !cast_names.contains(&src) || !cast_names.contains(&tgt) {
            continue;
        }
        return ConflictMove {
            action: format!("加压：{src} 与 {tgt} 正面对峙，赌注未解，不得只靠对话过渡。"),
            parties: vec![src.to_string(), tgt.to_string()],
        };
    }
    let parties: Vec<String> = cast.iter().take(2).map(|c| c.name.clone()).collect();
    let names = if parties.is_empty() {
        protagonist.to_string()
    } else {
        parties.join("、")
    };
    ConflictMove {
        action: format!("{names} 必须在本拍与阻力正面对峙，不得只靠对话过渡。"),
        parties,
    }
}

fn compile_emotion(
    chars: &[Character],
    cast: &[CastMember],
    pool: &DbPool,
    story_id: &str,
    protagonist: &str,
) -> EmotionBeat {
    let cast_names: Vec<&str> = cast.iter().map(|c| c.name.as_str()).collect();
    for c in chars {
        if !cast_names.contains(&c.name.as_str()) && !cast_names.is_empty() {
            continue;
        }
        let wound = c.emotional_wound.as_deref().unwrap_or("");
        let need = c.emotional_need.as_deref().unwrap_or("");
        let core = c.emotional_core.as_deref().unwrap_or("");
        if wound.is_empty() && need.is_empty() && core.is_empty() {
            continue;
        }
        let mut bits = vec![];
        if !core.is_empty() {
            bits.push(format!("内核={}", core));
        }
        if !wound.is_empty() {
            bits.push(format!("伤口={}", wound));
        }
        if !need.is_empty() {
            bits.push(format!("需求={}", need));
        }
        return EmotionBeat {
            summary: format!("{}：{}", c.name, bits.join("，")),
        };
    }
    let rels = CharacterRelationshipRepository::new(pool.clone())
        .get_by_story(story_id)
        .unwrap_or_default();
    if let Some(r) = rels.iter().find(|r| {
        let a = r.emotional_bond.as_deref().unwrap_or("");
        let b = r.reverse_emotional_bond.as_deref().unwrap_or("");
        !a.is_empty() && !b.is_empty() && a != b
    }) {
        let src = chars
            .iter()
            .find(|c| c.id == r.source_character_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| protagonist.to_string());
        let tgt = r
            .target_character_name
            .clone()
            .unwrap_or_else(|| "对方".into());
        return EmotionBeat {
            summary: format!(
                "{}→{}：{} vs {}→{}：{}",
                src,
                tgt,
                r.emotional_bond.as_deref().unwrap_or("未明"),
                tgt,
                src,
                r.reverse_emotional_bond.as_deref().unwrap_or("未明"),
            ),
        };
    }
    EmotionBeat {
        summary: format!("本拍必须让 {protagonist} 的需求受阻并露出情绪代价。"),
    }
}

pub(crate) fn compile_next_node(pool: &DbPool, story_id: &str) -> String {
    let fallback = "在硬约束内把当前冲突推进一步，不得原地复述末句。".to_string();
    let outline = StoryOutlineRepository::new(pool.clone())
        .get_by_story(story_id)
        .ok()
        .flatten()
        .map(|o| o.content)
        .unwrap_or_default();
    if outline.trim().is_empty() {
        return fallback;
    }
    let scenes = SceneRepository::new(pool.clone())
        .get_by_story(story_id)
        .unwrap_or_default();
    let mut recent: Vec<&str> = scenes
        .iter()
        .filter_map(|s| s.outline_content.as_deref())
        .collect();
    recent.reverse();
    recent.truncate(3);
    let covered = recent.join("");
    let candidates: Vec<&str> = outline
        .split(['\n', '。', '！', '？', ';', '；'])
        .map(str::trim)
        .filter(|s| s.chars().count() >= 8)
        .collect();
    for cand in &candidates {
        let key: String = cand.chars().take(20).collect();
        if key.is_empty() {
            continue;
        }
        if !covered.contains(&key) {
            return cand.chars().take(200).collect();
        }
    }
    fallback
}

fn last_n_sentences(text: &str, n: usize, max_chars: usize) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || n == 0 {
        return None;
    }
    let delimiters: &[char] = &['。', '？', '！', '.', '?', '!'];
    let mut ends: Vec<usize> = Vec::new();
    for (i, c) in trimmed.char_indices() {
        if delimiters.contains(&c) {
            ends.push(i + c.len_utf8());
        }
    }
    let slice = if ends.is_empty() {
        let total = trimmed.chars().count();
        if total > max_chars {
            trimmed.chars().skip(total - max_chars).collect::<String>()
        } else {
            trimmed.to_string()
        }
    } else {
        let take = n.min(ends.len());
        let start_byte = if take >= ends.len() {
            0
        } else {
            ends[ends.len() - take - 1]
        };
        let rest = trimmed[start_byte..].trim_start();
        let total = rest.chars().count();
        if total > max_chars {
            rest.chars().skip(total - max_chars).collect::<String>()
        } else {
            rest.to_string()
        }
    };
    let out = slice.trim();
    if out.is_empty() {
        None
    } else {
        Some(out.to_string())
    }
}

/// 末句硬锚点：复制自 `agents::orchestrator::build_ending_anchor`，避免
/// coordinator 耦合编排器。有正文时返回非空；空正文返回空串。
pub fn ending_anchor(current_content: &str) -> String {
    let plain = crate::agency::continue_assets::strip_editor_markup(current_content);
    let Some(last) = last_n_sentences(&plain, 2, 280) else {
        return String::new();
    };
    format!(
        "【续写硬锚点（句法衔接；人物/地点/未决以节拍任务与状态网为准）】\n\
         正文已写到此处，下一句必须无缝衔接，禁止另起开篇、禁止重写醒来/失忆/初入场景。\n\
         人物、地点、未决问题以节拍任务与状态网为准；末句只约束句法衔接。\n\
         ——已有正文末句——\n\
         {last}\n\
         ——请紧接上句继续写（可换段，但人物/地点/目标/未决问题必须承接）——"
    )
}

/// 主创 user prompt：卡全文 → 状态网 → Bundle → 指令 → 卡摘要 → 状态摘要 →
/// 末句锚点。
pub fn render_writer_user_prompt(
    bundle_prompt: &str,
    card: &SceneBeatCard,
    instruction: &str,
    current_content: &str,
    state: Option<&crate::agency::beat_state::BeatState>,
) -> String {
    let state_full = state
        .map(|s| format!("\n\n{}", s.render_full()))
        .unwrap_or_default();
    let state_tail = state
        .map(|s| format!("\n\n{}", s.render_tail_summary()))
        .unwrap_or_default();
    format!(
        "{card_full}{state_full}\n\n{bundle}\n\n【本次创作指令】\n{instruction}\n\n\
         须在节拍任务硬约束内落实指令核心意图。\n\n{card_tail}{state_tail}\n\n{ending}",
        card_full = card.render_full(),
        bundle = bundle_prompt,
        instruction = instruction,
        card_tail = card.render_tail_summary(),
        ending = ending_anchor(current_content),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        connection::create_test_pool,
        repositories::{CreateCharacterRequest, CreateStoryRequest, StoryRepository},
    };

    fn story_req(title: &str) -> CreateStoryRequest {
        CreateStoryRequest {
            title: title.into(),
            description: None,
            genre: Some("玄幻".into()),
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        }
    }

    fn char_req(story_id: &str, name: &str) -> CreateCharacterRequest {
        CreateCharacterRequest {
            story_id: story_id.into(),
            name: name.into(),
            background: None,
            personality: None,
            goals: None,
            appearance: None,
            gender: None,
            age: None,
            source: None,
            is_auto_generated: None,
            emotional_core: None,
            emotional_trigger: None,
            emotional_wound: None,
            emotional_need: None,
        }
    }

    fn seed_story_minimal(pool: &DbPool) -> String {
        let story = StoryRepository::new(pool.clone())
            .create(story_req("最小卡"))
            .unwrap();
        CharacterRepository::new(pool.clone())
            .create(char_req(&story.id, "阿岩"))
            .unwrap();
        story.id
    }

    fn seed_three_chars_one_silent(pool: &DbPool) -> String {
        let story = StoryRepository::new(pool.clone())
            .create(story_req("三角色一沉寂"))
            .unwrap();
        let repo = CharacterRepository::new(pool.clone());
        let mut a = char_req(&story.id, "阿岩");
        a.emotional_wound = Some("被师门放逐".into());
        a.emotional_need = Some("讨回公道".into());
        let a = repo.create(a).unwrap();
        repo.create(char_req(&story.id, "林雪")).unwrap();
        let b = repo.create(char_req(&story.id, "顾长夜")).unwrap();
        CharacterRelationshipRepository::new(pool.clone())
            .create(
                &story.id,
                &a.id,
                &b.id,
                "仇敌",
                None,
                None,
                Some("恨"),
                Some(0.9),
                Some("戒备"),
                Some(0.6),
            )
            .unwrap();
        StoryOutlineRepository::new(pool.clone())
            .create(
                &story.id,
                "夜宴破裂。林雪当众质问阿岩。顾长夜抽身。",
                None,
                3,
                None,
            )
            .unwrap();
        let scene_repo = SceneRepository::new(pool.clone());
        let mut conn = pool.get().unwrap();
        let tx = conn.transaction().unwrap();
        for seq in 1..=3 {
            let scene = scene_repo
                .create_in_tx(&tx, &story.id, seq, Some("章"))
                .unwrap();
            scene_repo
                .update_in_tx(
                    &tx,
                    &scene.id,
                    &crate::db::repositories::SceneUpdate {
                        setting_location: Some("雨巷".into()),
                        characters_present: Some(vec!["阿岩".into(), "顾长夜".into()]),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        tx.commit().unwrap();
        story.id
    }

    #[test]
    fn beat_card_does_not_teleport_silent_without_character_move() {
        let pool = create_test_pool().unwrap();
        let sid = seed_three_chars_one_silent(&pool);
        {
            let conn = pool.get().unwrap();
            crate::creative_engine::expansion::write_beat_counters(
                &conn,
                &sid,
                crate::creative_engine::expansion::BeatCounters {
                    append_beats: 1,
                    last_conflict_beat: 1,
                    last_cast_refresh_beat: 1,
                    last_location_beat: 1,
                    last_foreshadow_beat: 1,
                },
            )
            .unwrap();
        }
        let card = compile_beat_card(&pool, &sid, "阿岩站在雨里。顾长夜冷笑。").unwrap();
        assert!(
            !card.cast.iter().any(|c| c.name == "林雪"),
            "无 CharacterMove 不得传送林雪 cast={:?}",
            card.cast
        );
        assert!(!card.conflict_move.action.is_empty());
        assert!(!card.emotion_beat.summary.is_empty());
        assert!(!card.next_outline_node.is_empty());
    }

    #[test]
    fn beat_card_cast_includes_silent_when_character_move_quota() {
        let pool = create_test_pool().unwrap();
        let sid = seed_three_chars_one_silent(&pool);
        {
            let conn = pool.get().unwrap();
            crate::creative_engine::expansion::write_beat_counters(
                &conn,
                &sid,
                crate::creative_engine::expansion::BeatCounters {
                    append_beats: 3,
                    last_conflict_beat: 3,
                    last_cast_refresh_beat: 0,
                    last_location_beat: 3,
                    last_foreshadow_beat: 3,
                },
            )
            .unwrap();
        }
        let card = compile_beat_card(&pool, &sid, "阿岩站在雨里。顾长夜冷笑。").unwrap();
        assert!(
            card.cast.iter().any(|c| c.name == "林雪"),
            "CharacterMove 应将林雪列入 cast={:?}",
            card.cast
        );
    }

    #[test]
    fn beat_card_never_empty_slots() {
        let pool = create_test_pool().unwrap();
        let sid = seed_story_minimal(&pool);
        let card = compile_beat_card(&pool, &sid, "他走了。").unwrap();
        assert!(!card.conflict_move.action.is_empty());
        assert!(!card.emotion_beat.summary.is_empty());
        assert!(!card.next_outline_node.is_empty());
    }

    #[test]
    fn writer_prompt_order_is_card_then_body_then_summary_then_ending() {
        let card = SceneBeatCard {
            cast: vec![CastMember {
                name: "林雪".into(),
                purpose: "回归质问".into(),
            }],
            conflict_move: ConflictMove {
                action: "加压：当众揭穿".into(),
                parties: vec!["林雪".into(), "阿岩".into()],
            },
            emotion_beat: EmotionBeat {
                summary: "林雪伤口=被抛弃".into(),
            },
            next_outline_node: "夜宴破裂".into(),
            expansion_quota: vec![],
            expansion_quota_text: None,
            setting_location: Some("夜宴".into()),
        };
        let prompt =
            render_writer_user_prompt("【红线】不可飞天", &card, "往下写", "他推开门。", None);
        let i_card = prompt.find("【本章节拍任务】").unwrap();
        let i_sum = prompt.find("【节拍摘要】").unwrap();
        let i_end = prompt
            .find("必须从上述末句")
            .unwrap_or_else(|| prompt.find("末句").unwrap());
        assert!(i_card < i_sum);
        assert!(i_sum < i_end);
        assert!(prompt.contains("林雪"));
        assert!(!prompt.contains("最高优先级"));
    }

    #[test]
    fn ending_anchor_empty_when_no_content() {
        assert!(ending_anchor("").is_empty());
        assert!(ending_anchor("   ").is_empty());
    }

    #[test]
    fn ending_anchor_strips_html_tags() {
        let html = "<p>他推开门。</p><p>雨还在下。</p>";
        let a = ending_anchor(html);
        assert!(!a.contains("<p>"), "{a}");
        assert!(!a.contains("</p>"), "{a}");
        assert!(a.contains("雨还在下"), "{a}");
        assert!(a.contains("他推开门"), "{a}");
        assert!(!a.contains("最高优先级"), "{a}");
    }

    #[test]
    fn cast_present_only_from_chapter_tail() {
        let pool = create_test_pool().unwrap();
        let story = StoryRepository::new(pool.clone())
            .create(story_req("末段点名"))
            .unwrap();
        CharacterRepository::new(pool.clone())
            .create(char_req(&story.id, "青梧甲"))
            .unwrap();
        CharacterRepository::new(pool.clone())
            .create(char_req(&story.id, "客栈乙"))
            .unwrap();
        let opening = "青梧甲在雨里立誓。";
        let middle = "闲笔。".repeat(800);
        let ending = "客栈乙扣上匣子。";
        let card =
            compile_beat_card(&pool, &story.id, &format!("{opening}{middle}{ending}")).unwrap();
        let present: Vec<_> = card
            .cast
            .iter()
            .filter(|c| c.purpose.contains("末段已在场"))
            .map(|c| c.name.as_str())
            .collect();
        assert!(
            present.contains(&"客栈乙"),
            "近文人物应标末段在场 cast={:?}",
            card.cast
        );
        assert!(
            !present.contains(&"青梧甲"),
            "开篇人物不得标成末段在场 cast={:?}",
            card.cast
        );
    }

    #[test]
    fn quota_text_for_beats_uses_pai_not_debug_enum() {
        let debt = crate::creative_engine::expansion::ExpansionDebt {
            conflict: 2,
            scene: 0,
            character: 0,
            foreshadow: 0,
        };
        let text = quota_text_for_beats(&debt).expect("quota");
        assert!(text.contains("本拍扩张任务"));
        assert!(text.contains("必须"));
        assert!(!text.contains("ConflictEscalation"));
        assert!(!text.contains("本章扩张任务"));
    }

    #[test]
    fn compile_next_node_does_not_rewind_to_first_sentence() {
        let pool = create_test_pool().unwrap();
        let sid = seed_story_minimal(&pool);
        StoryOutlineRepository::new(pool.clone())
            .create(&sid, "开篇灵堂。钟楼破阵。龙脉重封。", None, 3, None)
            .unwrap();
        let scene_repo = SceneRepository::new(pool.clone());
        let mut conn = pool.get().unwrap();
        let tx = conn.transaction().unwrap();
        let scene = scene_repo.create_in_tx(&tx, &sid, 1, Some("章")).unwrap();
        scene_repo
            .update_in_tx(
                &tx,
                &scene.id,
                &crate::db::repositories::SceneUpdate {
                    outline_content: Some(
                        "进度：开篇灵堂。\n进度：钟楼破阵。\n进度：龙脉重封。".into(),
                    ),
                    ..Default::default()
                },
            )
            .unwrap();
        tx.commit().unwrap();
        let node = compile_next_node(&pool, &sid);
        assert!(!node.starts_with("开篇灵堂"), "rewound: {node}");
        assert!(node.contains("把当前冲突推进一步") || node.contains("不得原地复述"));
    }

    #[test]
    fn compile_conflict_ignores_offstage_enemy() {
        let pool = create_test_pool().unwrap();
        let sid = seed_three_chars_one_silent(&pool);
        let chars = CharacterRepository::new(pool.clone())
            .get_by_story(&sid)
            .unwrap();
        let cast = vec![CastMember {
            name: "林雪".into(),
            purpose: "末段已在场，承接行动".into(),
        }];
        let mv = compile_conflict(&chars, &cast, &pool, &sid, "林雪");
        assert!(!mv.parties.iter().any(|p| p == "顾长夜"));
        assert!(!mv.action.contains("顾长夜"));
    }

    #[test]
    fn third_compile_after_two_idle_appends_has_conflict_quota_zh() {
        let pool = create_test_pool().unwrap();
        let sid = seed_three_chars_one_silent(&pool);
        let scenes = SceneRepository::new(pool.clone())
            .get_by_story(&sid)
            .unwrap();
        let scene_id = scenes.last().unwrap().id.clone();
        let card = compile_beat_card(&pool, &sid, "阿岩站在雨里。").unwrap();
        crate::agency::persist::persist_append_with_card(
            &pool,
            &scene_id,
            "阿岩站在雨里。",
            &"续写增量正文。".repeat(30),
            &card,
        )
        .unwrap();
        crate::agency::persist::persist_append_with_card(
            &pool,
            &scene_id,
            "阿岩站在雨里。",
            &"续写增量正文。".repeat(30),
            &card,
        )
        .unwrap();
        let card3 = compile_beat_card(&pool, &sid, "阿岩站在雨里。").unwrap();
        let text = card3.expansion_quota_text.clone().unwrap_or_default();
        let full = card3.render_full();
        assert!(
            text.contains("冲突") || full.contains("本拍扩张任务"),
            "quota missing: {full}"
        );
        assert!(!full.contains("ConflictEscalation"));
    }
}
