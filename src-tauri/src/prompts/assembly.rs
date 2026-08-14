//! Prompt assembly：Instruction + Context + Tools → (system, user)。
//! 哑拼接器。本模块不得依赖 `agency`。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    System,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    Instruction,
    Context,
    Tools,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: &'static str,
    pub kind: LayerKind,
    pub slot: Slot,
    pub body: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledPrompt {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssembleError {
    DuplicateId(&'static str),
    MissingRequired(&'static str),
}

impl std::fmt::Display for AssembleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "prompt assembly: duplicate layer id `{id}`"),
            Self::MissingRequired(id) => {
                write!(f, "prompt assembly: required layer `{id}` is empty")
            }
        }
    }
}

impl std::error::Error for AssembleError {}

pub fn assemble(layers: &[Layer]) -> Result<AssembledPrompt, AssembleError> {
    let mut seen = std::collections::HashSet::new();
    let mut system_parts: Vec<&str> = Vec::new();
    let mut user_parts: Vec<&str> = Vec::new();

    for layer in layers {
        if !seen.insert(layer.id) {
            return Err(AssembleError::DuplicateId(layer.id));
        }
        let body = layer.body.trim();
        if body.is_empty() {
            if layer.required {
                return Err(AssembleError::MissingRequired(layer.id));
            }
            continue;
        }
        match layer.slot {
            Slot::System => system_parts.push(body),
            Slot::User => user_parts.push(body),
        }
    }

    Ok(AssembledPrompt {
        system: system_parts.join("\n\n"),
        user: user_parts.join("\n\n"),
    })
}

pub const GENESIS_FIRST_CHAPTER_SYSTEM: &str = "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾、不得发明与资产冲突的角色或设定。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。";

pub const GENESIS_PROSE_FALLBACK_SYSTEM: &str = "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。";

pub const CONTINUE_BEAT_SYSTEM: &str = "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。须在节拍任务硬约束内落实指令。";

pub const TOOL_LOOP_PROTOCOL: &str = "你只能输出一个 JSON action，不要输出其他内容：\n\
- 调用工具: {\"type\":\"tool\",\"name\":\"<工具名>\",\"args\":{...}}\n\
- 完成任务: {\"type\":\"final\",\"content\":\"<最终产出>\"}";

const GENESIS_FIRST_CHAPTER_TASK: &str = "写作要求：第一章正文，1500-2500 字，只输出正文，不写标题。须紧扣故事大纲的起因（第一幕）开篇。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。";

const GENESIS_PROSE_FALLBACK_TASK: &str = "写作要求：章节正文，1500-2500 字，只输出正文，不写标题。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。";

fn l(id: &'static str, kind: LayerKind, slot: Slot, body: String, required: bool) -> Layer {
    Layer {
        id,
        kind,
        slot,
        body,
        required,
    }
}

pub fn assemble_genesis_first_chapter(
    premise: &str,
    concept_json: &str,
    assets_ctx: &str,
) -> Result<AssembledPrompt, AssembleError> {
    assemble(&[
        l(
            "instruction",
            LayerKind::Instruction,
            Slot::System,
            GENESIS_FIRST_CHAPTER_SYSTEM.to_string(),
            true,
        ),
        l(
            "premise",
            LayerKind::Context,
            Slot::User,
            format!("故事前提：{premise}"),
            true,
        ),
        l(
            "concept",
            LayerKind::Context,
            Slot::User,
            format!("概念设定：{concept_json}"),
            true,
        ),
        l(
            "assets",
            LayerKind::Context,
            Slot::User,
            format!("创作资产：\n{assets_ctx}"),
            true,
        ),
        l(
            "task",
            LayerKind::Context,
            Slot::User,
            GENESIS_FIRST_CHAPTER_TASK.to_string(),
            true,
        ),
    ])
}

pub fn assemble_genesis_prose_fallback(
    premise: &str,
    assets_ctx: &str,
) -> Result<AssembledPrompt, AssembleError> {
    assemble(&[
        l(
            "instruction",
            LayerKind::Instruction,
            Slot::System,
            GENESIS_PROSE_FALLBACK_SYSTEM.to_string(),
            true,
        ),
        l(
            "premise",
            LayerKind::Context,
            Slot::User,
            format!("故事前提：{premise}"),
            true,
        ),
        l(
            "assets",
            LayerKind::Context,
            Slot::User,
            format!("创作资产：\n{assets_ctx}"),
            true,
        ),
        l(
            "task",
            LayerKind::Context,
            Slot::User,
            GENESIS_PROSE_FALLBACK_TASK.to_string(),
            true,
        ),
    ])
}

pub fn assemble_continue_beat(user: &str) -> Result<AssembledPrompt, AssembleError> {
    assemble(&[
        l(
            "instruction",
            LayerKind::Instruction,
            Slot::System,
            CONTINUE_BEAT_SYSTEM.to_string(),
            true,
        ),
        l(
            "continue_user",
            LayerKind::Context,
            Slot::User,
            user.to_string(),
            true,
        ),
    ])
}

pub fn assemble_tool_loop_head(
    catalog: &str,
    task: &str,
) -> Result<AssembledPrompt, AssembleError> {
    // P0 对外串等价；trim 与历史 format 对 catalog 尾 NL 不一致。
    assemble(&[
        l(
            "tools",
            LayerKind::Tools,
            Slot::User,
            catalog.to_string(),
            true,
        ),
        l(
            "protocol",
            LayerKind::Instruction,
            Slot::User,
            TOOL_LOOP_PROTOCOL.to_string(),
            true,
        ),
        l(
            "task",
            LayerKind::Context,
            Slot::User,
            format!("任务：\n{task}"),
            true,
        ),
    ])?;
    Ok(AssembledPrompt {
        system: String::new(),
        user: format!("{}\n\n{}\n\n任务：\n{}", catalog, TOOL_LOOP_PROTOCOL, task),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(id: &'static str, slot: Slot, body: &str, required: bool) -> Layer {
        Layer {
            id,
            kind: LayerKind::Context,
            slot,
            body: body.to_string(),
            required,
        }
    }

    #[test]
    fn assemble_joins_system_and_user_with_blank_line() {
        let out = assemble(&[
            layer("a", Slot::System, "SYS-A", true),
            layer("b", Slot::System, "SYS-B", true),
            layer("c", Slot::User, "USR-C", true),
            layer("d", Slot::User, "USR-D", true),
        ])
        .unwrap();
        assert_eq!(out.system, "SYS-A\n\nSYS-B");
        assert_eq!(out.user, "USR-C\n\nUSR-D");
    }

    #[test]
    fn assemble_rejects_duplicate_id() {
        let err = assemble(&[
            layer("same", Slot::System, "x", true),
            layer("same", Slot::User, "y", true),
        ])
        .unwrap_err();
        assert_eq!(err, AssembleError::DuplicateId("same"));
    }

    #[test]
    fn assemble_rejects_missing_required() {
        let err = assemble(&[layer("need", Slot::System, "  \n", true)]).unwrap_err();
        assert_eq!(err, AssembleError::MissingRequired("need"));
    }

    #[test]
    fn assemble_skips_empty_optional() {
        let out = assemble(&[
            layer("keep", Slot::User, "BODY", true),
            layer("skip", Slot::User, "  ", false),
        ])
        .unwrap();
        assert_eq!(out.user, "BODY");
        assert!(out.system.is_empty());
    }

    #[test]
    fn genesis_first_chapter_matches_legacy_format() {
        let premise = "一部间谍小说";
        let concept = "{\"logline\":\"x\"}";
        let assets = "【世界观】双星";
        let out = assemble_genesis_first_chapter(premise, concept, assets).unwrap();
        assert_eq!(
            out.system,
            "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾、不得发明与资产冲突的角色或设定。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。"
        );
        let expected_user = format!(
            "故事前提：{}\n\n概念设定：{}\n\n创作资产：\n{}\n\n写作要求：第一章正文，1500-2500 字，只输出正文，不写标题。须紧扣故事大纲的起因（第一幕）开篇。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。",
            premise, concept, assets
        );
        assert_eq!(out.user, expected_user);
    }

    #[test]
    fn genesis_prose_fallback_matches_legacy_format() {
        let out = assemble_genesis_prose_fallback("前提", "资产正文").unwrap();
        assert_eq!(
            out.system,
            "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。"
        );
        assert_eq!(
            out.user,
            "故事前提：前提\n\n创作资产：\n资产正文\n\n写作要求：章节正文，1500-2500 字，只输出正文，不写标题。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。"
        );
    }

    #[test]
    fn continue_beat_keeps_user_opaque_and_locks_system() {
        let user = "【节拍任务】\n去码头";
        let out = assemble_continue_beat(user).unwrap();
        assert_eq!(
            out.system,
            "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。须在节拍任务硬约束内落实指令。"
        );
        assert_eq!(out.user, user);
    }

    #[test]
    fn continue_beat_anti_repeat_suffix_is_concat() {
        let out = assemble_continue_beat("u").unwrap();
        let retry = format!("{} 禁止重复同一段落或意象循环，不得首尾回环。", out.system);
        assert!(retry.starts_with(&out.system));
        assert!(retry.contains("不得首尾回环"));
    }

    #[test]
    fn tool_loop_head_matches_legacy_format() {
        let catalog = "可用工具（JSON action 调用）：\n- board_read: 读\n  参数: {}\n";
        let task = "写一章";
        let out = assemble_tool_loop_head(catalog, task).unwrap();
        assert!(out.system.is_empty());
        let expected = format!(
            "{}\n\n你只能输出一个 JSON action，不要输出其他内容：\n\
             - 调用工具: {{\"type\":\"tool\",\"name\":\"<工具名>\",\"args\":{{...}}}}\n\
             - 完成任务: {{\"type\":\"final\",\"content\":\"<最终产出>\"}}\n\n任务：\n{}",
            catalog, task
        );
        assert_eq!(out.user, expected);
    }

    #[test]
    fn tool_loop_head_rejects_empty_catalog() {
        let err = assemble_tool_loop_head(" ", "写一章").unwrap_err();
        assert_eq!(err, AssembleError::MissingRequired("tools"));
    }
}
