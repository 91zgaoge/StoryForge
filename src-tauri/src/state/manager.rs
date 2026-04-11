use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

/// Global story state manager
pub struct StoryStateManager {
    states: Arc<Mutex<HashMap<String, StoryState>>>,
    current_story_id: Arc<Mutex<Option<String>>>,
}

/// Complete story state for runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryState {
    pub story_id: String,
    pub story_info: StoryInfo,
    pub characters: HashMap<String, CharacterState>,
    pub chapters: Vec<ChapterState>,
    pub plot_progression: PlotProgression,
    pub world_state: WorldState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryInfo {
    pub title: String,
    pub description: Option<String>,
    pub genre: String,
    pub tone: String,
    pub pacing: String,
    pub target_chapters: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub id: String,
    pub name: String,
    pub arc_progress: f32, // 0.0 - 1.0
    pub current_emotion: String,
    pub relationships: HashMap<String, Relationship>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub target_id: String,
    pub affinity: f32, // -1.0 to 1.0
    pub relationship_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterState {
    pub id: String,
    pub number: u32,
    pub title: Option<String>,
    pub status: ChapterStatus,
    pub word_count: u32,
    pub key_events: Vec<String>,
    pub pov_character: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChapterStatus {
    Planned,
    Outlined,
    Writing,
    Completed,
    Revising,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotProgression {
    pub current_arc: String,
    pub tension_level: f32, // 0.0 - 1.0
    pub plot_points_hit: Vec<String>,
    pub foreshadowing_queue: Vec<String>,
    pub unresolved_conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub locations: HashMap<String, LocationState>,
    pub lore_elements: Vec<LoreElement>,
    pub timeline: Vec<TimelineEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationState {
    pub id: String,
    pub name: String,
    pub current_occupants: Vec<String>,
    pub atmosphere: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoreElement {
    pub id: String,
    pub name: String,
    pub content: String,
    pub is_revealed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub chapter_id: Option<String>,
}

impl StoryStateManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            current_story_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn create_state(&self, story_id: String, info: StoryInfo) -> StoryState {
        let state = StoryState {
            story_id: story_id.clone(),
            story_info: info,
            characters: HashMap::new(),
            chapters: Vec::new(),
            plot_progression: PlotProgression {
                current_arc: "introduction".to_string(),
                tension_level: 0.0,
                plot_points_hit: Vec::new(),
                foreshadowing_queue: Vec::new(),
                unresolved_conflicts: Vec::new(),
            },
            world_state: WorldState {
                locations: HashMap::new(),
                lore_elements: Vec::new(),
                timeline: Vec::new(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.states.lock().unwrap().insert(story_id.clone(), state.clone());
        *self.current_story_id.lock().unwrap() = Some(story_id);
        state
    }

    pub fn get_state(&self, story_id: &str) -> Option<StoryState> {
        self.states.lock().unwrap().get(story_id).cloned()
    }

    pub fn update_state(&self, story_id: &str, updater: impl FnOnce(&mut StoryState)) -> Result<(), String> {
        let mut states = self.states.lock().unwrap();
        if let Some(state) = states.get_mut(story_id) {
            updater(state);
            state.updated_at = Utc::now();
            Ok(())
        } else {
            Err("Story state not found".to_string())
        }
    }

    pub fn set_current_story(&self, story_id: String) {
        *self.current_story_id.lock().unwrap() = Some(story_id);
    }

    pub fn get_current_story(&self) -> Option<String> {
        self.current_story_id.lock().unwrap().clone()
    }

    pub fn add_character(&self, story_id: &str, character: CharacterState) -> Result<(), String> {
        self.update_state(story_id, |state| {
            state.characters.insert(character.id.clone(), character);
        })
    }

    pub fn update_chapter_status(&self, story_id: &str, chapter_id: &str, status: ChapterStatus) -> Result<(), String> {
        self.update_state(story_id, |state| {
            if let Some(chapter) = state.chapters.iter_mut().find(|c| c.id == chapter_id) {
                chapter.status = status;
            }
        })
    }

    pub fn add_plot_point(&self, story_id: &str, point: String) -> Result<(), String> {
        self.update_state(story_id, |state| {
            state.plot_progression.plot_points_hit.push(point);
        })
    }

    pub fn get_all_states(&self) -> Vec<StoryState> {
        self.states.lock().unwrap().values().cloned().collect()
    }
}

impl Default for StoryStateManager {
    fn default() -> Self {
        Self::new()
    }
}