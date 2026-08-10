# 角色情感属性与情感关系 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 角色从诞生即带情感基因（4 项情感属性 + 双向情感关系），writer 上下文注入情感约束与情感张力驱动，前端可编辑情感关系，情感弧光成为故事驱动力。

**Architecture:** 纯 ALTER TABLE 加列（V123/V124，零新表零新依赖）→ 模型/DTO/Repository 全链路读写 → Agency 概念包与 materialize 落库 → 向导路径对齐 → writer 上下文注入（build_writer_context_from_db 纯函数提取）→ 前端 bug 修复与情感表单 → emotional_ledger 纯计算张力/弧光（对标 RotationLedger 模式）。

**Tech Stack:** Rust（src-tauri，rusqlite/r2d2 SQLite、serde_json）、React+TS（src-frontend，vitest）。

## Global Constraints

- 仓库 `/Users/yuzaimu/projects/StoryForge`，master 直接工作；中文 conventional commit；**只提交，不推送、不打 tag、不改版本号**（v0.34.0 已发布，本期为下一版本内容；CHANGELOG/版本留待用户决策）
- `.recovery/` 目录勿动勿提交；pre-commit 钩子（cargo +nightly fmt + prettier）不得绕过
- 已确认基线：`cargo test --lib` = **1234 passed / 2 ignored**；`npx vitest run` = 352 passed / 3 skipped（AGENTS.md 记载值；实现者以开工时实测为准并记录）；最新迁移 V122 → 本计划用 V123/V124
- DB 测试统一用 `crate::db::connection::create_test_pool()`
- TDD：每步先写失败测试（RED）→ 实现（GREEN）→ 重构 → 验证
- 已确认事实：`materialize.rs` 写入旧版 `characters` 表（build_continue_writer_context 经旧版回退读取）；前端 bug：`genesis.ts:24` 发送 `character_a_id`/`character_b_id`，后端期望 `source_character_id`/`target_character_id`
- 依赖顺序：Task 1 是地基必须先完成；Task 2/3/5 依赖 Task 1；Task 4 依赖 Task 1+2；Task 6 依赖 Task 1+4

---

### Task 1: DB 迁移 + 模型/DTO/Repository 扩展

**目标**：characters 表加 4 列情感属性；character_relationships 表加 4 列情感维度；Rust 模型/DTO/Repository 全链路读写。

**Files:**
- Create: `src-tauri/src/db/migrations/V123__characters_emotional_attrs.sql`
- Create: `src-tauri/src/db/migrations/V124__relationship_emotional_bond.sql`
- Modify: `src-tauri/src/db/models.rs`（Character :1136、CharacterRelationship :403、EmotionalBond 枚举）
- Modify: `src-tauri/src/db/dto.rs`（CreateCharacterRequest）
- Modify: `src-tauri/src/db/repositories/character_repository.rs`
- Modify: `src-tauri/src/db/repositories/character_relationship_repository.rs`

- [ ] **Step 1: RED — 写失败测试**

`src-tauri/src/db/repositories/character_repository.rs`（tests 模块追加）：

```rust
#[test]
fn test_create_character_with_emotional_attrs() {
    let pool = create_test_pool().unwrap();
    let story = StoryRepository::new(pool.clone()).create(story_req("情感属性测试")).unwrap();
    let repo = CharacterRepository::new(pool.clone());
    let ch = repo.create(CreateCharacterRequest {
        story_id: story.id.clone(),
        name: "阿岩".to_string(),
        background: Some("孤儿".into()),
        personality: Some("偏执".into()),
        goals: Some("夺回令牌".into()),
        appearance: None, gender: None, age: None, source: None, is_auto_generated: None,
        emotional_core: Some("表面冷漠内心炽热".into()),
        emotional_trigger: Some("被背叛时暴怒".into()),
        emotional_wound: Some("童年被师父抛弃".into()),
        emotional_need: Some("渴望被认可".into()),
    }).unwrap();
    let read = repo.get_by_id(&ch.id).unwrap().unwrap();
    assert_eq!(read.emotional_core.as_deref(), Some("表面冷漠内心炽热"));
    assert_eq!(read.emotional_trigger.as_deref(), Some("被背叛时暴怒"));
    assert_eq!(read.emotional_wound.as_deref(), Some("童年被师父抛弃"));
    assert_eq!(read.emotional_need.as_deref(), Some("渴望被认可"));
}

#[test]
fn test_update_character_emotional_attrs() {
    let pool = create_test_pool().unwrap();
    let story = StoryRepository::new(pool.clone()).create(story_req("情感更新测试")).unwrap();
    let repo = CharacterRepository::new(pool.clone());
    let ch = repo.create(req(&story.id, "林雪")).unwrap();
    repo.update_emotional(&ch.id,
        Some("愧疚驱动".into()), Some("看到令牌时闪回".into()),
        Some("未能阻止惨案".into()), Some("赎罪".into())).unwrap();
    let read = repo.get_by_id(&ch.id).unwrap().unwrap();
    assert_eq!(read.emotional_core.as_deref(), Some("愧疚驱动"));
    assert_eq!(read.emotional_need.as_deref(), Some("赎罪"));
}

#[test]
fn test_get_by_story_includes_emotional_attrs() {
    let pool = create_test_pool().unwrap();
    let story = StoryRepository::new(pool.clone()).create(story_req("批量情感测试")).unwrap();
    let repo = CharacterRepository::new(pool.clone());
    repo.create(CreateCharacterRequest {
        story_id: story.id.clone(), name: "甲".into(),
        background: None, personality: None, goals: None,
        appearance: None, gender: None, age: None, source: None, is_auto_generated: None,
        emotional_core: Some("愤怒".into()),
        emotional_trigger: None, emotional_wound: None, emotional_need: None,
    }).unwrap();
    let chars = repo.get_by_story(&story.id).unwrap();
    assert_eq!(chars.len(), 1);
    assert_eq!(chars[0].emotional_core.as_deref(), Some("愤怒"));
    assert!(chars[0].emotional_trigger.is_none());
}
```

`src-tauri/src/db/repositories/character_relationship_repository.rs`（tests 模块新建）：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_test_pool, dto::CreateStoryRequest, repositories::{StoryRepository, CharacterRepository}, dto::CreateCharacterRequest};

    fn setup(pool: &crate::db::DbPool, story_id: &str) -> (String, String) {
        let story = StoryRepository::new(pool.clone())
            .create(CreateStoryRequest { title: "测试".into(), description: None, genre: None, style_dna_id: None, genre_profile_id: None, methodology_id: None, reference_book_id: None }).unwrap();
        let conn = pool.get().unwrap();
        conn.execute("UPDATE stories SET id = ?1 WHERE id = ?2", rusqlite::params![story_id, story.id]).unwrap();
        let repo = CharacterRepository::new(pool.clone());
        let a = repo.create(CreateCharacterRequest { story_id: story_id.into(), name: "甲".into(), background: None, personality: None, goals: None, appearance: None, gender: None, age: None, source: None, is_auto_generated: None, emotional_core: None, emotional_trigger: None, emotional_wound: None, emotional_need: None }).unwrap();
        let b = repo.create(CreateCharacterRequest { story_id: story_id.into(), name: "乙".into(), background: None, personality: None, goals: None, appearance: None, gender: None, age: None, source: None, is_auto_generated: None, emotional_core: None, emotional_trigger: None, emotional_wound: None, emotional_need: None }).unwrap();
        (a.id, b.id)
    }

    #[test]
    fn test_create_relationship_with_emotional_bond() {
        let pool = create_test_pool().unwrap();
        let (a, b) = setup(&pool, "s1");
        let repo = CharacterRelationshipRepository::new(pool.clone());
        let rel = repo.create("s1", &a, &b, "师徒",
            Some("面和心不和"), None,
            Some("欺骗"), Some(0.9),
            Some("崇拜"), Some(0.7)).unwrap();
        assert_eq!(rel.emotional_bond.as_deref(), Some("欺骗"));
        assert_eq!(rel.emotional_intensity, Some(0.9));
        assert_eq!(rel.reverse_emotional_bond.as_deref(), Some("崇拜"));
        assert_eq!(rel.reverse_emotional_intensity, Some(0.7));
    }

    #[test]
    fn test_get_by_story_includes_emotional_fields() {
        let pool = create_test_pool().unwrap();
        let (a, b) = setup(&pool, "s2");
        let repo = CharacterRelationshipRepository::new(pool.clone());
        repo.create("s2", &a, &b, "同门", None, None,
            Some("恨"), Some(0.8), Some("恐惧"), Some(0.6)).unwrap();
        let rels = repo.get_by_story("s2").unwrap();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].emotional_bond.as_deref(), Some("恨"));
        assert_eq!(rels[0].reverse_emotional_bond.as_deref(), Some("恐惧"));
    }

    #[test]
    fn test_update_relationship_emotional_bond() {
        let pool = create_test_pool().unwrap();
        let (a, b) = setup(&pool, "s3");
        let repo = CharacterRelationshipRepository::new(pool.clone());
        let rel = repo.create("s3", &a, &b, "同门", None, None,
            Some("恨"), Some(0.8), None, None).unwrap();
        repo.update_emotional(&rel.id,
            Some("恨"), Some(1.0),
            Some("愧疚"), Some(0.5)).unwrap();
        let read = repo.get_by_id(&rel.id).unwrap().unwrap();
        assert_eq!(read.emotional_intensity, Some(1.0));
        assert_eq!(read.reverse_emotional_bond.as_deref(), Some("愧疚"));
    }
}
```

预期：编译失败——`Character` 无 `emotional_core` 等字段；`CharacterRelationship` 无 `emotional_bond` 等字段；`CreateCharacterRequest` 无情感字段；`CharacterRelationshipRepository::create` 签名不匹配。

- [ ] **Step 2: GREEN — 迁移文件**

`src-tauri/src/db/migrations/V123__characters_emotional_attrs.sql`：

```sql
ALTER TABLE characters ADD COLUMN emotional_core TEXT;
ALTER TABLE characters ADD COLUMN emotional_trigger TEXT;
ALTER TABLE characters ADD COLUMN emotional_wound TEXT;
ALTER TABLE characters ADD COLUMN emotional_need TEXT;
```

`src-tauri/src/db/migrations/V124__relationship_emotional_bond.sql`：

```sql
ALTER TABLE character_relationships ADD COLUMN emotional_bond TEXT;
ALTER TABLE character_relationships ADD COLUMN emotional_intensity REAL DEFAULT 0.5;
ALTER TABLE character_relationships ADD COLUMN reverse_emotional_bond TEXT;
ALTER TABLE character_relationships ADD COLUMN reverse_emotional_intensity REAL DEFAULT 0.5;
```

注册方式以实际为准：纯 SQL 迁移由目录自动扫描注册（本项目 MigrationRunner::load_migrations 目录扫描，V119/V122 先例均未在 mod.rs 注册）。若 `v_characters` 视图显式列字段，在 V123 中 `DROP VIEW IF EXISTS v_characters; CREATE VIEW ...` 重建；若是 `SELECT *` 则自动包含（先查该视图定义再定）。

- [ ] **Step 3: GREEN — Character 模型**（`db/models.rs:1136`）

```rust
pub struct Character {
    // ... 现有 13 字段不变 ...
    pub emotional_core: Option<String>,
    pub emotional_trigger: Option<String>,
    pub emotional_wound: Option<String>,
    pub emotional_need: Option<String>,
}
```

`from_entity`（`:1170`）追加：

```rust
emotional_core: attr_string(attrs, "emotional_core"),
emotional_trigger: attr_string(attrs, "emotional_trigger"),
emotional_wound: attr_string(attrs, "emotional_wound"),
emotional_need: attr_string(attrs, "emotional_need"),
```

`to_attributes`（`:1189`）追加 4 个 key。

- [ ] **Step 4: GREEN — CharacterRelationship 模型**（`db/models.rs:403`）

```rust
pub struct CharacterRelationship {
    // ... 现有 9 字段不变 ...
    pub emotional_bond: Option<String>,
    pub emotional_intensity: Option<f32>,
    pub reverse_emotional_bond: Option<String>,
    pub reverse_emotional_intensity: Option<f32>,
}
```

- [ ] **Step 5: GREEN — CreateCharacterRequest DTO**（`db/dto.rs`）追加 4 个 `Option<String>` 字段。

- [ ] **Step 6: GREEN — CharacterRepository**

- `create_in_tx`（`:16`）：attributes JSON 加 4 key；INSERT SQL 加 4 列；params 追加
- `row_to_character`（`:186`）：SELECT 加 4 列（索引 14-17）；结构体构造加 4 字段
- `get_by_story` fallback SELECT（`:144`）和 `get_by_id` fallback SELECT（`:171`）加 4 列
- `update`（`:214`）：attributes JSON 更新逻辑加 4 key；legacy UPDATE SQL COALESCE 加 4 列
- 新增 `update_emotional(&self, id, core, trigger, wound, need)` 方法
- `get_from_view` SELECT（`:503`）加 4 列（视图处理见 Step 2）

- [ ] **Step 7: GREEN — CharacterRelationshipRepository**

- `create`（`:14`）：签名加 4 参数 `emotional_bond: Option<&str>, emotional_intensity: Option<f32>, reverse_emotional_bond: Option<&str>, reverse_emotional_intensity: Option<f32>`；INSERT SQL 加 4 列；返回结构体加 4 字段
- `get_by_id`（`:59`）/`get_by_story`（`:96`）：SELECT 加 4 列；`query_row`/`query_map` 加 4 字段读取
- `update`（`:135`）：动态 SET builder 加 4 个可选字段
- 新增 `update_emotional(&self, id, bond, intensity, reverse_bond, reverse_intensity)` 方法

- [ ] **Step 8: REFACTOR**

- `EmotionalBond` 枚举（`db/models.rs`）：19 variant + `Display`（中文）+ `FromStr`（变体兼容）。暂只定义类型不强制使用
- 检查 `v_characters` 视图是否需重建

- [ ] **Step 9: 验证**

```bash
cd src-tauri && cargo test --lib -- character_repository emotional 2>&1 | tail -5
cd src-tauri && cargo test --lib -- character_relationship_repository 2>&1 | tail -5
cd src-tauri && cargo test --lib 2>&1 | tail -3
```

预期：1234 + ~6 = ~1240 passed。`cargo +nightly fmt`。

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/db/
git commit -m "新增(角色): 情感属性与情感关系 DB 层——V123/V124 加列 + 模型/DTO/Repository 全链路读写"
```

---

### Task 2: Agency 路径——SeedCharacter/SeedRelationship/ConceptPack + prompt + materialize

**目标**：创世概念包产出带情感属性的角色卡 + 角色间情感关系；materialize 落库到 characters 表（含情感列）和 character_relationships 表。

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（SeedCharacter :567、ConceptPack :580、concept_pack prompt :1664、genesis_fastpath :1511）
- Modify: `src-tauri/src/agency/materialize.rs`（character 分支 :34-86、relationship 分支新增）
- Modify: `resources/prompts/agency/agency_producer_system.md`

**Interfaces:**
- Consumes: Task 1 的 DB 列与 Repository
- Produces: `SeedRelationship`（source/target/relationship_type/emotional_bond/emotional_intensity/reverse_emotional_bond/reverse_emotional_intensity/description）；`ConceptPack.relationships: Vec<SeedRelationship>`；materialize 的 "relationship" 分支

- [ ] **Step 1: RED — 写失败测试**

`src-tauri/src/agency/materialize.rs`（tests 模块追加）：

```rust
#[test]
fn test_materialize_character_with_emotional_attrs() {
    let pool = create_test_pool().unwrap();
    story(&pool, "s1");
    let items = vec![item(
        "character", "主角",
        r#"{"name":"阿苔","background":"拾荒者","personality":"坚韧","goals":"找到星环",
           "emotional_core":"压抑的愤怒","emotional_trigger":"被背叛时暴怒",
           "emotional_wound":"目睹母亲惨死","emotional_need":"被认可"}"#,
    )];
    assert_eq!(materialize_assets(&pool, "s1", &items), 1);
    let conn = pool.get().unwrap();
    let core: String = conn.query_row(
        "SELECT emotional_core FROM characters WHERE story_id='s1'", [], |r| r.get(0)
    ).unwrap();
    assert_eq!(core, "压抑的愤怒");
    let wound: String = conn.query_row(
        "SELECT emotional_wound FROM characters WHERE story_id='s1'", [], |r| r.get(0)
    ).unwrap();
    assert_eq!(wound, "目睹母亲惨死");
}

#[test]
fn test_materialize_relationship_with_emotional_bond() {
    let pool = create_test_pool().unwrap();
    story(&pool, "s1");
    let items = vec![
        item("character", "甲", r#"{"name":"甲","background":"","personality":"","goals":""}"#),
        item("character", "乙", r#"{"name":"乙","background":"","personality":"","goals":""}"#),
        item("relationship", "关系", r#"[{"source":"甲","target":"乙","relationship_type":"师徒",
           "emotional_bond":"欺骗","emotional_intensity":0.9,
           "reverse_emotional_bond":"崇拜","reverse_emotional_intensity":0.7,
           "description":"面和心不和"}]"#),
    ];
    materialize_assets(&pool, "s1", &items);
    let conn = pool.get().unwrap();
    let bond: String = conn.query_row(
        "SELECT emotional_bond FROM character_relationships WHERE story_id='s1'", [], |r| r.get(0)
    ).unwrap();
    assert_eq!(bond, "欺骗");
    let rev_bond: String = conn.query_row(
        "SELECT reverse_emotional_bond FROM character_relationships WHERE story_id='s1'", [], |r| r.get(0)
    ).unwrap();
    assert_eq!(rev_bond, "崇拜");
}
```

`src-tauri/src/agency/coordinator.rs`（tests 模块追加）：

```rust
#[test]
fn test_seed_character_deserializes_emotional_fields() {
    let json = r#"{"name":"阿岩","background":"孤儿","personality":"偏执","goals":"夺回令牌",
      "emotional_core":"表面冷漠内心炽热","emotional_trigger":"被背叛时暴怒",
      "emotional_wound":"童年被师父抛弃","emotional_need":"渴望被认可"}"#;
    let ch: SeedCharacter = serde_json::from_str(json).unwrap();
    assert_eq!(ch.emotional_core, "表面冷漠内心炽热");
    assert_eq!(ch.emotional_trigger, "被背叛时暴怒");
}

#[test]
fn test_seed_character_backward_compat_without_emotional() {
    let json = r#"{"name":"甲","background":"背景","personality":"性格","goals":"目标"}"#;
    let ch: SeedCharacter = serde_json::from_str(json).unwrap();
    assert_eq!(ch.name, "甲");
    assert_eq!(ch.emotional_core, "");
}

#[test]
fn test_concept_pack_deserializes_relationships() {
    let json = r#"{"title":"书名","genre":"奇幻","logline":"一句话",
      "characters":[{"name":"甲","background":"","personality":"","goals":"",
        "emotional_core":"愤怒","emotional_trigger":"","emotional_wound":"","emotional_need":""}],
      "relationships":[{"source":"甲","target":"乙","relationship_type":"师徒",
        "emotional_bond":"欺骗","emotional_intensity":0.9,
        "reverse_emotional_bond":"崇拜","reverse_emotional_intensity":0.7,
        "description":"面和心不和"}]}"#;
    let pack: ConceptPack = serde_json::from_str(json).unwrap();
    assert_eq!(pack.relationships.len(), 1);
    assert_eq!(pack.relationships[0].emotional_bond, "欺骗");
    assert_eq!(pack.relationships[0].reverse_emotional_bond, "崇拜");
}

#[test]
fn test_concept_pack_backward_compat_without_relationships() {
    let json = r#"{"title":"书名","characters":[{"name":"甲","background":"","personality":"","goals":""}]}"#;
    let pack: ConceptPack = serde_json::from_str(json).unwrap();
    assert!(pack.relationships.is_empty());
}
```

预期：编译失败——`SeedCharacter` 无情感字段；`ConceptPack` 无 relationships；`SeedRelationship` 不存在；`materialize_assets` 无 relationship 分支。

- [ ] **Step 2: GREEN — SeedCharacter 扩展**（`coordinator.rs:567`）

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeedCharacter {
    #[serde(alias = "character_name")]
    pub name: String,
    #[serde(default, alias = "background_story", alias = "backstory")]
    pub background: String,
    #[serde(default, alias = "character")]
    pub personality: String,
    #[serde(default, alias = "goal", alias = "motivation")]
    pub goals: String,
    #[serde(default, alias = "emotion_core")]
    pub emotional_core: String,
    #[serde(default, alias = "emotion_trigger")]
    pub emotional_trigger: String,
    #[serde(default, alias = "emotion_wound")]
    pub emotional_wound: String,
    #[serde(default, alias = "emotion_need")]
    pub emotional_need: String,
}
```

- [ ] **Step 3: GREEN — SeedRelationship 新增**（SeedCharacter 之后）

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeedRelationship {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub relationship_type: String,
    pub emotional_bond: String,
    #[serde(default = "default_intensity")]
    pub emotional_intensity: f32,
    #[serde(default)]
    pub reverse_emotional_bond: String,
    #[serde(default = "default_intensity")]
    pub reverse_emotional_intensity: f32,
    #[serde(default)]
    pub description: String,
}
fn default_intensity() -> f32 { 0.5 }
```

- [ ] **Step 4: GREEN — ConceptPack 扩展**（`coordinator.rs:580`）

```rust
pub struct ConceptPack {
    pub title: String,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub logline: String,
    #[serde(default)]
    pub characters: Vec<SeedCharacter>,
    #[serde(default)]
    pub relationships: Vec<SeedRelationship>,
}
```

- [ ] **Step 5: GREEN — concept_pack prompt 改造**（`coordinator.rs:1664`）

```rust
let raw = concept_llm.complete_json(
    "你是小说策划，只输出 JSON。",
    &format!(
        "故事前提：{}\n\n输出 JSON：{{\"title\":\"书名\",\"genre\":\"类型\",\"logline\":\"一句话简介\",\"characters\":[{{\"name\":\"真名\",\"background\":\"背景\",\"personality\":\"性格\",\"goals\":\"欲望/目标\",\"emotional_core\":\"情感内核\",\"emotional_trigger\":\"情感触发\",\"emotional_wound\":\"情感创伤\",\"emotional_need\":\"情感需求\"}}],\"relationships\":[{{\"source\":\"角色A名\",\"target\":\"角色B名\",\"relationship_type\":\"社会关系\",\"emotional_bond\":\"A对B的真实情感\",\"emotional_intensity\":0.8,\"reverse_emotional_bond\":\"B对A的真实情感\",\"reverse_emotional_intensity\":0.6,\"description\":\"关系概述\"}}]}}\n\n要求（强制）：1.每个角色必须含全部8个字段，emotional_*不得为空 2.relationships不得为空 3.须含至少一条强负面情感（恨/欺骗/恐惧/嫉妒/毁灭欲） 4.intensity取0.0-1.0 5.情感关系可与表面社会关系不一致（2-3张角色卡）",
        premise
    ),
    TaskType::Brainstorming,
    2048,
).await?;
```

- [ ] **Step 6: GREEN — materialize.rs character 分支改造**（`:34-86`）

提取 4 情感字段：

```rust
let emotional_core = v.get("emotional_core").and_then(|x| x.as_str()).unwrap_or("").to_string();
let emotional_trigger = v.get("emotional_trigger").and_then(|x| x.as_str()).unwrap_or("").to_string();
let emotional_wound = v.get("emotional_wound").and_then(|x| x.as_str()).unwrap_or("").to_string();
let emotional_need = v.get("emotional_need").and_then(|x| x.as_str()).unwrap_or("").to_string();
```

UPDATE SQL 加 4 列：

```sql
UPDATE characters SET background=?3, personality=?4, goals=?5,
  emotional_core=?7, emotional_trigger=?8, emotional_wound=?9, emotional_need=?10,
  updated_at=?6
WHERE story_id=?1 AND name=?2
```

INSERT SQL 加 4 列（emotional_core/emotional_trigger/emotional_wound/emotional_need）。

- [ ] **Step 7: GREEN — materialize.rs relationship 分支新增**（character 分支之后）

```rust
"relationship" => {
    let rels: Vec<SeedRelationship> = parse_lenient(&item.content).unwrap_or_default();
    for rel in rels {
        let source_id: Option<String> = conn.query_row(
            "SELECT id FROM characters WHERE story_id=?1 AND name=?2",
            params![story_id, rel.source], |r| r.get(0)
        ).ok();
        let target_id: Option<String> = conn.query_row(
            "SELECT id FROM characters WHERE story_id=?1 AND name=?2",
            params![story_id, rel.target], |r| r.get(0)
        ).ok();
        match (source_id, target_id) {
            (Some(sid), Some(tid)) => {
                let id = uuid::Uuid::new_v4().to_string();
                let ts = now();
                match conn.execute(
                    "INSERT INTO character_relationships (id, story_id, source_character_id, \
                     target_character_id, relationship_type, description, emotional_bond, \
                     emotional_intensity, reverse_emotional_bond, reverse_emotional_intensity, \
                     created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                    params![id, story_id, sid, tid, rel.relationship_type,
                            if rel.description.is_empty() { None } else { Some(&rel.description) },
                            if rel.emotional_bond.is_empty() { None } else { Some(&rel.emotional_bond) },
                            rel.emotional_intensity,
                            if rel.reverse_emotional_bond.is_empty() { None } else { Some(&rel.reverse_emotional_bond) },
                            rel.reverse_emotional_intensity, ts],
                ) {
                    Ok(n) => count += n,
                    Err(e) => log::warn!("materialize: 写入关系失败: {}", e),
                }
            }
            _ => log::warn!("materialize: 关系 {} -> {} 找不到角色，跳过", rel.source, rel.target),
        }
    }
}
```

- [ ] **Step 8: GREEN — genesis_fastpath 关系入黑板**（`coordinator.rs:1511` 角色卡入黑板之后）

```rust
if !pack.relationships.is_empty() {
    let rel_json = serde_json::to_string(&pack.relationships).unwrap_or_default();
    board.write(&rid, AgentRole::Producer, BoardZone::Asset,
        "relationships", "角色情感关系", &rel_json, "relationship")?;
}
```

- [ ] **Step 9: GREEN — agency_producer_system.md prompt 改造**：角色卡 4 字段约定改为 8 字段 + relationship 条目格式。

- [ ] **Step 10: REFACTOR**

- `materialize.rs` 的 `find_character_id_by_name` 提取为 helper
- `item_type` 别名归一化加 `"emotional_relationship" | "bond" => "relationship"`

- [ ] **Step 11: 验证**

```bash
cd src-tauri && cargo test --lib -- materialize 2>&1 | tail -5
cd src-tauri && cargo test --lib -- coordinator::tests 2>&1 | tail -5
cd src-tauri && cargo test --lib 2>&1 | tail -3
```

预期：~1240 + ~6 = ~1246 passed。`cargo +nightly fmt`。

- [ ] **Step 12: Commit**

```bash
git add src-tauri/src/agency/ resources/prompts/agency/
git commit -m "新增(创世): 概念包产出情感角色卡+情感关系——SeedRelationship/ConceptPack/materialize 落库"
```

---

### Task 3: 向导路径——CharacterProfileOption + prompt

**目标**：向导路径（NovelCreationAgent）的角色谱也强制情感属性，与 Agency 路径对齐。

**Files:**
- Modify: `src-tauri/src/domain/novel_creation.rs`（CharacterProfileOption :20）
- Modify: `resources/prompts/creation/novel_creation_character_roster.md`
- Modify: `src-tauri/src/agents/novel_creation.rs`（fallback prompt :220-253、落库调用处）

- [ ] **Step 1: RED — 写失败测试**

`src-tauri/src/agents/novel_creation.rs`（tests 模块追加）：

```rust
#[test]
fn test_character_profile_option_deserializes_emotional_fields() {
    let json = r#"{"id":"c1","name":"阿岩","personality":"偏执","background":"孤儿",
      "goals":"夺回令牌","voice_style":"冷硬",
      "emotional_core":"表面冷漠内心炽热","emotional_trigger":"被背叛时暴怒",
      "emotional_wound":"童年被师父抛弃","emotional_need":"渴望被认可"}"#;
    let opt: CharacterProfileOption = serde_json::from_str(json).unwrap();
    assert_eq!(opt.emotional_core, "表面冷漠内心炽热");
    assert_eq!(opt.emotional_wound, "童年被师父抛弃");
}

#[test]
fn test_character_profile_option_backward_compat() {
    let json = r#"{"id":"c1","name":"甲","personality":"","background":"","goals":"","voice_style":""}"#;
    let opt: CharacterProfileOption = serde_json::from_str(json).unwrap();
    assert_eq!(opt.emotional_core, "");
}
```

- [ ] **Step 2: GREEN — CharacterProfileOption 扩展**（`domain/novel_creation.rs:20`）

```rust
pub struct CharacterProfileOption {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub background: String,
    pub goals: String,
    pub voice_style: String,
    #[serde(default)]
    pub emotional_core: String,
    #[serde(default)]
    pub emotional_trigger: String,
    #[serde(default)]
    pub emotional_wound: String,
    #[serde(default)]
    pub emotional_need: String,
}
```

- [ ] **Step 3: GREEN — 向导 prompt 改造**

`resources/prompts/creation/novel_creation_character_roster.md`：补全角色卡 schema 要求 8 字段。
`agents/novel_creation.rs:220-253` fallback prompt：JSON 示例从 6 字段改为 10 字段（加 4 情感属性）。

- [ ] **Step 4: GREEN — 向导落库改造**：向导路径创建角色时（`creation_commands.rs` 或 `agents/novel_creation.rs` 中 CharacterRepository::create 调用处），将情感属性填入 `CreateCharacterRequest`。

- [ ] **Step 5: 验证**

```bash
cd src-tauri && cargo test --lib -- novel_creation 2>&1 | tail -5
cd src-tauri && cargo test --lib 2>&1 | tail -3
```

预期：~1246 + ~2 = ~1248 passed。`cargo +nightly fmt`。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/domain/novel_creation.rs src-tauri/src/agents/novel_creation.rs resources/prompts/creation/
git commit -m "新增(向导): 角色谱强制情感属性——CharacterProfileOption +4 字段 + prompt/落库对齐"
```

---

### Task 4: Writer 上下文注入——情感属性 + 情感关系

**目标**：`build_continue_writer_context` 注入角色情感属性段 + 角色情感关系段，让 writer 生成正文时看到角色的情感基因和人际情感张力。

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（提取 `build_writer_context_from_db` 纯函数、角色段改造 :3186-3198、情感关系段新增）

**Interfaces:**
- Consumes: Task 1 的模型字段、Task 2 落库的数据
- Produces: `pub(crate) fn build_writer_context_from_db(pool: &DbPool, story_id: &str) -> String`（Task 6 复用）

- [ ] **Step 1: RED — 写失败测试**

`src-tauri/src/agency/coordinator.rs`（tests 模块追加）。测试策略：`build_continue_writer_context` 是 `&self` 方法需 Coordinator 实例，故先提取纯函数 `build_writer_context_from_db(pool, story_id) -> String`，直接测纯函数：

```rust
#[test]
fn test_build_continue_writer_context_includes_emotional_attrs() {
    // 创建 story + character（含情感属性），调用纯函数
    // 断言返回 ctx 含 "情感内核" 标签和具体值
}

#[test]
fn test_build_continue_writer_context_includes_emotional_relationships() {
    // 创建 story + 2 characters + relationship with emotional_bond
    // 断言 ctx 含 "情感关系" 段和 emotional_bond 值
}
```

- [ ] **Step 2: GREEN — 提取纯函数**

```rust
/// 从 DB 构建 writer 上下文（纯函数，可测试）。
pub(crate) fn build_writer_context_from_db(pool: &DbPool, story_id: &str) -> String {
    // ... 现有 build_continue_writer_context 的 db 闭包内容 ...
}
```

`build_continue_writer_context` 改为：

```rust
pub(crate) async fn build_continue_writer_context(&self, story_id: &str) -> String {
    let pool = self.pool.clone();
    let sid = story_id.to_string();
    self.db(move || build_writer_context_from_db(&pool, &sid))
        .await
        .unwrap_or_default()
}
```

- [ ] **Step 3: GREEN — 角色段改造**（`:3186-3198`）

```rust
let mut parts = vec![
    format!("性格：{}", c.personality.as_deref().unwrap_or("-")),
    format!("目标：{}", c.goals.as_deref().unwrap_or("-")),
    format!("背景：{}", c.background.as_deref().unwrap_or("-")),
];
if let Some(ref core) = c.emotional_core { if !core.is_empty() { parts.push(format!("情感内核：{}", core)); } }
if let Some(ref trigger) = c.emotional_trigger { if !trigger.is_empty() { parts.push(format!("情感触发：{}", trigger)); } }
if let Some(ref wound) = c.emotional_wound { if !wound.is_empty() { parts.push(format!("情感创伤：{}", wound)); } }
if let Some(ref need) = c.emotional_need { if !need.is_empty() { parts.push(format!("情感需求：{}", need)); } }
let line = format!("【角色·{}】{}\n", c.name, parts.join("｜"));
```

- [ ] **Step 4: GREEN — 情感关系段新增**（角色段之后，世界观段之前）

```rust
let rels = CharacterRelationshipRepository::new(pool.clone())
    .get_by_story(&sid).unwrap_or_default();
if !rels.is_empty() {
    ctx.push_str("【角色情感关系（创作约束：角色间的真实情感，可与表面社会关系不一致）】\n");
    for r in &rels {
        let src_name = chars.iter().find(|c| c.id == r.source_character_id)
            .map(|c| c.name.as_str()).unwrap_or("?");
        let tgt_name = r.target_character_name.as_deref().unwrap_or("?");
        let bond = r.emotional_bond.as_deref().unwrap_or("未明");
        let intensity = r.emotional_intensity.unwrap_or(0.5);
        let rev_bond = r.reverse_emotional_bond.as_deref().unwrap_or("未明");
        let rev_intensity = r.reverse_emotional_intensity.unwrap_or(0.5);
        let line = format!(
            "■ {} -> {}：社会关系={} ｜ 情感={}[{:.1}]（{} -> {}：{}[{:.1}]）\n",
            src_name, tgt_name, r.relationship_type,
            bond, intensity,
            tgt_name, src_name, rev_bond, rev_intensity,
        );
        if ctx.chars().count() + line.chars().count() > 6000 { break; }
        ctx.push_str(&line);
    }
    ctx.push_str("要求：角色言行须与其情感关系一致；不得让角色做出与其情感矛盾的行为（除非剧情有转变理由）。\n\n");
}
```

- [ ] **Step 5: 验证**

```bash
cd src-tauri && cargo test --lib -- coordinator::tests::test_build_writer_context 2>&1 | tail -5
cd src-tauri && cargo test --lib 2>&1 | tail -3
```

预期：~1248 + ~2 = ~1250 passed。`cargo +nightly fmt`。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/agency/coordinator.rs
git commit -m "新增(创作): writer 上下文注入角色情感属性+情感关系（build_writer_context_from_db 纯函数提取）"
```

---

### Task 5: 前端——Bug 修复 + 关系编辑 UI（含后端命令参数对齐）

**目标**：修复 `character_a_id`/`character_b_id` 参数名 bug；前端类型加情感字段；关系创建/编辑表单加情感字段；后端命令加 4 个 Option 参数。

**Files:**
- Modify: `src-frontend/src/services/api/genesis.ts`（:22-28）
- Modify: `src-frontend/src/types/index.ts`（CharacterRelationship :277）
- Modify: `src-frontend/src/components/CharacterRelationshipForm.tsx`
- Modify: `src-frontend/src/pages/Characters.tsx`（RelationshipCard :44-166）
- Modify: `src-frontend/src/hooks/useCharacterRelationships.ts`
- Modify: `src-tauri/src/revision_commands.rs`（create :513 / update :568）
- Test: `src-frontend/src/services/api/__tests__/genesis.test.ts`（新）
- Test: `src-frontend/src/components/__tests__/CharacterRelationshipForm.test.tsx`（新）

- [ ] **Step 1: RED — 写失败测试**

`src-frontend/src/services/api/__tests__/genesis.test.ts`：

```typescript
import { describe, it, expect, vi } from 'vitest';
import { createCharacterRelationship } from '../genesis';

describe('createCharacterRelationship', () => {
  it('sends source_character_id and target_character_id (not character_a_id)', async () => {
    const mockInvoke = vi.fn().mockResolvedValue({});
    vi.mock('../core', () => ({ loggedInvoke: mockInvoke }));
    await createCharacterRelationship({
      story_id: 's1',
      source_character_id: 'char-a',
      target_character_id: 'char-b',
      relationship_type: '师徒',
      emotional_bond: '欺骗',
      emotional_intensity: 0.9,
      reverse_emotional_bond: '崇拜',
      reverse_emotional_intensity: 0.7,
    });
    expect(mockInvoke).toHaveBeenCalledWith('create_character_relationship', expect.objectContaining({
      source_character_id: 'char-a',
      target_character_id: 'char-b',
      emotional_bond: '欺骗',
    }));
    const args = mockInvoke.mock.calls[0][1];
    expect(args).not.toHaveProperty('character_a_id');
    expect(args).not.toHaveProperty('character_b_id');
  });
});
```

`src-frontend/src/components/__tests__/CharacterRelationshipForm.test.tsx`：

```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CharacterRelationshipForm } from '../CharacterRelationshipForm';

describe('CharacterRelationshipForm', () => {
  it('renders emotional bond fields', () => {
    render(<CharacterRelationshipForm storyId="s1" characters={[]} isOpen={true} onClose={() => {}} />);
    expect(screen.getByText(/A对B的情感/)).toBeInTheDocument();
    expect(screen.getByText(/B对A的情感/)).toBeInTheDocument();
  });
});
```

（mock 方式与组件 props 以 src-frontend 既有测试风格为准调整。）

- [ ] **Step 2: GREEN — 修复前端 API**（`genesis.ts:22-28`）

```typescript
export const createCharacterRelationship = (params: {
  story_id: string;
  source_character_id: string;
  target_character_id: string;
  relationship_type: string;
  description?: string;
  emotional_bond?: string;
  emotional_intensity?: number;
  reverse_emotional_bond?: string;
  reverse_emotional_intensity?: number;
}) => loggedInvoke<CharacterRelationship>('create_character_relationship', params);

export const updateCharacterRelationship = (
  relationshipId: string,
  updates: {
    relationship_type?: string;
    description?: string;
    emotional_bond?: string;
    emotional_intensity?: number;
    reverse_emotional_bond?: string;
    reverse_emotional_intensity?: number;
  }
) => loggedInvoke<void>('update_character_relationship', {
  relationship_id: relationshipId,
  ...updates,
});
```

- [ ] **Step 3: GREEN — 前端类型扩展**（`types/index.ts:277`）

```typescript
export interface CharacterRelationship {
  id: string;
  story_id: string;
  source_character_id: string;
  target_character_id: string;
  target_character_name?: string;
  relationship_type: string;
  description?: string;
  dynamic?: string;
  emotional_bond?: string;
  emotional_intensity?: number;
  reverse_emotional_bond?: string;
  reverse_emotional_intensity?: number;
  created_at: string;
}
```

- [ ] **Step 4: GREEN — CharacterRelationshipForm 加情感字段**：新增 state `emotionalBond`/`emotionalIntensity`/`reverseEmotionalBond`/`reverseEmotionalIntensity`；表单加 4 个输入（情感类型 select + 强度 range slider），submit 时传入。

- [ ] **Step 5: GREEN — Characters.tsx RelationshipCard 展示情感**：展示 emotional_bond + intensity + reverse；编辑表单加 4 字段。

- [ ] **Step 6: GREEN — 后端命令参数对齐**（`revision_commands.rs:513`/`:568`）：create/update 各加 4 个 Option 参数，传递给 repository。

- [ ] **Step 7: GREEN — useCharacterRelationships hook 扩展**：useCreate/useUpdate 参数类型加情感字段。

- [ ] **Step 8: 验证**

```bash
cd src-frontend && npx tsc --noEmit
cd src-frontend && npx vitest run -- genesis CharacterRelationshipForm 2>&1 | tail -5
cd src-tauri && cargo test --lib -- revision_commands 2>&1 | tail -5
cd src-tauri && cargo test --lib 2>&1 | tail -3
```

预期：~1250 + ~3 = ~1253 passed Rust；vitest +2。`cargo +nightly fmt`。

- [ ] **Step 9: Commit**

```bash
git add src-frontend/ src-tauri/src/revision_commands.rs
git commit -m "修复(前端): 角色关系参数名 bug（character_a_id→source_character_id）+ 情感字段编辑 UI"
```

---

### Task 6: 下游消费——情感张力种子 + 情感弧光推导

**目标**：从角色情感属性 + 情感关系计算"情感张力种子"，注入 writer 上下文作为故事驱动力。对标 RotationLedger 的纯计算 + prompt 渲染模式。

**Files:**
- Create: `src-tauri/src/agency/emotional_ledger.rs`
- Modify: `src-tauri/src/agency/mod.rs`（注册模块）
- Modify: `src-tauri/src/agency/coordinator.rs`（build_writer_context_from_db 注入）
- Modify: `src-tauri/src/agents/orchestrator.rs`（build_progression_anchor 注入）

**Interfaces:**
- Consumes: Task 1 模型字段、Task 4 `build_writer_context_from_db`
- Produces: `load_tensions(pool, story_id) -> Vec<InterpersonalTension>`、`load_arcs(pool, story_id) -> Vec<EmotionalArc>`、`render_tensions_for_prompt`、`render_arcs_for_prompt`

- [ ] **Step 1: RED — 写失败测试**

`src-tauri/src/agency/emotional_ledger.rs`（新文件，先放测试）：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_tension_from_deception_high_intensity() {
        let tension = compute_interpersonal_tension("欺骗", 0.9, "崇拜", 0.7);
        assert!(tension.tension_type.contains("未揭穿的欺骗"));
        assert!(tension.pressure >= 0.7);
        assert_eq!(tension.accumulated_chapters, 0);
    }

    #[test]
    fn test_compute_tension_mutual_hate() {
        let tension = compute_interpersonal_tension("恨", 0.8, "恨", 0.8);
        assert!(tension.tension_type.contains("对抗"));
        assert!(tension.pressure >= 0.6);
    }

    #[test]
    fn test_compute_tension_low_intensity_neutral() {
        let tension = compute_interpersonal_tension("信任", 0.3, "信任", 0.3);
        assert!(tension.pressure < 0.3);
    }

    #[test]
    fn test_render_tensions_for_prompt() {
        let tensions = vec![
            InterpersonalTension {
                source_name: "阿岩".into(),
                target_name: "林雪".into(),
                tension_type: "未揭穿的欺骗".into(),
                pressure: 0.8,
                accumulated_chapters: 2,
                suggested_action: "揭穿或加深欺骗".into(),
            },
        ];
        let text = render_tensions_for_prompt(&tensions);
        assert!(text.contains("阿岩 -> 林雪"));
        assert!(text.contains("未揭穿的欺骗"));
        assert!(text.contains("0.8"));
        assert!(text.contains("揭穿或加深欺骗"));
    }

    #[test]
    fn test_compute_emotional_arc_from_attributes() {
        let arc = compute_emotional_arc(
            "童年被师父抛弃",
            "渴望被认可",
            "容易被愤怒驱动",
        );
        assert!(arc.start_emotion.contains("不安全") || arc.start_emotion.contains("恐惧"));
        assert!(arc.end_emotion.contains("认可") || arc.end_emotion.contains("自信"));
        assert!(!arc.catalyst.is_empty());
    }

    #[test]
    fn test_render_arc_for_prompt() {
        let arc = EmotionalArc {
            character_name: "阿岩".into(),
            start_emotion: "不安全感".into(),
            current_emotion: "压抑的愤怒".into(),
            end_emotion: "被认可的自信".into(),
            catalyst: "被背叛时暴怒".into(),
            stage: ArcStage::Brewing,
        };
        let text = render_arc_for_prompt(&arc);
        assert!(text.contains("阿岩"));
        assert!(text.contains("不安全感"));
        assert!(text.contains("被认可的自信"));
        assert!(text.contains("酝酿"));
    }

    #[test]
    fn test_empty_tensions_render_empty() {
        let text = render_tensions_for_prompt(&[]);
        assert!(text.is_empty());
    }
}
```

预期：编译失败——模块不存在。

- [ ] **Step 2: GREEN — 实现 emotional_ledger.rs**

```rust
//! 情感张力账本（Emotional Tension Ledger）。
//! 纯 Rust 零 LLM 计算，从 DB 读取角色情感属性 + 情感关系，
//! 计算人际张力种子 + 情感弧光，渲染为 prompt 注入文本。
//! 对标 RotationLedger 的 load_sync / render_for_prompt 模式。

use crate::db::DbPool;

/// 人际情感张力（从情感关系推导的剧情驱动力）
#[derive(Debug, Clone)]
pub struct InterpersonalTension {
    pub source_name: String,
    pub target_name: String,
    pub tension_type: String,
    pub pressure: f32,
    pub accumulated_chapters: u32,
    pub suggested_action: String,
}

/// 角色情感弧光（从情感属性推导的成长/堕落轨迹）
#[derive(Debug, Clone)]
pub struct EmotionalArc {
    pub character_name: String,
    pub start_emotion: String,
    pub current_emotion: String,
    pub end_emotion: String,
    pub catalyst: String,
    pub stage: ArcStage,
}

#[derive(Debug, Clone, Copy)]
pub enum ArcStage { Brewing, Escalating, Climax, Transforming, Resolving }

/// 从情感关系计算单条人际张力
pub fn compute_interpersonal_tension(
    bond: &str, intensity: f32,
    reverse_bond: &str, reverse_intensity: f32,
) -> InterpersonalTension {
    let tension_type = classify_tension(bond, reverse_bond);
    let pressure = compute_pressure(bond, intensity, reverse_bond, reverse_intensity);
    let suggested_action = suggest_action(&tension_type, pressure);
    InterpersonalTension {
        source_name: String::new(),
        target_name: String::new(),
        tension_type,
        pressure,
        accumulated_chapters: 0,
        suggested_action,
    }
}

fn classify_tension(bond: &str, reverse_bond: &str) -> String {
    if bond.contains("欺骗") || bond.contains("谎言") {
        if !reverse_bond.contains("恨") && !reverse_bond.contains("欺骗") {
            return "未揭穿的欺骗".into();
        }
    }
    if bond.contains("恨") && reverse_bond.contains("恨") {
        return "对抗".into();
    }
    if bond.contains("执念") || bond.contains("痴迷") {
        return "单方面执念".into();
    }
    if bond.contains("毁灭") {
        return "毁灭倾向".into();
    }
    if bond.contains("复仇") || bond.contains("报复") {
        return "复仇驱动".into();
    }
    if bond.contains("嫉妒") || bond.contains("妒") {
        return "嫉妒暗涌".into();
    }
    if bond.contains("愧疚") || bond.contains("内疚") {
        return "愧疚与怨恨".into();
    }
    format!("{}与{}", bond, reverse_bond)
}

fn compute_pressure(bond: &str, intensity: f32, _reverse_bond: &str, reverse_intensity: f32) -> f32 {
    let base = (intensity + reverse_intensity) / 2.0;
    let negative_boost = if bond.contains("恨") || bond.contains("欺骗") || bond.contains("毁灭") || bond.contains("复仇") {
        0.15
    } else { 0.0 };
    (base + negative_boost).min(1.0)
}

fn suggest_action(tension_type: &str, pressure: f32) -> String {
    if pressure > 0.7 {
        match tension_type {
            "未揭穿的欺骗" => "本节应让欺骗接近暴露或加深一层".into(),
            "对抗" => "本节应让对抗升级为直接冲突".into(),
            "毁灭倾向" => "本节应让毁灭冲动找到宣泄口或被遏制".into(),
            _ => "本节应让张力释放或升级".into(),
        }
    } else if pressure > 0.4 {
        format!("本节可加深{}的铺垫", tension_type)
    } else {
        "暂无紧迫驱动，保持情感暗流".into()
    }
}

/// 从角色情感属性推导弧光
pub fn compute_emotional_arc(wound: &str, need: &str, core: &str) -> EmotionalArc {
    let start_emotion = infer_start_emotion(wound);
    let end_emotion = infer_end_emotion(need);
    let current_emotion = core.to_string();
    let catalyst = format!("当{}发生时", wound);
    EmotionalArc {
        character_name: String::new(),
        start_emotion,
        current_emotion,
        end_emotion,
        catalyst,
        stage: ArcStage::Brewing,
    }
}

fn infer_start_emotion(wound: &str) -> String {
    if wound.contains("抛弃") || wound.contains("遗弃") { return "不安全感/被遗弃恐惧".into(); }
    if wound.contains("背叛") { return "不信任/防御".into(); }
    if wound.contains("惨死") || wound.contains("死亡") || wound.contains("杀害") { return "创伤后恐惧/无力感".into(); }
    if wound.contains("失败") || wound.contains("屈辱") { return "自我怀疑/羞耻".into(); }
    if wound.contains("失去") { return "丧失感/空洞".into(); }
    format!("源自({})的情感创伤", wound)
}

fn infer_end_emotion(need: &str) -> String {
    if need.contains("认可") || need.contains("肯定") { return "被认可的自信".into(); }
    if need.contains("归属") { return "归属感/安定".into(); }
    if need.contains("掌控") || need.contains("控制") { return "掌控的从容".into(); }
    if need.contains("爱") { return "被爱的温暖".into(); }
    if need.contains("自由") { return "自由的释然".into(); }
    if need.contains("复仇") || need.contains("报复") { return "复仇后的空虚或释然".into(); }
    format!("满足({})", need)
}

/// 从 DB 加载所有角色情感关系，计算张力列表
pub fn load_tensions(pool: &DbPool, story_id: &str) -> Vec<InterpersonalTension> {
    use crate::db::repositories::{CharacterRelationshipRepository, CharacterRepository};
    let rels = CharacterRelationshipRepository::new(pool.clone())
        .get_by_story(story_id).unwrap_or_default();
    let chars = CharacterRepository::new(pool.clone())
        .get_by_story(story_id).unwrap_or_default();
    rels.iter().map(|r| {
        let mut t = compute_interpersonal_tension(
            r.emotional_bond.as_deref().unwrap_or("未明"),
            r.emotional_intensity.unwrap_or(0.5),
            r.reverse_emotional_bond.as_deref().unwrap_or("未明"),
            r.reverse_emotional_intensity.unwrap_or(0.5),
        );
        t.source_name = chars.iter().find(|c| c.id == r.source_character_id)
            .map(|c| c.name.clone()).unwrap_or_default();
        t.target_name = r.target_character_name.clone().unwrap_or_default();
        t
    }).collect()
}

/// 从 DB 加载所有角色情感弧光
pub fn load_arcs(pool: &DbPool, story_id: &str) -> Vec<EmotionalArc> {
    let chars = crate::db::repositories::CharacterRepository::new(pool.clone())
        .get_by_story(story_id).unwrap_or_default();
    chars.iter().filter_map(|c| {
        let wound = c.emotional_wound.as_deref()?;
        let need = c.emotional_need.as_deref()?;
        let core = c.emotional_core.as_deref().unwrap_or("");
        if wound.is_empty() && need.is_empty() { return None; }
        let mut arc = compute_emotional_arc(wound, need, core);
        arc.character_name = c.name.clone();
        Some(arc)
    }).collect()
}

/// 渲染张力为 prompt 文本
pub fn render_tensions_for_prompt(tensions: &[InterpersonalTension]) -> String {
    if tensions.is_empty() { return String::new(); }
    let mut lines = vec!["【情感张力驱动（以下是角色间未释放的情感压力，本节须让至少一条张力推进或释放）】".to_string()];
    for t in tensions {
        lines.push(format!(
            "■ {} -> {}：{}（压力 {:.1}，积压 {} 章）-> {}",
            t.source_name, t.target_name, t.tension_type,
            t.pressure, t.accumulated_chapters, t.suggested_action,
        ));
    }
    lines.join("\n") + "\n"
}

/// 渲染单个弧光
pub fn render_arc_for_prompt(arc: &EmotionalArc) -> String {
    let stage_text = match arc.stage {
        ArcStage::Brewing => "酝酿期",
        ArcStage::Escalating => "升级期",
        ArcStage::Climax => "高潮期",
        ArcStage::Transforming => "转变期",
        ArcStage::Resolving => "收束期",
    };
    format!(
        "【{} 的情感弧光（{}）】起点：{} -> 当前：{} -> 终点：{}\n催化剂：{}\n",
        arc.character_name, stage_text,
        arc.start_emotion, arc.current_emotion, arc.end_emotion,
        arc.catalyst,
    )
}

/// 渲染所有弧光
pub fn render_arcs_for_prompt(arcs: &[EmotionalArc]) -> String {
    if arcs.is_empty() { return String::new(); }
    arcs.iter().map(render_arc_for_prompt).collect::<Vec<_>>().join("")
}
```

- [ ] **Step 3: GREEN — 模块注册**（`agency/mod.rs`）：`pub mod emotional_ledger;`

- [ ] **Step 4: GREEN — 注入 writer 上下文**（`build_writer_context_from_db`，情感关系段之后）

```rust
let tensions = crate::agency::emotional_ledger::load_tensions(&pool, &sid);
let tension_text = crate::agency::emotional_ledger::render_tensions_for_prompt(&tensions);
if !tension_text.is_empty() {
    ctx.push_str(&tension_text);
}

let arcs = crate::agency::emotional_ledger::load_arcs(&pool, &sid);
let arc_text = crate::agency::emotional_ledger::render_arcs_for_prompt(&arcs);
if !arc_text.is_empty() {
    ctx.push_str(&arc_text);
}
```

- [ ] **Step 5: GREEN — TriShot 路径同步注入**（`agents/orchestrator.rs` `build_progression_anchor`）：同样调用 load_tensions + load_arcs 渲染为锚点段。

- [ ] **Step 6: REFACTOR**

- 关键词匹配可提取为常量表
- ArcStage 阶段推进逻辑后续可从 character_states 推导（本期固定 Brewing）

- [ ] **Step 7: 验证（全量回归清单）**

```bash
cd src-tauri && cargo test --lib -- emotional_ledger 2>&1 | tail -10
cd src-tauri && cargo test --lib 2>&1 | tail -3          # ~1253 + ~7 = ~1260 passed / 2 ignored
cd src-tauri && cargo +nightly fmt -- --check
cd src-frontend && npx tsc --noEmit
cd src-frontend && npx vitest run 2>&1 | grep -E "passed|failed"
python3 scripts/architecture_guard.py
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/agency/ src-tauri/src/agents/orchestrator.rs
git commit -m "新增(创作): 情感张力账本——人际张力种子+情感弧光注入 writer（emotional_ledger）"
```

---

## 文件清单汇总

| # | 文件 | 改动类型 | Task |
|---|---|---|---|
| 1 | `src-tauri/src/db/migrations/V123__characters_emotional_attrs.sql` | 新建 | 1 |
| 2 | `src-tauri/src/db/migrations/V124__relationship_emotional_bond.sql` | 新建 | 1 |
| 3 | `src-tauri/src/db/models.rs` | Character/CharacterRelationship +4 字段 + EmotionalBond 枚举 | 1 |
| 4 | `src-tauri/src/db/dto.rs` | CreateCharacterRequest +4 字段 | 1 |
| 5 | `src-tauri/src/db/repositories/character_repository.rs` | 全链路读写 + update_emotional | 1 |
| 6 | `src-tauri/src/db/repositories/character_relationship_repository.rs` | 全链路读写 + update_emotional | 1 |
| 7 | `src-tauri/src/agency/coordinator.rs` | Seed/ConceptPack/prompt/黑板/writer 注入 | 2,4,6 |
| 8 | `src-tauri/src/agency/materialize.rs` | character +4 字段 + relationship 分支 | 2 |
| 9 | `resources/prompts/agency/agency_producer_system.md` | 8 字段 + relationship 格式 | 2 |
| 10 | `src-tauri/src/domain/novel_creation.rs` | CharacterProfileOption +4 字段 | 3 |
| 11 | `resources/prompts/creation/novel_creation_character_roster.md` | 要求 8 字段 | 3 |
| 12 | `src-tauri/src/agents/novel_creation.rs` | fallback prompt + 落库 | 3 |
| 13 | `src-frontend/src/services/api/genesis.ts` | 参数名修复 + 情感字段 | 5 |
| 14 | `src-frontend/src/types/index.ts` | CharacterRelationship +4 字段 | 5 |
| 15 | `src-frontend/src/components/CharacterRelationshipForm.tsx` | 情感表单 | 5 |
| 16 | `src-frontend/src/pages/Characters.tsx` | RelationshipCard 情感展示 | 5 |
| 17 | `src-frontend/src/hooks/useCharacterRelationships.ts` | hook 参数扩展 | 5 |
| 18 | `src-tauri/src/revision_commands.rs` | create/update +4 参数 | 5 |
| 19 | `src-tauri/src/agency/emotional_ledger.rs` | 新建 | 6 |
| 20 | `src-tauri/src/agency/mod.rs` | 注册模块 | 6 |
| 21 | `src-tauri/src/agents/orchestrator.rs` | build_progression_anchor 注入 | 6 |
