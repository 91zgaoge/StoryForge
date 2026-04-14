//! LLM Service - 统一的大语言模型服务
//! 
//! 提供同步生成和流式生成两种模式
//! 支持多提供商配置管理和自动切换

use super::adapter::{GenerateRequest, GenerateResponse};
use super::openai::OpenAiAdapter;
use crate::config::settings::{AppConfig, LlmProfile, LlmProvider};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

/// 流式生成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub chunk: String,
    pub is_first: bool,
    pub is_last: bool,
    pub model: String,
}

/// 生成完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationComplete {
    pub full_text: String,
    pub model: String,
    pub tokens_used: i32,
    pub cost: f64,
    pub duration_ms: u64,
}

/// 生成错误事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationError {
    pub error: String,
    pub error_code: String,
}

/// LLM服务 - 管理所有LLM调用
pub struct LlmService {
    app_handle: AppHandle,
    config: Arc<Mutex<AppConfig>>,
}

impl LlmService {
    pub fn new(app_handle: AppHandle) -> Self {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        
        let config = AppConfig::load(&app_dir).unwrap_or_default();
        
        Self {
            app_handle,
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// 重新加载配置
    pub fn reload_config(&self) {
        let app_dir = self.app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        
        if let Ok(config) = AppConfig::load(&app_dir) {
            if let Ok(mut guard) = self.config.lock() {
                *guard = config;
            }
        }
    }

    /// 获取当前活跃的LLM配置
    fn get_active_profile(&self) -> Option<LlmProfile> {
        let guard = self.config.lock().ok()?;
        guard.get_active_llm_profile().cloned()
    }

    /// 获取指定ID的LLM配置
    fn get_profile_by_id(&self, profile_id: &str) -> Option<LlmProfile> {
        let guard = self.config.lock().ok()?;
        guard.llm_profiles.get(profile_id).cloned()
    }

    /// 创建适配器
    fn create_adapter(&self, profile: &LlmProfile) -> Result<Box<dyn super::LlmAdapter>, String> {
        match profile.provider {
            LlmProvider::OpenAI => {
                Ok(Box::new(OpenAiAdapter::new(
                    profile.api_key.clone(),
                    profile.model.clone(),
                    profile.api_base.clone(),
                    profile.max_tokens,
                    profile.temperature,
                )))
            }
            LlmProvider::Anthropic => {
                // TODO: 实现Anthropic适配器
                Err("Anthropic adapter not yet implemented".to_string())
            }
            LlmProvider::Ollama => {
                // TODO: 实现Ollama适配器
                Err("Ollama adapter not yet implemented".to_string())
            }
            _ => Err(format!("Provider {:?} not supported", profile.provider)),
        }
    }

    /// 同步生成文本
    pub async fn generate(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<GenerateResponse, String> {
        let profile = self.get_active_profile()
            .ok_or("No active LLM profile configured")?;
        
        let adapter = self.create_adapter(&profile)?;
        
        let request = GenerateRequest {
            prompt,
            max_tokens,
            temperature,
        };
        
        adapter.generate(request).await
            .map_err(|e| format!("Generation failed: {}", e))
    }

    /// 使用指定模型配置同步生成文本
    pub async fn generate_with_profile(
        &self,
        profile_id: &str,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<GenerateResponse, String> {
        let profile = self.get_profile_by_id(profile_id)
            .ok_or_else(|| format!("LLM profile '{}' not found", profile_id))?;
        
        let adapter = self.create_adapter(&profile)?;
        
        let request = GenerateRequest {
            prompt,
            max_tokens,
            temperature,
        };
        
        adapter.generate(request).await
            .map_err(|e| format!("Generation failed: {}", e))
    }

    /// 流式生成文本
    /// 
    /// 通过Tauri事件向前端发送生成进度
    /// 事件名称: `llm-stream-chunk`, `llm-stream-complete`, `llm-stream-error`
    pub async fn generate_stream(
        &self,
        request_id: String,
        prompt: String,
        context: Option<String>,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<(), String> {
        let start_time = std::time::Instant::now();
        
        let profile = self.get_active_profile()
            .ok_or("No active LLM profile configured")?;
        
        // 构建增强提示词
        let enhanced_prompt = self.build_writing_prompt(&prompt, context.as_deref());
        
        log::info!("[LLM] Starting stream generation with request_id: {}", request_id);
        log::debug!("[LLM] Prompt: {}...", &enhanced_prompt[..enhanced_prompt.len().min(100)]);
        
        // 目前使用模拟流式生成，后续可接入真实SSE
        // TODO: 实现真实的流式API调用
        let simulated_text = self.simulate_generation(&enhanced_prompt).await;
        
        // 模拟流式发送
        let chars: Vec<char> = simulated_text.chars().collect();
        let chunk_size = 3; // 每次发送3个字符
        
        for (i, chunk) in chars.chunks(chunk_size).enumerate() {
            let chunk_str: String = chunk.iter().collect();
            let is_first = i == 0;
            
            let stream_chunk = StreamChunk {
                chunk: chunk_str,
                is_first,
                is_last: false,
                model: profile.model.clone(),
            };
            
            // 发送事件到前端
            let _ = self.app_handle.emit(&format!("llm-stream-chunk-{}", request_id), stream_chunk);
            
            // 模拟打字延迟
            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        }
        
        // 发送完成事件
        let duration = start_time.elapsed().as_millis() as u64;
        let complete = GenerationComplete {
            full_text: simulated_text.clone(),
            model: profile.model.clone(),
            tokens_used: simulated_text.len() as i32 / 2, // 粗略估计
            cost: 0.001, // 粗略估计
            duration_ms: duration,
        };
        
        let _ = self.app_handle.emit(&format!("llm-stream-complete-{}", request_id), complete);
        
        log::info!("[LLM] Stream generation completed in {}ms", duration);
        
        Ok(())
    }

    /// 构建写作专用提示词
    fn build_writing_prompt(&self, user_input: &str, context: Option<&str>) -> String {
        let mut prompt = String::new();
        
        // 系统提示
        prompt.push_str("你是一位专业的小说创作助手，擅长中文写作。\n\n");
        
        // 上下文
        if let Some(ctx) = context {
            prompt.push_str("【前文上下文】\n");
            prompt.push_str(ctx);
            prompt.push_str("\n\n");
        }
        
        // 用户输入
        prompt.push_str("【续写要求】\n");
        prompt.push_str(user_input);
        prompt.push_str("\n\n");
        
        // 输出要求
        prompt.push_str("请直接输出续写内容，不要添加解释。保持文风一致，情节连贯。");
        
        prompt
    }

    /// 模拟生成（临时实现）
    async fn simulate_generation(&self, prompt: &str) -> String {
        // 根据提示词长度决定生成长度
        let prompt_len = prompt.len();
        
        let samples = vec![
            "夜风轻轻拂过窗棂，带来远处桂花的香气。她放下手中的笔，望向窗外那轮明月，心中涌起无限思绪。过去的点点滴滴，仿佛都在这清冷的月光下浮现。".to_string(),
            "他的声音低沉而温柔，像是大提琴的最后一个音符，在空气中缓缓消散。那一刻，她感觉时间都静止了，只剩下心跳的声音在耳畔回响。".to_string(),
            "雨点开始敲打屋顶，节奏清晰而有力，仿佛大自然在谱写一首独特的乐章。她靠在窗边，听着这雨声，思绪也随之飘向了远方。".to_string(),
            "烛光摇曳，在墙上投下舞动的影子。她轻抚那本泛黄的书页，指尖传来岁月的温度。那些文字，那些故事，仿佛都有了生命。".to_string(),
        ];
        
        // 根据prompt哈希选择一个样本
        let hash: usize = prompt.bytes().map(|b| b as usize).sum();
        let index = hash % samples.len();
        
        samples[index].clone()
    }

    /// 测试连接
    pub async fn test_connection(&self) -> Result<(bool, u64), String> {
        let profile = self.get_active_profile()
            .ok_or("No active LLM profile configured")?;
        
        let start = std::time::Instant::now();
        
        // 发送一个简单的测试请求
        let test_prompt = "Hello, respond with 'OK' only.";
        
        match self.generate(test_prompt.to_string(), Some(10), Some(0.0)).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                Ok((true, latency))
            }
            Err(e) => Err(e),
        }
    }
}

/// 全局LLM服务实例
static LLM_SERVICE: once_cell::sync::OnceCell<std::sync::Mutex<Option<LlmService>>> = once_cell::sync::OnceCell::new();

/// 初始化LLM服务
pub fn init_llm_service(app_handle: AppHandle) {
    let service = LlmService::new(app_handle);
    let _ = LLM_SERVICE.set(std::sync::Mutex::new(Some(service)));
}

/// 获取LLM服务
pub fn get_llm_service() -> Option<LlmService> {
    LLM_SERVICE.get()
        .and_then(|s| s.lock().ok())
        .and_then(|s| s.as_ref().cloned())
}

impl Clone for LlmService {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            config: Arc::clone(&self.config),
        }
    }
}
