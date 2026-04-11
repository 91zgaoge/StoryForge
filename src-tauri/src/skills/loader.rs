use super::*;
use serde_yaml;

pub struct SkillLoader {
    skills_dir: PathBuf,
}

impl SkillLoader {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self { skills_dir }
    }
    
    /// Load skill from directory
    pub fn load_from_directory(&self,
        dir: &Path,
    ) -> Result<Skill, String> {
        let manifest_path = dir.join("skill.yaml");
        if !manifest_path.exists() {
            return Err("skill.yaml not found".to_string());
        }
        
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| e.to_string())?;
        let manifest: SkillManifest = serde_yaml::from_str(&manifest_content)
            .map_err(|e| e.to_string())?;
        
        // Determine runtime type
        let runtime = self.load_runtime(&manifest, dir)?;
        
        Ok(Skill {
            manifest,
            path: dir.to_path_buf(),
            is_enabled: true,
            loaded_at: Utc::now(),
            runtime,
        })
    }
    
    /// Load skill from single file
    pub fn load_from_file(&self,
        file: &Path,
    ) -> Result<Skill, String> {
        let content = fs::read_to_string(file).map_err(|e| e.to_string())?;
        
        // Try YAML first
        if let Ok(manifest) = serde_yaml::from_str::<SkillManifest>(&content) {
            let runtime = self.infer_runtime_from_manifest(&manifest)?;
            return Ok(Skill {
                manifest,
                path: file.to_path_buf(),
                is_enabled: true,
                loaded_at: Utc::now(),
                runtime,
            });
        }
        
        // Try JSON
        if let Ok(manifest) = serde_json::from_str::<SkillManifest>(&content) {
            let runtime = self.infer_runtime_from_manifest(&manifest)?;
            return Ok(Skill {
                manifest,
                path: file.to_path_buf(),
                is_enabled: true,
                loaded_at: Utc::now(),
                runtime,
            });
        }
        
        Err("Failed to parse skill file".to_string())
    }
    
    /// Download and load skill from URL
    pub async fn download_and_load(
        &self,
        url: &str,
    ) -> Result<Skill, String> {
        let response = reqwest::get(url).await
            .map_err(|e| e.to_string())?;
        
        let content = response.text().await
            .map_err(|e| e.to_string())?;
        
        // Try to parse as manifest
        if let Ok(manifest) = serde_yaml::from_str::<SkillManifest>(&content) {
            let runtime = self.infer_runtime_from_manifest(&manifest)?;
            let path = self.skills_dir.join(&manifest.id);
            return Ok(Skill {
                manifest,
                path,
                is_enabled: true,
                loaded_at: Utc::now(),
                runtime,
            });
        }
        
        Err("Failed to download skill".to_string())
    }
    
    /// Save skill to directory
    pub fn save_to_directory(
        &self,
        skill: &Skill,
        dir: &Path,
    ) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        
        let manifest_path = dir.join("skill.yaml");
        let yaml = serde_yaml::to_string(&skill.manifest)
            .map_err(|e| e.to_string())?;
        fs::write(manifest_path, yaml).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    fn load_runtime(
        &self,
        manifest: &SkillManifest,
        dir: &Path,
    ) -> Result<SkillRuntime, String> {
        match manifest.entry_point.as_str() {
            ep if ep.ends_with(".prompt") => {
                self.load_prompt_runtime(dir, ep)
            }
            ep if ep.ends_with(".json") || ep == "mcp" => {
                self.load_mcp_runtime(dir, ep)
            }
            _ => Err("Unknown skill type".to_string()),
        }
    }
    
    fn load_prompt_runtime(
        &self,
        dir: &Path,
        entry: &str,
    ) -> Result<SkillRuntime, String> {
        let prompt_path = dir.join(entry);
        let content = fs::read_to_string(prompt_path)
            .map_err(|e| e.to_string())?;
        
        // Parse prompt file: system prompt + user template
        let parts: Vec<&str> = content.split("---").collect();
        let system_prompt = parts.get(0).unwrap_or(&"").trim().to_string();
        let user_template = parts.get(1).unwrap_or(&"").trim().to_string();
        
        Ok(SkillRuntime::Prompt(PromptRuntime {
            system_prompt,
            user_prompt_template: user_template,
        }))
    }
    
    fn load_mcp_runtime(
        &self,
        dir: &Path,
        entry: &str,
    ) -> Result<SkillRuntime, String> {
        let mcp_path = dir.join(entry);
        let content = fs::read_to_string(mcp_path)
            .map_err(|e| e.to_string())?;
        
        let config: McpServerConfig = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;
        
        Ok(SkillRuntime::Mcp(McpRuntime {
            server_config: config,
        }))
    }
    
    fn infer_runtime_from_manifest(
        &self,
        manifest: &SkillManifest,
    ) -> Result<SkillRuntime, String> {
        match manifest.entry_point.as_str() {
            ep if ep.ends_with(".prompt") => {
                Ok(SkillRuntime::Prompt(PromptRuntime {
                    system_prompt: String::new(),
                    user_prompt_template: String::new(),
                }))
            }
            _ => Ok(SkillRuntime::Native(NativeRuntime {
                handler: Arc::new(DummyHandler),
            })),
        }
    }
}

struct DummyHandler;

impl SkillHandler for DummyHandler {
    fn execute(
        &self,
        _context: &AgentContext,
        _params: HashMap<String, serde_json::Value>,
    ) -> Result<SkillResult, Box<dyn std::error::Error>> {
        Ok(SkillResult {
            success: true,
            data: serde_json::json!({}),
            error: None,
            execution_time_ms: 0,
        })
    }
}
