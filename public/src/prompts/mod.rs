use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 提示词模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PromptCategory,
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub variables: Vec<PromptVariable>,
    pub tags: Vec<String>,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PromptCategory {
    Writing,      // 写作
    Analysis,     // 分析
    Revision,     // 修改
    Brainstorm,   // 头脑风暴
    Character,    // 角色
    Plot,         // 情节
    Custom,       // 自定义
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariable {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// 提示词管理器
pub struct PromptManager {
    storage_path: PathBuf,
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    pub fn new(app_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let storage_path = app_dir.join("prompts");
        fs::create_dir_all(&storage_path)?;

        let mut manager = Self {
            storage_path,
            templates: HashMap::new(),
        };

        manager.load_builtin_templates();
        manager.load_custom_templates()?;

        Ok(manager)
    }

    /// 加载内置模板
    fn load_builtin_templates(&mut self) {
        let builtins = vec![
            PromptTemplate {
                id: "writing_chapter".to_string(),
                name: "章节写作".to_string(),
                description: "根据大纲生成完整章节".to_string(),
                category: PromptCategory::Writing,
                system_prompt: "你是一位专业中文小说作家...".to_string(),
                user_prompt_template: "请为第{chapter_number}章写作，大纲：{outline}".to_string(),
                variables: vec![
                    PromptVariable { name: "chapter_number".to_string(), description: "章节号".to_string(), required: true, default_value: None },
                    PromptVariable { name: "outline".to_string(), description: "章节大纲".to_string(), required: true, default_value: None },
                ],
                tags: vec!["写作".to_string(), "章节".to_string()],
                is_builtin: true,
                created_at: chrono::Local::now().to_rfc3339(),
                updated_at: chrono::Local::now().to_rfc3339(),
            },
            PromptTemplate {
                id: "analyze_plot".to_string(),
                name: "情节分析".to_string(),
                description: "分析故事情节的连贯性和张力".to_string(),
                category: PromptCategory::Analysis,
                system_prompt: "你是一位专业的小说编辑...".to_string(),
                user_prompt_template: "请分析以下章节情节：{content}".to_string(),
                variables: vec![
                    PromptVariable { name: "content".to_string(), description: "章节内容".to_string(), required: true, default_value: None },
                ],
                tags: vec!["分析".to_string(), "情节".to_string()],
                is_builtin: true,
                created_at: chrono::Local::now().to_rfc3339(),
                updated_at: chrono::Local::now().to_rfc3339(),
            },
        ];

        for template in builtins {
            self.templates.insert(template.id.clone(), template);
        }
    }

    /// 加载自定义模板
    fn load_custom_templates(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let custom_path = self.storage_path.join("custom.json");
        if custom_path.exists() {
            let content = fs::read_to_string(&custom_path)?;
            let custom: Vec<PromptTemplate> = serde_json::from_str(&content)?;
            for template in custom {
                self.templates.insert(template.id.clone(), template);
            }
        }
        Ok(())
    }

    /// 保存自定义模板
    pub fn save_custom_templates(&self) -> Result<(), Box<dyn std::error::Error>> {
        let custom: Vec<&PromptTemplate> = self.templates
            .values()
            .filter(|t| !t.is_builtin)
            .collect();
        let content = serde_json::to_string_pretty(&custom)?;
        fs::write(self.storage_path.join("custom.json"), content)?;
        Ok(())
    }

    /// 获取所有模板
    pub fn get_all_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }

    /// 按分类获取模板
    pub fn get_templates_by_category(&self, category: PromptCategory) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    /// 获取单个模板
    pub fn get_template(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.get(id)
    }

    /// 创建模板
    pub fn create_template(&mut self, mut template: PromptTemplate) -> Result<(), String> {
        if template.id.is_empty() {
            template.id = format!("custom_{}", uuid::Uuid::new_v4());
        }
        template.is_builtin = false;
        template.created_at = chrono::Local::now().to_rfc3339();
        template.updated_at = chrono::Local::now().to_rfc3339();

        self.templates.insert(template.id.clone(), template);
        self.save_custom_templates().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 更新模板
    pub fn update_template(&mut self, id: &str, updates: PromptTemplate) -> Result<(), String> {
        if let Some(template) = self.templates.get_mut(id) {
            if template.is_builtin {
                return Err("Cannot modify builtin templates".to_string());
            }
            template.name = updates.name;
            template.description = updates.description;
            template.category = updates.category;
            template.system_prompt = updates.system_prompt;
            template.user_prompt_template = updates.user_prompt_template;
            template.variables = updates.variables;
            template.tags = updates.tags;
            template.updated_at = chrono::Local::now().to_rfc3339();

            self.save_custom_templates().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Template not found".to_string())
        }
    }

    /// 删除模板
    pub fn delete_template(&mut self, id: &str) -> Result<(), String> {
        if let Some(template) = self.templates.get(id) {
            if template.is_builtin {
                return Err("Cannot delete builtin templates".to_string());
            }
            self.templates.remove(id);
            self.save_custom_templates().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Template not found".to_string())
        }
    }

    /// 渲染模板
    pub fn render_template(&self, id: &str, variables: &HashMap<String, String>) -> Result<(String, String), String> {
        let template = self.templates.get(id).ok_or("Template not found")?;

        let mut user_prompt = template.user_prompt_template.clone();
        for (key, value) in variables {
            user_prompt = user_prompt.replace(&format!("{{{}}}", key), value);
        }

        Ok((template.system_prompt.clone(), user_prompt))
    }
}
