//! 拍级状态网。设计：docs/plans/2026-08-15-continue-quality-closure-design.md
//! §7

use crate::{
    agency::{
        beat_card::{CastMember, SceneBeatCard},
        continue_director::DirectorLock,
    },
    creative_engine::expansion::debt::QuotaItem,
};

#[derive(Debug, Clone)]
pub struct OpenThread {
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct BeatState {
    pub present: Vec<String>,
    pub locations: Vec<(String, String)>,
    pub threads: Vec<OpenThread>,
    /// 角色表内、本拍未在场的已登记名。探针用来拦场外开篇。
    pub offshot: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BeatProbe {
    pub named_cast: usize,
    pub gaps: Vec<String>,
}

impl BeatState {
    pub fn render_full(&self) -> String {
        let loc = self
            .locations
            .iter()
            .map(|(n, l)| format!("{n}={l}"))
            .collect::<Vec<_>>()
            .join("；");
        let mut lines = vec![
            "【本拍状态网】".into(),
            format!("在场：{}", self.present.join("、")),
            format!("地点：{loc}"),
        ];
        if self.threads.is_empty() {
            lines.push("未决：承接当前冲突，不得原地复述末句。".into());
        } else {
            let t = self
                .threads
                .iter()
                .enumerate()
                .map(|(i, th)| format!("{}. {}", i + 1, th.text))
                .collect::<Vec<_>>()
                .join(" ");
            lines.push(format!("未决：{t}"));
        }
        lines.push("在场者可以不出声。禁止写成他们不在场。禁止把同一人的别名写成另一个人。".into());
        lines.join("\n")
    }

    pub fn render_tail_summary(&self) -> String {
        let thread = self
            .threads
            .first()
            .map(|t| t.text.chars().take(40).collect::<String>())
            .unwrap_or_default();
        format!(
            "【状态摘要】在场：{}｜未决：{}",
            self.present.join("、"),
            thread
        )
    }
}

pub fn compile_beat_state(
    present: &[String],
    location: Option<&str>,
    next_node: &str,
    overdue: &[String],
    tail: &str,
    progress_lines: &[String],
) -> BeatState {
    let loc = location.unwrap_or("").trim();
    let locations = if loc.is_empty() {
        vec![]
    } else {
        present
            .iter()
            .map(|n| (n.clone(), loc.to_string()))
            .collect()
    };
    let mut threads: Vec<OpenThread> = Vec::new();
    fn push(threads: &mut Vec<OpenThread>, raw: &str) {
        let t: String = raw.chars().take(80).collect();
        let t = t.trim().to_string();
        if t.is_empty() {
            return;
        }
        if threads.iter().any(|x| x.text == t) {
            return;
        }
        if threads.len() < 5 {
            threads.push(OpenThread { text: t });
        }
    }
    push(&mut threads, next_node);
    for o in overdue {
        push(&mut threads, o);
    }
    for p in progress_lines {
        if threads.len() >= 5 {
            break;
        }
        if p.contains("进度：") {
            push(&mut threads, p.trim());
        }
    }
    const SIGNALS: &[&str] = &["必须", "之前", "否则", "子时", "七日"];
    for sent in tail.split(['。', '！', '？', '\n']) {
        if threads.len() >= 5 {
            break;
        }
        if SIGNALS.iter().any(|s| sent.contains(s)) {
            push(&mut threads, sent);
        }
    }
    BeatState {
        present: present.to_vec(),
        locations,
        threads,
        offshot: Vec::new(),
    }
}

pub fn probe_increment(
    increment: &str,
    card: &SceneBeatCard,
    state: &BeatState,
    quota: &[QuotaItem],
    lock: Option<&DirectorLock>,
) -> BeatProbe {
    let cast_names: Vec<String> = card.cast.iter().map(|c| c.name.clone()).collect();
    let matched = crate::agency::continue_assets::match_character_names(&cast_names, increment);
    let named_cast = matched.len();
    let mut gaps = Vec::new();
    if quota.contains(&QuotaItem::ConflictEscalation) {
        let living_parties: Vec<&String> = card
            .conflict_move
            .parties
            .iter()
            .filter(|p| !card.dead.iter().any(|d| d == *p))
            .collect();
        let one_living = living_parties
            .iter()
            .any(|p| matched.iter().any(|n| n == *p));
        let verb = crate::agency::continue_assets::has_conflict_verb(increment);
        if !living_parties.is_empty() && !one_living && !verb {
            gaps.push("未落实冲突加压".into());
        }
    }
    if quota.contains(&QuotaItem::NewScene) {
        let stayed = card
            .setting_location
            .as_deref()
            .map(|loc| increment.contains(loc))
            .unwrap_or(false);
        let moved = ["离开", "潜入", "进入", "前往"]
            .iter()
            .any(|v| increment.contains(v));
        if stayed || !moved {
            gaps.push("未离开当前场景".into());
        }
    }
    if quota.contains(&QuotaItem::CharacterMove) {
        let silent: Vec<&CastMember> = card
            .cast
            .iter()
            .filter(|c| c.purpose.contains("沉寂") || c.purpose.contains("入场"))
            .collect();
        if silent.iter().any(|c| !matched.iter().any(|n| n == &c.name)) {
            gaps.push("沉寂角色未入场".into());
        }
    }
    if !quota.contains(&QuotaItem::NewScene) && !state.offshot.is_empty() {
        let opening: String = increment.chars().take(80).collect();
        let off = crate::agency::continue_assets::match_character_names(&state.offshot, &opening);
        let on = crate::agency::continue_assets::match_character_names(&state.present, &opening);
        if !off.is_empty() && on.is_empty() {
            gaps.push("增量以场外角色开篇".into());
        }
    }
    if crate::agency::continue_assets::increment_replays_completed_deaths(increment, &card.dead) {
        gaps.push("重演已完成的死亡或行刺".into());
    }
    if let Some(lock) = lock {
        gaps.extend(crate::agency::continue_director::subject_split_gaps(
            increment, lock,
        ));
        gaps.extend(crate::agency::continue_director::kin_inversion_gaps(
            increment, lock,
        ));
        gaps.extend(crate::agency::continue_director::dead_acting_gaps(
            increment, lock,
        ));
    }
    BeatProbe { named_cast, gaps }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agency::beat_card::{ConflictMove, EmotionBeat, SceneBeatCard};

    #[test]
    fn beat_state_includes_next_node_and_overdue() {
        let st = compile_beat_state(
            &["沈砚".into(), "白芷".into()],
            Some("钟楼"),
            "子时前破金煞",
            &["五阵未破".into()],
            "沈砚握着罗盘。必须在子时前动手，否则龙脉裂口。",
            &["进度：灵堂托梦".to_string()],
        );
        assert!(st.present.contains(&"沈砚".into()));
        assert!(st.locations.iter().any(|(n, l)| n == "沈砚" && l == "钟楼"));
        assert!(st
            .threads
            .iter()
            .any(|t| t.text.contains("金煞") || t.text.contains("子时")));
        assert!(st.threads.iter().any(|t| t.text.contains("五阵")));
        assert!(st.threads.len() <= 5);
        let full = st.render_full();
        assert!(full.contains("【本拍状态网】"));
        assert!(full.contains("未决"));
    }

    #[test]
    fn probe_reports_missing_cast_and_unshifted_location() {
        let card = SceneBeatCard {
            cast: vec![
                CastMember {
                    name: "阿岩".into(),
                    purpose: "末段已在场".into(),
                },
                CastMember {
                    name: "林雪".into(),
                    purpose: "末段已在场".into(),
                },
            ],
            conflict_move: ConflictMove {
                action: "加压".into(),
                parties: vec!["阿岩".into(), "林雪".into()],
            },
            emotion_beat: EmotionBeat {
                summary: "怒".into(),
            },
            next_outline_node: "夜宴破裂".into(),
            expansion_quota: vec![QuotaItem::NewScene, QuotaItem::ConflictEscalation],
            expansion_quota_text: None,
            setting_location: Some("雨巷".into()),
            open_review_issues: vec![],
            dead: vec![],
        };
        let state = BeatState {
            present: vec!["阿岩".into(), "林雪".into()],
            locations: vec![("阿岩".into(), "雨巷".into())],
            threads: vec![],
            offshot: vec![],
        };
        let probe = probe_increment(
            "他叹了口气，继续喝茶。",
            &card,
            &state,
            &[QuotaItem::NewScene, QuotaItem::ConflictEscalation],
            None,
        );
        assert!(!probe.gaps.is_empty());
        assert!(probe.gaps.join("").contains("在场") || probe.named_cast < 2);
    }

    #[test]
    fn probe_rejects_offshot_pov_opening() {
        let card = SceneBeatCard {
            cast: vec![
                CastMember {
                    name: "苏亦铁".into(),
                    purpose: "末段已在场".into(),
                },
                CastMember {
                    name: "曹元佩".into(),
                    purpose: "末段已在场".into(),
                },
            ],
            conflict_move: ConflictMove {
                action: "加压".into(),
                parties: vec!["苏亦铁".into(), "曹元佩".into()],
            },
            emotion_beat: EmotionBeat {
                summary: "惊".into(),
            },
            next_outline_node: "留在大堂".into(),
            expansion_quota: vec![],
            expansion_quota_text: None,
            setting_location: Some("镇北王府大堂".into()),
            open_review_issues: vec![],
            dead: vec![],
        };
        let state = BeatState {
            present: vec!["苏亦铁".into(), "曹元佩".into()],
            locations: vec![("苏亦铁".into(), "镇北王府大堂".into())],
            threads: vec![],
            offshot: vec!["费迪南三世".into()],
        };
        let probe = probe_increment(
            "费迪南三世在都城宫殿里批阅奏折，烟火节的税单堆满御案。",
            &card,
            &state,
            &[],
            None,
        );
        assert!(
            probe.gaps.iter().any(|g| g.contains("场外")),
            "须拦截场外开篇 gaps={:?}",
            probe.gaps
        );
    }

    #[test]
    fn probe_rejects_replay_of_completed_stab() {
        let card = SceneBeatCard {
            cast: vec![
                CastMember {
                    name: "苏亦铁".into(),
                    purpose: "末段已在场".into(),
                },
                CastMember {
                    name: "景亲王".into(),
                    purpose: "末段已在场".into(),
                },
            ],
            conflict_move: ConflictMove {
                action: "加压".into(),
                parties: vec!["苏亦铁".into(), "景亲王".into()],
            },
            emotion_beat: EmotionBeat {
                summary: "悲愤".into(),
            },
            next_outline_node: "当众驳斥谋反".into(),
            expansion_quota: vec![],
            expansion_quota_text: None,
            setting_location: Some("镇北王府大堂".into()),
            open_review_issues: vec![],
            dead: vec!["苏会山".into(), "明成公主".into()],
        };
        let state = BeatState {
            present: vec!["苏亦铁".into(), "景亲王".into()],
            locations: vec![("苏亦铁".into(), "镇北王府大堂".into())],
            threads: vec![],
            offshot: vec![],
        };
        let rewind = "明成公主将短刃狠狠刺入了苏会山的胸口。苏会山头脸崩裂。";
        let probe = probe_increment(rewind, &card, &state, &[], None);
        assert!(
            probe.gaps.iter().any(|g| g.contains("重演")),
            "须拦截重演刺杀 gaps={:?}",
            probe.gaps
        );
        let forward = "苏亦铁扑向父亲的尸体。景亲王的护卫大喊谋反。曹元佩僵在座上。";
        let ok = probe_increment(forward, &card, &state, &[], None);
        assert!(
            !ok.gaps.iter().any(|g| g.contains("重演")),
            "点名尸体不得算重演 gaps={:?}",
            ok.gaps
        );
    }

    #[test]
    fn probe_does_not_gap_silent_present() {
        let card = SceneBeatCard {
            cast: vec![
                CastMember {
                    name: "苏亦铁".into(),
                    purpose: "可沉默".into(),
                },
                CastMember {
                    name: "曹元佩".into(),
                    purpose: "可沉默".into(),
                },
            ],
            conflict_move: ConflictMove {
                action: "加压".into(),
                parties: vec!["苏亦铁".into()],
            },
            emotion_beat: EmotionBeat {
                summary: "悲".into(),
            },
            next_outline_node: String::new(),
            expansion_quota: vec![],
            expansion_quota_text: None,
            setting_location: Some("大堂".into()),
            open_review_issues: vec![],
            dead: vec!["苏会山".into()],
        };
        let state = BeatState {
            present: vec!["苏亦铁".into(), "曹元佩".into()],
            locations: vec![],
            threads: vec![],
            offshot: vec![],
        };
        let probe = probe_increment(
            "苏亦铁扑向父亲的尸体，指尖触到冰冷的骨骼。",
            &card,
            &state,
            &[],
            None,
        );
        assert!(
            !probe.gaps.iter().any(|g| g.contains("丢掉已在场者")),
            "gaps={:?}",
            probe.gaps
        );
    }
}
