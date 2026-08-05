//! 统一 Prompt 模板系统
//!
//! 每个叙事元素的 Prompt 用于拆书提取（从文本分析）；Generate 模式已随
//! v0.31 Genesis 生成族提示词删除。

use crate::db::DbPool;

/// v0.21.0: 从 PromptRegistry 读取模板并渲染变量
///
/// 若 registry 不可用或 key 不存在，回退到提供的默认模板。
fn resolve_and_render(
    prompt_id: &str,
    default_template: &str,
    vars: &[(&str, &str)],
    pool: Option<&DbPool>,
) -> String {
    let template = if let Some(pool) = pool {
        crate::prompts::registry::resolve_prompt(pool, prompt_id)
            .unwrap_or_else(|_| default_template.to_string())
    } else {
        crate::prompts::registry::resolve_prompt_default(prompt_id)
            .unwrap_or_else(|| default_template.to_string())
    };

    let mut vars_map = std::collections::HashMap::new();
    for (k, v) in vars {
        vars_map.insert(k.to_string(), v.to_string());
    }
    crate::prompts::engine::TemplateEngine::render_with_conditions(&template, &vars_map)
}

// ==================== 故事概念 Prompt ====================

pub fn story_concept_prompt(context: &str, pool: Option<&DbPool>) -> String {
    resolve_and_render(
        "narrative_story_concept_extract",
        r#"你是一位资深小说编辑。请从以下小说文本中，提取故事的基本信息。

文本片段：
{{text}}

请用 JSON 格式回复：
{
  "title": "小说标题（如无法确定则为null）",
  "author": "作者姓名（文本中可识别则填写，否则为null）",
  "description": "一句话简介（30-50字，如无法确定则为null）",
  "genre": "题材（如：玄幻、都市、穿越、科幻、武侠等）",
  "tone": "文风基调（如：热血、暗黑、轻松、沉重）",
  "pacing": "叙事节奏（如：快节奏、慢热、跌宕起伏）",
  "themes": ["主题1", "主题2"],
  "target_length": "估计篇幅"
}

要求：
1. 基于文本内容推断，不要虚构
2. 如某信息文本中未体现，标记为null
3. 只输出 JSON，不要其他内容"#,
        &[("text", context)],
        pool,
    )
}

// ==================== 世界观 Prompt ====================

pub fn world_building_prompt(
    story_title: &str,
    genre: &str,
    context: &str,
    pool: Option<&DbPool>,
) -> String {
    resolve_and_render(
        "narrative_world_building_extract",
        r#"你是一位世界观分析专家。请从以下小说文本中，提取世界观设定。

故事：《{{title}}》
题材：{{genre}}

文本片段：
{{text}}

请用 JSON 格式回复：
{
  "concept": "世界观核心概念（50-100字，基于文本推断）",
  "rules": [
    {"name": "规则名称", "description": "规则描述", "rule_type": "physical|magic|social|historical", "importance": 8}
  ],
  "history": "世界历史背景（基于文本推断，200-300字）",
  "key_locations": ["关键地点1", "关键地点2"],
  "power_system": "力量体系概述（如有）"
}

要求：
1. 基于文本内容推断，不要虚构
2. 规则从文本中的描写归纳总结
3. 只输出 JSON"#,
        &[("title", story_title), ("genre", genre), ("text", context)],
        pool,
    )
}

// ==================== 角色 Prompt ====================

pub fn character_prompt(
    story_title: &str,
    genre: &str,
    context: &str,
    pool: Option<&DbPool>,
) -> String {
    resolve_and_render(
        "narrative_character_extract",
        r#"你是一位角色分析专家。请从以下小说文本中，提取所有出现的人物角色。

故事：《{{title}}》
题材：{{genre}}

文本片段：
{{text}}

请用 JSON 格式回复：
{
  "characters": [
    {
      "name": "人物姓名",
      "role_type": "角色定位（主角/反派/配角/龙套/提及）",
      "personality": "性格特征（基于文本描写）",
      "background": "背景故事（基于文本推断）",
      "goals": "核心目标（如有）",
      "fears": "深层恐惧（如有）",
      "appearance": "外貌描写（如有）",
      "gender": "男/女/其他",
      "age": 25,
      "importance_score": 7,
      "relationships": [{"target_name": "另一个角色名", "relation_type": "关系性质", "description": "关系描述"}]
    }
  ]
}

要求：
1. 只提取文本中实际出现或有明确描写的人物
2. 仅被提及但未出场，role_type 标记为"提及"
3. importance_score 根据重要性打分（1-10）
4. 只输出 JSON"#,
        &[("title", story_title), ("genre", genre), ("text", context)],
        pool,
    )
}

// ==================== 场景 Prompt ====================

pub fn scene_prompt(
    story_title: &str,
    genre: &str,
    context: &str,
    pool: Option<&DbPool>,
) -> String {
    resolve_and_render(
        "narrative_scene_extract",
        r#"你是一位场景分析专家。请从以下小说文本中，提取所有场景/章节。

故事：《{{title}}》
题材：{{genre}}

文本片段：
{{text}}

请用 JSON 格式回复：
{
  "scenes": [
    {
      "sequence_number": 1,
      "title": "场景标题（如有）",
      "summary": "场景内容概要（100-200字）",
      "dramatic_goal": "本场景的戏剧目标（基于内容推断）",
      "external_pressure": "外部压力/阻碍（如有）",
      "conflict_type": "man_vs_man|man_vs_self|...",
      "setting_location": "地点",
      "setting_time": "时间",
      "characters_present": ["角色名1", "角色名2"],
      "key_events": ["关键事件1", "关键事件2"],
      "emotional_tone": "情感基调（如：紧张/温馨/悲伤/激昂）"
    }
  ]
}

要求：
1. 按文本顺序排列场景
2. 提取每个场景的核心冲突和情感基调
3. 列出场景中出场的所有人物
4. 只输出 JSON"#,
        &[("title", story_title), ("genre", genre), ("text", context)],
        pool,
    )
}

// ==================== 伏笔 Prompt ====================

pub fn foreshadowing_prompt(
    story_title: &str,
    genre: &str,
    context: &str,
    pool: Option<&DbPool>,
) -> String {
    resolve_and_render(
        "narrative_foreshadowing_extract",
        r#"你是一位伏笔分析专家。请从以下小说文本中，提取所有伏笔（已埋设的暗示和线索）。

故事：《{{title}}》
题材：{{genre}}

文本片段：
{{text}}

请用 JSON 格式回复：
{
  "foreshadowings": [
    {
      "content": "伏笔内容描述（基于文本中的具体描写）",
      "importance": 8,
      "target_act": 2,
      "hint_style": "暗示风格（如：环境隐喻、对话暗示、物品象征、预言梦境）",
      "setup_scene": "埋设伏笔的场景描述"
    }
  ]
}

要求：
1. 只提取文本中实际存在的暗示和线索
2. 区分已明确回收的伏笔和尚未回收的伏笔
3. importance 根据伏笔对整体故事的重要性打分
4. 只输出 JSON"#,
        &[("title", story_title), ("genre", genre), ("text", context)],
        pool,
    )
}

// ==================== 故事线/弧光 Prompt ====================

pub fn story_arc_prompt(story_title: &str, context: &str, pool: Option<&DbPool>) -> String {
    resolve_and_render(
        "narrative_story_arc_extract",
        r#"你是一位故事线分析专家。请从以下小说章节概要中，提取故事线结构。

故事：《{{title}}》

章节概要：
{{text}}

请用 JSON 格式回复：
{
  "main_arc": "主线故事（基于概要推断）",
  "sub_arcs": ["支线1", "支线2"],
  "climaxes": ["高潮点1", "高潮点2"],
  "turning_points": ["转折点1", "转折点2"]
}

要求：
1. 基于章节概要推断故事结构
2. 如果文本不完整，标注待补充
3. 只输出 JSON"#,
        &[("title", story_title), ("text", context)],
        pool,
    )
}

// ==================== 提示词框架目录 (v0.23.61) ====================

/// 生成紧凑的提示词框架目录 JSON，供 Call 1 最快模型选择创作框架。
pub fn build_prompt_framework_catalog() -> String {
    serde_json::json!({
        "methodologies": [
            {"id": "snowflake", "name": "雪花法", "steps": 10, "适合": "规划型作者"},
            {"id": "hero_journey", "name": "英雄之旅", "stages": 12, "适合": "史诗/奇幻/冒险"},
            {"id": "scene_structure", "name": "场景结构法", "适合": "电影化写作"},
            {"id": "character_depth", "name": "角色深度模型", "适合": "角色驱动型"},
            {"id": "hdwb", "name": "高密度世界构建", "phases": 4, "适合": "复杂世界观"}
        ],
        "quality_gates": [
            {"id": "pipeline_review", "用途": "深度审稿(5维评分)"},
            {"id": "audit_quality_inspector", "用途": "11维审计(后台静默)"},
            {"id": "mini_review_system", "用途": "轻量合同检查(默认)"}
        ],
        "contextual_injectors": [
            {"id": "writer_contract_constraints", "触发": "故事合同已设置时"},
            {"id": "writer_chase_debt", "触发": "有未回收伏笔时"},
            {"id": "writer_narrative_event_history", "触发": "已有前文内容时"}
        ]
    })
    .to_string()
}
