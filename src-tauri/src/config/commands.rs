//! Settings management commands for Tauri

use super::settings::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::{command, AppHandle, Manager};

/// 模型类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Chat,
    Embedding,
    Multimodal,
    Image,
}

/// 通用模型配置（前端传来的）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub model_type: ModelType,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub dimensions: Option<usize>,
    pub capabilities: Option<Vec<String>>,
    pub is_default: Option<bool>,
    pub enabled: Option<bool>,
}

/// 应用设置导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettingsExport {
    pub version: String,
    pub exported_at: String,
    pub settings: AppSettingsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettingsData {
    pub models: HashMap<String, Vec<serde_json::Value>>,
    pub active_models: HashMap<String, String>,
    pub agent_mappings: Vec<AgentMapping>,
    pub general: GeneralSettings,
    pub privacy: PrivacySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneralSettings {
    pub theme: String,
    pub language: String,
    pub auto_save: bool,
    pub auto_save_interval: u64,
    pub font_size: u32,
    pub line_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrivacySettings {
    pub share_usage_data: bool,
    pub store_api_keys_securely: bool,
}

/// 获取应用设置
#[command]
pub fn get_settings(app_handle: AppHandle) -> Result<AppSettingsData, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    
    let config = AppConfig::load(&app_dir).map_err(|e| e.to_string())?;
    
    let mut models: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    
    // 转换LLM配置：含 Vision 能力的放入 multimodal，其余放入 chat
    let mut chat_models: Vec<serde_json::Value> = vec![];
    let mut multimodal_models: Vec<serde_json::Value> = vec![];
    
    for p in config.llm_profiles.values() {
        let is_multimodal = p.capabilities.contains(&super::settings::ModelCapability::Vision);
        let model_json = serde_json::json!({
            "id": p.id,
            "name": p.name,
            "description": p.description,
            "provider": p.provider,
            "model": p.model,
            "type": if is_multimodal { "multimodal" } else { "chat" },
            "temperature": p.temperature,
            "max_tokens": p.max_tokens,
            "timeout_seconds": p.timeout_seconds,
            "is_default": p.is_default,
            "capabilities": p.capabilities,
            "enabled": true,
            "api_key": if p.api_key.is_empty() { None } else { Some("***") },
            "api_base": p.api_base,
        });
        if is_multimodal {
            multimodal_models.push(model_json);
        } else {
            chat_models.push(model_json);
        }
    }
    models.insert("chat".to_string(), chat_models);
    models.insert("multimodal".to_string(), multimodal_models);
    
    // 转换Embedding配置
    let embedding_models: Vec<serde_json::Value> = config
        .embedding_profiles
        .values()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "description": p.description,
                "provider": p.provider,
                "model": p.model,
                "type": "embedding",
                "dimensions": p.dimensions,
                "max_input_tokens": p.max_input_tokens,
                "is_default": p.is_default,
                "enabled": true,
                "api_key": if p.api_key.is_empty() { None } else { Some("***") },
                "api_base": p.api_base,
            })
        })
        .collect();
    models.insert("embedding".to_string(), embedding_models);
    
    // 图像模型暂空
    models.insert("image".to_string(), vec![]);
    
    let active_models = vec![
        ("chat".to_string(), config.active_llm_profile.unwrap_or_default()),
        ("embedding".to_string(), config.active_embedding_profile.unwrap_or_default()),
        ("multimodal".to_string(), String::new()),
        ("image".to_string(), String::new()),
    ]
    .into_iter()
    .collect();
    
    let agent_mappings: Vec<AgentMapping> = config.agent_mappings.values().cloned().collect();

    Ok(AppSettingsData {
        models,
        active_models,
        agent_mappings,
        general: GeneralSettings {
            theme: "dark".to_string(),
            language: "zh-CN".to_string(),
            auto_save: true,
            auto_save_interval: 30,
            font_size: 16,
            line_height: 1.6,
        },
        privacy: PrivacySettings {
            share_usage_data: false,
            store_api_keys_securely: true,
        },
    })
}

/// 保存设置
#[command]
pub fn save_settings(settings: AppSettingsData, app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    
    let mut config = AppConfig::load(&app_dir).map_err(|e| e.to_string())?;
    
    // 保存活跃配置
    if let Some(chat_id) = settings.active_models.get("chat") {
        if !chat_id.is_empty() {
            config.active_llm_profile = Some(chat_id.clone());
        }
    }
    if let Some(emb_id) = settings.active_models.get("embedding") {
        if !emb_id.is_empty() {
            config.active_embedding_profile = Some(emb_id.clone());
        }
    }

    // 保存 Agent 映射
    for mapping in settings.agent_mappings {
        config.agent_mappings.insert(mapping.agent_id.clone(), mapping);
    }
    
    config.save(&app_dir).map_err(|e| e.to_string())
}

/// 导出设置
#[command]
pub fn export_settings(app_handle: AppHandle) -> Result<AppSettingsExport, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    
    let settings = get_settings(app_handle)?;
    
    Ok(AppSettingsExport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        settings,
    })
}

/// 导入设置
#[command]
pub fn import_settings(data: AppSettingsExport, app_handle: AppHandle) -> Result<(), String> {
    // 验证版本兼容性
    let current_version = env!("CARGO_PKG_VERSION");
    let import_version = &data.version;
    
    // 简单版本检查（主版本号必须相同）
    let current_major = current_version.split('.').next().unwrap_or("0");
    let import_major = import_version.split('.').next().unwrap_or("0");
    
    if current_major != import_major {
        return Err(format!(
            "版本不兼容: 当前版本 {}，导入版本 {}",
            current_version, import_version
        ));
    }
    
    save_settings(data.settings, app_handle)?;
    Ok(())
}

/// 获取所有模型
#[command]
pub fn get_models(app_handle: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let settings = get_settings(app_handle)?;
    let mut all_models: Vec<serde_json::Value> = vec![];
    
    for (model_type, models) in settings.models {
        for mut model in models {
            if let Some(obj) = model.as_object_mut() {
                obj.insert("type".to_string(), serde_json::json!(model_type));
            }
            all_models.push(model);
        }
    }
    
    Ok(all_models)
}

/// 创建模型配置
#[command]
pub fn create_model(config: ModelConfigInput, app_handle: AppHandle) -> Result<serde_json::Value, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    
    let mut app_config = AppConfig::load(&app_dir).map_err(|e| e.to_string())?;
    
    let model_id = config.id.clone().unwrap_or_else(|| format!("model-{}", uuid::Uuid::new_v4()));
    let model_name = config.name.clone();
    let model_type_str = format!("{:?}", config.model_type);
    
    match config.model_type {
        ModelType::Chat => {
            let capabilities = config
                .capabilities
                .unwrap_or_default()
                .into_iter()
                .filter_map(|c| match c.as_str() {
                    "chat" => Some(ModelCapability::Chat),
                    "completion" => Some(ModelCapability::Completion),
                    "function_calling" => Some(ModelCapability::FunctionCalling),
                    "json_mode" => Some(ModelCapability::JsonMode),
                    "vision" => Some(ModelCapability::Vision),
                    "long_context" => Some(ModelCapability::LongContext),
                    _ => None,
                })
                .collect();
            
            let provider = match config.provider.as_str() {
                "anthropic" => LlmProvider::Anthropic,
                "azure" => LlmProvider::Azure,
                "ollama" => LlmProvider::Ollama,
                "deepseek" => LlmProvider::DeepSeek,
                "qwen" => LlmProvider::Qwen,
                "custom" => LlmProvider::Custom,
                _ => LlmProvider::OpenAI,
            };
            
            let profile = LlmProfile {
                id: model_id.clone(),
                name: config.name,
                description: config.description,
                provider,
                model: config.model,
                api_key: config.api_key.unwrap_or_default(),
                api_base: config.api_base,
                max_tokens: config.max_tokens.unwrap_or(2000),
                temperature: config.temperature.unwrap_or(0.7),
                timeout_seconds: 120,
                is_default: config.is_default.unwrap_or(false),
                capabilities,
            };
            
            app_config.add_llm_profile(profile).map_err(|e| e.to_string())?;
        }
        ModelType::Embedding => {
            let provider = match config.provider.as_str() {
                "azure" => EmbeddingProvider::Azure,
                "ollama" => EmbeddingProvider::Ollama,
                "local" => EmbeddingProvider::Local,
                "custom" => EmbeddingProvider::Custom,
                _ => EmbeddingProvider::OpenAI,
            };
            
            let profile = EmbeddingProfile {
                id: model_id.clone(),
                name: config.name,
                description: config.description,
                provider,
                model: config.model,
                api_key: config.api_key.unwrap_or_default(),
                api_base: config.api_base,
                dimensions: config.dimensions.unwrap_or(1536),
                max_input_tokens: 8192,
                is_default: config.is_default.unwrap_or(false),
            };
            
            app_config.add_embedding_profile(profile).map_err(|e| e.to_string())?;
        }
        ModelType::Multimodal => {
            let provider = match config.provider.as_str() {
                "anthropic" => LlmProvider::Anthropic,
                "azure" => LlmProvider::Azure,
                "ollama" => LlmProvider::Ollama,
                "deepseek" => LlmProvider::DeepSeek,
                "qwen" => LlmProvider::Qwen,
                "custom" => LlmProvider::Custom,
                _ => LlmProvider::OpenAI,
            };
            
            let mut capabilities = config
                .capabilities
                .unwrap_or_default()
                .into_iter()
                .filter_map(|c| match c.as_str() {
                    "chat" => Some(ModelCapability::Chat),
                    "completion" => Some(ModelCapability::Completion),
                    "function_calling" => Some(ModelCapability::FunctionCalling),
                    "json_mode" => Some(ModelCapability::JsonMode),
                    "vision" => Some(ModelCapability::Vision),
                    "long_context" => Some(ModelCapability::LongContext),
                    _ => None,
                })
                .collect::<Vec<_>>();
            
            if !capabilities.contains(&ModelCapability::Vision) {
                capabilities.push(ModelCapability::Vision);
            }
            
            let profile = LlmProfile {
                id: model_id.clone(),
                name: config.name,
                description: config.description,
                provider,
                model: config.model,
                api_key: config.api_key.unwrap_or_default(),
                api_base: config.api_base,
                max_tokens: config.max_tokens.unwrap_or(2000),
                temperature: config.temperature.unwrap_or(0.7),
                timeout_seconds: 120,
                is_default: config.is_default.unwrap_or(false),
                capabilities,
            };
            
            app_config.add_llm_profile(profile).map_err(|e| e.to_string())?;
        }
        ModelType::Image => {
            // TODO: 实现图像生成模型
            return Err("图像生成模型暂未实现".to_string());
        }
    }
    
    app_config.save(&app_dir).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "id": model_id,
        "name": model_name,
        "type": model_type_str,
    }))
}

/// 更新模型配置
#[command]
pub fn update_model(id: String, config: ModelConfigInput, app_handle: AppHandle) -> Result<(), String> {
    // 简化实现：删除旧配置，创建新配置
    delete_model(id.clone(), app_handle.clone())?;
    let mut new_config = config;
    new_config.id = Some(id);
    create_model(new_config, app_handle)?;
    Ok(())
}

/// 删除模型配置
#[command]
pub fn delete_model(id: String, app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    
    let mut config = AppConfig::load(&app_dir).map_err(|e| e.to_string())?;
    
    // 尝试删除LLM配置
    if config.llm_profiles.contains_key(&id) {
        config.remove_llm_profile(&id).map_err(|e| e.to_string())?;
    }
    // 尝试删除Embedding配置
    else if config.embedding_profiles.contains_key(&id) {
        config.remove_embedding_profile(&id).map_err(|e| e.to_string())?;
    }
    else {
        return Err(format!("Model '{}' not found", id));
    }
    
    config.save(&app_dir).map_err(|e| e.to_string())
}

/// 设置活跃模型
#[command]
pub fn set_active_model(model_type: String, model_id: String, app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    
    let mut config = AppConfig::load(&app_dir).map_err(|e| e.to_string())?;
    
    match model_type.as_str() {
        "chat" => config.set_active_llm_profile(&model_id).map_err(|e| e.to_string())?,
        "embedding" => config.set_active_embedding_profile(&model_id).map_err(|e| e.to_string())?,
        _ => return Err(format!("Unknown model type: {}", model_type)),
    }
    
    config.save(&app_dir).map_err(|e| e.to_string())
}

/// 获取Agent模型映射
#[command]
pub fn get_agent_mappings(app_handle: AppHandle) -> Result<Vec<AgentMapping>, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    
    let config = AppConfig::load(&app_dir).map_err(|e| e.to_string())?;
    Ok(config.agent_mappings.values().cloned().collect())
}

/// 更新Agent模型映射
#[command]
pub fn update_agent_mapping(mapping: AgentMapping, app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    
    let mut config = AppConfig::load(&app_dir).map_err(|e| e.to_string())?;
    config.agent_mappings.insert(mapping.agent_id.clone(), mapping);
    config.save(&app_dir).map_err(|e| e.to_string())?;
    Ok(())
}

/// 测试模型连接
#[command]
pub async fn test_model_connection(model_id: String, app_handle: AppHandle) -> Result<serde_json::Value, String> {
    let settings = get_settings(app_handle)?;
    
    // 查找模型
    let mut found_model: Option<&serde_json::Value> = None;
    for models in settings.models.values() {
        if let Some(model) = models.iter().find(|m| m.get("id").and_then(|v| v.as_str()) == Some(&model_id)) {
            found_model = Some(model);
            break;
        }
    }
    
    if found_model.is_none() {
        return Err(format!("Model '{}' not found", model_id));
    }
    
    // TODO: 实际测试连接
    // 模拟延迟测试
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    Ok(serde_json::json!({
        "success": true,
        "latency": 500,
    }))
}
