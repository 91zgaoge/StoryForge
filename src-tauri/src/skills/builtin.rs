use super::*;

pub fn get_builtin_skills() -> Vec<Skill> {
    vec![
        create_style_enhancer_skill(),
        create_plot_twist_skill(),
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
