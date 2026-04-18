use super::*;

pub fn get_builtin_skills() -> Vec<Skill> {
    vec![
        create_style_enhancer_skill(),
        create_plot_twist_skill(),
        create_text_formatter_skill(),
    ]
}

fn create_style_enhancer_skill() -> Skill {
    Skill {
        manifest: SkillManifest {
            id: "builtin.style_enhancer".to_string(),
            name: "文风增强器".to_string(),
            version: "1.0.0".to_string(),
            description: "增强文本的文学性和表现力".to_string(),
            author: "CINEMA-AI".to_string(),
            category: SkillCategory::Style,
            entry_point: "style_enhancer.prompt".to_string(),
            parameters: vec![
                SkillParameter {
                    name: "content".to_string(),
                    description: "需要增强的文本内容".to_string(),
                    param_type: ParameterType::Text,
                    required: true,
                    default: None,
                },
            ],
            capabilities: vec!["style_enhancement".to_string()],
            hooks: vec![],
            config: HashMap::new(),
        },
        path: PathBuf::from("builtin"),
        is_enabled: true,
        loaded_at: Utc::now(),
        runtime: SkillRuntime::Prompt(PromptRuntime {
            system_prompt: "你是一个专业的文学编辑".to_string(),
            user_prompt_template: "请增强以下文本：{content}".to_string(),
        }),
    }
}

fn create_plot_twist_skill() -> Skill {
    Skill {
        manifest: SkillManifest {
            id: "builtin.plot_twist".to_string(),
            name: "情节反转生成器".to_string(),
            version: "1.0.0".to_string(),
            description: "生成出人意料的情节反转".to_string(),
            author: "CINEMA-AI".to_string(),
            category: SkillCategory::Plot,
            entry_point: "plot_twist.prompt".to_string(),
            parameters: vec![
                SkillParameter {
                    name: "context".to_string(),
                    description: "故事上下文".to_string(),
                    param_type: ParameterType::Text,
                    required: true,
                    default: None,
                },
            ],
            capabilities: vec!["plot_generation".to_string()],
            hooks: vec![],
            config: HashMap::new(),
        },
        path: PathBuf::from("builtin"),
        is_enabled: true,
        loaded_at: Utc::now(),
        runtime: SkillRuntime::Prompt(PromptRuntime {
            system_prompt: "你是一个擅长情节设计的编剧".to_string(),
            user_prompt_template: "请基于以下上下文生成反转：{context}".to_string(),
        }),
    }
}

fn create_text_formatter_skill() -> Skill {
    Skill {
        manifest: SkillManifest {
            id: "builtin.text_formatter".to_string(),
            name: "文本排版器".to_string(),
            version: "1.0.0".to_string(),
            description: "对小说正文进行智能排版，优化段落结构、标点使用和对话格式".to_string(),
            author: "CINEMA-AI".to_string(),
            category: SkillCategory::Style,
            entry_point: "text_formatter.prompt".to_string(),
            parameters: vec![
                SkillParameter {
                    name: "content".to_string(),
                    description: "需要排版的文本内容".to_string(),
                    param_type: ParameterType::Text,
                    required: true,
                    default: None,
                },
            ],
            capabilities: vec!["text_formatting".to_string()],
            hooks: vec![],
            config: HashMap::new(),
        },
        path: PathBuf::from("builtin"),
        is_enabled: true,
        loaded_at: Utc::now(),
        runtime: SkillRuntime::Prompt(PromptRuntime {
            system_prompt: "你是一位专业的中文小说排版编辑。你的任务是对输入的小说正文进行智能排版优化。请遵循以下规则：\n1. 合理分段：根据语义和场景转换进行分段，避免过长段落\n2. 对话格式：确保对话单独成段，使用正确的引号和标点\n3. 场景转换：场景或视角转换时添加空行分隔\n4. 标点规范：修正错误的标点使用，统一全角标点\n5. 保留原意：不改变原文的内容和表达意图\n6. 输出纯文本，不需要添加任何解释或说明".to_string(),
            user_prompt_template: "请对以下小说正文进行智能排版优化，只返回排版后的正文内容，不要添加任何解释：\n\n{content}".to_string(),
        }),
    }
}
