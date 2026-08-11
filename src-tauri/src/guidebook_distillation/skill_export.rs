//! SKILL.md 导出：把自定义方法论渲染为 book-to-skill 同款 Agent Skills 格式。

use super::models::*;

/// name slug 化：小写、空白转 -、去非字母数字/CJK 字符；全空回退默认
pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for c in name.trim().chars() {
        if c.is_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&c) {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        "custom-methodology".to_string()
    } else {
        trimmed
    }
}

/// 渲染 SKILL.md（空段省略）
pub fn render_skill_md(cm: &CustomMethodology, source_book: Option<&str>) -> String {
    let mut md = String::new();
    let desc = cm.description.as_deref().unwrap_or(&cm.name);
    md.push_str(&format!(
        "---\nname: {}\ndescription: \"创作方法论：{}。写小说/续写/设计情节冲突时使用。\"\n---\n\n",
        slugify(&cm.name),
        desc.replace('"', "'")
    ));
    md.push_str(&format!("# {}\n\n", cm.name));
    let source = source_book
        .map(|t| format!("《{}》（指导书提炼）", t))
        .unwrap_or_else(|| "指导书提炼".to_string());
    md.push_str(&format!(
        "**来源**：{} | **生成**：{}\n\n",
        source,
        chrono::Local::now().format("%Y-%m-%d")
    ));

    // 方法论步骤
    md.push_str("## 创作方法论（按步骤执行）\n\n");
    for (i, s) in cm.steps.iter().enumerate() {
        md.push_str(&format!("{}. **{}**：{}\n", i + 1, s.title, s.instruction));
        for c in &s.checklist {
            md.push_str(&format!("   - 检查：{}\n", c));
        }
    }

    // 技巧模式库
    if !cm.patterns.is_empty() {
        md.push_str("\n## 技巧模式库\n\n");
        for t in &cm.patterns {
            md.push_str(&format!(
                "**{}**\n- 何时用：{}\n- 怎么做：{}\n\n",
                t.name, t.when_to_use, t.how
            ));
        }
    }

    // 决策速查
    if !cm.cheatsheet.decision_rules.is_empty() {
        md.push_str("\n## 决策速查\n\n");
        for r in &cm.cheatsheet.decision_rules {
            md.push_str(&format!("- {}\n", r));
        }
    }

    // 反模式
    if !cm.cheatsheet.anti_patterns.is_empty() {
        md.push_str("\n## 反模式（务必避免）\n\n");
        for a in &cm.cheatsheet.anti_patterns {
            md.push_str(&format!("- **{}**：{}\n", a.what, a.why));
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guidebook_distillation::models::*;

    fn sample_cm() -> CustomMethodology {
        CustomMethodology {
            id: "custom_x".into(),
            guidebook_id: None,
            name: "冲突驱动法".into(),
            description: Some("以冲突为引擎".into()),
            steps: vec![MethodologyStep {
                title: "立冲突".into(),
                instruction: "确立核心冲突".into(),
                checklist: vec!["冲突明确吗？".into()],
            }],
            patterns: vec![Technique {
                name: "场景目标法".into(),
                when_to_use: "每场开场".into(),
                how: "给 POV 角色具体目标".into(),
            }],
            cheatsheet: Cheatsheet {
                decision_rules: vec!["当节奏拖沓时删场景，因为每场景须推进冲突".into()],
                anti_patterns: vec![AntiPattern {
                    what: "信息倾倒".into(),
                    why: "读者失去探索欲".into(),
                }],
            },
            enabled: true,
            created_at: chrono::Local::now(),
            updated_at: chrono::Local::now(),
        }
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Snowflake Method"), "snowflake-method");
        assert_eq!(slugify("冲突 驱动 法"), "冲突-驱动-法");
        assert_eq!(slugify("Save the Cat!"), "save-the-cat");
        assert_eq!(slugify("  "), "custom-methodology");
    }

    #[test]
    fn render_skill_md_full_sections() {
        let md = render_skill_md(&sample_cm(), Some("故事的故事"));
        assert!(md.starts_with("---\nname: 冲突驱动法\n"));
        assert!(md.contains("description:"));
        assert!(md.contains("# 冲突驱动法"));
        assert!(md.contains("《故事的故事》"));
        assert!(md.contains("## 创作方法论（按步骤执行）"));
        assert!(md.contains("**立冲突**：确立核心冲突"));
        assert!(md.contains("冲突明确吗？"));
        assert!(md.contains("## 技巧模式库"));
        assert!(md.contains("**场景目标法**"));
        assert!(md.contains("何时用：每场开场"));
        assert!(md.contains("## 决策速查"));
        assert!(md.contains("当节奏拖沓时删场景"));
        assert!(md.contains("## 反模式（务必避免）"));
        assert!(md.contains("**信息倾倒**：读者失去探索欲"));
    }

    #[test]
    fn render_skill_md_omits_empty_sections() {
        let mut cm = sample_cm();
        cm.patterns = vec![];
        cm.cheatsheet = Cheatsheet::default();
        let md = render_skill_md(&cm, None);
        assert!(md.contains("## 创作方法论（按步骤执行）"));
        assert!(!md.contains("## 技巧模式库"));
        assert!(!md.contains("## 决策速查"));
        assert!(!md.contains("## 反模式"));
        assert!(!md.contains("《"));
    }
}
