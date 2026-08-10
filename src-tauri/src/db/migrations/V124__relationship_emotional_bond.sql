-- 角色关系情感维度（双向情感纽带）
ALTER TABLE character_relationships ADD COLUMN emotional_bond TEXT;
ALTER TABLE character_relationships ADD COLUMN emotional_intensity REAL DEFAULT 0.5;
ALTER TABLE character_relationships ADD COLUMN reverse_emotional_bond TEXT;
ALTER TABLE character_relationships ADD COLUMN reverse_emotional_intensity REAL DEFAULT 0.5;
