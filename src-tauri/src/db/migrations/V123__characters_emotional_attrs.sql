-- 角色情感属性（身份级静态属性，创建时强制）
ALTER TABLE characters ADD COLUMN emotional_core TEXT;
ALTER TABLE characters ADD COLUMN emotional_trigger TEXT;
ALTER TABLE characters ADD COLUMN emotional_wound TEXT;
ALTER TABLE characters ADD COLUMN emotional_need TEXT;

-- 重建兼容视图 v_characters，加入情感属性列（从 kg_entities.attributes JSON 提取）
DROP VIEW IF EXISTS v_characters;
CREATE VIEW v_characters AS
SELECT
    id,
    story_id,
    name,
    json_extract(attributes, '$.background') AS background,
    json_extract(attributes, '$.personality') AS personality,
    json_extract(attributes, '$.goals') AS goals,
    json_extract(attributes, '$.appearance') AS appearance,
    json_extract(attributes, '$.gender') AS gender,
    json_extract(attributes, '$.age') AS age,
    json_extract(attributes, '$.dynamic_traits') AS dynamic_traits,
    source,
    is_auto_generated,
    first_seen AS created_at,
    last_updated AS updated_at,
    json_extract(attributes, '$.emotional_core') AS emotional_core,
    json_extract(attributes, '$.emotional_trigger') AS emotional_trigger,
    json_extract(attributes, '$.emotional_wound') AS emotional_wound,
    json_extract(attributes, '$.emotional_need') AS emotional_need
FROM kg_entities
WHERE entity_type = 'Character' AND is_archived = 0;
