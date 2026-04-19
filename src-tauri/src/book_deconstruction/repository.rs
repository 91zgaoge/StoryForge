//! Reference Book Repository
//!
//! 参考书籍、人物、场景的数据库存取层。

use super::models::*;
use crate::db::DbPool;
use chrono::Local;
use rusqlite::{params, OptionalExtension};

pub struct ReferenceBookRepository {
    pool: DbPool,
}

impl ReferenceBookRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 创建参考书籍记录
    pub fn create(&self, book: &ReferenceBook) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO reference_books (id, title, author, genre, word_count, file_format, file_hash, file_path, world_setting, plot_summary, story_arc, analysis_status, analysis_progress, analysis_error, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                book.id,
                book.title,
                book.author,
                book.genre,
                book.word_count,
                book.file_format,
                book.file_hash,
                book.file_path,
                book.world_setting,
                book.plot_summary,
                book.story_arc,
                book.analysis_status.to_string(),
                book.analysis_progress,
                book.analysis_error,
                book.created_at.to_rfc3339(),
                book.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 根据ID获取
    pub fn get_by_id(&self, id: &str) -> Result<Option<ReferenceBook>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, author, genre, word_count, file_format, file_hash, file_path, world_setting, plot_summary, story_arc, analysis_status, analysis_progress, analysis_error, created_at, updated_at
             FROM reference_books WHERE id = ?1"
        )?;
        
        let book = stmt.query_row([id], |row| {
            let status_str: String = row.get(11)?;
            let status = status_str.parse().unwrap_or(AnalysisStatus::Pending);
            
            Ok(ReferenceBook {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                genre: row.get(3)?,
                word_count: row.get(4)?,
                file_format: row.get(5)?,
                file_hash: row.get(6)?,
                file_path: row.get(7)?,
                world_setting: row.get(8)?,
                plot_summary: row.get(9)?,
                story_arc: row.get(10)?,
                analysis_status: status,
                analysis_progress: row.get(12)?,
                analysis_error: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        }).optional()?;
        
        Ok(book)
    }

    /// 根据文件哈希获取（去重检查）
    pub fn get_by_hash(&self, hash: &str) -> Result<Option<ReferenceBook>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, author, genre, word_count, file_format, file_hash, file_path, world_setting, plot_summary, story_arc, analysis_status, analysis_progress, analysis_error, created_at, updated_at
             FROM reference_books WHERE file_hash = ?1"
        )?;
        
        let book = stmt.query_row([hash], |row| {
            let status_str: String = row.get(11)?;
            let status = status_str.parse().unwrap_or(AnalysisStatus::Pending);
            
            Ok(ReferenceBook {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                genre: row.get(3)?,
                word_count: row.get(4)?,
                file_format: row.get(5)?,
                file_hash: row.get(6)?,
                file_path: row.get(7)?,
                world_setting: row.get(8)?,
                plot_summary: row.get(9)?,
                story_arc: row.get(10)?,
                analysis_status: status,
                analysis_progress: row.get(12)?,
                analysis_error: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        }).optional()?;
        
        Ok(book)
    }

    /// 获取列表
    pub fn list_all(&self) -> Result<Vec<ReferenceBookSummary>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, author, genre, word_count, file_format, analysis_status, analysis_progress, created_at
             FROM reference_books ORDER BY created_at DESC"
        )?;
        
        let books = stmt.query_map([], |row| {
            Ok(ReferenceBookSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                genre: row.get(3)?,
                word_count: row.get(4)?,
                file_format: row.get(5)?,
                analysis_status: row.get(6)?,
                analysis_progress: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(books)
    }

    /// 更新分析状态和进度
    pub fn update_status(
        &self,
        id: &str,
        status: AnalysisStatus,
        progress: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE reference_books SET analysis_status = ?1, analysis_progress = ?2, updated_at = ?3 WHERE id = ?4",
            params![status.to_string(), progress, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// 更新分析结果
    pub fn update_analysis_result(
        &self,
        id: &str,
        genre: Option<&str>,
        world_setting: Option<&str>,
        plot_summary: Option<&str>,
        story_arc: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE reference_books SET genre = ?1, world_setting = ?2, plot_summary = ?3, story_arc = ?4, updated_at = ?5 WHERE id = ?6",
            params![genre, world_setting, plot_summary, story_arc, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// 更新错误信息
    pub fn update_error(
        &self,
        id: &str,
        error: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE reference_books SET analysis_status = 'failed', analysis_error = ?1, updated_at = ?2 WHERE id = ?3",
            params![error, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// 删除
    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM reference_books WHERE id = ?1", [id])?;
        Ok(())
    }
}

// ==================== 人物仓库 ====================

pub struct ReferenceCharacterRepository {
    pool: DbPool,
}

impl ReferenceCharacterRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, character: &ReferenceCharacter) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO reference_characters (id, book_id, name, role_type, personality, appearance, relationships, key_scenes, importance_score, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                character.id,
                character.book_id,
                character.name,
                character.role_type,
                character.personality,
                character.appearance,
                character.relationships,
                character.key_scenes,
                character.importance_score,
                character.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn create_batch(
        &self,
        characters: &[ReferenceCharacter],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        
        for character in characters {
            tx.execute(
                "INSERT INTO reference_characters (id, book_id, name, role_type, personality, appearance, relationships, key_scenes, importance_score, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    character.id,
                    character.book_id,
                    character.name,
                    character.role_type,
                    character.personality,
                    character.appearance,
                    character.relationships,
                    character.key_scenes,
                    character.importance_score,
                    character.created_at.to_rfc3339(),
                ],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    pub fn get_by_book(
        &self,
        book_id: &str,
    ) -> Result<Vec<ReferenceCharacter>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, book_id, name, role_type, personality, appearance, relationships, key_scenes, importance_score, created_at
             FROM reference_characters WHERE book_id = ?1 ORDER BY importance_score DESC, name"
        )?;
        
        let characters = stmt.query_map([book_id], |row| {
            Ok(ReferenceCharacter {
                id: row.get(0)?,
                book_id: row.get(1)?,
                name: row.get(2)?,
                role_type: row.get(3)?,
                personality: row.get(4)?,
                appearance: row.get(5)?,
                relationships: row.get(6)?,
                key_scenes: row.get(7)?,
                importance_score: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(characters)
    }

    pub fn delete_by_book(&self, book_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM reference_characters WHERE book_id = ?1", [book_id])?;
        Ok(())
    }
}

// ==================== 场景仓库 ====================

pub struct ReferenceSceneRepository {
    pool: DbPool,
}

impl ReferenceSceneRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, scene: &ReferenceScene) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO reference_scenes (id, book_id, sequence_number, title, summary, characters_present, key_events, conflict_type, emotional_tone, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                scene.id,
                scene.book_id,
                scene.sequence_number,
                scene.title,
                scene.summary,
                scene.characters_present,
                scene.key_events,
                scene.conflict_type,
                scene.emotional_tone,
                scene.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn create_batch(
        &self,
        scenes: &[ReferenceScene],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        
        for scene in scenes {
            tx.execute(
                "INSERT INTO reference_scenes (id, book_id, sequence_number, title, summary, characters_present, key_events, conflict_type, emotional_tone, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    scene.id,
                    scene.book_id,
                    scene.sequence_number,
                    scene.title,
                    scene.summary,
                    scene.characters_present,
                    scene.key_events,
                    scene.conflict_type,
                    scene.emotional_tone,
                    scene.created_at.to_rfc3339(),
                ],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    pub fn get_by_book(
        &self,
        book_id: &str,
    ) -> Result<Vec<ReferenceScene>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, book_id, sequence_number, title, summary, characters_present, key_events, conflict_type, emotional_tone, created_at
             FROM reference_scenes WHERE book_id = ?1 ORDER BY sequence_number"
        )?;
        
        let scenes = stmt.query_map([book_id], |row| {
            Ok(ReferenceScene {
                id: row.get(0)?,
                book_id: row.get(1)?,
                sequence_number: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                characters_present: row.get(5)?,
                key_events: row.get(6)?,
                conflict_type: row.get(7)?,
                emotional_tone: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(scenes)
    }

    pub fn delete_by_book(&self, book_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM reference_scenes WHERE book_id = ?1", [book_id])?;
        Ok(())
    }
}
