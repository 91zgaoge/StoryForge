use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSkill {
    pub skill_type: String,
    pub character_id: String,
    pub version: String,
    pub last_updated: String,
    pub updated_by_chapter: u32,
    pub base_profile: BaseProfile,
    pub dynamic_traits: Vec<DynamicTrait>,
    pub relationships: HashMap<String, Relationship>,
    pub writing_guidance: WritingGuidance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseProfile {
    pub name: String,
    pub age: u32,
    pub background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTrait {
    pub trait_desc: String,
    pub source_chapter: u32,
    pub evidence: String,
    pub confidence: f32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relation_type: String,
    pub level: i8,
    pub dynamics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingGuidance {
    pub speech_pattern: String,
    pub inner_thoughts: String,
    pub action_style: String,
}

impl CharacterSkill {
    pub fn to_prompt_context(&self) -> String {
        let traits: Vec<String> = self.dynamic_traits.iter()
            .filter(|t| t.status == "active")
            .map(|t| format!("- {} (ch.{}, {}%)",
                t.trait_desc, t.source_chapter, (t.confidence * 100.0) as u32))
            .collect();
        
        format!(
            r#"Character: {}
Background: {}
Traits:
{}
Speech: {}
Thoughts: {}"#,
            self.base_profile.name,
            self.base_profile.background,
            traits.join("\n"),
            self.writing_guidance.speech_pattern,
            self.writing_guidance.inner_thoughts
        )
    }
}
