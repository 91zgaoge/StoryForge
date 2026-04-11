//! Global State Manager
//! 
//! 单例模式，确保所有 Agent 读写同一份状态

use super::schema::*;
use crate::error::{CinemaError, Result};
use parking_lot::RwLock;
use std::sync::Arc;
use std::path::PathBuf;
use chrono::Utc;
use tracing::{info, debug, warn};

/// 状态管理器 - 线程安全的单例
pub struct StateManager {
    state: Arc<RwLock<StoryState>>,
    storage_path: PathBuf,
    auto_persist: bool,
}

impl StateManager {
    /// 创建新的状态管理器
    pub fn new(storage_path: PathBuf) -> Self {
        let state = Self::load_or_init(&storage_path);
        Self {
            state: Arc::new(RwLock::new(state)),
            storage_path,
            auto_persist: true,
        }
    }
    
    /// 从存储加载或初始化新状态
    fn load_or_init(path: &PathBuf) -> StoryState {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    match serde_json::from_str::<StoryState>(&content) {
                        Ok(state) => {
                            info!("Loaded story state from {:?}", path);
                            return state;
                        }
                        Err(e) => {
                            warn!("Failed to parse state: {}, creating new", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read state file: {}, creating new", e);
                }
            }
        }
        StoryState::default()
    }
    
    /// 获取当前状态（只读）
    pub fn get_state(&self) -> StoryState {
        self.state.read().clone()
    }
    
    /// 更新状态
    pub fn update_state<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut StoryState),
    {
        let mut state = self.state.write();
        updater(&mut state);
        state.metadata.last_updated = Utc::now();
        drop(state); // 释放锁
        
        if self.auto_persist {
            self.persist()?;
        }
        
        Ok(())
    }
    
    /// 进入下一章
    pub fn next_chapter(&self) -> Result<u32> {
        let mut new_chapter = 0;
        
        self.update_state(|state| {
            state.metadata.current_chapter += 1;
            new_chapter = state.metadata.current_chapter;
            info!("Advancing to chapter {}", new_chapter);
        })?;
        
        Ok(new_chapter)
    }
    
    /// 获取指定角色
    pub fn get_character(&self, char_id: &str) -> Option<Character> {
        self.state.read().characters.get(char_id).cloned()
    }
    
    /// 更新角色动态特质
    pub fn add_character_trait(
        &self,
        char_id: &str,
        trait: &str,
        chapter: u32,
        confidence: f32,
        evidence: &str,
    ) -> Result<()> {
        self.update_state(|state| {
            if let Some(char) = state.characters.get_mut(char_id) {
                // 检查是否已存在
                let exists = char.dynamic_traits.iter()
                    .any(|t| t.trait == trait && t.status != TraitStatus::Deprecated);
                
                if exists {
                    // 提升现有特质置信度
                    for t in &mut char.dynamic_traits {
                        if t.trait == trait {
                            t.confidence = (t.confidence + 0.1).min(1.0);
                            debug!("Boosted trait '{}' confidence to {}", trait, t.confidence);
                        }
                    }
                } else {
                    // 添加新特质
                    char.dynamic_traits.push(DynamicTrait {
                        trait: trait.to_string(),
                        source_chapter: chapter,
                        confidence: confidence.clamp(0.0, 1.0),
                        evidence: evidence.to_string(),
                        status: TraitStatus::Active,
                    });
                    info!(
                        "Added new trait '{}' to character {} at chapter {}",
                        trait, char_id, chapter
                    );
                }
            } else {
                warn!("Character {} not found, cannot add trait", char_id);
            }
        })
    }
    
    /// 更新写作风格
    pub fn adjust_style(
        &self,
        parameter: &str,
        new_value: &str,
        reason: &str,
    ) -> Result<()> {
        self.update_state(|state| {
            let chapter = state.metadata.current_chapter;
            let old_value = match parameter {
                "tone" => state.writing_style.tone.clone(),
                "pacing" => format!("{:?}", state.writing_style.pacing),
                _ => "unknown".to_string(),
            };
            
            // 应用新值
            match parameter {
                "tone" => state.writing_style.tone = new_value.to_string(),
                "pacing" => {
                    state.writing_style.pacing = match new_value {
                        "slow" => Pacing::Slow,
                        "medium" => Pacing::Medium,
                        "fast" => Pacing::Fast,
                        "dynamic" => Pacing::Dynamic,
                        _ => state.writing_style.pacing.clone(),
                    };
                }
                _ => {}
            }
            
            // 记录历史
            state.writing_style.evolution_history.push(StyleAdjustment {
                chapter,
                parameter: parameter.to_string(),
                old_value,
                new_value: new_value.to_string(),
                reason: reason.to_string(),
            });
            
            info!("Adjusted style parameter '{}' at chapter {}", parameter, chapter);
        })
    }
    
    /// 持久化状态到磁盘
    pub fn persist(&self) -> Result<()> {
        let state = self.state.read();
        let json = serde_json::to_string_pretty(&*state)
            .map_err(|e| CinemaError::Serialization(e))?;
        
        // 确保目录存在
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&self.storage_path, json)?;
        debug!("Persisted state to {:?}", self.storage_path);
        
        Ok(())
    }
    
    /// 导出为特定格式
    pub fn export(&self, format: ExportFormat) -> Result<String> {
        let state = self.state.read();
        
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&*state)
                    .map_err(|e| CinemaError::Serialization(e))
            }
            ExportFormat::Toml => {
                toml::to_string_pretty(&*state)
                    .map_err(|e| CinemaError::Validation(e.to_string()))
            }
            ExportFormat::Markdown => {
                Ok(self.export_markdown(&state))
            }
        }
    }
    
    fn export_markdown(&self, state: &StoryState) -> String {
        let mut md = format!("# {}\n\n", state.metadata.title);
        md.push_str(&format!("当前章节: {}\n\n", state.metadata.current_chapter));
        
        md.push_str("## 角色\n\n");
        for (id, char) in &state.characters {
            md.push_str(&format!("### {} ({})\n\n", char.name, id));
            md.push_str(&format!("**核心欲望**: {}\n\n", char.base_profile.core_desire));
            
            if !char.dynamic_traits.is_empty() {
                md.push_str("**动态特质**:\n");
                for t in &char.dynamic_traits {
                    if t.status == TraitStatus::Active {
                        md.push_str(&format!(
                            "- {} (第{}章发现, 置信度{:.0}%)\n",
                            t.trait, t.source_chapter, t.confidence * 100.0
                        ));
                    }
                }
                md.push('\n');
            }
        }
        
        md
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Toml,
    Markdown,
}
