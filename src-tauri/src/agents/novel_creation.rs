//! 小说创建 Agent
//!
//! 负责引导式生成小说核心要素：世界观、角色谱、文字风格
#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use serde_json;

pub use crate::domain::novel_creation::{
    CharacterProfileOption, WorldBuildingOption, WritingStyleOption,
};
use crate::{db::DbPool, llm::LlmService, router::TaskType};

/// 小说创建 Agent
pub struct NovelCreationAgent {
    llm_service: LlmService,
    pool: DbPool,
}

/// 生成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// 生成数量（默认3）
    pub count: usize,
    /// 创意程度 (0.0-1.0)
    pub creativity: f32,
    /// 详细程度
    pub detail_level: DetailLevel,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            count: 3,
            creativity: 0.8,
            detail_level: DetailLevel::Normal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetailLevel {
    Brief,
    Normal,
    Detailed,
}

impl NovelCreationAgent {
    pub fn new(llm_service: LlmService, pool: DbPool) -> Self {
        Self { llm_service, pool }
    }

    /// 第一步：根据用户输入生成世界观选项
    pub async fn generate_world_building_options(
        &self,
        user_input: &str,
        options: &GenerationOptions,
    ) -> Result<Vec<WorldBuildingOption>, Box<dyn std::error::Error>> {
        // v0.21.0: 优先从 PromptRegistry 读取
        let prompt = if let Some(tpl) =
            crate::prompts::registry::resolve_prompt(&self.pool, "novel_creation_world_options")
                .ok()
                .or_else(|| {
                    crate::prompts::registry::resolve_prompt_default("novel_creation_world_options")
                }) {
            let mut vars = std::collections::HashMap::new();
            vars.insert("count".to_string(), options.count.to_string());
            vars.insert("input".to_string(), user_input.to_string());
            crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &vars)
        } else {
            format!(
                r#"作为一位资深世界观设计师，请基于以下用户输入，创建{}个独特的世界观概念。

用户输入：{}

要求：
1. 每个世界观应该有独特的核心概念
2. 包含基本的世界规则（3-5条）
3. 有历史背景概述
4. 包含2-3个主要文化设定

请以JSON格式输出，格式如下：
{{
  "world_buildings": [
    {{
      "id": "wb_1",
      "concept": "世界观核心概念（20-50字）",
      "rules": [
        {{"name": "规则名称", "description": "规则描述", "rule_type": "Magic", "importance": 8}}
      ],
      "history": "历史背景（100-200字）",
      "cultures": [
        {{"name": "文化名称", "description": "文化描述", "customs": ["习俗1", "习俗2"], "values": ["价值观1", "价值观2"]}}
      ]
    }}
  ]
}}

注意：
- 世界观类型可以是：玄幻、科幻、都市、历史、武侠、悬疑等
- 规则类型包括：Magic（魔法）、Technology（科技）、Social（社会）、Physical（物理）
- importance 范围 1-10
- 确保JSON格式正确"#,
                options.count, user_input
            )
        };

        let response = self
            .llm_service
            .generate_for_task(
                TaskType::WorldBuilding,
                prompt,
                None,
                None,
                Some("世界观选项"),
            )
            .await?;
        match Self::parse_world_options_response(&response.content) {
            Ok(options) => Ok(options),
            Err(e) => {
                let snippet: String = response.content.chars().take(200).collect();
                log::warn!(
                    "novel_creation: 世界观选项解析失败 err={} raw_len={} snippet={:?}",
                    e,
                    response.content.len(),
                    snippet
                );
                Err(e.into())
            }
        }
    }

    /// 解析世界观选项响应为纯函数（issue #14）：先用 narrative 健壮提取器剥离
    /// markdown 围栏 / 修复字符串内未转义换行，再反序列化。模型常将 JSON 包裹
    /// 在 ` ```json ... ``` ` 中，旧实现直接 `serde_json::from_str` 全量内容会
    /// 静默失败。提取为独立函数便于单测（无需 mock LlmService）。
    fn parse_world_options_response(content: &str) -> Result<Vec<WorldBuildingOption>, String> {
        let sanitized = crate::narrative::extract_and_sanitize_json(content)
            .unwrap_or_else(|_| content.to_string());
        let parsed: serde_json::Value = serde_json::from_str(&sanitized)
            .map_err(|e| format!("世界观选项 JSON 解析失败: {}", e))?;
        parsed["world_buildings"]
            .as_array()
            .ok_or_else(|| "Invalid response format: 缺少 world_buildings 数组".to_string())?
            .iter()
            .map(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| format!("世界观项反序列化失败: {}", e))
            })
            .collect()
    }

    /// 第二步：根据世界观生成角色谱选项
    pub async fn generate_character_profiles(
        &self,
        world_building: &WorldBuildingOption,
        options: &GenerationOptions,
    ) -> Result<Vec<Vec<CharacterProfileOption>>, Box<dyn std::error::Error>> {
        let world_info = format!(
            "世界观概念：{}\n历史背景：{}\n文化设定：{}",
            world_building.concept,
            world_building.history,
            world_building
                .cultures
                .iter()
                .map(|c| format!("{} - {}", c.name, c.description))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // v0.21.0: 优先从 PromptRegistry 读取
        let prompt = if let Some(tpl) =
            crate::prompts::registry::resolve_prompt(&self.pool, "novel_creation_character_roster")
                .ok()
                .or_else(|| {
                    crate::prompts::registry::resolve_prompt_default(
                        "novel_creation_character_roster",
                    )
                }) {
            let mut vars = std::collections::HashMap::new();
            vars.insert("count".to_string(), options.count.to_string());
            vars.insert("input".to_string(), world_info.to_string());
            crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &vars)
        } else {
            format!(
                r#"作为一位角色设计专家，请基于以下世界观，创建{}组不同的角色配置。

{}

要求：
1. 每组包含3-5个核心角色
2. 角色应该代表不同的立场和功能（主角、反派、导师、盟友等）
3. 角色性格应该鲜明，有冲突和互补
4. 考虑世界观对角色塑造的影响

请以JSON格式输出，格式如下：
{{
  "character_sets": [
    [
      {{
        "id": "char_1_1",
        "name": "角色姓名",
        "personality": "性格特点（30-50字）",
        "background": "背景故事（50-100字）",
        "goals": "目标动机（30-50字）",
        "voice_style": "语言风格（20-30字）"
      }}
    ]
  ]
}}

注意：
- 姓名应符合世界观文化背景，且具有辨识度
- 禁止使用林、陈、王、李、张、刘等最常见单字姓；禁止单字名
- 同一组角色姓氏不得重复
- 角色应有明确外貌、性别、年龄描述，避免千人一面
- 每组角色之间应有内在联系和冲突
- 确保JSON格式正确"#,
                options.count, world_info
            )
        };

        let response = self
            .llm_service
            .generate_for_task(
                TaskType::WorldBuilding,
                prompt,
                None,
                None,
                Some("角色谱选项"),
            )
            .await?;
        let parsed: serde_json::Value = serde_json::from_str(&response.content)?;

        let sets: Vec<Vec<CharacterProfileOption>> = parsed["character_sets"]
            .as_array()
            .ok_or("Invalid response format")?
            .iter()
            .map(|arr| {
                arr.as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|v| serde_json::from_value(v.clone()).unwrap())
                    .collect()
            })
            .collect();

        Ok(sets)
    }

    /// 第三步：生成文字风格选项
    pub async fn generate_writing_styles(
        &self,
        genre: &str,
        world_building: &WorldBuildingOption,
        options: &GenerationOptions,
    ) -> Result<Vec<WritingStyleOption>, Box<dyn std::error::Error>> {
        // v0.21.0: 优先从 PromptRegistry 读取
        let prompt = if let Some(tpl) =
            crate::prompts::registry::resolve_prompt(&self.pool, "novel_creation_writing_style")
                .ok()
                .or_else(|| {
                    crate::prompts::registry::resolve_prompt_default("novel_creation_writing_style")
                }) {
            let mut vars = std::collections::HashMap::new();
            vars.insert("count".to_string(), options.count.to_string());
            vars.insert(
                "input".to_string(),
                format!("{} {}", genre, world_building.concept),
            );
            crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &vars)
        } else {
            format!(
                r#"作为一位资深文学编辑，请基于以下小说类型和世界观，创建{}种不同的文字风格。

小说类型：{}
世界观概念：{}

要求：
1. 每种风格应该有独特的名称和描述
2. 明确语调和节奏特点
3. 提供词汇水平和句式结构说明
4. 每种风格配一段示例文本（100-150字）

请以JSON格式输出，格式如下：
{{
  "writing_styles": [
    {{
      "id": "ws_1",
      "name": "风格名称",
      "description": "风格描述（30-50字）",
      "tone": "语调特点",
      "pacing": "节奏特点",
      "vocabulary_level": "词汇水平",
      "sentence_structure": "句式结构",
      "sample_text": "示例文本（100-150字）"
    }}
  ]
}}

注意：
- 风格应该适合所选小说类型
- 示例文本应该能体现该风格特点
- 确保JSON格式正确"#,
                options.count, genre, world_building.concept
            )
        };

        let response = self
            .llm_service
            .generate_for_task(
                TaskType::CreativeWriting,
                prompt,
                None,
                None,
                Some("文字风格选项"),
            )
            .await?;
        let parsed: serde_json::Value = serde_json::from_str(&response.content)?;

        let options: Vec<WritingStyleOption> = parsed["writing_styles"]
            .as_array()
            .ok_or("Invalid response format")?
            .iter()
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .collect();

        Ok(options)
    }

    /// 生成首个场景建议
    pub async fn generate_first_scene(
        &self,
        world_building: &WorldBuildingOption,
        characters: &[CharacterProfileOption],
        writing_style: &WritingStyleOption,
    ) -> Result<SceneProposal, Box<dyn std::error::Error>> {
        let char_info = characters
            .iter()
            .map(|c| format!("{}：{}，{}", c.name, c.personality, c.goals))
            .collect::<Vec<_>>()
            .join("\n");

        // v0.21.0: 优先从 PromptRegistry 读取
        let prompt = if let Some(tpl) =
            crate::prompts::registry::resolve_prompt(&self.pool, "novel_creation_opening_scene")
                .ok()
                .or_else(|| {
                    crate::prompts::registry::resolve_prompt_default("novel_creation_opening_scene")
                }) {
            let mut vars = std::collections::HashMap::new();
            vars.insert(
                "input".to_string(),
                format!(
                    "{} {} {}",
                    world_building.concept, char_info, writing_style.name
                ),
            );
            crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &vars)
        } else {
            format!(
                r#"作为一位场景设计专家，请基于以下设定，设计一个开场场景。

世界观：{}
角色：
{}
文字风格：{}

要求：
1. 场景应该有强烈的戏剧冲突或悬念
2. 展示主要角色的特点和关系
3. 体现世界观的独特元素
4. 符合指定的文字风格

请以JSON格式输出：
{{
  "scene": {{
    "title": "场景标题",
    "dramatic_goal": "戏剧目标",
    "external_pressure": "外部压迫",
    "conflict_type": "冲突类型（ManVsMan/ManVsSelf/ManVsSociety/ManVsNature/ManVsTechnology/ManVsFate/ManVsSupernatural）",
    "setting_location": "地点",
    "setting_time": "时间",
    "setting_atmosphere": "氛围",
    "content": "场景正文（500-800字）"
  }}
}}

注意：
- 场景应该能吸引读者继续阅读
- 确保JSON格式正确"#,
                world_building.concept, char_info, writing_style.name
            )
        };

        let response = self
            .llm_service
            .generate_for_task(
                TaskType::CreativeWriting,
                prompt,
                None,
                None,
                Some("首场景"),
            )
            .await?;
        let parsed: serde_json::Value = serde_json::from_str(&response.content)?;

        let scene: SceneProposal = serde_json::from_value(parsed["scene"].clone())?;
        Ok(scene)
    }
}

/// 场景建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneProposal {
    pub title: String,
    pub dramatic_goal: String,
    pub external_pressure: String,
    pub conflict_type: String,
    pub setting_location: String,
    pub setting_time: String,
    pub setting_atmosphere: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WORLD_JSON: &str = r#"{"world_buildings":[{"id":"wb_1","concept":"双星废土文明","rules":[{"name":"星环律","description":"资源受星环周期约束","rule_type":"physical","importance":9}],"history":"千年前双星相撞","cultures":[{"name":"拾荒者","description":"废土游民","customs":["以物易物"],"values":["生存至上"]}]}]}"#;

    #[test]
    fn test_parse_world_options_clean_json() {
        let opts = NovelCreationAgent::parse_world_options_response(VALID_WORLD_JSON).unwrap();
        assert_eq!(opts.len(), 1);
        assert_eq!(opts[0].id, "wb_1");
        assert_eq!(opts[0].concept, "双星废土文明");
        assert_eq!(opts[0].rules.len(), 1);
        assert_eq!(opts[0].rules[0].importance, 9);
        assert_eq!(opts[0].cultures[0].customs, vec!["以物易物".to_string()]);
    }

    #[test]
    fn test_parse_world_options_markdown_fenced() {
        // issue #14：模型将 JSON 包裹在 ```json ... ``` 代码块中，旧实现直接
        // serde_json::from_str 全量内容会失败；现先经 extract_and_sanitize_json
        // 剥离围栏再解析。
        let raw = format!(
            "好的，以下是世界观选项：\n```json\n{}\n```\n希望你喜欢。",
            VALID_WORLD_JSON
        );
        let opts = NovelCreationAgent::parse_world_options_response(&raw).unwrap();
        assert_eq!(opts.len(), 1);
        assert_eq!(opts[0].concept, "双星废土文明");
    }

    #[test]
    fn test_parse_world_options_missing_key_errors() {
        // prompt 误写 "concepts" 而 code 读 "world_buildings" 的回归守卫
        let raw =
            r#"{"concepts":[{"id":"wb_1","concept":"x","rules":[],"history":"","cultures":[]}]}"#;
        let err = NovelCreationAgent::parse_world_options_response(raw).unwrap_err();
        assert!(
            err.contains("world_buildings"),
            "错误信息应指出缺少 world_buildings 数组: {}",
            err
        );
    }
}
