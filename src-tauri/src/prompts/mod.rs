#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub variables: Vec<String>,
    pub is_builtin: bool,
}

pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };
        manager.load_builtin_templates();
        manager
    }

    fn load_builtin_templates(&mut self) {
        let builtins = vec![
            PromptTemplate {
                id: "writing_chapter".to_string(),
                name: "章节写作".to_string(),
                description: "根据大纲生成完整章节".to_string(),
                category: "writing".to_string(),
                system_prompt: "你是一位专业中文小说作家...".to_string(),
                user_prompt_template: "请为第{chapter_number}章写作，大纲：{outline}".to_string(),
                variables: vec!["chapter_number".to_string(), "outline".to_string()],
                is_builtin: true,
            },
            PromptTemplate {
                id: "analyze_plot".to_string(),
                name: "情节分析".to_string(),
                description: "分析故事情节".to_string(),
                category: "analysis".to_string(),
                system_prompt: "你是一位专业编辑...".to_string(),
                user_prompt_template: "请分析：{content}".to_string(),
                variables: vec!["content".to_string()],
                is_builtin: true,
            },
        ];

        for template in builtins {
            self.templates.insert(template.id.clone(), template);
        }
    }

    pub fn get_all_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }

    pub fn get_template(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.get(id)
    }

    pub fn create_template(&mut self,
        mut template: PromptTemplate
    ) -> Result<(), String> {
        if template.id.is_empty() {
            template.id = format!("custom_{}", uuid::Uuid::new_v4());
        }
        template.is_builtin = false;
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    pub fn delete_template(&mut self, id: &str) -> Result<(), String> {
        if let Some(t) = self.templates.get(id) {
            if t.is_builtin {
                return Err("Cannot delete builtin template".to_string());
            }
            self.templates.remove(id);
            Ok(())
        } else {
            Err("Template not found".to_string())
        }
    }
}
