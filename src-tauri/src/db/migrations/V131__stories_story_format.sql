-- V131: story format (novel | short_drama) + optional production constraints JSON
ALTER TABLE stories ADD COLUMN story_format TEXT NOT NULL DEFAULT 'novel';
ALTER TABLE stories ADD COLUMN production_constraints TEXT;
