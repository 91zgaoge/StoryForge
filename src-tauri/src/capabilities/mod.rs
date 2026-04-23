//! Capability Registry

pub mod evolution;
pub use evolution::{CapabilityEvolutionEngine, ExecutionRecord};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityParam {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub when_to_use: String,
    pub input_description: String,
    pub output_description: String,
    pub parameters: Vec<CapabilityParam>,
    pub source_type: CapabilitySource,
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySource {
    Agent,
    Skill,
    McpTool,
    SystemCommand,
}

pub struct CapabilityRegistry {
    capabilities: Vec<Capability>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self { capabilities: Vec::new() }
    }

    pub fn register(&mut self, capability: Capability) {
        self.capabilities.retain(|c| c.id != capability.id);
        self.capabilities.push(capability);
    }

    pub fn get_all(&self) -> &[Capability] {
        &self.capabilities
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.id == id)
    }

    pub fn to_llm_context(&self) -> String {
        let mut ctx = String::from("Available capabilities:\n\n");
        for cap in &self.capabilities {
            ctx.push_str(&format!("- {} ({}): {}\n", cap.name, cap.id, cap.description));
            ctx.push_str(&format!("  when_to_use: {}\n", cap.when_to_use));
        }
        ctx
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self { Self::new() }
}

pub fn build_default_registry() -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::new();

    // Agents
    registry.register(Capability {
        id: "writer".to_string(),
        name: "Writer Agent".to_string(),
        description: "Generates creative prose, dialogue, and narrative content based on story context and instructions.".to_string(),
        when_to_use: "Use when you need to write new story content, continue a scene, or generate narrative text.".to_string(),
        input_description: "Story context, writing instructions, style preferences, and optional outline beats.".to_string(),
        output_description: "Generated prose text, scene draft, or narrative continuation.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Target story ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "instruction".to_string(), description: "Writing prompt or instruction".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "style_dna_id".to_string(), description: "Optional style DNA ID".to_string(), required: false, param_type: "string".to_string() },
            CapabilityParam { name: "max_tokens".to_string(), description: "Maximum output tokens".to_string(), required: false, param_type: "integer".to_string() },
        ],
        source_type: CapabilitySource::Agent,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "inspector".to_string(),
        name: "Inspector Agent".to_string(),
        description: "Reviews generated content for quality, consistency, style adherence, and narrative logic.".to_string(),
        when_to_use: "Use after content generation to validate quality before accepting or publishing.".to_string(),
        input_description: "Draft text to inspect, along with expected style rules and continuity constraints.".to_string(),
        output_description: "Quality score (0-100), issues list, and optional rewrite suggestions.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Target story ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "draft".to_string(), description: "Text to inspect".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "check_style".to_string(), description: "Whether to check style DNA adherence".to_string(), required: false, param_type: "boolean".to_string() },
        ],
        source_type: CapabilitySource::Agent,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "outline_planner".to_string(),
        name: "Outline Planner Agent".to_string(),
        description: "Generates or refines story outlines, scene structures, and plot arcs.".to_string(),
        when_to_use: "Use when planning a new story, restructuring an existing one, or designing scene sequences.".to_string(),
        input_description: "Story premise, genre, target length, and optional existing outline fragments.".to_string(),
        output_description: "Structured outline with acts, scenes, beats, and turning points.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Target story ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "methodology".to_string(), description: "Outline methodology (e.g., snowflake, three-act, hero's journey)".to_string(), required: false, param_type: "string".to_string() },
            CapabilityParam { name: "chapter_count".to_string(), description: "Desired number of chapters".to_string(), required: false, param_type: "integer".to_string() },
        ],
        source_type: CapabilitySource::Agent,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "style_mimic".to_string(),
        name: "Style Mimic Agent".to_string(),
        description: "Analyzes and mimics a specific writing style from sample text or style DNA profiles.".to_string(),
        when_to_use: "Use when you want to adapt generated content to match a specific author's voice or custom style DNA.".to_string(),
        input_description: "Sample text or style DNA identifier, plus content to rewrite in that style.".to_string(),
        output_description: "Rewritten text matching the target style characteristics.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Target story ID".to_string(), required: false, param_type: "string".to_string() },
            CapabilityParam { name: "style_dna_id".to_string(), description: "Style DNA identifier".to_string(), required: false, param_type: "string".to_string() },
            CapabilityParam { name: "sample_text".to_string(), description: "Sample text to analyze style from".to_string(), required: false, param_type: "string".to_string() },
            CapabilityParam { name: "content".to_string(), description: "Content to rewrite in target style".to_string(), required: true, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::Agent,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "plot_analyzer".to_string(),
        name: "Plot Analyzer Agent".to_string(),
        description: "Analyzes plot structure, pacing, tension curves, and foreshadowing payoffs across the story.".to_string(),
        when_to_use: "Use when diagnosing plot issues, checking pacing, or validating foreshadowing resolution.".to_string(),
        input_description: "Full story text or scene list with summaries, plus specific analysis focus.".to_string(),
        output_description: "Plot analysis report with pacing metrics, tension graph, and payoff recommendations.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Target story ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "focus".to_string(), description: "Analysis focus: pacing, tension, foreshadowing, structure".to_string(), required: false, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::Agent,
        metadata: HashMap::new(),
    });

    // System commands
    registry.register(Capability {
        id: "create_story".to_string(),
        name: "Create Story".to_string(),
        description: "Creates a new story project with title, description, and genre.".to_string(),
        when_to_use: "Use when the user wants to start a new novel or story project.".to_string(),
        input_description: "Story title, optional description, and optional genre.".to_string(),
        output_description: "Newly created story object with ID and metadata.".to_string(),
        parameters: vec![
            CapabilityParam { name: "title".to_string(), description: "Story title".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "description".to_string(), description: "Short description or synopsis".to_string(), required: false, param_type: "string".to_string() },
            CapabilityParam { name: "genre".to_string(), description: "Story genre".to_string(), required: false, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::SystemCommand,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "create_chapter".to_string(),
        name: "Create Chapter".to_string(),
        description: "Creates a new chapter within an existing story.".to_string(),
        when_to_use: "Use when adding a new chapter to an existing story project.".to_string(),
        input_description: "Story ID, chapter number, and optional title, outline, and content.".to_string(),
        output_description: "Newly created chapter object.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Parent story ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "chapter_number".to_string(), description: "Chapter number (1-based)".to_string(), required: true, param_type: "integer".to_string() },
            CapabilityParam { name: "title".to_string(), description: "Chapter title".to_string(), required: false, param_type: "string".to_string() },
            CapabilityParam { name: "outline".to_string(), description: "Chapter outline".to_string(), required: false, param_type: "string".to_string() },
            CapabilityParam { name: "content".to_string(), description: "Initial chapter content".to_string(), required: false, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::SystemCommand,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "create_character".to_string(),
        name: "Create Character".to_string(),
        description: "Creates a new character within an existing story.".to_string(),
        when_to_use: "Use when adding a new character to a story project.".to_string(),
        input_description: "Story ID, character name, and optional background or personality description.".to_string(),
        output_description: "Newly created character object.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Parent story ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "name".to_string(), description: "Character name".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "background".to_string(), description: "Character background or biography".to_string(), required: false, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::SystemCommand,
        metadata: HashMap::new(),
    });

    // Skills
    registry.register(Capability {
        id: "builtin.style_enhancer".to_string(),
        name: "Style Enhancer".to_string(),
        description: "Enhances prose style by applying rhetorical techniques, rhythm adjustments, and vocabulary enrichment.".to_string(),
        when_to_use: "Use when text feels flat or needs stylistic elevation without changing meaning.".to_string(),
        input_description: "Raw or draft text to enhance, plus optional style constraints.".to_string(),
        output_description: "Stylistically enhanced text preserving original meaning.".to_string(),
        parameters: vec![
            CapabilityParam { name: "content".to_string(), description: "Text to enhance".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "intensity".to_string(), description: "Enhancement intensity: subtle, moderate, strong".to_string(), required: false, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::Skill,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "builtin.plot_twist".to_string(),
        name: "Plot Twist Generator".to_string(),
        description: "Generates unexpected but logical plot twists based on existing story context and character motivations.".to_string(),
        when_to_use: "Use when a scene or chapter needs an unexpected turn or revelation.".to_string(),
        input_description: "Story context, character states, and optional twist type preference.".to_string(),
        output_description: "Plot twist suggestion with setup and payoff notes.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Target story ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "twist_type".to_string(), description: "Desired twist type: revelation, reversal, betrayal, discovery".to_string(), required: false, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::Skill,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "builtin.text_formatter".to_string(),
        name: "Text Formatter".to_string(),
        description: "Intelligently formats and cleans up text, handling paragraph breaks, dialogue formatting, and punctuation.".to_string(),
        when_to_use: "Use when importing raw text or cleaning up draft formatting.".to_string(),
        input_description: "Unformatted or poorly formatted text.".to_string(),
        output_description: "Properly formatted text with consistent paragraph and dialogue structure.".to_string(),
        parameters: vec![
            CapabilityParam { name: "content".to_string(), description: "Text to format".to_string(), required: true, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::Skill,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "builtin.character_voice".to_string(),
        name: "Character Voice Generator".to_string(),
        description: "Generates dialogue or monologue matching a specific character's voice, speech patterns, and personality.".to_string(),
        when_to_use: "Use when writing dialogue for a specific character and wanting consistent voice.".to_string(),
        input_description: "Character information, scene context, and dialogue intent.".to_string(),
        output_description: "Dialogue or monologue in the character's distinctive voice.".to_string(),
        parameters: vec![
            CapabilityParam { name: "story_id".to_string(), description: "Target story ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "character_id".to_string(), description: "Target character ID".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "context".to_string(), description: "Scene context or dialogue prompt".to_string(), required: true, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::Skill,
        metadata: HashMap::new(),
    });

    registry.register(Capability {
        id: "builtin.emotion_pacing".to_string(),
        name: "Emotion Pacing Controller".to_string(),
        description: "Analyzes and adjusts emotional pacing in a scene, controlling tension build-up and release.".to_string(),
        when_to_use: "Use when a scene's emotional rhythm feels off or needs stronger tension management.".to_string(),
        input_description: "Scene text and desired emotional arc or target intensity.".to_string(),
        output_description: "Adjusted scene text with improved emotional pacing and tension curve.".to_string(),
        parameters: vec![
            CapabilityParam { name: "content".to_string(), description: "Scene text to adjust".to_string(), required: true, param_type: "string".to_string() },
            CapabilityParam { name: "target_intensity".to_string(), description: "Target emotional intensity: low, medium, high, climax".to_string(), required: false, param_type: "string".to_string() },
        ],
        source_type: CapabilitySource::Skill,
        metadata: HashMap::new(),
    });

    registry
}
