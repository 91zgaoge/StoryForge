#![allow(dead_code)]
use super::{DbPool, Story, Character, Chapter, CreateStoryRequest, CreateCharacterRequest, CreateChapterRequest, DynamicTrait};
use chrono::Local;
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

pub struct StoryRepository {
    pool: DbPool,
}

impl StoryRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, req: CreateStoryRequest) -> Result<Story, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO stories (id, title, description, genre, tone, pacing, style_dna_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![&id, &req.title, req.description, req.genre, "dark", "medium", req.style_dna_id, now.to_rfc3339(), now.to_rfc3339()],
        )?;
        
        Ok(Story {
            id,
            title: req.title,
            description: req.description,
            genre: req.genre,
            tone: Some("dark".to_string()),
            pacing: Some("medium".to_string()),
            style_dna_id: req.style_dna_id,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_all(&self) -> Result<Vec<Story>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, title, description, genre, tone, pacing, style_dna_id, created_at, updated_at FROM stories ORDER BY updated_at DESC"
        )?;
        
        let stories = stmt.query_map([], |row| {
            let created_str: String = row.get(7)?;
            let updated_str: String = row.get(8)?;
            Ok(Story {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                genre: row.get(3)?,
                tone: row.get(4)?,
                pacing: row.get(5)?,
                style_dna_id: row.get(6)?,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(stories)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Story>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, title, description, genre, tone, pacing, style_dna_id, created_at, updated_at FROM stories WHERE id = ?1"
        )?;
        
        let story = stmt.query_row([id], |row| {
            let created_str: String = row.get(7)?;
            let updated_str: String = row.get(8)?;
            Ok(Story {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                genre: row.get(3)?,
                tone: row.get(4)?,
                pacing: row.get(5)?,
                style_dna_id: row.get(6)?,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        }).optional()?;
        
        Ok(story)
    }

    pub fn update(&self, id: &str, req: &super::UpdateStoryRequest) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();

        let count = conn.execute(
            "UPDATE stories SET title = COALESCE(?2, title), description = COALESCE(?3, description),
             tone = COALESCE(?4, tone), pacing = COALESCE(?5, pacing), style_dna_id = COALESCE(?6, style_dna_id), updated_at = ?7 WHERE id = ?1",
            params![id, req.title, req.description, req.tone, req.pacing, req.style_dna_id, now],
        )?;
        Ok(count)
    }

    pub fn delete(&self, id: &str) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let count = conn.execute("DELETE FROM stories WHERE id = ?1", [id])?;
        Ok(count)
    }
}

pub struct CharacterRepository {
    pool: DbPool,
}

impl CharacterRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, req: CreateCharacterRequest) -> Result<Character, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        let traits_json = "[]";
        
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO characters (id, story_id, name, background, personality, goals, dynamic_traits, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![&id, &req.story_id, &req.name, req.background, "", "", traits_json, now.to_rfc3339(), now.to_rfc3339()],
        )?;
        
        Ok(Character {
            id,
            story_id: req.story_id,
            name: req.name,
            background: req.background,
            personality: None,
            goals: None,
            dynamic_traits: vec![],
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_by_story(&self, story_id: &str) -> Result<Vec<Character>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, background, personality, goals, dynamic_traits, created_at, updated_at FROM characters WHERE story_id = ?1"
        )?;

        let characters = stmt.query_map([story_id], |row| {
            let traits_json: String = row.get(6)?;
            let dynamic_traits: Vec<DynamicTrait> = serde_json::from_str(&traits_json).unwrap_or_default();
            let created_str: String = row.get(7)?;
            let updated_str: String = row.get(8)?;

            Ok(Character {
                id: row.get(0)?,
                story_id: row.get(1)?,
                name: row.get(2)?,
                background: row.get(3)?,
                personality: row.get(4)?,
                goals: row.get(5)?,
                dynamic_traits,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(characters)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Character>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, background, personality, goals, dynamic_traits, created_at, updated_at FROM characters WHERE id = ?1"
        )?;

        let character = stmt.query_row([id], |row| {
            let traits_json: String = row.get(6)?;
            let dynamic_traits: Vec<DynamicTrait> = serde_json::from_str(&traits_json).unwrap_or_default();
            let created_str: String = row.get(7)?;
            let updated_str: String = row.get(8)?;

            Ok(Character {
                id: row.get(0)?,
                story_id: row.get(1)?,
                name: row.get(2)?,
                background: row.get(3)?,
                personality: row.get(4)?,
                goals: row.get(5)?,
                dynamic_traits,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        }).optional()?;

        Ok(character)
    }

    pub fn update(&self, id: &str, name: Option<String>, background: Option<String>, personality: Option<String>, goals: Option<String>) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();

        let count = conn.execute(
            "UPDATE characters SET name = COALESCE(?2, name), background = COALESCE(?3, background),
             personality = COALESCE(?4, personality), goals = COALESCE(?5, goals), updated_at = ?6 WHERE id = ?1",
            params![id, name, background, personality, goals, now],
        )?;
        Ok(count)
    }

    pub fn delete(&self, id: &str) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let count = conn.execute("DELETE FROM characters WHERE id = ?1", [id])?;
        Ok(count)
    }
}

pub struct ChapterRepository {
    pool: DbPool,
}

impl ChapterRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, req: CreateChapterRequest) -> Result<Chapter, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        let word_count = req.content.as_ref().map(|c| c.len() as i32);

        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO chapters (id, story_id, chapter_number, title, outline, content, word_count, model_used, cost, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &id, &req.story_id, req.chapter_number, req.title, req.outline, req.content,
                word_count, "", 0.0, now.to_rfc3339(), now.to_rfc3339()
            ],
        )?;

        Ok(Chapter {
            id,
            story_id: req.story_id,
            chapter_number: req.chapter_number,
            title: req.title,
            outline: req.outline,
            content: req.content,
            word_count,
            model_used: None,
            cost: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_by_story(&self, story_id: &str) -> Result<Vec<Chapter>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, chapter_number, title, outline, content, word_count, model_used, cost, created_at, updated_at FROM chapters WHERE story_id = ?1 ORDER BY chapter_number"
        )?;

        let chapters = stmt.query_map([story_id], |row| {
            let created_str: String = row.get(9)?;
            let updated_str: String = row.get(10)?;
            Ok(Chapter {
                id: row.get(0)?,
                story_id: row.get(1)?,
                chapter_number: row.get(2)?,
                title: row.get(3)?,
                outline: row.get(4)?,
                content: row.get(5)?,
                word_count: row.get(6)?,
                model_used: row.get(7)?,
                cost: row.get(8)?,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(chapters)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Chapter>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, chapter_number, title, outline, content, word_count, model_used, cost, created_at, updated_at FROM chapters WHERE id = ?1"
        )?;

        let chapter = stmt.query_row([id], |row| {
            let created_str: String = row.get(9)?;
            let updated_str: String = row.get(10)?;
            Ok(Chapter {
                id: row.get(0)?,
                story_id: row.get(1)?,
                chapter_number: row.get(2)?,
                title: row.get(3)?,
                outline: row.get(4)?,
                content: row.get(5)?,
                word_count: row.get(6)?,
                model_used: row.get(7)?,
                cost: row.get(8)?,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        }).optional()?;

        Ok(chapter)
    }

    pub fn update(&self, id: &str, title: Option<String>, outline: Option<String>, content: Option<String>, word_count: Option<i32>) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        let word_count = word_count.or_else(|| content.as_ref().map(|c| c.len() as i32));

        let count = conn.execute(
            "UPDATE chapters SET title = COALESCE(?2, title), outline = COALESCE(?3, outline),
             content = COALESCE(?4, content), word_count = COALESCE(?5, word_count), updated_at = ?6 WHERE id = ?1",
            params![id, title, outline, content, word_count, now],
        )?;
        Ok(count)
    }

    pub fn delete(&self, id: &str) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let count = conn.execute("DELETE FROM chapters WHERE id = ?1", [id])?;
        Ok(count)
    }
}
