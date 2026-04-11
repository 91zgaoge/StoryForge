use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Result;
use std::path::Path;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_db(app_dir: &Path) -> Result<DbPool, Box<dyn std::error::Error>> {
    let db_path = app_dir.join("cinema_ai.db");
    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)?;
    
    // Initialize tables
    let mut conn = pool.get()?;
    create_tables(&mut conn)?;
    
    Ok(pool)
}

fn create_tables(conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- Stories table
        CREATE TABLE IF NOT EXISTS stories (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            genre TEXT,
            tone TEXT,
            pacing TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Characters table
        CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            name TEXT NOT NULL,
            background TEXT,
            personality TEXT,
            goals TEXT,
            dynamic_traits TEXT, -- JSON array
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- Chapters table
        CREATE TABLE IF NOT EXISTS chapters (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            chapter_number INTEGER NOT NULL,
            title TEXT,
            outline TEXT,
            content TEXT,
            word_count INTEGER,
            model_used TEXT,
            cost REAL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE,
            UNIQUE(story_id, chapter_number)
        );

        -- Create indexes
        CREATE INDEX IF NOT EXISTS idx_characters_story ON characters(story_id);
        CREATE INDEX IF NOT EXISTS idx_chapters_story ON chapters(story_id);
        CREATE INDEX IF NOT EXISTS idx_chapters_number ON chapters(story_id, chapter_number);
        "#
    )?;
    Ok(())
}
