//! 提炼结果校验清洗器（对标 book-to-skill validate_skill.py）。
//! 纯 Rust 确定性清洗：剔除空条目、按名去重、字段截断。落库前调用。

use super::models::*;

/// 清洗统计（质量指标，log 输出）
#[derive(Debug, Default, PartialEq)]
pub struct CleanReport {
    pub removed_techniques: usize,
    pub deduped_techniques: usize,
    pub removed_rules: usize,
    pub removed_anti_patterns: usize,
    pub truncated_fields: usize,
}

const FIELD_MAX: usize = 200;
const STEP_TITLE_MAX: usize = 20;
const STEP_INSTRUCTION_MAX: usize = 500;

fn clip(s: &str, max: usize, report: &mut CleanReport) -> String {
    let t = s.trim();
    if t.chars().count() > max {
        report.truncated_fields += 1;
        t.chars().take(max).collect()
    } else {
        t.to_string()
    }
}

/// 校验并清洗提炼产物。硬校验（方法论结构）已在 distiller 内完成，
/// 此处只做确定性软清洗，不会失败。
pub fn validate_and_clean(mut output: DistillationOutput) -> (DistillationOutput, CleanReport) {
    let mut report = CleanReport::default();

    // techniques：剔除空 name → 按 name 去重（保留首个）→ 字段截断
    let before = output.techniques.len();
    output.techniques.retain(|t| !t.name.trim().is_empty());
    report.removed_techniques = before - output.techniques.len();
    let mut seen = std::collections::HashSet::new();
    let before = output.techniques.len();
    output
        .techniques
        .retain(|t| seen.insert(t.name.trim().to_string()));
    report.deduped_techniques = before - output.techniques.len();
    for t in &mut output.techniques {
        t.name = clip(&t.name, FIELD_MAX, &mut report);
        t.when_to_use = clip(&t.when_to_use, FIELD_MAX, &mut report);
        t.how = clip(&t.how, FIELD_MAX, &mut report);
    }

    // decision_rules：剔除空白 → 去重 → 截断
    let before = output.cheatsheet.decision_rules.len();
    output
        .cheatsheet
        .decision_rules
        .retain(|r| !r.trim().is_empty());
    let mut seen = std::collections::HashSet::new();
    output
        .cheatsheet
        .decision_rules
        .retain(|r| seen.insert(r.trim().to_string()));
    report.removed_rules = before - output.cheatsheet.decision_rules.len();
    for r in &mut output.cheatsheet.decision_rules {
        *r = clip(r, FIELD_MAX, &mut report);
    }

    // anti_patterns：剔除空 what → 按 what 去重
    let before = output.cheatsheet.anti_patterns.len();
    output
        .cheatsheet
        .anti_patterns
        .retain(|a| !a.what.trim().is_empty());
    let mut seen = std::collections::HashSet::new();
    output
        .cheatsheet
        .anti_patterns
        .retain(|a| seen.insert(a.what.trim().to_string()));
    report.removed_anti_patterns = before - output.cheatsheet.anti_patterns.len();
    for a in &mut output.cheatsheet.anti_patterns {
        a.what = clip(&a.what, FIELD_MAX, &mut report);
        a.why = clip(&a.why, FIELD_MAX, &mut report);
    }

    // steps：title/instruction 截断
    for s in &mut output.methodology.steps {
        s.title = clip(&s.title, STEP_TITLE_MAX, &mut report);
        s.instruction = clip(&s.instruction, STEP_INSTRUCTION_MAX, &mut report);
    }

    (output, report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guidebook_distillation::models::*;

    fn output_with(
        techniques: Vec<Technique>,
        rules: Vec<String>,
        anti: Vec<AntiPattern>,
    ) -> DistillationOutput {
        DistillationOutput {
            metadata: LlmGuidebookMetadataResponse {
                title: None,
                author: None,
                subject: None,
            },
            methodology: LlmMethodologyResponse {
                name: "测试法".into(),
                description: None,
                steps: vec![LlmMethodologyStepResponse {
                    title: "s".into(),
                    instruction: "i".into(),
                    checklist: vec![],
                }],
            },
            techniques,
            cheatsheet: Cheatsheet {
                decision_rules: rules,
                anti_patterns: anti,
            },
        }
    }

    #[test]
    fn removes_blank_name_techniques_and_dedupes() {
        let out = output_with(
            vec![
                Technique {
                    name: "  ".into(),
                    when_to_use: "w".into(),
                    how: "h".into(),
                },
                Technique {
                    name: "雪花法".into(),
                    when_to_use: "w1".into(),
                    how: "h1".into(),
                },
                Technique {
                    name: "雪花法".into(),
                    when_to_use: "w2".into(),
                    how: "h2".into(),
                },
            ],
            vec![],
            vec![],
        );
        let (cleaned, report) = validate_and_clean(out);
        assert_eq!(cleaned.techniques.len(), 1);
        assert_eq!(cleaned.techniques[0].when_to_use, "w1"); // 保留首个
        assert_eq!(report.removed_techniques, 1);
        assert_eq!(report.deduped_techniques, 1);
    }

    #[test]
    fn removes_blank_rules_and_anti_patterns() {
        let out = output_with(
            vec![],
            vec![
                "".into(),
                "  ".into(),
                "当X做Y，因为Z".into(),
                "当X做Y，因为Z".into(),
            ],
            vec![
                AntiPattern {
                    what: " ".into(),
                    why: "w".into(),
                },
                AntiPattern {
                    what: "流水账".into(),
                    why: "无冲突".into(),
                },
                AntiPattern {
                    what: "流水账".into(),
                    why: "重复".into(),
                },
            ],
        );
        let (cleaned, report) = validate_and_clean(out);
        assert_eq!(cleaned.cheatsheet.decision_rules, vec!["当X做Y，因为Z"]);
        assert_eq!(cleaned.cheatsheet.anti_patterns.len(), 1);
        assert_eq!(report.removed_rules, 3); // 2 空白 + 1 重复
        assert_eq!(report.removed_anti_patterns, 2);
    }

    #[test]
    fn truncates_overlong_fields() {
        let long = "x".repeat(300);
        let out = output_with(
            vec![Technique {
                name: "t".into(),
                when_to_use: long.clone(),
                how: long.clone(),
            }],
            vec![long.clone()],
            vec![],
        );
        let (cleaned, report) = validate_and_clean(out);
        assert_eq!(cleaned.techniques[0].when_to_use.chars().count(), 200);
        assert_eq!(cleaned.techniques[0].how.chars().count(), 200);
        assert_eq!(cleaned.cheatsheet.decision_rules[0].chars().count(), 200);
        assert_eq!(report.truncated_fields, 3);
    }

    #[test]
    fn truncates_step_title_and_instruction() {
        let mut out = output_with(vec![], vec![], vec![]);
        out.methodology.steps[0].title = "t".repeat(30);
        out.methodology.steps[0].instruction = "i".repeat(600);
        let (cleaned, report) = validate_and_clean(out);
        assert_eq!(cleaned.methodology.steps[0].title.chars().count(), 20);
        assert_eq!(
            cleaned.methodology.steps[0].instruction.chars().count(),
            500
        );
        assert_eq!(report.truncated_fields, 2);
    }

    #[test]
    fn clean_output_is_noop_on_valid_input() {
        let out = output_with(
            vec![Technique {
                name: "t".into(),
                when_to_use: "w".into(),
                how: "h".into(),
            }],
            vec!["r".into()],
            vec![AntiPattern {
                what: "a".into(),
                why: "b".into(),
            }],
        );
        let (_, report) = validate_and_clean(out);
        assert_eq!(report, CleanReport::default());
    }
}
