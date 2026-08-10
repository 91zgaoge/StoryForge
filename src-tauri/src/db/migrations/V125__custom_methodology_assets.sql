ALTER TABLE custom_methodologies ADD COLUMN patterns_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE custom_methodologies ADD COLUMN cheatsheet_json TEXT NOT NULL DEFAULT '{}';
