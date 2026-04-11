use crate::error::{CinemaError, Result};
use crate::skills::CharacterSkill;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct SkillLoader {
    skills_dir: PathBuf,
}

impl SkillLoader {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self { skills_dir }
    }
    
    pub async fn load_character(&self, char_id: &str) -> Result<CharacterSkill> {
        let path = self.skills_dir
            .join("characters")
            .join(format!("{}.json", char_id));
        
        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| CinemaError::Io(e))?;
        
        let skill: CharacterSkill = serde_json::from_str(&content)
            .map_err(|e| CinemaError::Serialization(e))?;
        
        Ok(skill)
    }
    
    pub async fn load_all_characters(&self
    ) -> Result<Vec<CharacterSkill>> {
        let mut skills = vec![];
        let chars_dir = self.skills_dir.join("characters");
        
        for entry in WalkDir::new(chars_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && 
               entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                let content = tokio::fs::read_to_string(entry.path()).await?;
                let skill: CharacterSkill = serde_json::from_str(&content)?;
                skills.push(skill);
            }
        }
        
        Ok(skills)
    }
    
    pub async fn save_character(
        &self,
        skill: &CharacterSkill,
    ) -> Result<()> {
        let path = self.skills_dir
            .join("characters")
            .join(format!("{}.json", skill.character_id));
        
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        
        let content = serde_json::to_string_pretty(skill)
            .map_err(|e| CinemaError::Serialization(e))?;
        
        tokio::fs::write(path, content).await?;
        Ok(())
    }
}
