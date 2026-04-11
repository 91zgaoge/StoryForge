use crate::error::{CinemaError, Result};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;
use tracing::info;

pub type DbPool = Pool<Sqlite>;

/// Initialize database connection pool
pub async fn init_db(database_url: &str) -> Result<DbPool> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(database_url).parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| CinemaError::Database(format!("Failed to create db directory: {}", e)))?;
    }

    info!("Initializing database at: {}", database_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| CinemaError::Database(format!("Failed to connect to database: {}", e)))?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .map_err(|e| CinemaError::Database(e.to_string()))?;

    info!("Database connection established");
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    info!("Running database migrations...");

    // Create chapters table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chapters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chapter_number INTEGER NOT NULL UNIQUE,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            outline TEXT,
            word_count INTEGER NOT NULL DEFAULT 0,
            model_used TEXT NOT NULL,
            cost REAL NOT NULL DEFAULT 0.0,
            generation_time_ms INTEGER NOT NULL DEFAULT 0,
            consistency_score REAL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CinemaError::Database(e.to_string()))?;

    // Create characters table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER,
            background TEXT,
            core_desire TEXT,
            fear TEXT,
            current_mood TEXT NOT NULL DEFAULT 'neutral',
            arc_status TEXT NOT NULL DEFAULT 'stable',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CinemaError::Database(e.to_string()))?;

    // Create dynamic_traits table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dynamic_traits (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            character_id TEXT NOT NULL,
            trait TEXT NOT NULL,
            source_chapter INTEGER NOT NULL,
            confidence REAL NOT NULL,
            evidence TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CinemaError::Database(e.to_string()))?;

    // Create story_metadata table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS story_metadata (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            title TEXT NOT NULL DEFAULT 'Untitled',
            current_chapter INTEGER NOT NULL DEFAULT 0,
            total_chapters INTEGER,
            writing_tone TEXT NOT NULL DEFAULT 'neutral',
            writing_pacing TEXT NOT NULL DEFAULT 'medium',
            vocabulary_density REAL NOT NULL DEFAULT 0.5,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CinemaError::Database(e.to_string()))?;

    // Create documents table for vector store
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            embedding BLOB,
            source_type TEXT NOT NULL,
            source_id TEXT NOT NULL,
            chapter INTEGER,
            importance REAL NOT NULL DEFAULT 1.0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CinemaError::Database(e.to_string()))?;

    // Create generation_logs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS generation_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chapter_number INTEGER NOT NULL,
            prompt TEXT NOT NULL,
            model_used TEXT NOT NULL,
            tokens_input INTEGER NOT NULL,
            tokens_output INTEGER NOT NULL,
            cost REAL NOT NULL,
            duration_ms INTEGER NOT NULL,
            success BOOLEAN NOT NULL,
            error_message TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CinemaError::Database(e.to_string()))?;

    // Insert default story metadata if not exists
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO story_metadata (id, title, current_chapter)
        VALUES (1, 'Untitled Story', 0)
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CinemaError::Database(e.to_string()))?;

    info!("Database migrations completed successfully");
    Ok(())
}
