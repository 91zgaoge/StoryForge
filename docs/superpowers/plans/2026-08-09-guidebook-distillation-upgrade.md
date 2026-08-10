# 技能书提炼优化 + 去 Pro 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 借鉴 book-to-skill 将技能书提炼升级为分层结构化提炼（技巧模式库+决策速查表注入续写），修复失败不可重试的恢复缺口，并去除技能书功能的 Pro 限制。

**Architecture:** 分块提炼 prompt 输出四类结构化资产（要点/技巧/决策规则/反模式）→ 合并分类聚合 → 落库 `custom_methodologies` 新增两列（V125）→ 续写注入在当前步骤之外按步骤轮转注入技巧与速查。重试复用已存文件重建任务。去 Pro 仅改订阅白名单与前端面板。

**Tech Stack:** Rust（Tauri 2, rusqlite, serde_json）、React + TypeScript（vitest, @testing-library/react）、react-query。

**设计文档：** `docs/plans/2026-08-09-guidebook-distillation-upgrade-design.md`（已批准）

## Global Constraints

- 测试基线：`cd src-tauri && cargo test --lib` = 1260 passed / 2 ignored；`cd src-frontend && npx vitest run` = 399 passed / 3 skipped。每个任务完成后回归不得低于基线（只能增加）。
- `.sql` 迁移文件放 `src-tauri/src/db/migrations/` 即被自动扫描执行，**无需在 mod.rs 注册**；Rust 迁移才需要注册。下一个版本号 V125。
- 测试 DB 用 `crate::db::connection::create_test_pool()`（内存库跑全量迁移）。
- 中文 conventional commit；不绕过 pre-commit 钩子；**不 push、不打 tag**。
- `.recovery/` 目录勿动勿提交。
- 不改动其它 Pro 守卫：`book_decomposition/upload_book`、Pipeline 三命令、`agents/service.rs:1960`（内置方法论）、`:2104`（StyleBlend）、`:2205`（personalizer）。
- prompt 文件带 frontmatter（`id/name/description/category/version/variables`），`version` 字段随内容变更递增。

---

### Task 1: V125 迁移 + 资产模型 + Repository 新列

**Files:**
- Create: `src-tauri/src/db/migrations/V125__custom_methodology_assets.sql`
- Modify: `src-tauri/src/guidebook_distillation/models.rs`（:86-118 自定义方法论区）
- Modify: `src-tauri/src/guidebook_distillation/repository.rs`（:189-227 create/row_to_cm，:253-288 update）
- Test: 同上两文件的 `#[cfg(test)]` 模块

**Interfaces:**
- Consumes: 现有 `CustomMethodology`、`parse_steps`、`CustomMethodologyRepository`
- Produces（后续任务依赖）:
  - `pub struct Technique { pub name: String, pub when_to_use: String, pub how: String }`
  - `pub struct AntiPattern { pub what: String, pub why: String }`
  - `pub struct Cheatsheet { pub decision_rules: Vec<String>, pub anti_patterns: Vec<AntiPattern> }`（实现 `Default`）
  - `pub fn parse_patterns(json: &str) -> Vec<Technique>`
  - `pub fn parse_cheatsheet(json: &str) -> Cheatsheet`
  - `CustomMethodology` 新增字段 `pub patterns: Vec<Technique>`、`pub cheatsheet: Cheatsheet`
  - `CustomMethodologyRepository::update(&self, id, name, description, steps, enabled, patterns: Option<&[Technique]>, cheatsheet: Option<&Cheatsheet>)`

- [ ] **Step 1: 写失败测试（models）**

在 `models.rs` tests 模块追加：

```rust
#[test]
fn parse_patterns_handles_valid_and_invalid() {
    let json = r#"[{"name":"雪花写作法","when_to_use":"搭建大纲时","how":"从一句话扩展到段落"}]"#;
    let p = parse_patterns(json);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].name, "雪花写作法");
    assert_eq!(p[0].when_to_use, "搭建大纲时");
    // 缺省字段容错
    let minimal = parse_patterns(r#"[{"name":"x"}]"#);
    assert_eq!(minimal[0].when_to_use, "");
    // 坏 JSON → 空
    assert!(parse_patterns("not json").is_empty());
}

#[test]
fn parse_cheatsheet_handles_valid_and_invalid() {
    let json = r#"{"decision_rules":["当冲突弱化时加码，因为张力是引擎"],"anti_patterns":[{"what":"流水账","why":"没有冲突驱动"}]}"#;
    let cs = parse_cheatsheet(json);
    assert_eq!(cs.decision_rules.len(), 1);
    assert_eq!(cs.anti_patterns[0].what, "流水账");
    // 坏 JSON → 默认空
    let empty = parse_cheatsheet("not json");
    assert!(empty.decision_rules.is_empty());
    assert!(empty.anti_patterns.is_empty());
}
```

- [ ] **Step 2: 写失败测试（repository）**

在 `repository.rs` tests 模块的 `custom_methodology_crud_flow` 之后追加：

```rust
#[test]
fn custom_methodology_patterns_and_cheatsheet_roundtrip() {
    let pool = create_test_pool().unwrap();
    let repo = CustomMethodologyRepository::new(pool);
    let cm = CustomMethodology {
        id: "custom_p1".into(),
        guidebook_id: None,
        name: "资产测试".into(),
        description: None,
        steps: vec![MethodologyStep {
            title: "s".into(),
            instruction: "i".into(),
            checklist: vec![],
        }],
        patterns: vec![Technique {
            name: "三幕结构".into(),
            when_to_use: "布局全书".into(),
            how: "建置-对抗-解决".into(),
        }],
        cheatsheet: Cheatsheet {
            decision_rules: vec!["当节奏拖沓时删场景，因为每场景须推进冲突".into()],
            anti_patterns: vec![AntiPattern {
                what: "信息倾倒".into(),
                why: "读者失去探索欲".into(),
            }],
        },
        enabled: true,
        created_at: Local::now(),
        updated_at: Local::now(),
    };
    repo.create(&cm).unwrap();
    let got = repo.get_by_id("custom_p1").unwrap().unwrap();
    assert_eq!(got.patterns.len(), 1);
    assert_eq!(got.patterns[0].name, "三幕结构");
    assert_eq!(got.cheatsheet.decision_rules.len(), 1);
    assert_eq!(got.cheatsheet.anti_patterns[0].why, "读者失去探索欲");
    // update 新字段
    repo.update(
        "custom_p1",
        None,
        None,
        None,
        None,
        Some(&[Technique {
            name: "新技巧".into(),
            when_to_use: String::new(),
            how: String::new(),
        }]),
        None,
    )
    .unwrap();
    let got = repo.get_by_id("custom_p1").unwrap().unwrap();
    assert_eq!(got.patterns[0].name, "新技巧");
    assert_eq!(got.cheatsheet.decision_rules.len(), 1); // 未传则不动
}
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -5`
Expected: 编译失败——`parse_patterns`/`parse_cheatsheet`/`Technique`/`Cheatsheet` 未定义，`CustomMethodology` 无 `patterns` 字段，`update` 参数数不匹配。

- [ ] **Step 4: 创建 V125 迁移**

Create `src-tauri/src/db/migrations/V125__custom_methodology_assets.sql`：

```sql
ALTER TABLE custom_methodologies ADD COLUMN patterns_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE custom_methodologies ADD COLUMN cheatsheet_json TEXT NOT NULL DEFAULT '{}';
```

- [ ] **Step 5: models.rs 实现资产类型**

在 `MethodologyStep` 定义之后、`CustomMethodology` 之前插入：

```rust
/// 技巧模式库条目（提炼自指导书的具名技巧）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Technique {
    pub name: String,
    #[serde(default)]
    pub when_to_use: String,
    #[serde(default)]
    pub how: String,
}

/// 反模式（避免什么 + 为什么）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AntiPattern {
    pub what: String,
    #[serde(default)]
    pub why: String,
}

/// 决策速查表（决策规则 + 反模式）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Cheatsheet {
    #[serde(default)]
    pub decision_rules: Vec<String>,
    #[serde(default)]
    pub anti_patterns: Vec<AntiPattern>,
}
```

`CustomMethodology` 结构体追加两字段：

```rust
    #[serde(default)]
    pub patterns: Vec<Technique>,
    #[serde(default)]
    pub cheatsheet: Cheatsheet,
```

`parse_steps` 之后追加：

```rust
/// 解析 patterns_json；坏数据返回空 vec
pub fn parse_patterns(json: &str) -> Vec<Technique> {
    serde_json::from_str(json).unwrap_or_default()
}

/// 解析 cheatsheet_json；坏数据返回默认空速查表
pub fn parse_cheatsheet(json: &str) -> Cheatsheet {
    serde_json::from_str(json).unwrap_or_default()
}
```

同时修复 models.rs 既有测试 `max_steps_at_least_one`：构造 `CustomMethodology` 处追加 `patterns: vec![], cheatsheet: Cheatsheet::default(),`。

- [ ] **Step 6: repository.rs 读写新列**

`create`（:189-206）INSERT 改为：

```rust
conn.execute(
    "INSERT INTO custom_methodologies (id, guidebook_id, name, description, steps_json, \
     patterns_json, cheatsheet_json, enabled, created_at, updated_at) \
     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
    params![
        cm.id,
        cm.guidebook_id,
        cm.name,
        cm.description,
        serde_json::to_string(&cm.steps)?,
        serde_json::to_string(&cm.patterns)?,
        serde_json::to_string(&cm.cheatsheet)?,
        cm.enabled as i32,
        cm.created_at.to_rfc3339(),
        cm.updated_at.to_rfc3339(),
    ],
)?;
```

`row_to_cm`（:208-227）：取列处追加

```rust
let patterns_json: String = row.get("patterns_json")?;
let cheatsheet_json: String = row.get("cheatsheet_json")?;
```

结构体构造中 `steps: parse_steps(&steps_json),` 之后追加：

```rust
            patterns: parse_patterns(&patterns_json),
            cheatsheet: parse_cheatsheet(&cheatsheet_json),
```

`update`（:253-288）签名与实现改为：

```rust
/// 更新名称/描述/步骤/启用状态/技巧模式库/决策速查（None 字段不动）
pub fn update(
    &self,
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
    steps: Option<&[MethodologyStep]>,
    enabled: Option<bool>,
    patterns: Option<&[Technique]>,
    cheatsheet: Option<&Cheatsheet>,
) -> RepoResult<()> {
    let conn = self.pool.get()?;
    // ... 既有 name/description/steps/enabled 四个 if let 保持不变 ...
    if let Some(p) = patterns {
        conn.execute(
            "UPDATE custom_methodologies SET patterns_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![serde_json::to_string(p)?, Local::now().to_rfc3339(), id],
        )?;
    }
    if let Some(c) = cheatsheet {
        conn.execute(
            "UPDATE custom_methodologies SET cheatsheet_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![serde_json::to_string(c)?, Local::now().to_rfc3339(), id],
        )?;
    }
    Ok(())
}
```

既有测试 `custom_methodology_crud_flow` 中三处 `repo.update(...)` 调用各追加 `None, None` 参数；`CustomMethodology` 构造处追加 `patterns: vec![], cheatsheet: Cheatsheet::default(),`。

- [ ] **Step 7: 运行测试确认通过 + 全量回归**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -5`
Expected: 全部通过（含新增 3 个测试）

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3`
Expected: ≥1263 passed / 2 ignored（其它模块中构造 `CustomMethodology` 的地方若有编译错误一并补齐 `patterns`/`cheatsheet` 字段；已知 `service.rs` 的 `seed_cm` 和 `run_distillation` 各有一处，前者在本任务顺手补 `patterns: vec![], cheatsheet: Cheatsheet::default(),`，后者在 Task 3 改）

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/db/migrations/V125__custom_methodology_assets.sql \
        src-tauri/src/guidebook_distillation/models.rs \
        src-tauri/src/guidebook_distillation/repository.rs \
        src-tauri/src/guidebook_distillation/service.rs
git commit -m "feat: 自定义方法论新增技巧模式库与决策速查表存储（V125）"
```

---

### Task 2: Distiller 结构化提炼（prompt + LLM 响应类型 + 聚合）

**Files:**
- Modify: `resources/prompts/distillation/distill_chunk.md`（全文替换）
- Modify: `resources/prompts/distillation/distill_merge.md`（全文替换）
- Modify: `src-tauri/src/guidebook_distillation/models.rs`（:142-172 LLM 响应类型区）
- Modify: `src-tauri/src/guidebook_distillation/distiller.rs`（:99-142 distill 编排、:175-301 distill_chunks/merge_points）
- Test: `src-tauri/src/guidebook_distillation/models.rs`、`distiller.rs` 的 tests 模块

**Interfaces:**
- Consumes: Task 1 的 `Technique`/`AntiPattern`/`Cheatsheet`
- Produces（Task 3 依赖）:
  - `LlmDistillChunkResponse { points, techniques, decision_rules, anti_patterns }`（全部 serde default，`points` 带 `alias = "key_points"`）
  - `LlmDistillMergeResponse { principles, techniques, decision_rules, anti_patterns }`（全部 serde default）
  - `pub struct ChunkAssets { pub points: Vec<String>, pub techniques: Vec<Technique>, pub decision_rules: Vec<String>, pub anti_patterns: Vec<AntiPattern> }`（`Default`）
  - `DistillationOutput { metadata, methodology, techniques: Vec<Technique>, cheatsheet: Cheatsheet }`

- [ ] **Step 1: 写失败测试（models LLM 响应类型）**

在 `models.rs` tests 模块追加：

```rust
#[test]
fn chunk_response_deserializes_structured_assets() {
    let json = r#"{"key_points":["要点一"],
      "techniques":[{"name":"雪花写作法","when_to_use":"搭大纲","how":"逐步扩展"}],
      "decision_rules":["当冲突弱时加码，因为张力是引擎"],
      "anti_patterns":[{"what":"流水账","why":"无冲突驱动"}]}"#;
    let r: LlmDistillChunkResponse = serde_json::from_str(json).unwrap();
    assert_eq!(r.points, vec!["要点一"]);
    assert_eq!(r.techniques[0].name, "雪花写作法");
    assert_eq!(r.decision_rules.len(), 1);
    assert_eq!(r.anti_patterns[0].why, "无冲突驱动");
}

#[test]
fn chunk_response_backward_compat_old_points_format() {
    // 旧格式（只有 points）不崩溃，新字段为空
    let r: LlmDistillChunkResponse =
        serde_json::from_str(r#"{"points":["要点"]}"#).unwrap();
    assert_eq!(r.points, vec!["要点"]);
    assert!(r.techniques.is_empty());
    assert!(r.decision_rules.is_empty());
    assert!(r.anti_patterns.is_empty());
}

#[test]
fn merge_response_deserializes_classified_assets() {
    let json = r#"{"principles":["原则一"],
      "techniques":[{"name":"t","when_to_use":"w","how":"h"}],
      "decision_rules":["r"],
      "anti_patterns":[{"what":"x","why":"y"}]}"#;
    let r: LlmDistillMergeResponse = serde_json::from_str(json).unwrap();
    assert_eq!(r.principles, vec!["原则一"]);
    assert_eq!(r.techniques.len(), 1);
    assert_eq!(r.anti_patterns[0].what, "x");
    // 旧格式兼容
    let old: LlmDistillMergeResponse =
        serde_json::from_str(r#"{"principles":["p"]}"#).unwrap();
    assert!(old.techniques.is_empty());
}
```

- [ ] **Step 2: 写失败测试（distiller 聚合与截断）**

在 `distiller.rs` tests 模块追加：

```rust
#[test]
fn chunk_assets_aggregate_extends_all_categories() {
    let mut a = ChunkAssets::default();
    a.extend(LlmDistillChunkResponse {
        points: vec!["p1".into()],
        techniques: vec![Technique {
            name: "t1".into(),
            when_to_use: String::new(),
            how: String::new(),
        }],
        decision_rules: vec!["r1".into()],
        anti_patterns: vec![],
    });
    a.extend(LlmDistillChunkResponse {
        points: vec!["p2".into()],
        techniques: vec![],
        decision_rules: vec![],
        anti_patterns: vec![AntiPattern {
            what: "w".into(),
            why: String::new(),
        }],
    });
    assert_eq!(a.points, vec!["p1", "p2"]);
    assert_eq!(a.techniques.len(), 1);
    assert_eq!(a.decision_rules.len(), 1);
    assert_eq!(a.anti_patterns.len(), 1);
    assert!(!a.is_empty());
    assert!(ChunkAssets::default().is_empty());
}

#[test]
fn merge_input_contains_four_sections_and_truncates() {
    let mut a = ChunkAssets::default();
    a.points.push("要点".repeat(300)); // 超长条 → 截断
    a.techniques.push(Technique {
        name: "雪花写作法".into(),
        when_to_use: "搭大纲".into(),
        how: "逐步扩展".into(),
    });
    a.decision_rules.push("当X时做Y，因为Z".into());
    a.anti_patterns.push(AntiPattern {
        what: "流水账".into(),
        why: "无冲突".into(),
    });
    let input = build_merge_input(&a);
    assert!(input.contains("【要点】"));
    assert!(input.contains("【技巧】"));
    assert!(input.contains("雪花写作法"));
    assert!(input.contains("【决策规则】"));
    assert!(input.contains("【反模式】"));
    assert!(input.contains("流水账"));
    // 单条 200 字截断：拼接行不含完整 300 字重复
    assert!(!input.contains(&"要点".repeat(300)));
}
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -5`
Expected: 编译失败——`LlmDistillChunkResponse` 无新字段、`ChunkAssets`/`build_merge_input` 不存在。

- [ ] **Step 4: models.rs 扩展 LLM 响应类型**

`LlmDistillChunkResponse`（:142-145）替换为：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDistillChunkResponse {
    #[serde(default, alias = "key_points")]
    pub points: Vec<String>,
    #[serde(default)]
    pub techniques: Vec<Technique>,
    #[serde(default)]
    pub decision_rules: Vec<String>,
    #[serde(default)]
    pub anti_patterns: Vec<AntiPattern>,
}
```

`LlmDistillMergeResponse`（:147-150）替换为：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDistillMergeResponse {
    #[serde(default)]
    pub principles: Vec<String>,
    #[serde(default)]
    pub techniques: Vec<Technique>,
    #[serde(default)]
    pub decision_rules: Vec<String>,
    #[serde(default)]
    pub anti_patterns: Vec<AntiPattern>,
}
```

`DistillationOutput`（:167-172）替换为：

```rust
/// 提炼流水线的最终产出
#[derive(Debug, Clone)]
pub struct DistillationOutput {
    pub metadata: LlmGuidebookMetadataResponse,
    pub methodology: LlmMethodologyResponse,
    pub techniques: Vec<Technique>,
    pub cheatsheet: Cheatsheet,
}
```

- [ ] **Step 5: distiller.rs 聚合结构与 merge 输入构建**

`use super::models::*;` 已覆盖新类型。在 `validate_methodology` 之前插入：

```rust
/// 分块结构化资产的聚合容器
#[derive(Debug, Default)]
pub struct ChunkAssets {
    pub points: Vec<String>,
    pub techniques: Vec<Technique>,
    pub decision_rules: Vec<String>,
    pub anti_patterns: Vec<AntiPattern>,
}

impl ChunkAssets {
    pub fn extend(&mut self, r: LlmDistillChunkResponse) {
        self.points.extend(r.points);
        self.techniques.extend(r.techniques);
        self.decision_rules.extend(r.decision_rules);
        self.anti_patterns.extend(r.anti_patterns);
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
            && self.techniques.is_empty()
            && self.decision_rules.is_empty()
            && self.anti_patterns.is_empty()
    }
}

/// 截断到 max 字（chars）
fn clip_chars(s: &str, max: usize) -> String {
    let t = s.trim();
    if t.chars().count() > max {
        t.chars().take(max).collect()
    } else {
        t.to_string()
    }
}

/// 构建 merge 输入：四类资产分区，单条 200 字、总量 12000 字截断
fn build_merge_input(assets: &ChunkAssets) -> String {
    let mut sections = Vec::new();
    if !assets.points.is_empty() {
        let lines = assets
            .points
            .iter()
            .map(|p| clip_chars(p, 200))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("【要点】\n{}", lines));
    }
    if !assets.techniques.is_empty() {
        let lines = assets
            .techniques
            .iter()
            .map(|t| {
                clip_chars(
                    &format!("{}｜何时用：{}｜怎么做：{}", t.name, t.when_to_use, t.how),
                    200,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("【技巧】\n{}", lines));
    }
    if !assets.decision_rules.is_empty() {
        let lines = assets
            .decision_rules
            .iter()
            .map(|r| clip_chars(r, 200))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("【决策规则】\n{}", lines));
    }
    if !assets.anti_patterns.is_empty() {
        let lines = assets
            .anti_patterns
            .iter()
            .map(|a| clip_chars(&format!("{}｜{}", a.what, a.why), 200))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("【反模式】\n{}", lines));
    }
    clip_chars(&sections.join("\n\n"), 12000)
}
```

- [ ] **Step 6: distiller.rs 编排改造**

`distill`（:108-141）：Step 2/3/4 段替换为：

```rust
        // Step 2: 分块提炼（10→70%，并发）
        let total = chunks.len();
        self.emit_progress(
            guidebook_id,
            "distilling",
            12,
            &format!("正在分块提炼创作资产（共 {} 块）...", total),
        )
        .await;
        let assets = self
            .distill_chunks(guidebook_id, chunks, &cancel_check)
            .await?;
        heartbeat();
        check_cancel()?;

        // Step 3: 合并去重（→85%）
        self.emit_progress(guidebook_id, "merging", 72, "正在分类合并创作资产...")
            .await;
        let merged = self.merge_assets(&assets).await?;
        heartbeat();
        check_cancel()?;

        // Step 4: 结构化方法论（→100%），JSON 失败重试一次
        self.emit_progress(guidebook_id, "merging", 88, "正在生成创作方法论...")
            .await;
        let methodology = match self
            .generate_methodology(&merged.principles, &book_title)
            .await
        {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "[GuidebookDistiller] methodology 首次生成失败，重试一次: {}",
                    e
                );
                self.generate_methodology(&merged.principles, &book_title)
                    .await?
            }
        };
        self.emit_progress(guidebook_id, "merging", 100, "提炼完成")
            .await;
        heartbeat();

        Ok(DistillationOutput {
            metadata,
            methodology,
            techniques: merged.techniques,
            cheatsheet: Cheatsheet {
                decision_rules: merged.decision_rules,
                anti_patterns: merged.anti_patterns,
            },
        })
```

`distill_chunks`（:175-259）：返回类型 `Result<Vec<String>, AnalysisError>` 改为 `Result<ChunkAssets, AnalysisError>`。task 闭包内：

- `call_llm(&llm, "guidebook_chunk", prompt, Some(2000), Some(0.3))` 的 max_tokens 改为 `Some(3000)`；
- 解析行改为 `let parsed: LlmDistillChunkResponse = parse_json_response(&resp)?;`，返回 `Ok::<LlmDistillChunkResponse, AnalysisError>(parsed)`；
- 聚合段（:245-258）改为：

```rust
        let mut all = ChunkAssets::default();
        while let Some(res) = set.join_next().await {
            match res {
                Ok(Ok(chunk_assets)) => all.extend(chunk_assets),
                Ok(Err(e)) => {
                    // 单块失败不致命：记录并继续
                    log::warn!("[GuidebookDistiller] 单块提炼失败，跳过: {}", e);
                }
                Err(e) => {
                    return Err(AnalysisError::LlmError(format!("Join error: {}", e)));
                }
            }
        }
        Ok(all)
```

`merge_points`（:261-301）整体替换为：

```rust
    async fn merge_assets(
        &self,
        assets: &ChunkAssets,
    ) -> Result<LlmDistillMergeResponse, AnalysisError> {
        if assets.is_empty() {
            return Err(AnalysisError::LlmError(
                "全书未提炼出任何创作要点".to_string(),
            ));
        }
        let joined = build_merge_input(assets);
        let prompt = self
            .render_prompt("distill_merge", &[("points", joined)])
            .ok_or_else(|| AnalysisError::LlmError("prompt distill_merge 未注册".into()))?;
        let resp = call_llm(
            &self.llm_service,
            "guidebook_merge",
            prompt,
            Some(4000),
            Some(0.3),
        )
        .await?;
        let parsed: LlmDistillMergeResponse = parse_json_response(&resp)?;
        if parsed.principles.is_empty() {
            return Err(AnalysisError::LlmError("合并后原则为空".to_string()));
        }
        Ok(parsed)
    }
```

- [ ] **Step 7: 替换两个 prompt 文件**

`resources/prompts/distillation/distill_chunk.md` 全文替换（version 0.33.2 → 0.36.0）：

```markdown
---
id: distill_chunk
name: "指导书分块资产提炼"
description: "从指导书文本片段中提炼结构化创作资产：要点、具名技巧、决策规则、反模式"
category: distillation
version: 0.36.0
variables:
  - text
---

你是一位小说创作方法论专家。以下是一本故事创作指导书的片段，请提炼其中所有**可操作**的创作资产。

要求：
1. 提取结构而非摘要：具名技巧、决策规则、反模式，而不是内容概括
2. 保留作者对技巧/框架的原始命名（如"雪花写作法"不得改写为"逐步扩展法"）
3. 用实践者口吻：写"当X时用Y"，不写"作者介绍了Y"
4. 忽略序言、致谢、出版信息等无实质内容的部分
5. 某一类没有可提炼内容时，该类返回空数组
6. 只输出 JSON，不要有任何其他文字

文本片段：
{{text}}

JSON格式：
{"key_points":["一句话可执行要点"],"techniques":[{"name":"技巧名（保留作者原始命名）","when_to_use":"何时使用","how":"具体怎么做"}],"decision_rules":["当X时做Y，因为Z"],"anti_patterns":[{"what":"应避免的做法","why":"为什么会导致失败"}]}
```

`resources/prompts/distillation/distill_merge.md` 全文替换（version 0.33.2 → 0.36.0）：

```markdown
---
id: distill_merge
name: "指导书资产分类合并"
description: "合并全书提炼的四类创作资产，分类去重并保留最具操作性的条目"
category: distillation
version: 0.36.0
variables:
  - points
---

你是一位小说创作方法论专家。以下是从一本故事创作指导书各章节提炼出的创作资产（分【要点】【技巧】【决策规则】【反模式】四类），请分类合并去重。

要求：
1. 语义相同的条目合并为一条，保留最准确的表述与作者原始命名
2. principles：按主题归类排序（冲突设计、人物塑造、结构节奏、世界观、对白等），保留最重要的 10-20 条，每条一句话
3. techniques：保留最实用、最具操作性的 5-15 条，每条必须含 name/when_to_use/how 三个字段
4. decision_rules：保留 5-10 条，保持"当X时做Y，因为Z"格式
5. anti_patterns：保留 3-8 条，每条含 what/why 两个字段
6. 只输出 JSON，不要有任何其他文字

原始资产列表：
{{points}}

JSON格式：
{"principles":["原则1","原则2"],"techniques":[{"name":"技巧名","when_to_use":"何时使用","how":"具体怎么做"}],"decision_rules":["当X时做Y，因为Z"],"anti_patterns":[{"what":"应避免的做法","why":"为什么会导致失败"}]}
```

- [ ] **Step 8: 运行测试确认通过 + 全量回归**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -5`
Expected: 全部通过（含新增 5 个测试）

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3`
Expected: ≥1268 passed / 2 ignored（`service.rs:138` 处 `DistillationOutput` 构造引用若因字段变化报错属 Task 3 范围——本任务先确保编译通过：若 `service.rs` 的 `output.methodology` 之外引用了旧字段才需处理，实际只引用 `metadata`/`methodology`，无需改）

- [ ] **Step 9: Commit**

```bash
git add resources/prompts/distillation/distill_chunk.md \
        resources/prompts/distillation/distill_merge.md \
        src-tauri/src/guidebook_distillation/models.rs \
        src-tauri/src/guidebook_distillation/distiller.rs
git commit -m "feat: 技能书提炼升级为四类结构化资产（借鉴 book-to-skill 提取结构而非摘要）"
```

---

### Task 3: 落库新资产 + 续写注入增强

**Files:**
- Modify: `src-tauri/src/guidebook_distillation/service.rs`（:188-235 run_distillation 落库段、:346-382 render_custom_methodology_extension）
- Test: `src-tauri/src/guidebook_distillation/service.rs` 的 `extension_tests` 模块

**Interfaces:**
- Consumes: Task 2 的 `DistillationOutput.techniques`/`DistillationOutput.cheatsheet`；Task 1 的 `CustomMethodology.patterns`/`cheatsheet`
- Produces: `render_custom_methodology_extension(pool, methodology_id, step) -> Option<String>` 签名不变；输出文本在原方法论段后追加 `【技巧参考】`（有 patterns 时）与 `【决策速查】`（有速查内容时）两段。`write_time_bundle.rs:214` 调用方零改动。

- [ ] **Step 1: 写失败测试**

`service.rs` 的 `seed_cm`（:389-413）构造处追加 `patterns: vec![], cheatsheet: Cheatsheet::default(),`（Task 1 已做则跳过）。在 `extension_tests` 模块追加：

```rust
    fn seed_cm_with_assets(pool: &DbPool) {
        CustomMethodologyRepository::new(pool.clone())
            .create(&CustomMethodology {
                id: "custom_a1".into(),
                guidebook_id: None,
                name: "冲突驱动法".into(),
                description: None,
                steps: vec![
                    MethodologyStep {
                        title: "立冲突".into(),
                        instruction: "确立核心冲突".into(),
                        checklist: vec![],
                    },
                    MethodologyStep {
                        title: "升级".into(),
                        instruction: "升级冲突".into(),
                        checklist: vec![],
                    },
                ],
                patterns: vec![
                    Technique {
                        name: "场景目标法".into(),
                        when_to_use: "每场开场".into(),
                        how: "给 POV 角色一个当场可达成的具体目标".into(),
                    },
                    Technique {
                        name: "灾难收尾".into(),
                        when_to_use: "场景结尾".into(),
                        how: "让目标受挫并引出新难题".into(),
                    },
                    Technique {
                        name: "情感节拍".into(),
                        when_to_use: "反应段".into(),
                        how: "先情感后理性再行动".into(),
                    },
                    Technique {
                        name: "第四技巧".into(),
                        when_to_use: "w4".into(),
                        how: "h4".into(),
                    },
                ],
                cheatsheet: Cheatsheet {
                    decision_rules: vec![
                        "当节奏拖沓时删场景，因为每场景必须推进冲突".into(),
                        "当人物扁平时加矛盾欲望，因为冲突来自内心".into(),
                    ],
                    anti_patterns: vec![AntiPattern {
                        what: "信息倾倒".into(),
                        why: "读者失去探索欲".into(),
                    }],
                },
                enabled: true,
                created_at: chrono::Local::now(),
                updated_at: chrono::Local::now(),
            })
            .unwrap();
    }

    #[test]
    fn render_extension_includes_techniques_and_cheatsheet() {
        let pool = create_test_pool().unwrap();
        seed_cm_with_assets(&pool);
        let text = render_custom_methodology_extension(&pool, "custom_a1", 1).unwrap();
        assert!(text.contains("【技巧参考】"));
        assert!(text.contains("场景目标法"));
        assert!(text.contains("给 POV 角色一个当场可达成的具体目标"));
        assert!(text.contains("【决策速查】"));
        assert!(text.contains("当节奏拖沓时删场景"));
        assert!(text.contains("避免：信息倾倒"));
    }

    #[test]
    fn render_extension_rotates_techniques_by_step() {
        let pool = create_test_pool().unwrap();
        seed_cm_with_assets(&pool);
        // step 1（idx 0）：从 patterns[0] 起取 3 条
        let s1 = render_custom_methodology_extension(&pool, "custom_a1", 1).unwrap();
        assert!(s1.contains("场景目标法"));
        assert!(s1.contains("灾难收尾"));
        assert!(s1.contains("情感节拍"));
        assert!(!s1.contains("第四技巧"));
        // step 2（idx 1）：轮转从 patterns[1] 起
        let s2 = render_custom_methodology_extension(&pool, "custom_a1", 2).unwrap();
        assert!(s2.contains("灾难收尾"));
        assert!(s2.contains("情感节拍"));
        assert!(s2.contains("第四技巧"));
        assert!(!s2.contains("场景目标法"));
    }

    #[test]
    fn render_extension_omits_sections_when_no_assets() {
        let pool = create_test_pool().unwrap();
        seed_cm(&pool, true); // 无 patterns/cheatsheet 的旧数据
        let text = render_custom_methodology_extension(&pool, "custom_t1", 1).unwrap();
        assert!(!text.contains("【技巧参考】"));
        assert!(!text.contains("【决策速查】"));
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation::service 2>&1 | tail -5`
Expected: FAIL——`render_extension_includes_techniques_and_cheatsheet` 断言失败（输出尚无两段）。

- [ ] **Step 3: run_distillation 落库新资产**

`service.rs` :195-213 的 `cm` 构造中 `steps: ...collect(),` 之后追加：

```rust
            patterns: output.techniques.clone(),
            cheatsheet: output.cheatsheet.clone(),
```

- [ ] **Step 4: render_custom_methodology_extension 注入增强**

`service.rs` :348-382 整函数替换为：

```rust
/// 续写注入点用：渲染自定义方法论当前步骤的约束文本 + 技巧参考 + 决策速查。
/// 未知 id / 已禁用 / 无步骤 → None（调用方静默跳过注入）。
pub fn render_custom_methodology_extension(
    pool: &DbPool,
    methodology_id: &str,
    step: i32,
) -> Option<String> {
    let cm = CustomMethodologyRepository::new(pool.clone())
        .get_by_id(methodology_id)
        .ok()
        .flatten()?;
    if !cm.enabled || cm.steps.is_empty() {
        return None;
    }
    let idx = ((step.max(1) as usize) - 1).min(cm.steps.len() - 1);
    let s = &cm.steps[idx];
    let checklist = if s.checklist.is_empty() {
        String::new()
    } else {
        format!(
            "\n检查清单：\n{}",
            s.checklist
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    let mut out = format!(
        "【创作方法论（{}·第{}步：{}）】\n{}{}",
        cm.name,
        idx + 1,
        s.title,
        s.instruction,
        checklist
    );

    // 技巧参考：按步骤轮转取最多 3 条（确定性、零额外 LLM 调用）
    if !cm.patterns.is_empty() {
        let start = idx % cm.patterns.len();
        let n = 3.min(cm.patterns.len());
        let lines = (0..n)
            .map(|k| {
                let t = &cm.patterns[(start + k) % cm.patterns.len()];
                format!(
                    "- {}：{}→{}",
                    clip_chars(&t.name, 40),
                    clip_chars(&t.when_to_use, 80),
                    clip_chars(&t.how, 160)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        out.push_str(&format!("\n\n【技巧参考】\n{}", lines));
    }

    // 决策速查：最多 3 条规则 + 1 条反模式
    if !cm.cheatsheet.decision_rules.is_empty() || !cm.cheatsheet.anti_patterns.is_empty() {
        let mut lines = Vec::new();
        for r in cm.cheatsheet.decision_rules.iter().take(3) {
            lines.push(format!("- {}", clip_chars(r, 120)));
        }
        if let Some(ap) = cm.cheatsheet.anti_patterns.first() {
            lines.push(format!(
                "- 避免：{}（{}）",
                clip_chars(&ap.what, 60),
                clip_chars(&ap.why, 80)
            ));
        }
        out.push_str(&format!("\n\n【决策速查】\n{}", lines.join("\n")));
    }

    Some(out)
}

/// 截断到 max 字（chars），防爆 token
fn clip_chars(s: &str, max: usize) -> String {
    let t = s.trim();
    if t.chars().count() > max {
        t.chars().take(max).collect()
    } else {
        t.to_string()
    }
}
```

- [ ] **Step 5: 运行测试确认通过 + 全量回归 + clippy**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -5`
Expected: 全部通过（含新增 3 个测试）

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3 && cargo clippy --lib 2>&1 | tail -3`
Expected: ≥1271 passed / 2 ignored；clippy 无新增警告（注意 `clip_chars` 与 distiller.rs 中同名函数不冲突——分属不同模块，各自私有）

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/guidebook_distillation/service.rs
git commit -m "feat: 续写注入自定义方法论的技巧参考与决策速查（按步骤轮转）"
```

---

### Task 4: 失败可重试（去重状态分支 + retry 命令）

**Files:**
- Modify: `src-tauri/src/guidebook_distillation/repository.rs`（追加 `reset_for_retry`）
- Modify: `src-tauri/src/guidebook_distillation/service.rs`（:55-66 去重分支、追加 `dedup_decision` + `retry_distillation`）
- Modify: `src-tauri/src/guidebook_distillation/commands.rs`（追加命令）
- Modify: `src-tauri/src/handlers.rs`（:206-214 注册块追加）
- Test: `repository.rs`、`service.rs` tests 模块

**Interfaces:**
- Consumes: 现有 `GuidebookRepository`、`CreateTaskRequest`、`parse_book`、`create_chunks`
- Produces:
  - `GuidebookRepository::reset_for_retry(&self, id: &str) -> RepoResult<()>`
  - `pub(crate) fn dedup_decision(status: &DistillationStatus) -> DedupDecision`（纯函数）
  - `GuidebookDistillationService::retry_distillation(&self, guidebook_id: &str) -> Result<(), ParseError>`
  - Tauri 命令 `retry_guidebook_distillation(guidebook_id: String)`（前端 Task 6 调用）

- [ ] **Step 1: 写失败测试**

`repository.rs` tests 模块追加：

```rust
    #[test]
    fn reset_for_retry_clears_status_progress_error() {
        let pool = create_test_pool().unwrap();
        let repo = GuidebookRepository::new(pool);
        repo.create(&sample_guidebook("g9")).unwrap();
        repo.update_error("g9", "LLM 超时").unwrap();
        let g = repo.get_by_id("g9").unwrap().unwrap();
        assert_eq!(g.status, DistillationStatus::Failed);
        assert!(g.error.is_some());
        repo.reset_for_retry("g9").unwrap();
        let g = repo.get_by_id("g9").unwrap().unwrap();
        assert_eq!(g.status, DistillationStatus::Pending);
        assert_eq!(g.progress, 0);
        assert!(g.error.is_none());
    }
```

`service.rs` tests 模块（`extension_tests` 内或新模块）追加：

```rust
    #[test]
    fn dedup_decision_only_failed_and_cancelled_retry() {
        assert!(matches!(
            dedup_decision(&DistillationStatus::Failed),
            DedupDecision::RetryExisting
        ));
        assert!(matches!(
            dedup_decision(&DistillationStatus::Cancelled),
            DedupDecision::RetryExisting
        ));
        for s in [
            DistillationStatus::Pending,
            DistillationStatus::Extracting,
            DistillationStatus::Distilling,
            DistillationStatus::Merging,
            DistillationStatus::Completed,
        ] {
            assert!(matches!(dedup_decision(&s), DedupDecision::ReturnExisting));
        }
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -5`
Expected: 编译失败——`reset_for_retry`/`dedup_decision`/`DedupDecision` 未定义。

- [ ] **Step 3: repository.rs 追加 reset_for_retry**

`update_error`（:137-144）之后插入：

```rust
    /// 重试前重置：清错误、回到 pending、进度归零
    pub fn reset_for_retry(&self, id: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE guidebooks SET status = 'pending', progress = 0, error = NULL, \
             updated_at = ?1 WHERE id = ?2",
            params![Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }
```

- [ ] **Step 4: service.rs 去重分支 + retry_distillation**

`upload_and_distill` 中 :60-66 的去重块替换为：

```rust
        if let Ok(Some(existing)) = repo.get_by_hash(&file_hash) {
            match dedup_decision(&existing.status) {
                DedupDecision::ReturnExisting => {
                    log::info!(
                        "[GuidebookDistillation] File already exists: {}",
                        existing.id
                    );
                    return Ok(existing.id);
                }
                DedupDecision::RetryExisting => {
                    // 失败/已取消的记录：复用已存文件重新提炼，不重复落文件
                    log::info!(
                        "[GuidebookDistillation] Retrying failed/cancelled distillation: {}",
                        existing.id
                    );
                    self.retry_distillation(&existing.id).await?;
                    return Ok(existing.id);
                }
            }
        }
```

文件末尾（`clip_chars` 之后、`#[cfg(test)]` 之前）插入：

```rust
/// hash 去重决策：失败/已取消的记录走重试，其余直接返回旧 id
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DedupDecision {
    ReturnExisting,
    RetryExisting,
}

pub(crate) fn dedup_decision(status: &DistillationStatus) -> DedupDecision {
    match status {
        DistillationStatus::Failed | DistillationStatus::Cancelled => DedupDecision::RetryExisting,
        _ => DedupDecision::ReturnExisting,
    }
}
```

`GuidebookDistillationService` 的 `cancel_distillation` 方法之后插入：

```rust
    /// 重试提炼：复用已存文件重建任务（仅 failed/cancelled 可重试）。
    /// 与 upload_and_distill 的任务创建/回退路径同款。
    pub async fn retry_distillation(&self, guidebook_id: &str) -> Result<(), ParseError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        let book = repo
            .get_by_id(guidebook_id)
            .map_err(|e| ParseError::StorageError(format!("查询指导书失败: {}", e)))?
            .ok_or_else(|| ParseError::StorageError("指导书不存在".to_string()))?;
        match book.status {
            DistillationStatus::Failed | DistillationStatus::Cancelled => {}
            _ => {
                return Err(ParseError::StorageError(
                    "仅失败或已取消的指导书可重试".to_string(),
                ))
            }
        }
        let file_path = book
            .file_path
            .clone()
            .ok_or_else(|| {
                ParseError::StorageError("原始文件记录缺失，请删除后重新上传".to_string())
            })?;
        let path = std::path::PathBuf::from(&file_path);
        if !path.exists() {
            return Err(ParseError::IoError(
                "原始文件已丢失，请删除后重新上传".to_string(),
            ));
        }
        let parsed = parse_book(&path, None)?;
        repo.reset_for_retry(guidebook_id)
            .map_err(|e| ParseError::StorageError(format!("重置状态失败: {}", e)))?;

        let payload = serde_json::json!({
            "guidebook_id": guidebook_id,
            "file_path": file_path,
        })
        .to_string();
        let task_req = CreateTaskRequest {
            name: format!("指导书提炼（重试）: {}", book.title),
            description: Some(format!("重试提炼 {} 字的指导书", parsed.word_count)),
            task_type: "guidebook_distillation".to_string(),
            schedule_type: "once".to_string(),
            cron_pattern: None,
            payload: Some(payload),
            enabled: Some(true),
            max_retries: Some(3),
            heartbeat_timeout_seconds: Some(600),
        };

        let task_service = self.app_handle.state::<TaskService>();
        match task_service.create_task(task_req) {
            Ok(task) => {
                let _ = repo.update_task_id(guidebook_id, &task.id);
                let _ = repo.update_status(guidebook_id, DistillationStatus::Pending, 0);
            }
            Err(e) => {
                log::error!(
                    "[GuidebookDistillation] 重试任务创建失败，回退直接后台提炼: {}",
                    e
                );
                let pool = self.pool.clone();
                let llm_service = self.llm_service.clone();
                let app_handle = self.app_handle.clone();
                let gid = guidebook_id.to_string();
                let chunks = create_chunks(&parsed);
                tauri::async_runtime::spawn(async move {
                    let service =
                        GuidebookDistillationService::new(pool.clone(), llm_service, app_handle);
                    if let Err(e) = service.run_distillation(&gid, &chunks, None, None).await {
                        log::error!("[GuidebookDistillation] 回退重试提炼失败 {}: {}", gid, e);
                        let repo = GuidebookRepository::new(pool.clone());
                        let _ = repo.update_error(&gid, &e.to_string());
                    }
                });
            }
        }
        Ok(())
    }
```

- [ ] **Step 5: commands.rs + handlers.rs 注册**

`commands.rs` 的 `cancel_guidebook_distillation` 之后插入：

```rust
/// 重试失败/已取消的提炼（复用已存文件，无需重新上传）
#[command(rename_all = "snake_case")]
pub async fn retry_guidebook_distillation(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    new_service(&app_handle)?
        .retry_distillation(&guidebook_id)
        .await
        .map_err(AppError::from)
}
```

`handlers.rs` :211 行 `guidebook_distillation::commands::cancel_guidebook_distillation,` 之后插入一行：

```rust
    guidebook_distillation::commands::retry_guidebook_distillation,
```

- [ ] **Step 6: 运行测试确认通过 + 全量回归**

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3 && cargo check 2>&1 | tail -3`
Expected: ≥1273 passed / 2 ignored；编译零错误

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/guidebook_distillation/repository.rs \
        src-tauri/src/guidebook_distillation/service.rs \
        src-tauri/src/guidebook_distillation/commands.rs \
        src-tauri/src/handlers.rs
git commit -m "feat: 技能书提炼失败可重试（复用已存文件，hash 去重按状态分支）"
```

---

### Task 5: 去 Pro（订阅白名单 + 前端面板去 Pro UI）

**Files:**
- Modify: `src-tauri/src/subscription/mod.rs`（:72-97 注释与白名单、tests :183-231）
- Modify: `src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx`（:331-466 面板主体）
- Rewrite: `src-frontend/src/components/guidebook-distillation/__tests__/GuidebookDistillationPanel.test.tsx`

**Interfaces:**
- Consumes: 现有 `has_feature_access`、`useSubscription`
- Produces: Free 用户 `has_feature_access(_, "guidebook_distillation") == true`；前端面板无 Pro 拦截/徽章/横幅/UpgradeModal。`commands.rs:38` 的后端门控**保留**（白名单命中即通过，作为防御层不动）。

- [ ] **Step 1: 改订阅测试为新行为（先红）**

`subscription/mod.rs` tests 中：

`free_user_has_basic_features_but_not_pro_features`（:183-192）替换为：

```rust
    #[test]
    fn free_user_has_basic_features_but_not_pro_features() {
        let svc = service();
        assert!(svc.has_feature_access("u-free", "writer").unwrap());
        assert!(svc.has_feature_access("u-free", "outline").unwrap());
        // v0.36.0：技能书提炼转为免费功能
        assert!(svc
            .has_feature_access("u-free", "guidebook_distillation")
            .unwrap());
        assert!(!svc.has_feature_access("u-free", "bootstrap").unwrap());
    }
```

`downgrade_back_to_free_revokes_pro_features`（:216-231）替换为：

```rust
    #[test]
    fn downgrade_back_to_free_revokes_pro_features() {
        let svc = service();
        svc.upgrade_subscription("u3", "pro", Some(30)).unwrap();
        assert!(svc.has_feature_access("u3", "book_deconstruction").unwrap());

        let status = svc.upgrade_subscription("u3", "free", None).unwrap();
        assert_eq!(status.tier, "free");
        assert!(!svc
            .has_feature_access("u3", "book_deconstruction")
            .unwrap());
        // 免费基础功能不受影响（含 v0.36.0 起免费的技能书提炼）
        assert!(svc.has_feature_access("u3", "writer").unwrap());
        assert!(svc
            .has_feature_access("u3", "guidebook_distillation")
            .unwrap());
    }
```

- [ ] **Step 2: 前端测试重写（先红）**

`GuidebookDistillationPanel.test.tsx` 全文替换为：

```tsx
import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { GuidebookDistillationPanel } from '../GuidebookDistillationPanel';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

const { subscriptionState, dialogOpenMock } = vi.hoisted(() => ({
  subscriptionState: { isPro: false },
  dialogOpenMock: vi.fn(),
}));

vi.mock('@/hooks/useSubscription', () => ({
  useSubscription: () => ({
    isPro: subscriptionState.isPro,
    fetchStatus: () => Promise.resolve(),
  }),
}));

vi.mock('@/hooks/useGuidebookDistillation', () => ({
  useGuidebooks: () => ({ data: [], isLoading: false }),
  useUploadGuidebook: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useDeleteGuidebook: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useGuidebookDistillationStatus: () => ({ data: null }),
  useCancelGuidebookDistillation: () => ({ mutateAsync: vi.fn() }),
  useGuidebookResult: () => ({ data: null, isLoading: false }),
  useUpdateCustomMethodology: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useDeleteCustomMethodology: () => ({ mutateAsync: vi.fn(), isPending: false }),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: (...args: unknown[]) => dialogOpenMock(...args),
}));

vi.mock('react-hot-toast', () => ({ default: { success: vi.fn(), error: vi.fn() } }));

describe('GuidebookDistillationPanel - 免费可用（v0.36.0）', () => {
  beforeEach(() => {
    subscriptionState.isPro = false;
    dialogOpenMock.mockReset();
  });

  it('Free 用户无 Pro 徽标与升级横幅', () => {
    render(<GuidebookDistillationPanel />, { wrapper });
    expect(screen.queryByText('Pro')).not.toBeInTheDocument();
    expect(screen.queryByText(/指导书提炼为 Pro 功能/)).not.toBeInTheDocument();
    expect(screen.queryByText('升级 Pro')).not.toBeInTheDocument();
  });

  it('Free 用户点击上传直接打开文件对话框', async () => {
    dialogOpenMock.mockResolvedValue(null);
    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    await user.click(screen.getByText('上传指导书'));
    expect(dialogOpenMock).toHaveBeenCalledTimes(1);
  });
});
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- subscription 2>&1 | tail -5`
Expected: FAIL——`free_user_has_basic_features_but_not_pro_features` 断言失败（白名单未加）。

Run: `cd src-frontend && npx vitest run -- GuidebookDistillationPanel 2>&1 | tail -5`
Expected: FAIL——Pro 徽标/横幅仍在，上传被拦截。

- [ ] **Step 4: subscription/mod.rs 白名单**

:72-77 注释与 :83-89 白名单替换为：

```rust
    /// 检查用户是否有权使用指定功能（订阅解锁功能，非模型配额）
    ///
    /// 细粒度功能权限映射：
    /// - Free 用户可用：基础写作、场景管理、角色管理、知识图谱查询、大纲、
    ///   技能书提炼（v0.36.0 起免费）
    /// - Pro 用户解锁：Bootstrap / Pipeline（Refine/Review/Finalize）/ 拆书 /
    ///   自动续写 / 自动修改
    pub fn has_feature_access(&self, user_id: &str, feature_id: &str) -> Result<bool, AppError> {
        let status = self.get_or_create_subscription(user_id)?;
        let is_pro = status.tier == "pro" || status.tier == "enterprise";

        // Free 用户可用的基础功能
        let free_features = [
            "writer",
            "scene_management",
            "character_management",
            "knowledge_graph_query",
            "outline",
            "guidebook_distillation",
        ];
```

（函数其余部分不变）

- [ ] **Step 5: GuidebookDistillationPanel.tsx 去 Pro UI**

改动点（均在面板主体组件 `GuidebookDistillationPanel` 与 imports）：

1. imports 删除：`Sparkles`（lucide 导入列表中）、`isSubscriptionRequired`（保留 `extractMessage`）、`useSubscription`、`UpgradeModal`。
2. 组件内删除：`const [showUpgradeModal, setShowUpgradeModal] = useState(false);`、`const { isPro, fetchStatus: refreshSubscription } = useSubscription();`
3. `handleUpload`（:340-365）替换为：

```tsx
  const handleUpload = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{ name: '指导书', extensions: ['txt', 'pdf', 'epub'] }],
      });
      if (selected && typeof selected === 'string') {
        const guidebookId = await uploadMutation.mutateAsync(selected);
        setSelectedId(guidebookId);
        toast.success('上传成功，开始提炼...');
      }
    } catch (error) {
      toast.error(`上传失败: ${extractMessage(error)}`);
    }
  };
```

4. 标题区（:384-391）删除 `{!isPro && (...Pro 徽章...)}` 条件块，只留 `指导书提炼` 文本。
5. 上传按钮（:396-408）删除 `title={isPro ? undefined : '...'}` 属性。
6. 删除整个升级横幅块（:411-426 `{!isPro && (...)}`）。
7. 删除文件尾部 `<UpgradeModal ... />`（:455-463）。

- [ ] **Step 6: 运行测试确认通过 + 全量回归**

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3`
Expected: ≥1273 passed / 2 ignored

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | grep -E "Tests|passed|failed"`
Expected: tsc 通过；≥399 passed / 3 skipped（面板测试总数变化属预期：5 个门控测试 → 2 个免费可用测试；总数不得有其它失败）

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/subscription/mod.rs \
        src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx \
        src-frontend/src/components/guidebook-distillation/__tests__/GuidebookDistillationPanel.test.tsx
git commit -m "feat: 技能书提炼转为免费功能（订阅白名单+前端去 Pro UI）"
```

---

### Task 6: 前端资产展示与编辑 + 重试按钮

**Files:**
- Modify: `src-frontend/src/types/guidebook-distillation.ts`
- Modify: `src-frontend/src/hooks/useGuidebookDistillation.ts`（:133-151 update hook 扩展 + 追加 retry hook）
- Modify: `src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx`（GuidebookCard 重试按钮、MethodologyEditor 资产编辑区）
- Test: `src-frontend/src/components/guidebook-distillation/__tests__/GuidebookDistillationPanel.test.tsx`（追加）

**Interfaces:**
- Consumes: Task 4 的 `retry_guidebook_distillation` 命令；Task 1 的 `update_custom_methodology` 扩展（本任务 Step 2 同步扩展后端命令参数）
- Produces:
  - TS 类型 `Technique { name, when_to_use, how }`、`AntiPattern { what, why }`、`Cheatsheet { decision_rules, anti_patterns }`；`CustomMethodology` 加 `patterns: Technique[]`、`cheatsheet: Cheatsheet`
  - hook `useRetryGuidebookDistillation()`（调 `retry_guidebook_distillation`，成功刷 guidebooks 列表）
  - `update_custom_methodology` 命令/前端 hook 支持 `patterns`/`cheatsheet` 参数

- [ ] **Step 1: 后端 update_custom_methodology 支持新字段（先补后端，前端才能调）**

`commands.rs` 的 `update_custom_methodology`（:156-175）替换为：

```rust
/// 更新自定义方法论（None 字段不动）
#[command(rename_all = "snake_case")]
pub async fn update_custom_methodology(
    id: String,
    name: Option<String>,
    description: Option<String>,
    steps: Option<Vec<MethodologyStep>>,
    enabled: Option<bool>,
    patterns: Option<Vec<Technique>>,
    cheatsheet: Option<Cheatsheet>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    CustomMethodologyRepository::new(pool)
        .update(
            &id,
            name.as_deref(),
            description.as_deref(),
            steps.as_deref(),
            enabled,
            patterns.as_deref(),
            cheatsheet.as_ref(),
        )
        .map_err(AppError::from)
}
```

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -3`
Expected: 通过（无行为变化，仅参数扩展）

- [ ] **Step 2: 前端测试（先红）**

`GuidebookDistillationPanel.test.tsx` 的 hoisted 区与 mock 扩展，并追加用例。hoisted 改为：

```tsx
const { subscriptionState, dialogOpenMock, retryMock } = vi.hoisted(() => ({
  subscriptionState: { isPro: false },
  dialogOpenMock: vi.fn(),
  retryMock: vi.fn(),
}));
```

`useGuidebookDistillation` mock 追加一行：

```tsx
  useRetryGuidebookDistillation: () => ({ mutateAsync: retryMock, isPending: false }),
```

`useGuidebooks` mock 改为可按用例切换数据——在 describe 内追加新 describe：

```tsx
describe('GuidebookCard - 失败重试（v0.36.0）', () => {
  beforeEach(() => {
    retryMock.mockReset();
  });

  it('失败状态的卡片显示重试按钮，点击调用 retry', async () => {
    // 直接渲染卡片列表：mock useGuidebooks 返回一条 failed 记录
    const failedBook = {
      id: 'g-fail',
      title: '失败的书',
      author: null,
      subject: null,
      word_count: 1000,
      file_format: 'txt',
      methodology_id: null,
      status: 'failed',
      progress: 0,
      created_at: '2026-08-09',
    };
    vi.mocked(
      (await import('@/hooks/useGuidebookDistillation')).useGuidebooks
    ).mockReturnValue({ data: [failedBook], isLoading: false } as never);

    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    const btn = screen.getByText('重试提炼');
    await user.click(btn);
    expect(retryMock).toHaveBeenCalledWith('g-fail');
  });

  it('completed 状态的卡片不显示重试按钮', async () => {
    const doneBook = {
      id: 'g-done',
      title: '完成的书',
      author: null,
      subject: null,
      word_count: 1000,
      file_format: 'txt',
      methodology_id: 'custom_x',
      status: 'completed',
      progress: 100,
      created_at: '2026-08-09',
    };
    vi.mocked(
      (await import('@/hooks/useGuidebookDistillation')).useGuidebooks
    ).mockReturnValue({ data: [doneBook], isLoading: false } as never);

    render(<GuidebookDistillationPanel />, { wrapper });
    expect(screen.queryByText('重试提炼')).not.toBeInTheDocument();
  });
});
```

注意：文件顶部 `useGuidebookDistillation` mock 从对象工厂改为 `vi.fn()` 形式以支持 `vi.mocked`——将该 mock 整体替换为：

```tsx
vi.mock('@/hooks/useGuidebookDistillation', () => ({
  useGuidebooks: vi.fn(() => ({ data: [], isLoading: false })),
  useUploadGuidebook: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useDeleteGuidebook: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useGuidebookDistillationStatus: () => ({ data: null }),
  useCancelGuidebookDistillation: () => ({ mutateAsync: vi.fn() }),
  useRetryGuidebookDistillation: () => ({ mutateAsync: retryMock, isPending: false }),
  useGuidebookResult: () => ({ data: null, isLoading: false }),
  useUpdateCustomMethodology: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useDeleteCustomMethodology: () => ({ mutateAsync: vi.fn(), isPending: false }),
}));
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cd src-frontend && npx vitest run -- GuidebookDistillationPanel 2>&1 | tail -5`
Expected: FAIL——无 `重试提炼` 按钮。

- [ ] **Step 4: 类型与 hook**

`types/guidebook-distillation.ts` 在 `MethodologyStep` 之后插入：

```typescript
export interface Technique {
  name: string;
  when_to_use: string;
  how: string;
}

export interface AntiPattern {
  what: string;
  why: string;
}

export interface Cheatsheet {
  decision_rules: string[];
  anti_patterns: AntiPattern[];
}
```

`CustomMethodology` 接口追加：

```typescript
  patterns: Technique[];
  cheatsheet: Cheatsheet;
```

`useGuidebookDistillation.ts` 的 `useUpdateCustomMethodology` 的 input 类型追加两字段：

```typescript
      patterns?: Technique[];
      cheatsheet?: Cheatsheet;
```

（`Technique`/`Cheatsheet` 加入顶部 type 导入。）

`useCancelGuidebookDistillation` 之后插入：

```typescript
export function useRetryGuidebookDistillation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (guidebookId: string) => {
      await loggedInvoke<void>('retry_guidebook_distillation', {
        guidebook_id: guidebookId,
      });
    },
    onSuccess: (_, guidebookId) => {
      queryClient.invalidateQueries({ queryKey: [DISTILL_STATUS_KEY, guidebookId] });
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
    },
  });
}
```

- [ ] **Step 5: GuidebookCard 重试按钮**

`GuidebookDistillationPanel.tsx` 的 `GuidebookCard` 内：

imports 的 lucide 列表追加 `RotateCcw`；hooks 导入追加 `useRetryGuidebookDistillation`。

`GuidebookCard` 组件内 `const cancelMutation = ...` 之后加：

```tsx
  const retryMutation = useRetryGuidebookDistillation();

  const handleRetry = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await retryMutation.mutateAsync(guidebook.id);
      toast.success('已重新开始提炼');
    } catch (error) {
      toast.error(`重试失败: ${extractMessage(error)}`);
    }
  };
```

状态行（`{ACTIVE_STATUSES.includes(status) && (...Loader2...)}` 之后、删除按钮之前）插入：

```tsx
          {(status === 'failed' || status === 'cancelled') && (
            <button
              onClick={handleRetry}
              disabled={retryMutation.isPending}
              title="复用已上传文件重新提炼"
              className="p-1.5 rounded-lg hover:bg-cinema-gold/10 text-gray-500 hover:text-cinema-gold transition-colors disabled:opacity-50"
            >
              <RotateCcw className={cn('w-3.5 h-3.5', retryMutation.isPending && 'animate-spin')} />
            </button>
          )}
```

为让测试的 `getByText('重试提炼')` 可断言，按钮加可视文本——上面按钮改为带文本版本：

```tsx
          {(status === 'failed' || status === 'cancelled') && (
            <button
              onClick={handleRetry}
              disabled={retryMutation.isPending}
              className="flex items-center gap-1 px-2 py-1 rounded-lg hover:bg-cinema-gold/10 text-gray-500 hover:text-cinema-gold transition-colors text-xs disabled:opacity-50"
            >
              <RotateCcw className={cn('w-3.5 h-3.5', retryMutation.isPending && 'animate-spin')} />
              重试提炼
            </button>
          )}
```

（前一个代码块仅为说明插入位置，实际采用本带文本版本。）

- [ ] **Step 6: MethodologyEditor 资产编辑区**

`MethodologyEditor` 内 `const [steps, setSteps] = ...` 之后追加 state：

```tsx
  const [patterns, setPatterns] = useState<
    Array<{ name: string; when_to_use: string; how: string }>
  >(methodology.patterns.map(t => ({ ...t })));
  const [decisionRules, setDecisionRules] = useState(
    methodology.cheatsheet.decision_rules.join('\n')
  );
  const [antiPatterns, setAntiPatterns] = useState<Array<{ what: string; why: string }>>(
    methodology.cheatsheet.anti_patterns.map(a => ({ ...a }))
  );
```

`handleSave` 的 `updateMutation.mutateAsync` 参数追加：

```tsx
        patterns: patterns.filter(t => t.name.trim().length > 0),
        cheatsheet: {
          decision_rules: decisionRules
            .split('\n')
            .map(line => line.trim())
            .filter(line => line.length > 0),
          anti_patterns: antiPatterns.filter(a => a.what.trim().length > 0),
        },
```

步骤编辑区之后、enabled checkbox 之前插入资产编辑 JSX：

```tsx
      <div className="space-y-3">
        <label className="block text-xs text-gray-400">技巧模式库（{patterns.length}）</label>
        {patterns.map((t, idx) => (
          <div
            key={idx}
            className="p-3 rounded-lg bg-cinema-800/50 border border-cinema-700 space-y-2"
          >
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={t.name}
                onChange={e =>
                  setPatterns(prev =>
                    prev.map((x, i) => (i === idx ? { ...x, name: e.target.value } : x))
                  )
                }
                placeholder="技巧名称"
                className="flex-1 px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-white text-sm focus:border-cinema-gold focus:outline-none"
              />
              <button
                onClick={() => setPatterns(prev => prev.filter((_, i) => i !== idx))}
                className="p-1.5 rounded hover:bg-red-500/10 text-gray-500 hover:text-red-400 transition-colors"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
            <input
              type="text"
              value={t.when_to_use}
              onChange={e =>
                setPatterns(prev =>
                  prev.map((x, i) => (i === idx ? { ...x, when_to_use: e.target.value } : x))
                )
              }
              placeholder="何时使用"
              className="w-full px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-white text-sm focus:border-cinema-gold focus:outline-none"
            />
            <textarea
              value={t.how}
              onChange={e =>
                setPatterns(prev =>
                  prev.map((x, i) => (i === idx ? { ...x, how: e.target.value } : x))
                )
              }
              placeholder="具体怎么做"
              rows={2}
              className="w-full px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-white text-sm focus:border-cinema-gold focus:outline-none resize-y"
            />
          </div>
        ))}
        <button
          onClick={() =>
            setPatterns(prev => [...prev, { name: '', when_to_use: '', how: '' }])
          }
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-cinema-700 text-gray-400 hover:text-white hover:border-cinema-600 transition-colors text-xs"
        >
          <Plus className="w-3.5 h-3.5" />
          添加技巧
        </button>
      </div>

      <div>
        <label className="block text-xs text-gray-400 mb-1">决策速查（一行一条：当X时做Y，因为Z）</label>
        <textarea
          value={decisionRules}
          onChange={e => setDecisionRules(e.target.value)}
          rows={3}
          className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-y"
        />
      </div>

      <div className="space-y-3">
        <label className="block text-xs text-gray-400">反模式（{antiPatterns.length}）</label>
        {antiPatterns.map((a, idx) => (
          <div key={idx} className="flex items-center gap-2">
            <input
              type="text"
              value={a.what}
              onChange={e =>
                setAntiPatterns(prev =>
                  prev.map((x, i) => (i === idx ? { ...x, what: e.target.value } : x))
                )
              }
              placeholder="应避免的做法"
              className="flex-1 px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-white text-sm focus:border-cinema-gold focus:outline-none"
            />
            <input
              type="text"
              value={a.why}
              onChange={e =>
                setAntiPatterns(prev =>
                  prev.map((x, i) => (i === idx ? { ...x, why: e.target.value } : x))
                )
              }
              placeholder="为什么会失败"
              className="flex-1 px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-white text-sm focus:border-cinema-gold focus:outline-none"
            />
            <button
              onClick={() => setAntiPatterns(prev => prev.filter((_, i) => i !== idx))}
              className="p-1.5 rounded hover:bg-red-500/10 text-gray-500 hover:text-red-400 transition-colors"
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          </div>
        ))}
        <button
          onClick={() => setAntiPatterns(prev => [...prev, { what: '', why: '' }])}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-cinema-700 text-gray-400 hover:text-white hover:border-cinema-600 transition-colors text-xs"
        >
          <Plus className="w-3.5 h-3.5" />
          添加反模式
        </button>
      </div>
```

- [ ] **Step 7: 运行测试确认通过 + 全量回归**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | grep -E "Tests|passed|failed"`
Expected: tsc 通过；≥401 passed / 3 skipped

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3 && cargo clippy --lib 2>&1 | tail -3 && cargo +nightly fmt -- --check`
Expected: ≥1273 passed；clippy 无新增；fmt 通过

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/guidebook_distillation/commands.rs \
        src-frontend/src/types/guidebook-distillation.ts \
        src-frontend/src/hooks/useGuidebookDistillation.ts \
        src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx \
        src-frontend/src/components/guidebook-distillation/__tests__/GuidebookDistillationPanel.test.tsx
git commit -m "feat: 前端技巧模式库/决策速查编辑与失败重试按钮"
```

---

## 依赖顺序

```
Task 1 (DB+模型) → Task 2 (Distiller) → Task 3 (落库+注入)
                 ↘ Task 4 (重试) ↗（独立，可与 2/3 并行，但都改 service.rs 建议串行）
Task 5 (去 Pro) —— 完全独立，可随时插入
Task 6 (前端) ← 依赖 Task 1（类型）+ Task 4（retry 命令）
```

推荐执行顺序：T1 → T2 → T3 → T4 → T5 → T6。

## 全量回归清单

| 检查 | 命令 | 预期 |
|---|---|---|
| Rust 单测 | `cd src-tauri && cargo test --lib` | ≥1273 passed / 2 ignored |
| Clippy | `cd src-tauri && cargo clippy --lib` | 零新增警告 |
| Fmt | `cd src-tauri && cargo +nightly fmt -- --check` | 通过 |
| TS 类型 | `cd src-frontend && npx tsc --noEmit` | 通过 |
| Vitest | `cd src-frontend && npx vitest run` | ≥401 passed / 3 skipped |
| 架构守卫 | `python3 scripts/architecture_guard.py` | 通过 |
