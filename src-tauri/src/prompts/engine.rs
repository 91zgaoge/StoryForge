//! Prompt Template Engine - 提示词模板引擎
//!
//! 将硬编码的提示词字符串替换为可维护的模板系统。
//! 支持变量替换 {{variable}} 和条件块 {{#if condition}}...{{/if}}

use std::collections::HashMap;

/// 提示词模板
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub user_prompt_template: String,
}

/// 模板引擎
pub struct TemplateEngine;

impl TemplateEngine {
    /// 渲染模板，替换 {{key}} 为对应值
    pub fn render(template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        // 简单变量替换: {{key}}
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}" , key);
            result = result.replace(&placeholder, value);
        }

        // 清理未替换的变量（保留原样或替换为空）
        // 这里选择保留原样，以便调试

        result
    }

    /// 条件渲染: {{#if key}}...{{/if}}
    pub fn render_with_conditions(template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        // 处理条件块
        loop {
            let start_tag = result.find("{{#if ");
            if start_tag.is_none() {
                break;
            }
            let start = start_tag.unwrap();
            let cond_end = result[start..].find("}}").unwrap() + start;
            let condition_key = result[start + 6..cond_end].trim();

            let end_tag = result[cond_end..].find("{{/if}}").unwrap() + cond_end;
            let block_content = result[cond_end + 2..end_tag].to_string();

            let has_value = variables.get(condition_key)
                .map(|v| !v.is_empty() && v != "无" && v != "暂无" && v != "暂无角色信息")
                .unwrap_or(false);

            let replacement = if has_value {
                block_content
            } else {
                String::new()
            };

            result.replace_range(start..end_tag + 7, &replacement);
        }

        // 然后处理普通变量
        Self::render(&result, variables)
    }
}

/// 内置提示词模板库
pub struct PromptLibrary;

impl PromptLibrary {
    /// 获取 Writer Agent 的系统提示词模板
    pub fn writer_system_template() -> &'static str {
        r#"你是一位专业中文小说作家，擅长根据上下文续写和改写内容。

【故事信息】
标题: {{story_title}}
类型: {{genre}}
风格: {{tone}} / 节奏: {{pacing}}

{{#if world_rules}}
【世界观规则】
{{world_rules}}
{{/if}}

{{#if characters}}
【角色信息】
{{characters}}
{{/if}}

{{#if previous_chapters}}
【前文摘要】
{{previous_chapters}}
{{/if}}

{{#if scene_structure}}
【当前场景结构】
{{scene_structure}}
{{/if}}

写作要求：
1. 保持文风一致，情节连贯自然
2. 人物行为符合性格设定
3. 适当加入环境描写和对话
4. 遵守世界观规则
5. 只输出需要的内容，不要添加解释"#
    }

    /// 获取 Writer Agent 的用户提示词模板（续写）
    pub fn writer_continue_template() -> &'static str {
        r#"请根据以上上下文，续写接下来的内容。

【写作要求】
{{instruction}}

【当前已有内容】
{{current_content}}

请直接输出续写内容，不要添加解释或重复上下文。"#
    }

    /// 获取 Writer Agent 的用户提示词模板（改写）
    pub fn writer_rewrite_template() -> &'static str {
        r#"请根据以上上下文，对以下文本进行修改。

【修改要求】
{{instruction}}

【需要修改的文本】
{{selected_text}}

【当前章节内容】
{{current_content}}

请只输出修改后的文本，不要添加解释或重复上下文。"#
    }

    /// 获取 Inspector Agent 的系统提示词模板
    pub fn inspector_system_template() -> &'static str {
        r#"你是一位专业的小说质检员，负责检查内容质量、逻辑连贯性和人物一致性。

【故事信息】
标题: {{story_title}}
类型: {{genre}}

{{#if characters}}
【角色设定】
{{characters}}
{{/if}}

检查维度：
1. 逻辑连贯性 - 情节是否通顺，有无矛盾
2. 人物一致性 - 角色行为是否符合设定
3. 文笔质量 - 语言是否流畅，描写是否生动
4. 节奏把控 - 快慢是否得当，有无冗余
5. 世界观一致性 - 是否违反已设定的规则

请提供：
1. 总体评分（0-100）
2. 各维度评分
3. 具体问题指出
4. 改进建议"#
    }

    /// 获取 Outline Planner 的系统提示词模板
    pub fn outline_planner_template() -> &'static str {
        r#"你是一位专业的故事结构顾问，擅长设计故事大纲和章节结构。

【故事创意】
{{premise}}

{{#if characters}}
【角色概要】
{{characters}}
{{/if}}

请使用三幕式结构设计大纲：
1. 第一幕（Setup，25%）：介绍世界、角色、冲突
2. 第二幕（Confrontation，50%）：升级冲突、揭示真相
3. 第三幕（Resolution，25%）：高潮对决、结局收场

每章需要包含：
- 戏剧目标：这章要完成什么叙事使命
- 外部压迫：环境/反派/事件对角色的压迫
- 冲突类型
- 情感弧线

请以清晰的层次结构输出。"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_render() {
        let template = "Hello, {{name}}!";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        assert_eq!(TemplateEngine::render(template, &vars), "Hello, World!");
    }

    #[test]
    fn test_conditional_render() {
        let template = "{{#if has_data}}Data: {{data}}{{/if}}End";
        let mut vars = HashMap::new();
        vars.insert("has_data".to_string(), "yes".to_string());
        vars.insert("data".to_string(), "123".to_string());
        assert_eq!(TemplateEngine::render_with_conditions(template, &vars), "Data: 123End");
    }

    #[test]
    fn test_conditional_skip() {
        let template = "{{#if missing}}Data: {{data}}{{/if}}End";
        let mut vars = HashMap::new();
        vars.insert("missing".to_string(), "".to_string());
        assert_eq!(TemplateEngine::render_with_conditions(template, &vars), "End");
    }
}
