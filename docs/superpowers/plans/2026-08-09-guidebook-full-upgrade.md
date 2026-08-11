# 技能书全面升级档实施计划（fold-in + 校验器 + SKILL.md 导出）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为技能书提炼增加增量合并（fold-in：新书资产融合进现有自定义方法论）、落库前确定性校验清洗、SKILL.md 导出（book-to-skill 同款格式）。

**Architecture:** fold-in 复用同一提炼流水线产出四类资产，新增 `distill_foldin` prompt 融合新旧资产后重跑 `distill_methodology` 生成连贯 steps，更新现有 CM（name/description/enabled 保留）并 clamp 引用故事的 methodology_step；合并意图持久化在 `guidebooks.merge_into_methodology_id`（V126）使重试沿用。校验器为纯 Rust 清洗函数接入落库前。SKILL.md 导出为纯渲染函数 + 命令 + 前端保存对话框。

**Tech Stack:** Rust（Tauri 2, rusqlite, serde_json）、React + TypeScript、@tauri-apps/plugin-dialog + plugin-fs。

**设计文档：** `docs/plans/2026-08-09-guidebook-full-upgrade-design.md`（已批准）

## Global Constraints

- 测试基线：`cd src-tauri && cargo test --lib` = 1273 passed / 2 ignored；`cd src-frontend && npx vitest run` = 398 passed / 3 skipped。每个任务完成后不得低于基线（只能增加）。
- `.sql` 迁移文件放 `src-tauri/src/db/migrations/` 自动扫描，无需注册。下一个版本号 V126。
- 测试 DB 用 `crate::db::connection::create_test_pool()`。
- 中文 conventional commit；不绕过 pre-commit 钩子；**不 push、不打 tag**。
- 勿动 `.recovery/`；提交前 `git status` 核对暂存区只含本任务文件。
- prompt 文件带 frontmatter（`id/name/description/category/version/variables`），新 prompt version 从 0.36.0 起。
- fold-in 复用 `LlmDistillMergeResponse` 作为融合输出类型（设计文档中的 LlmDistillFoldinResponse 取消，YAGNI——结构完全相同）。

---

### Task 1: V126 迁移 + Guidebook 模型/Repository + upload merge_into 参数

**Files:**
- Create: `src-tauri/src/db/migrations/V126__guidebooks_merge_into.sql`
- Modify: `src-tauri/src/guidebook_distillation/models.rs`（Guidebook 结构体 :53-70）
- Modify: `src-tauri/src/guidebook_distillation/repository.rs`（GuidebookRepository create/row_to_guidebook/list_all）
- Modify: `src-tauri/src/guidebook_distillation/service.rs`（upload_and_distill 签名与校验）
- Modify: `src-tauri/src/guidebook_distillation/commands.rs`（upload_guidebook 加参数）
- Test: 各文件 tests 模块

**Interfaces:**
- Consumes: 现有 `Guidebook`、`GuidebookRepository`、`upload_and_distill`
- Produces（后续任务依赖）:
  - `Guidebook` 新字段 `pub merge_into_methodology_id: Option<String>`
  - `GuidebookListItem` 新字段 `pub merge_into_methodology_id: Option<String>`
  - `GuidebookDistillationService::upload_and_distill(&self, file_path: &Path, merge_into: Option<&str>) -> Result<String, ParseError>`
  - Tauri 命令 `upload_guidebook(file_path: String, merge_into: Option<String>)`

- [ ] **Step 1: 写失败测试（repository）**

`repository.rs` tests 模块的 `guidebook_crud_flow` 之后追加：

```rust
    #[test]
    fn guidebook_merge_into_roundtrip() {
        let pool = create_test_pool().unwrap();
        let repo = GuidebookRepository::new(pool);
        let mut book = sample_guidebook("gm1");
        book.merge_into_methodology_id = Some("custom_target".into());
        repo.create(&book).unwrap();
        let got = repo.get_by_id("gm1").unwrap().unwrap();
        assert_eq!(
            got.merge_into_methodology_id.as_deref(),
            Some("custom_target")
        );
        // 列表项也带该字段
        let items = repo.list_all().unwrap();
        assert_eq!(
            items[0].merge_into_methodology_id.as_deref(),
            Some("custom_target")
        );
        // 普通上传为 None
        repo.create(&sample_guidebook("gm2")).unwrap();
        let got2 = repo.get_by_id("gm2").unwrap().unwrap();
        assert!(got2.merge_into_methodology_id.is_none());
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -5`
Expected: 编译失败——`Guidebook`/`GuidebookListItem` 无 `merge_into_methodology_id` 字段。

- [ ] **Step 3: V126 迁移 + 模型字段**

Create `src-tauri/src/db/migrations/V126__guidebooks_merge_into.sql`：

```sql
ALTER TABLE guidebooks ADD COLUMN merge_into_methodology_id TEXT;
```

`models.rs` 的 `Guidebook` 结构体（`task_id` 之后）追加：

```rust
    pub merge_into_methodology_id: Option<String>,
```

`GuidebookListItem`（`methodology_id` 之后）追加：

```rust
    pub merge_into_methodology_id: Option<String>,
```

- [ ] **Step 4: repository 读写新列**

`GuidebookRepository::create`（:22-48）INSERT 改为 16 列：

```rust
conn.execute(
    "INSERT INTO guidebooks (id, title, author, subject, word_count, file_format, \
     file_hash, file_path, methodology_id, status, progress, error, task_id, \
     merge_into_methodology_id, created_at, updated_at)
     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
    params![
        book.id,
        book.title,
        book.author,
        book.subject,
        book.word_count,
        book.file_format,
        book.file_hash,
        book.file_path,
        book.methodology_id,
        book.status.to_string(),
        book.progress,
        book.error,
        book.task_id,
        book.merge_into_methodology_id,
        book.created_at.to_rfc3339(),
        book.updated_at.to_rfc3339(),
    ],
)?;
```

`row_to_guidebook` 结构体构造中 `task_id: row.get("task_id")?,` 之后追加：

```rust
            merge_into_methodology_id: row.get("merge_into_methodology_id")?,
```

`list_all`（:91-112）SELECT 列表 `methodology_id,` 之后加 `merge_into_methodology_id,`，`query_map` 构造中 `methodology_id: row.get("methodology_id")?,` 之后追加：

```rust
                merge_into_methodology_id: row.get("merge_into_methodology_id")?,
```

`sample_guidebook` 测试辅助函数构造处追加 `merge_into_methodology_id: None,`。

- [ ] **Step 5: upload_and_distill 加 merge_into 参数与校验**

`service.rs` 的 `upload_and_distill` 签名改为：

```rust
    pub async fn upload_and_distill(
        &self,
        file_path: &Path,
        merge_into: Option<&str>,
    ) -> Result<String, ParseError> {
```

函数体开头（`self.validate_file(file_path)?;` 之后）追加校验：

```rust
        // fold-in：合并目标必须是存在的自定义方法论
        if let Some(target) = merge_into {
            if !crate::domain::methodology::is_custom_methodology_id(target) {
                return Err(ParseError::InvalidFormat(
                    "合并目标必须是自定义方法论（custom_ 前缀）".to_string(),
                ));
            }
            let exists = CustomMethodologyRepository::new(self.pool.clone())
                .get_by_id(target)
                .ok()
                .flatten()
                .is_some();
            if !exists {
                return Err(ParseError::StorageError(format!(
                    "合并目标方法论 {} 不存在",
                    target
                )));
            }
        }
```

`book` 构造（`task_id: None,` 之后）追加：

```rust
            merge_into_methodology_id: merge_into.map(|s| s.to_string()),
```

`commands.rs` 的 `upload_guidebook` 改为：

```rust
/// 上传指导书并开始提炼；merge_into 非空时合并进指定自定义方法论（fold-in）
#[command(rename_all = "snake_case")]
pub async fn upload_guidebook(
    file_path: String,
    merge_into: Option<String>,
    app_handle: AppHandle,
) -> Result<String, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let user_id = identity::resolve_user_id(&app_handle, &pool);
    let subscription = SubscriptionService::new(pool.clone());
    if !subscription.has_feature_access(&user_id, "guidebook_distillation")? {
        return Err(AppError::subscription_required(
            "guidebook_distillation",
            "指导书提炼功能需要 Pro 订阅，请升级以继续使用",
        ));
    }
    let service = new_service(&app_handle)?;
    service
        .upload_and_distill(std::path::Path::new(&file_path), merge_into.as_deref())
        .await
        .map_err(AppError::from)
}
```

- [ ] **Step 6: 修复编译（调用点）**

`service.rs` 与 `commands.rs` 之外无其它 `upload_and_distill` 调用点（executor 走 payload 不经过它）。全仓 `Guidebook`/`GuidebookListItem` 构造点补齐新字段——已知仅 `service.rs` 的 `book` 构造（Step 5 已改）与 repository 测试辅助 `sample_guidebook`（Step 4 已改）。

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: 零错误（如有遗漏构造点逐一补 `merge_into_methodology_id: None`）

- [ ] **Step 7: 运行测试确认通过 + 全量回归**

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3`
Expected: ≥1274 passed / 2 ignored

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/db/migrations/V126__guidebooks_merge_into.sql \
        src-tauri/src/guidebook_distillation/models.rs \
        src-tauri/src/guidebook_distillation/repository.rs \
        src-tauri/src/guidebook_distillation/service.rs \
        src-tauri/src/guidebook_distillation/commands.rs
git commit -m "feat: fold-in 数据基础——guidebooks 记录合并目标（V126）+ upload 参数校验"
```

---

### Task 2: fold-in 提炼流水线（prompt + distiller 分支 + CM 更新 + step clamp）

**Files:**
- Create: `resources/prompts/distillation/distill_foldin.md`
- Modify: `src-tauri/src/guidebook_distillation/distiller.rs`（distill 签名 + foldin 分支 + foldin_assets 方法）
- Modify: `src-tauri/src/guidebook_distillation/service.rs`（run_distillation fold-in 分支 + clamp_stories_step）
- Test: `distiller.rs`、`service.rs` tests 模块

**Interfaces:**
- Consumes: U1 的 `guidebook.merge_into_methodology_id`；T1-T3 的 `CustomMethodology`/`DistillationOutput`/`ChunkAssets`
- Produces:
  - `GuidebookDistiller::distill(&self, guidebook_id, chunks, fold_in: Option<&CustomMethodology>, heartbeat, cancel_check) -> Result<DistillationOutput, AnalysisError>`
  - 纯函数 `build_foldin_input(existing: &CustomMethodology, new_assets: &ChunkAssets) -> String`（可测试）
  - `GuidebookDistillationService::clamp_stories_step(&self, methodology_id: &str, max_steps: i32)`（私有，借测试模块验证 SQL 效果）

- [ ] **Step 1: 写失败测试（distiller foldin 输入构建）**

`distiller.rs` tests 模块追加：

```rust
    fn sample_cm() -> CustomMethodology {
        CustomMethodology {
            id: "custom_t".into(),
            guidebook_id: None,
            name: "旧方法论".into(),
            description: None,
            steps: vec![MethodologyStep {
                title: "旧步骤".into(),
                instruction: "旧指令".into(),
                checklist: vec![],
            }],
            patterns: vec![Technique {
                name: "旧技巧".into(),
                when_to_use: "w".into(),
                how: "h".into(),
            }],
            cheatsheet: Cheatsheet {
                decision_rules: vec!["旧规则".into()],
                anti_patterns: vec![AntiPattern {
                    what: "旧反模式".into(),
                    why: "why".into(),
                }],
            },
            enabled: true,
            created_at: chrono::Local::now(),
            updated_at: chrono::Local::now(),
        }
    }

    #[test]
    fn foldin_input_contains_existing_and_new_sections() {
        let cm = sample_cm();
        let mut new_assets = ChunkAssets::default();
        new_assets.points.push("新要点".into());
        new_assets.techniques.push(Technique {
            name: "新技巧".into(),
            when_to_use: String::new(),
            how: String::new(),
        });
        let input = build_foldin_input(&cm, &new_assets);
        assert!(input.contains("【现有方法论资产】"));
        assert!(input.contains("旧技巧"));
        assert!(input.contains("旧规则"));
        assert!(input.contains("旧反模式"));
        assert!(input.contains("【新提炼资产】"));
        assert!(input.contains("新要点"));
        assert!(input.contains("新技巧"));
    }

    #[test]
    fn foldin_input_truncates_to_budget() {
        let mut cm = sample_cm();
        cm.patterns = (0..200)
            .map(|i| Technique {
                name: format!("技巧{}", i),
                when_to_use: "w".repeat(100),
                how: "h".repeat(200),
            })
            .collect();
        let input = build_foldin_input(&cm, &ChunkAssets::default());
        assert!(input.chars().count() <= 12050); // 12000 + 段标题余量
    }
```

- [ ] **Step 2: 写失败测试（service clamp）**

`service.rs` tests 模块追加（用 StoryRepository 造故事，避免裸 INSERT 撞 stories 表 NOT NULL 列）：

```rust
    #[test]
    fn clamp_stories_step_caps_to_new_max() {
        use crate::db::{dto::CreateStoryRequest, repositories::StoryRepository};
        let pool = create_test_pool().unwrap();
        let story = StoryRepository::new(pool.clone())
            .create(CreateStoryRequest {
                title: "clamp 测试".into(),
                description: None,
                genre: None,
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .unwrap();
        {
            let conn = pool.get().unwrap();
            conn.execute(
                "UPDATE stories SET methodology_id = 'custom_clamp', methodology_step = 5 WHERE id = ?1",
                rusqlite::params![story.id],
            )
            .unwrap();
        }
        clamp_stories_step_in(&pool, "custom_clamp", 3).unwrap();
        let conn = pool.get().unwrap();
        let step: i32 = conn
            .query_row(
                "SELECT methodology_step FROM stories WHERE id = ?1",
                rusqlite::params![story.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(step, 3);
    }
```

注意：`CreateStoryRequest` 字段以 `src-tauri/src/db/dto.rs` 当前定义为准（上述字段来自既有测试用法，若多出带默认的字段按编译错误补齐 `None`）。函数做成纯的可测形式：

- [ ] **Step 3: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- guidebook_distillation 2>&1 | tail -5`
Expected: 编译失败——`build_foldin_input`/`clamp_stories_step_in` 未定义，`distill` 签名不匹配（下一步改）。

- [ ] **Step 4: distill_foldin prompt**

Create `resources/prompts/distillation/distill_foldin.md`：

```markdown
---
id: distill_foldin
name: "指导书资产增量融合"
description: "将新指导书提炼的资产与现有方法论资产融合去重，产出更强的统一资产集"
category: distillation
version: 0.36.0
variables:
  - existing
  - new
---

你是一位小说创作方法论专家。一位创作者已有一套从指导书提炼的创作方法论资产，现在又提炼了一本新指导书的资产。请将两者**融合为一套更强的统一资产**。

要求：
1. 语义相同的条目合并为一条，保留最准确的表述与作者原始命名（两本书对同一技巧命名不同时，并列保留如"场景目标（SCU）"）
2. 冲突的指导（一本说应该X、另一本说应该非X）两条都保留，并在表述中注明适用情境差异
3. principles：按主题归类排序，保留最重要的 15-25 条
4. techniques：保留最实用、最具操作性的 8-20 条，每条必须含 name/when_to_use/how
5. decision_rules：保留 6-12 条，保持"当X时做Y，因为Z"格式
6. anti_patterns：保留 4-10 条，每条含 what/why
7. 只输出 JSON，不要有任何其他文字

【现有方法论资产】
{{existing}}

【新提炼资产】
{{new}}

JSON格式：
{"principles":["原则1","原则2"],"techniques":[{"name":"技巧名","when_to_use":"何时使用","how":"具体怎么做"}],"decision_rules":["当X时做Y，因为Z"],"anti_patterns":[{"what":"应避免的做法","why":"为什么会导致失败"}]}
```

注意：frontmatter variables 中 `new` 是模板引擎变量名——若 `TemplateEngine::render_with_conditions` 对变量名 `new` 有保留字冲突，改用 `new_assets`（实现时先 grep `prompts/engine.rs` 确认无保留字问题，有则同步改 prompt 与调用处）。

- [ ] **Step 5: distiller.rs foldin 支持**

`distill` 签名改为：

```rust
    pub async fn distill(
        &self,
        guidebook_id: &str,
        chunks: &[TextChunk],
        fold_in: Option<&CustomMethodology>,
        heartbeat_callback: Option<Box<dyn Fn() + Send + Sync>>,
        cancel_check: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<DistillationOutput, AnalysisError> {
```

Step 3 段（原 `let merged = self.merge_assets(&assets).await?;`）替换为：

```rust
        // Step 3: 合并去重（→85%）；fold-in 时与现有方法论资产融合
        let merged = if let Some(existing_cm) = fold_in {
            self.emit_progress(guidebook_id, "merging", 72, "正在与现有方法论融合...")
                .await;
            self.foldin_assets(existing_cm, &assets).await?
        } else {
            self.emit_progress(guidebook_id, "merging", 72, "正在分类合并创作资产...")
                .await;
            self.merge_assets(&assets).await?
        };
```

（原 Step 3 的独立 emit_progress 行删除，进度文案并入分支内。）

`merge_assets` 之后追加：

```rust
    /// fold-in：新资产与现有方法论资产融合（输出与 merge 同构）
    async fn foldin_assets(
        &self,
        existing: &CustomMethodology,
        new_assets: &ChunkAssets,
    ) -> Result<LlmDistillMergeResponse, AnalysisError> {
        if new_assets.is_empty() {
            return Err(AnalysisError::LlmError(
                "全书未提炼出任何创作要点".to_string(),
            ));
        }
        let prompt = self
            .render_prompt(
                "distill_foldin",
                &[
                    ("existing", build_existing_assets_input(existing)),
                    ("new", build_merge_input(new_assets)),
                ],
            )
            .ok_or_else(|| AnalysisError::LlmError("prompt distill_foldin 未注册".into()))?;
        let resp = call_llm(
            &self.llm_service,
            "guidebook_foldin",
            prompt,
            Some(4000),
            Some(0.3),
        )
        .await?;
        let parsed: LlmDistillMergeResponse = parse_json_response(&resp)?;
        if parsed.principles.is_empty() {
            return Err(AnalysisError::LlmError("融合后原则为空".to_string()));
        }
        Ok(parsed)
    }
```

`build_merge_input` 之后追加两个纯函数：

```rust
/// 现有 CM 资产渲染为 foldin 输入（复用四类分区格式，复用 steps 的 principles 位）
fn build_existing_assets_input(cm: &CustomMethodology) -> String {
    let assets = ChunkAssets {
        points: cm
            .steps
            .iter()
            .map(|s| format!("{}：{}", s.title, s.instruction))
            .collect(),
        techniques: cm.patterns.clone(),
        decision_rules: cm.cheatsheet.decision_rules.clone(),
        anti_patterns: cm.cheatsheet.anti_patterns.clone(),
    };
    build_merge_input(&assets)
}

/// foldin 完整输入（带两大段标题，总量 12000 字截断）
fn build_foldin_input(existing: &CustomMethodology, new_assets: &ChunkAssets) -> String {
    let combined = format!(
        "【现有方法论资产】\n{}\n\n【新提炼资产】\n{}",
        build_existing_assets_input(existing),
        build_merge_input(new_assets)
    );
    clip_chars(&combined, 12000)
}
```

注意：`foldin_assets` 中 `build_foldin_input` 与直接传两段的关系——render_prompt 需要 `existing`/`new` 两个变量分别对应 prompt 中的 `{{existing}}`/`{{new}}`，所以 `foldin_assets` 里**不需要** `build_foldin_input`；`build_foldin_input` 仅供测试预算验证与备用。修正：`foldin_assets` 的 render_prompt 调用（上面代码）传 `build_existing_assets_input(existing)` 与 `clip_chars(&build_merge_input(new_assets), 6000)`（新书段单独限 6000，现有段由 build_merge_input 内部限 12000——实现时把 `build_existing_assets_input` 结果也 `clip_chars(_, 6000)`，保证总输入 ≤12000+段标题）。测试 `foldin_input_truncates_to_budget` 对 `build_foldin_input` 的断言保留。

- [ ] **Step 6: service.rs run_distillation fold-in 分支**

`run_distillation` 中 `let distiller = ...` 之后、`let output = distiller.distill(...)` 调用改为：

```rust
        // fold-in：从 guidebook 记录读合并意图（重试路径同样生效）
        let fold_target = repo
            .get_by_id(guidebook_id)
            .ok()
            .flatten()
            .and_then(|g| g.merge_into_methodology_id);
        let fold_cm = fold_target.as_deref().and_then(|mid| {
            CustomMethodologyRepository::new(self.pool.clone())
                .get_by_id(mid)
                .ok()
                .flatten()
        });
        if fold_target.is_some() && fold_cm.is_none() {
            return Err(AnalysisError::StorageError(
                "合并目标方法论已不存在".to_string(),
            ));
        }

        let output = distiller
            .distill(
                guidebook_id,
                chunks,
                fold_cm.as_ref(),
                heartbeat,
                cancel_check,
            )
            .await?;
```

落库段（原"落库：自定义方法论"整段）改为分支：

```rust
        if let Some(target_cm) = fold_cm {
            // fold-in：更新现有 CM（name/description/enabled 保留），替换 steps/patterns/cheatsheet
            let new_steps: Vec<MethodologyStep> = output
                .methodology
                .steps
                .iter()
                .map(|s| MethodologyStep {
                    title: s.title.clone(),
                    instruction: s.instruction.clone(),
                    checklist: s.checklist.clone(),
                })
                .collect();
            let cm_repo = CustomMethodologyRepository::new(self.pool.clone());
            cm_repo
                .update(
                    &target_cm.id,
                    None,
                    None,
                    Some(&new_steps),
                    None,
                    Some(&output.techniques),
                    Some(&output.cheatsheet),
                )
                .map_err(|e| AnalysisError::StorageError(e.to_string()))?;
            // 引用故事的 step 顶格 clamp 到新 max_steps
            let max_steps = (new_steps.len() as i32).max(1);
            clamp_stories_step_in(&self.pool, &target_cm.id, max_steps)
                .map_err(|e| AnalysisError::StorageError(e.to_string()))?;
            repo.update_distilled(
                guidebook_id,
                output.metadata.title.as_deref(),
                output.metadata.author.as_deref(),
                output.metadata.subject.as_deref(),
                &target_cm.id,
            )
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;
            repo.update_status(guidebook_id, DistillationStatus::Completed, 100)
                .map_err(|e| AnalysisError::StorageError(e.to_string()))?;
            log::info!(
                "[GuidebookDistillation] {} fold-in 完成 → 合并进 {}（{}）",
                guidebook_id,
                target_cm.id,
                target_cm.name
            );
            return Ok(());
        }

        // 原有新建路径（落库：自定义方法论 ...）保持不变
```

文件尾部（`dedup_decision` 之后）追加可测纯函数：

```rust
/// 引用某方法论的故事 methodology_step 顶格 clamp（fold-in 后 steps 数可能变少）
pub(crate) fn clamp_stories_step_in(
    pool: &DbPool,
    methodology_id: &str,
    max_steps: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE stories SET methodology_step = ?1 \
         WHERE methodology_id = ?2 AND methodology_step > ?1",
        rusqlite::params![max_steps, methodology_id],
    )?;
    Ok(())
}
```

- [ ] **Step 7: 运行测试确认通过 + 全量回归**

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3`
Expected: ≥1277 passed / 2 ignored

- [ ] **Step 8: Commit**

```bash
git add resources/prompts/distillation/distill_foldin.md \
        src-tauri/src/guidebook_distillation/distiller.rs \
        src-tauri/src/guidebook_distillation/service.rs
git commit -m "feat: fold-in 增量融合——新指导书资产并入现有方法论并重生成连贯步骤"
```

---

### Task 3: 提炼结果校验清洗器

**Files:**
- Create: `src-tauri/src/guidebook_distillation/validator.rs`
- Modify: `src-tauri/src/guidebook_distillation/mod.rs`（注册模块）
- Modify: `src-tauri/src/guidebook_distillation/service.rs`（run_distillation 落库前接入）
- Test: `src-tauri/src/guidebook_distillation/validator.rs` tests 模块

**Interfaces:**
- Consumes: `DistillationOutput`、`Technique`、`AntiPattern`、`Cheatsheet`、`MethodologyStep`
- Produces:
  - `pub struct CleanReport { pub removed_techniques: usize, pub deduped_techniques: usize, pub removed_rules: usize, pub removed_anti_patterns: usize, pub truncated_fields: usize }`（派生 Debug/Default/PartialEq）
  - `pub fn validate_and_clean(output: DistillationOutput) -> (DistillationOutput, CleanReport)`
  - service.rs 落库前调用，清洗后 log::info 质量指标

- [ ] **Step 1: 写失败测试**

Create `src-tauri/src/guidebook_distillation/validator.rs`，内容为本任务 Step 1-3 的测试 + Step 4 的实现。先写测试（文件尾部 `#[cfg(test)] mod tests`）：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::guidebook_distillation::models::*;

    fn output_with(techniques: Vec<Technique>, rules: Vec<String>, anti: Vec<AntiPattern>) -> DistillationOutput {
        DistillationOutput {
            metadata: LlmGuidebookMetadataResponse {
                title: None,
                author: None,
                subject: None,
            },
            methodology: LlmMethodologyResponse {
                name: "测试法".into(),
                description: None,
                steps: vec![LlmMethodologyStepResponse {
                    title: "s".into(),
                    instruction: "i".into(),
                    checklist: vec![],
                }],
            },
            techniques,
            cheatsheet: Cheatsheet {
                decision_rules: rules,
                anti_patterns: anti,
            },
        }
    }

    #[test]
    fn removes_blank_name_techniques_and_dedupes() {
        let out = output_with(
            vec![
                Technique { name: "  ".into(), when_to_use: "w".into(), how: "h".into() },
                Technique { name: "雪花法".into(), when_to_use: "w1".into(), how: "h1".into() },
                Technique { name: "雪花法".into(), when_to_use: "w2".into(), how: "h2".into() },
            ],
            vec![],
            vec![],
        );
        let (cleaned, report) = validate_and_clean(out);
        assert_eq!(cleaned.techniques.len(), 1);
        assert_eq!(cleaned.techniques[0].when_to_use, "w1"); // 保留首个
        assert_eq!(report.removed_techniques, 1);
        assert_eq!(report.deduped_techniques, 1);
    }

    #[test]
    fn removes_blank_rules_and_anti_patterns() {
        let out = output_with(
            vec![],
            vec!["".into(), "  ".into(), "当X做Y，因为Z".into(), "当X做Y，因为Z".into()],
            vec![
                AntiPattern { what: " ".into(), why: "w".into() },
                AntiPattern { what: "流水账".into(), why: "无冲突".into() },
                AntiPattern { what: "流水账".into(), why: "重复".into() },
            ],
        );
        let (cleaned, report) = validate_and_clean(out);
        assert_eq!(cleaned.cheatsheet.decision_rules, vec!["当X做Y，因为Z"]);
        assert_eq!(cleaned.cheatsheet.anti_patterns.len(), 1);
        assert_eq!(report.removed_rules, 2); // 2 空白 + 1 重复 = 3？见断言口径
        assert_eq!(report.removed_anti_patterns, 2);
    }

    #[test]
    fn truncates_overlong_fields() {
        let long = "x".repeat(300);
        let out = output_with(
            vec![Technique {
                name: "t".into(),
                when_to_use: long.clone(),
                how: long.clone(),
            }],
            vec![long.clone()],
            vec![],
        );
        let (cleaned, report) = validate_and_clean(out);
        assert_eq!(cleaned.techniques[0].when_to_use.chars().count(), 200);
        assert_eq!(cleaned.techniques[0].how.chars().count(), 200);
        assert_eq!(cleaned.cheatsheet.decision_rules[0].chars().count(), 200);
        assert_eq!(report.truncated_fields, 3);
    }

    #[test]
    fn truncates_step_title_and_instruction() {
        let mut out = output_with(vec![], vec![], vec![]);
        out.methodology.steps[0].title = "t".repeat(30);
        out.methodology.steps[0].instruction = "i".repeat(600);
        let (cleaned, report) = validate_and_clean(out);
        assert_eq!(cleaned.methodology.steps[0].title.chars().count(), 20);
        assert_eq!(cleaned.methodology.steps[0].instruction.chars().count(), 500);
        assert_eq!(report.truncated_fields, 2);
    }

    #[test]
    fn clean_output_is_noop_on_valid_input() {
        let out = output_with(
            vec![Technique { name: "t".into(), when_to_use: "w".into(), how: "h".into() }],
            vec!["r".into()],
            vec![AntiPattern { what: "a".into(), why: "b".into() }],
        );
        let (_, report) = validate_and_clean(out);
        assert_eq!(report, CleanReport::default());
    }
}
```

注意 `removes_blank_rules_and_anti_patterns` 中断言口径：decision_rules 4 条输入（2 空白 + 2 重复）→ 1 条，`removed_rules` 应计空白剔除 2 + 重复剔除 1 = 3。修正断言为 `assert_eq!(report.removed_rules, 3);`。CleanReport 不区分剔除原因，统称 removed。

- [ ] **Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- validator 2>&1 | tail -5`
Expected: 编译失败——模块未注册、`validate_and_clean`/`CleanReport` 未定义。

- [ ] **Step 3: mod.rs 注册**

`src-tauri/src/guidebook_distillation/mod.rs` 追加一行：

```rust
pub mod validator;
```

- [ ] **Step 4: validator.rs 实现**

文件顶部（测试模块之前）：

```rust
//! 提炼结果校验清洗器（对标 book-to-skill validate_skill.py）。
//! 纯 Rust 确定性清洗：剔除空条目、按名去重、字段截断。落库前调用。

use super::models::*;

/// 清洗统计（质量指标，log 输出）
#[derive(Debug, Default, PartialEq)]
pub struct CleanReport {
    pub removed_techniques: usize,
    pub deduped_techniques: usize,
    pub removed_rules: usize,
    pub removed_anti_patterns: usize,
    pub truncated_fields: usize,
}

const FIELD_MAX: usize = 200;
const STEP_TITLE_MAX: usize = 20;
const STEP_INSTRUCTION_MAX: usize = 500;

fn clip(s: &str, max: usize, report: &mut CleanReport) -> String {
    let t = s.trim();
    if t.chars().count() > max {
        report.truncated_fields += 1;
        t.chars().take(max).collect()
    } else {
        t.to_string()
    }
}

/// 校验并清洗提炼产物。硬校验（方法论结构）已在 distiller 内完成，
/// 此处只做确定性软清洗，不会失败。
pub fn validate_and_clean(mut output: DistillationOutput) -> (DistillationOutput, CleanReport) {
    let mut report = CleanReport::default();

    // techniques：剔除空 name → 按 name 去重（保留首个）→ 字段截断
    let before = output.techniques.len();
    output.techniques.retain(|t| !t.name.trim().is_empty());
    report.removed_techniques = before - output.techniques.len();
    let mut seen = std::collections::HashSet::new();
    let before = output.techniques.len();
    output
        .techniques
        .retain(|t| seen.insert(t.name.trim().to_string()));
    report.deduped_techniques = before - output.techniques.len();
    for t in &mut output.techniques {
        t.name = clip(&t.name, FIELD_MAX, &mut report);
        t.when_to_use = clip(&t.when_to_use, FIELD_MAX, &mut report);
        t.how = clip(&t.how, FIELD_MAX, &mut report);
    }

    // decision_rules：剔除空白 → 去重 → 截断
    let before = output.cheatsheet.decision_rules.len();
    output.cheatsheet.decision_rules.retain(|r| !r.trim().is_empty());
    let mut seen = std::collections::HashSet::new();
    output
        .cheatsheet
        .decision_rules
        .retain(|r| seen.insert(r.trim().to_string()));
    report.removed_rules = before - output.cheatsheet.decision_rules.len();
    for r in &mut output.cheatsheet.decision_rules {
        *r = clip(r, FIELD_MAX, &mut report);
    }

    // anti_patterns：剔除空 what → 按 what 去重
    let before = output.cheatsheet.anti_patterns.len();
    output.cheatsheet.anti_patterns.retain(|a| !a.what.trim().is_empty());
    let mut seen = std::collections::HashSet::new();
    output
        .cheatsheet
        .anti_patterns
        .retain(|a| seen.insert(a.what.trim().to_string()));
    report.removed_anti_patterns = before - output.cheatsheet.anti_patterns.len();
    for a in &mut output.cheatsheet.anti_patterns {
        a.what = clip(&a.what, FIELD_MAX, &mut report);
        a.why = clip(&a.why, FIELD_MAX, &mut report);
    }

    // steps：title/instruction 截断
    for s in &mut output.methodology.steps {
        s.title = clip(&s.title, STEP_TITLE_MAX, &mut report);
        s.instruction = clip(&s.instruction, STEP_INSTRUCTION_MAX, &mut report);
    }

    (output, report)
}
```

- [ ] **Step 5: 接入 run_distillation**

`service.rs` 的 `run_distillation` 中 `let output = distiller.distill(...).await?;` 之后插入：

```rust
        // 落库前确定性清洗（剔除空条目/去重/截断）
        let (output, clean_report) =
            crate::guidebook_distillation::validator::validate_and_clean(output);
        log::info!(
            "[GuidebookDistillation] {} 清洗: 技巧 {}（剔 {} 重 {}）规则 {}（剔 {}）反模式 {}（剔 {}）截断 {} 处",
            guidebook_id,
            output.techniques.len(),
            clean_report.removed_techniques,
            clean_report.deduped_techniques,
            output.cheatsheet.decision_rules.len(),
            clean_report.removed_rules,
            output.cheatsheet.anti_patterns.len(),
            clean_report.removed_anti_patterns,
            clean_report.truncated_fields,
        );
```

注意：后续对 `output` 的使用（fold-in 分支与新建分支）无需改动，`output` 被 shadow 为清洗后版本。

- [ ] **Step 6: 运行测试确认通过 + 全量回归 + clippy**

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3 && cargo clippy --lib 2>&1 | tail -3`
Expected: ≥1282 passed / 2 ignored；clippy 无新增

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/guidebook_distillation/validator.rs \
        src-tauri/src/guidebook_distillation/mod.rs \
        src-tauri/src/guidebook_distillation/service.rs
git commit -m "feat: 提炼产物落库前确定性校验清洗（对标 book-to-skill validate_skill）"
```

---

### Task 4: SKILL.md 导出

**Files:**
- Create: `src-tauri/src/guidebook_distillation/skill_export.rs`
- Modify: `src-tauri/src/guidebook_distillation/mod.rs`（注册模块）
- Modify: `src-tauri/src/guidebook_distillation/commands.rs`（export 命令）
- Modify: `src-tauri/src/handlers.rs`（注册）
- Test: `skill_export.rs` tests 模块 + 前端（U5 不做导出按钮——本任务含前端按钮）

**Interfaces:**
- Consumes: `CustomMethodology`、`GuidebookRepository`
- Produces:
  - `pub fn render_skill_md(cm: &CustomMethodology, source_book: Option<&str>) -> String`
  - `pub fn slugify(name: &str) -> String`
  - Tauri 命令 `export_methodology_skill(id: String) -> Result<String, AppError>`（返回 Markdown 文本）
  - 前端 `MethodologyEditor` 加"导出 SKILL.md"按钮（调命令 → save 对话框 → plugin-fs 写文件，模式参照 `src-frontend/src/hooks/useExport.ts:60-87`）

- [ ] **Step 1: 写失败测试（后端渲染）**

`skill_export.rs` 的 tests 模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::guidebook_distillation::models::*;

    fn sample_cm() -> CustomMethodology {
        CustomMethodology {
            id: "custom_x".into(),
            guidebook_id: None,
            name: "冲突驱动法".into(),
            description: Some("以冲突为引擎".into()),
            steps: vec![MethodologyStep {
                title: "立冲突".into(),
                instruction: "确立核心冲突".into(),
                checklist: vec!["冲突明确吗？".into()],
            }],
            patterns: vec![Technique {
                name: "场景目标法".into(),
                when_to_use: "每场开场".into(),
                how: "给 POV 角色具体目标".into(),
            }],
            cheatsheet: Cheatsheet {
                decision_rules: vec!["当节奏拖沓时删场景，因为每场景须推进冲突".into()],
                anti_patterns: vec![AntiPattern {
                    what: "信息倾倒".into(),
                    why: "读者失去探索欲".into(),
                }],
            },
            enabled: true,
            created_at: chrono::Local::now(),
            updated_at: chrono::Local::now(),
        }
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Snowflake Method"), "snowflake-method");
        assert_eq!(slugify("冲突 驱动 法"), "冲突-驱动-法");
        assert_eq!(slugify("Save the Cat!"), "save-the-cat");
        assert_eq!(slugify("  "), "custom-methodology");
    }

    #[test]
    fn render_skill_md_full_sections() {
        let md = render_skill_md(&sample_cm(), Some("故事的故事"));
        assert!(md.starts_with("---\nname: 冲突驱动法\n"));
        assert!(md.contains("description:"));
        assert!(md.contains("# 冲突驱动法"));
        assert!(md.contains("《故事的故事》"));
        assert!(md.contains("## 创作方法论（按步骤执行）"));
        assert!(md.contains("**立冲突**：确立核心冲突"));
        assert!(md.contains("冲突明确吗？"));
        assert!(md.contains("## 技巧模式库"));
        assert!(md.contains("**场景目标法**"));
        assert!(md.contains("何时用：每场开场"));
        assert!(md.contains("## 决策速查"));
        assert!(md.contains("当节奏拖沓时删场景"));
        assert!(md.contains("## 反模式（务必避免）"));
        assert!(md.contains("**信息倾倒**：读者失去探索欲"));
    }

    #[test]
    fn render_skill_md_omits_empty_sections() {
        let mut cm = sample_cm();
        cm.patterns = vec![];
        cm.cheatsheet = Cheatsheet::default();
        let md = render_skill_md(&cm, None);
        assert!(md.contains("## 创作方法论（按步骤执行）"));
        assert!(!md.contains("## 技巧模式库"));
        assert!(!md.contains("## 决策速查"));
        assert!(!md.contains("## 反模式"));
        assert!(!md.contains("《"));
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test --lib -- skill_export 2>&1 | tail -3`
Expected: 编译失败——模块/函数未定义。

- [ ] **Step 3: mod.rs 注册 + skill_export.rs 实现**

`mod.rs` 追加 `pub mod skill_export;`。

`skill_export.rs`（测试模块之前）：

```rust
//! SKILL.md 导出：把自定义方法论渲染为 book-to-skill 同款 Agent Skills 格式。

use super::models::*;

/// name slug 化：小写、空白转 -、去非字母数字/CJK 字符；全空回退默认
pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for c in name.trim().chars() {
        if c.is_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&c) {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        "custom-methodology".to_string()
    } else {
        trimmed
    }
}

/// 渲染 SKILL.md（空段省略）
pub fn render_skill_md(cm: &CustomMethodology, source_book: Option<&str>) -> String {
    let mut md = String::new();
    let desc = cm.description.as_deref().unwrap_or(&cm.name);
    md.push_str(&format!(
        "---\nname: {}\ndescription: \"创作方法论：{}。写小说/续写/设计情节冲突时使用。\"\n---\n\n",
        slugify(&cm.name),
        desc.replace('"', "'")
    ));
    md.push_str(&format!("# {}\n\n", cm.name));
    let source = source_book
        .map(|t| format!("《{}》（指导书提炼）", t))
        .unwrap_or_else(|| "指导书提炼".to_string());
    md.push_str(&format!(
        "**来源**：{} | **生成**：{}\n\n",
        source,
        chrono::Local::now().format("%Y-%m-%d")
    ));

    // 方法论步骤
    md.push_str("## 创作方法论（按步骤执行）\n\n");
    for (i, s) in cm.steps.iter().enumerate() {
        md.push_str(&format!("{}. **{}**：{}\n", i + 1, s.title, s.instruction));
        for c in &s.checklist {
            md.push_str(&format!("   - 检查：{}\n", c));
        }
    }

    // 技巧模式库
    if !cm.patterns.is_empty() {
        md.push_str("\n## 技巧模式库\n\n");
        for t in &cm.patterns {
            md.push_str(&format!(
                "**{}**\n- 何时用：{}\n- 怎么做：{}\n\n",
                t.name, t.when_to_use, t.how
            ));
        }
    }

    // 决策速查
    if !cm.cheatsheet.decision_rules.is_empty() {
        md.push_str("\n## 决策速查\n\n");
        for r in &cm.cheatsheet.decision_rules {
            md.push_str(&format!("- {}\n", r));
        }
    }

    // 反模式
    if !cm.cheatsheet.anti_patterns.is_empty() {
        md.push_str("\n## 反模式（务必避免）\n\n");
        for a in &cm.cheatsheet.anti_patterns {
            md.push_str(&format!("- **{}**：{}\n", a.what, a.why));
        }
    }

    md
}
```

- [ ] **Step 4: export 命令 + 注册**

`commands.rs` 的 `delete_custom_methodology` 之后追加：

```rust
/// 导出自定义方法论为 SKILL.md 文本（book-to-skill 同款格式）
#[command(rename_all = "snake_case")]
pub async fn export_methodology_skill(
    id: String,
    app_handle: AppHandle,
) -> Result<String, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let cm_repo = CustomMethodologyRepository::new(pool.clone());
    let cm = cm_repo
        .get_by_id(&id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("custom_methodology", &id))?;
    let source_book = cm
        .guidebook_id
        .as_deref()
        .and_then(|gid| {
            super::repository::GuidebookRepository::new(pool)
                .get_by_id(gid)
                .ok()
                .flatten()
        })
        .map(|g| g.title);
    Ok(super::skill_export::render_skill_md(&cm, source_book.as_deref()))
}
```

`handlers.rs` 在 `guidebook_distillation::commands::delete_custom_methodology,` 之后追加：

```rust
    guidebook_distillation::commands::export_methodology_skill,
```

- [ ] **Step 5: 前端导出按钮 + 测试**

`src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx`：

imports 追加：lucide 的 `FileDown`；`loggedInvoke` 从 `@/services/tauri`（检查现有 import——面板目前不经 loggedInvoke，直接 `const { loggedInvoke } = await import('@/services/tauri')` 或在顶部 import）。

`MethodologyEditor` 内 `handleDelete` 之后追加：

```tsx
  const [exporting, setExporting] = useState(false);
  const handleExportSkill = async () => {
    setExporting(true);
    try {
      const { loggedInvoke } = await import('@/services/tauri');
      const markdown = await loggedInvoke<string>('export_methodology_skill', {
        id: methodology.id,
      });
      const { save } = await import('@tauri-apps/plugin-dialog');
      const { writeFile } = await import('@tauri-apps/plugin-fs');
      const filePath = await save({
        filters: [{ name: 'SKILL.md', extensions: ['md'] }],
        defaultPath: `${methodology.name}.md`,
      });
      if (!filePath) return;
      await writeFile(filePath, new TextEncoder().encode(markdown));
      toast.success(`SKILL.md 已导出: ${filePath.split(/[\\/]/).pop()}`);
    } catch (error) {
      toast.error(`导出失败: ${extractMessage(error)}`);
    } finally {
      setExporting(false);
    }
  };
```

底部按钮区（删除方法论按钮之前）插入：

```tsx
        <button
          onClick={handleExportSkill}
          disabled={exporting}
          title="导出为 book-to-skill 同款 SKILL.md，可在 Claude Code / Copilot CLI 中加载"
          className="flex items-center gap-1.5 px-3 py-2 rounded-lg border border-cinema-700 text-gray-400 hover:text-white hover:border-cinema-600 transition-colors text-sm disabled:opacity-50"
        >
          {exporting ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <FileDown className="w-3.5 h-3.5" />
          )}
          导出 SKILL.md
        </button>
```

前端测试：`GuidebookDistillationPanel.test.tsx` 暂不加导出用例（save 对话框 + fs mock 成本高，渲染逻辑已由后端测试覆盖）——仅加一条按钮存在性断言到免费可用 describe：

```tsx
  it('方法论编辑器含导出 SKILL.md 按钮', () => {
    // 该用例依赖 useGuidebookResult 返回带 methodology 的数据，
    // 见下方 mock 切换说明；若实现时成本高可省略本用例
  });
```

（实现时若 mock 切换成本高于收益，删除此用例并在报告注明——导出核心逻辑已由后端 render_skill_md 测试覆盖。）

- [ ] **Step 6: 运行测试确认通过 + 全量回归**

Run: `cd src-tauri && cargo test --lib 2>&1 | tail -3`
Expected: ≥1286 passed / 2 ignored

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | grep -E "Tests|passed"`
Expected: tsc 通过；≥398 passed / 3 skipped

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/guidebook_distillation/skill_export.rs \
        src-tauri/src/guidebook_distillation/mod.rs \
        src-tauri/src/guidebook_distillation/commands.rs \
        src-tauri/src/handlers.rs \
        src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx \
        src-frontend/src/components/guidebook-distillation/__tests__/GuidebookDistillationPanel.test.tsx
git commit -m "feat: 自定义方法论导出 SKILL.md（book-to-skill 同款 Agent Skills 格式）"
```

---

### Task 5: 前端 fold-in 上传 UI + 卡片合并标注

**Files:**
- Modify: `src-frontend/src/types/guidebook-distillation.ts`（GuidebookListItem 加字段）
- Modify: `src-frontend/src/hooks/useGuidebookDistillation.ts`（useUploadGuidebook 参数 + 方法论列表 hook 复用）
- Modify: `src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx`（合并选择 UI + 卡片标注）
- Test: `src-frontend/src/components/guidebook-distillation/__tests__/GuidebookDistillationPanel.test.tsx`（追加）

**Interfaces:**
- Consumes: U1 的 `upload_guidebook(file_path, merge_into)`；U4 已改的 Panel 文件
- Produces:
  - TS `GuidebookListItem.merge_into_methodology_id: string | null`
  - `useUploadGuidebook` mutationFn 参数改为 `{ filePath: string; mergeInto?: string }`
  - 上传流程：选完文件后，若存在自定义方法论 → 弹内联选择（新建/合并到某个 CM）→ 再调 upload

- [ ] **Step 1: 写失败测试**

`GuidebookDistillationPanel.test.tsx` 追加 describe（hoisted 区加 `uploadMock`）：

hoisted 改为：

```tsx
const { subscriptionState, dialogOpenMock, retryMock, uploadMock } = vi.hoisted(() => ({
  subscriptionState: { isPro: false },
  dialogOpenMock: vi.fn(),
  retryMock: vi.fn(),
  uploadMock: vi.fn(),
}));
```

`useGuidebookDistillation` mock 中 `useUploadGuidebook` 改为：

```tsx
  useUploadGuidebook: () => ({ mutateAsync: uploadMock, isPending: false }),
```

并追加 `useAllMethodologies` mock（fold-in 选择列表的数据源，已确认存在于 `src-frontend/src/hooks/useMethodologies.ts`，queryKey `['all-methodologies']`，返回 `MethodologyInfo[]`）：

```tsx
vi.mock('@/hooks/useMethodologies', () => ({
  useAllMethodologies: () => ({
    data: [
      { id: 'custom_a', name: '方法论A', is_custom: true, enabled: true },
      { id: 'snowflake', name: '雪花写作法', is_custom: false, enabled: true },
    ],
    isLoading: false,
  }),
}));
```

追加 describe：

```tsx
describe('fold-in 上传选择（v0.36.0）', () => {
  beforeEach(() => {
    uploadMock.mockReset();
    uploadMock.mockResolvedValue('g-new');
    dialogOpenMock.mockReset();
    vi.mocked(
      (await import('@/hooks/useGuidebookDistillation')).useGuidebooks
    ).mockReturnValue({ data: [], isLoading: false } as never);
  });

  it('选完文件后展示新建/合并选择，选新建直接上传（merge_into 为空）', async () => {
    dialogOpenMock.mockResolvedValue('/tmp/book.txt');
    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    await user.click(screen.getByText('上传指导书'));
    // 出现 fold-in 选择区
    const createBtn = await screen.findByText('新建方法论');
    await user.click(createBtn);
    expect(uploadMock).toHaveBeenCalledWith({ filePath: '/tmp/book.txt', mergeInto: undefined });
  });

  it('选择合并到现有方法论时带 mergeInto 上传', async () => {
    dialogOpenMock.mockResolvedValue('/tmp/book.txt');
    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    await user.click(screen.getByText('上传指导书'));
    const mergeBtn = await screen.findByText(/合并到：方法论A/);
    await user.click(mergeBtn);
    expect(uploadMock).toHaveBeenCalledWith({ filePath: '/tmp/book.txt', mergeInto: 'custom_a' });
  });
});
```

注意：beforeEach 中的 `await import` 语法在 vitest beforeEach 里可用（async beforeEach）；实现时若 lint 报错改为在 describe 外用模块级 import 后 vi.mocked。卡片合并标注测试（`merge_into_methodology_id` 非空显示"已合并"字样）可作为第三条用例，模式同 T6 的卡片测试。

- [ ] **Step 2: 运行测试确认失败**

Run: `cd src-frontend && npx vitest run -- GuidebookDistillationPanel 2>&1 | tail -5`
Expected: FAIL——无"新建方法论"/"合并到："按钮；uploadMock 调用形状不符。

- [ ] **Step 3: 类型与 hook**

`types/guidebook-distillation.ts` 的 `GuidebookListItem` 追加：

```typescript
  merge_into_methodology_id: string | null;
```

`useGuidebookDistillation.ts` 的 `useUploadGuidebook` 改为：

```typescript
export function useUploadGuidebook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (input: { filePath: string; mergeInto?: string }) => {
      return await loggedInvoke<string>('upload_guidebook', {
        file_path: input.filePath,
        merge_into: input.mergeInto ?? null,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
    },
  });
}
```

方法论清单数据源：复用 `useAllMethodologies`（`@/hooks/useMethodologies`），过滤 `m.is_custom && m.enabled` 得到合并候选。

- [ ] **Step 4: Panel 合并选择 UI + 卡片标注**

`GuidebookDistillationPanel` 组件内：

state 追加：

```tsx
  const [pendingFile, setPendingFile] = useState<string | null>(null);
```

`handleUpload` 改为：选完文件后不直接上传——若存在启用的自定义方法论，存 `pendingFile` 展示选择区；否则直接 `doUpload(filePath)`：

```tsx
  const doUpload = async (filePath: string, mergeInto?: string) => {
    try {
      const guidebookId = await uploadMutation.mutateAsync({ filePath, mergeInto });
      setSelectedId(guidebookId);
      toast.success(mergeInto ? '上传成功，开始增量融合...' : '上传成功，开始提炼...');
    } catch (error) {
      toast.error(`上传失败: ${extractMessage(error)}`);
    }
  };

  const handleUpload = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{ name: '指导书', extensions: ['txt', 'pdf', 'epub'] }],
      });
      if (selected && typeof selected === 'string') {
        if (customMethodologies.length > 0) {
          setPendingFile(selected);
        } else {
          await doUpload(selected);
        }
      }
    } catch (error) {
      toast.error(`上传失败: ${extractMessage(error)}`);
    }
  };
```

其中 `customMethodologies` 来自方法论清单 hook 过滤 `m.is_custom && m.enabled`。

选择区 JSX（上传按钮区之后、列表之前；`pendingFile` 非空时渲染）：

```tsx
      {pendingFile && (
        <div className="mb-6 max-w-3xl rounded-xl border border-cinema-gold/30 bg-cinema-gold/5 px-4 py-3 space-y-3">
          <p className="text-sm text-gray-300">
            已选择《{pendingFile.split(/[\\/]/).pop()}》——提炼为新方法论，还是合并进现有方法论？
          </p>
          <div className="flex flex-wrap items-center gap-2">
            <button
              onClick={async () => {
                const f = pendingFile;
                setPendingFile(null);
                await doUpload(f);
              }}
              className="px-3 py-1.5 rounded-lg bg-cinema-gold/20 text-cinema-gold text-sm hover:bg-cinema-gold/30 transition-colors"
            >
              新建方法论
            </button>
            {customMethodologies.map(m => (
              <button
                key={m.id}
                onClick={async () => {
                  const f = pendingFile;
                  setPendingFile(null);
                  await doUpload(f, m.id);
                }}
                className="px-3 py-1.5 rounded-lg border border-cinema-700 text-gray-400 hover:text-white hover:border-cinema-600 transition-colors text-sm"
              >
                合并到：{m.name}
              </button>
            ))}
            <button
              onClick={() => setPendingFile(null)}
              className="px-3 py-1.5 rounded-lg text-gray-500 hover:text-gray-300 text-sm transition-colors"
            >
              取消
            </button>
          </div>
        </div>
      )}
```

卡片标注：`GuidebookCard` 的标题行下方（作者/字数行内）追加：

```tsx
            {guidebook.merge_into_methodology_id && (
              <span className="text-xs text-cinema-gold/70">增量融合</span>
            )}
```

- [ ] **Step 5: 运行测试确认通过 + 全量回归**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | grep -E "Tests|passed"`
Expected: tsc 通过；≥401 passed / 3 skipped

- [ ] **Step 6: Commit**

```bash
git add src-frontend/src/types/guidebook-distillation.ts \
        src-frontend/src/hooks/useGuidebookDistillation.ts \
        src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx \
        src-frontend/src/components/guidebook-distillation/__tests__/GuidebookDistillationPanel.test.tsx
git commit -m "feat: 前端 fold-in 上传选择（新建/合并）与卡片增量融合标注"
```

---

## 依赖顺序

```
U1 (V126+参数) → U2 (fold-in 流水线) → U5 (前端 UI)
U3 (校验器) —— 独立（U1 后即可），接入点在 run_distillation
U4 (SKILL.md 导出) —— 完全独立
推荐执行顺序：U1 → U2 → U3 → U4 → U5
```

## 全量回归清单

| 检查 | 命令 | 预期 |
|---|---|---|
| Rust 单测 | `cd src-tauri && cargo test --lib` | ≥1286 passed / 2 ignored |
| Clippy | `cd src-tauri && cargo clippy --lib` | 零新增警告 |
| Fmt | `cd src-tauri && cargo +nightly fmt -- --check` | 通过 |
| TS 类型 | `cd src-frontend && npx tsc --noEmit` | 通过 |
| Vitest | `cd src-frontend && npx vitest run` | ≥401 passed / 3 skipped |
| 架构守卫 | `python3 scripts/architecture_guard.py` | 通过 |
