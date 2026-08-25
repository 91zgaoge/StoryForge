# StoryMoss Agent 指南

> 本文件包含 AI 助手需要了解的项目背景、编码风格、工具配置与强制构建规则。

## 项目背景

**StoryMoss (草苔)** — AI 辅助小说创作桌面应用

- **项目根目录**: `/Users/yuzaimu/projects/StoryMoss`
- **版本**: v0.54.0
- **GitHub**: https://github.com/91zgaoge/StoryMoss
- **技术栈**: Tauri 2.4 + Rust 1.95.0 + React 18 + TypeScript 5.8 + Vite 6 + SQLite + LanceDB
- **双界面**: 幕前 `/frontstage.html`（沉浸式写作），幕后 `/index.html`（工作室管理）

## 关键教训（必读）

> 完整档案见 `docs/archive/LESSONS_LEARNED.md`。以下是代价最高的一条，任何涉及启动流程/`State`/窗口创建的改动前必读。

**tauri setup 建窗顺序竞态（v0.33.5 根治，Windows 启动闪退 BEX64/c0000409）**：

- tauri 的 `app::setup()` **先创建 `tauri.conf.json` 配置窗口、后调 `.setup()` 闭包**。Windows 上 WebView2 环境创建会泵消息循环数秒，前端加载完立即发 IPC，若此时 `State` 尚未 `manage()` → `state() called before manage()` panic → 发生在 WebView2 COM 回调（`extern "C"`）内无法解退 → 进程直接 abort，无日志。
- **铁律**：任何 `State` 必须在第一个窗口/WebView 创建前 `manage()`；配置窗口一律 `create: false`，由 setup 末尾在所有状态就绪后用 `WebviewWindowBuilder::from_config` 显式创建（见 `src-tauri/src/app.rs`）。
- `extern "C"` 边界（COM 回调/WNDPROC/WebView IPC）内的代码必须不 panic。
- 诊断 Windows GUI 崩溃无日志时：临时切控制台子系统（去掉 `windows_subsystem = "windows"`）从终端拿 panic 消息；项目已内置启动面包屑（`startup_trace.rs`）与早期 panic hook。

## 编码风格

- **Rust**: `snake_case`，`Result<T, E>`，异步 `async/await`，数据库 `rusqlite` + `r2d2`。
- **TypeScript**: `camelCase`，函数组件 + Hooks，Zustand 状态管理，TanStack Query 调用后端。
- **AI 原生组件**: `src-frontend/src/components/ui/ai/`（P1 生成体验：AiLoading/AiThinking/AiStreamingText/AiPromptBar/AiApprovalCard；P2 代理与任务：AiContextCards/AiToolChips/AiRecommendationCard/AiTaskRows；P3 数据展示：AiSearchList/AiCodeBlock/AiDiffTable/AiFilterTable/AiRecordsTable/AiInsightCards），只引用 `--ai-*` 语义令牌（幕后 tokens.css / 幕前 frontstage.css 各自定义），不写死颜色；tint 缺口用 color-mix 内联零扩令牌，契约现为 17 变量（P4 新增 `--ai-on-accent`；v0.46.0 起跟随当前窗主题 `onAccent`，不再写死 #ffffff）；动画用 tailwind.config.js 注册的 ai keyframes 工具类；受控组件，禁止引入自运行演示逻辑；组件内嵌私有动效/图表（如 AiInsightCards 的 MiniLineChart）不复用为公共 API。v0.49.1 卸掉幕前划词浮条 `AiSelectionActions`。

## 开发命令

```bash
# 前端开发服务器
cd src-frontend && npm run dev

# 启动 Tauri 桌面应用
cd src-tauri && cargo tauri dev

# 构建生产版本
cd src-tauri && cargo tauri build

# 测试与检查
cd src-tauri && cargo test --lib
cd src-frontend && npx tsc --noEmit
npx vitest run
npm test                              # Playwright E2E
node scripts/cdp-inspect.js           # CDP 截图
```

## Pre-commit 格式守卫

仓库内置 `.githooks/pre-commit`：提交前自动检查本次 staged 的 Rust（`cargo +nightly fmt -- --check`）与前端（`prettier --check`）代码是否已格式化，未格式化则拒绝提交，对齐 CI 的 fmt 检查。

- **首次克隆后启用**：`git config core.hooksPath .githooks`
- **行为**：仅检查本次 `git add` 进来的 `.rs` / `.ts` / `.tsx` / `.css` / `.json` 代码文件，纯文档/配置提交不受影响；失败时打印 diff 并给出修复命令。
- **修复**：按提示执行 `(cd src-tauri && cargo +nightly fmt)` 或 `(cd src-frontend && npm run format)`，再 `git add -u && git commit`。
- **紧急绕过**：`git commit --no-verify`（仅限紧急情况，CI 仍会兜底检查）。

## 强制构建规则（用户级）

1. **每次修改代码后**：先推送到 GitHub，触发 GitHub Actions 全平台构建。
2. **本地构建仅在用户明确要求时执行**：推送后由 GitHub Actions 负责全平台构建（macOS `.dmg` / Windows `.exe`+`.msi` / Linux `.AppImage`+`.deb`）。**除非用户明确要求「构建」/「打包」/「生成本地安装包」，否则不要在本地执行 `cargo tauri build`**——本地 `cargo test --lib` / `cargo check` / `tsc` / `vitest` 等验证命令照常运行，仅省略耗时的打包构建。此规则为用户级永久指令，优先级高于本节其它条目。
3. **版本号统一**：`Git tag`、`Cargo.toml`、`src-tauri/tauri.conf.json`、`src-frontend/package.json` 必须一致。
4. **每次推送必须更新** `README.md` 与以下文档：`CHANGELOG.md`、`AGENTS.md`、`PROJECT_STATUS.md`、`ROADMAP.md`、`ARCHITECTURE.md`、`TESTING.md`、`docs/USER_GUIDE.md`。
5. **版本标签**：每次推送使用新 tag，禁止 force push 覆盖已有 tag。
   ```bash
   git tag -a vX.Y.Z -m "..." && git push origin vX.Y.Z
   ```
6. **网站 Release 保留策略**：`.github/scripts/upload-releases-ftp.mjs` 每次上传后会自动清理 `/releases` 目录，仅保留最近 5 个版本的安装包（`RELEASE_RETENTION_COUNT=5`，可通过环境变量覆盖），防止服务器空间不足。禁止删除 `latest.json` 与无版本号文件（如 `StoryMoss_aarch64.app.tar.gz`）。v0.30.50 起上传**前**也会先清理（磁盘满 552 自愈——旧版仅上传后清理，磁盘已满时清理永远轮不到）；磁盘满等紧急情况可手动运行 `Cleanup Releases` 工作流（`.github/workflows/cleanup-releases.yml`，workflow_dispatch，可选保留数），它调用脚本的 `--cleanup-only` 模式，不上传只清理。
7. **网站下载页内容及时同步**（用户级永久指令）：落地页（`landing/`）下载区**运行时**从 `https://storymoss.top/releases/latest.json` 拉取版本号并拼出下载链接（`landing/src/hooks/useLatestRelease.ts`），因此每次发版后下载页版本号与链接**自动跟随最新 release，无需重新部署落地页**。注意：发版（tag push）**不**触发 `deploy-landing.yml`（它只在 `landing/**` 变更时构建部署），运行时 fetch 才是保持下载页新鲜的机制。两条强制维护义务：
   - **兜底版本必须随发版 bump**：`useLatestRelease.ts` 的 `FALLBACK_VERSION` 必须与 `Cargo.toml` / `src-frontend/package.json` 的版本号同步更新--这是 fetch 失败（离线/服务器故障）时下载链接仍指向有效版本的最后一道防线，否则兜底链接会指向已被保留策略删除的旧版本而 404。
   - **文件名规律变更时必须校验**：`buildReleaseUrls` 内的 bundle 命名（`StoryMoss_{version}_aarch64.dmg` / `_x64_zh-CN.msi` / `_amd64.AppImage`）在 Tauri bundle 命名或语言包变更时需重新对照线上 `latest.json` 核对。
8. **实现后核验功能和设计，持续迭代直到可上线**（用户级永久指令）：写完代码、勾完计划、提交/发版都不算完成。必须对照设计不变量/契约/验收指标与用户可点路径核验，发现缺口就修再核验，循环直到可上线。未跑通设计验收探针，不得宣称症状已修复。详见 `.cursor/rules/verify-until-shippable.mdc`。

## 提交信息格式

```
<type>: <subject>

type:
  feat / fix / docs / style / refactor / test / chore
```

## 重要文档

- [README.md](./README.md)
- [docs/USER_GUIDE.md](./docs/USER_GUIDE.md)
- [ARCHITECTURE.md](./ARCHITECTURE.md)
- [TESTING.md](./TESTING.md)
- [CHANGELOG.md](./CHANGELOG.md)
- [ROADMAP.md](./ROADMAP.md)
- [docs/archive/AGENTS_HISTORY.md](./docs/archive/AGENTS_HISTORY.md) — 完整历史版本记录
- [docs/archive/LESSONS_LEARNED.md](./docs/archive/LESSONS_LEARNED.md) — 项目修复过程中积累的经验教训与反模式

## 当前编译状态

- `cargo check` ✅ 零错误
- `cargo test -p storymoss` ✅ 1542 passed / 2 ignored（+16 原生 tools）
- `npx tsc --noEmit` ✅
- `npx vitest run` ✅ 602 passed / 3 skipped
- `npx playwright test` ✅ 本版未重跑 E2E
- `cargo +nightly fmt` ✅
- `cargo clippy --lib` ✅ 本版未重跑
- `npm run format:check` ✅
- `python3 scripts/architecture_guard.py` ✅

## 最近完成的功能

### v0.54.0 - Agency ToolLoop 原生 function calling

创世/管理/编辑的 ToolLoop 把角色白名单工具以 JSON Schema 发给模型；优先执行原生 `tool_calls`，没有则回退现网文本 JSON action。续写 `write_beat_once` 仍是单次 `complete()`，请求不带 tools。不改三档路由、不把主创拉回 ToolLoop。

- **验证**：`cargo test --lib` 1542 passed / 2 ignored（+16）。
- **契约**：`native_tool_calls_preferred_over_text_json`；`text_json_action_still_parses_when_tool_calls_empty`；`generate_request_omits_tools_field_when_none`；`tool_specs_for_role_producer_is_json_schema`；`continue_beat_complete_does_not_require_tools`；既有 tool_loop mock 只实现 `complete` 仍绿。
- **未关闭**：真机须再跑创世/资产路径看管理 Agent 是否少一轮解析失败；**不得宣称 ToolLoop JSON 熔断或续写质量已修复**。第二至四期（半卡展开 / 节拍卡冻结 / 续写对错范例）未做。

### v0.53.6 - 已写完的行刺/死亡不再被续写重演

大堂行刺、苏会山与明成公主已死之后，节拍卡仍把死人当在场、下一拍仍是行刺，主创会把刺杀再写一遍。本版：死人退出行动阵容；已完成高潮不再当下一拍；末句锚点与探针拦重演。

- **验证**：`cargo test --lib` 1526 passed / 2 ignored（+7）。
- **契约**：`wedding_climax_marks_king_and_princess_dead_not_son`；`wedding_climax_dead_are_not_acting_cast_or_conflict`；`compile_next_node_skips_stab_already_written_in_prose`；`probe_rejects_replay_of_completed_stab`。
- **未关闭**：须用「飞身扑上」同一开头再跑真机；**不得宣称唱反调/续写质量已修复**。

### v0.53.5 - 大纲重写弹出确认框（确认 / 取消 / 重写）

生成后续故事大纲 / 场景大纲后，幕前弹出可编辑对话框。确认才写入幕后；取消废弃；重写再生成一轮。纸面不改。

- **验证**：`cargo test --lib` 1519 passed / 2 ignored（+2）；`npx vitest run` 602 passed / 3 skipped（+5）。
- **契约**：`execute` 确认前不写库；`confirm_persists_user_edited_outlines_as_given`；幕前确认/取消/重写。
- **未关闭**：须用原句再跑真机；不得宣称续写质量已修复。

### v0.53.4 - 「写后续的故事大纲」不再进续写正文

真机该句走 Agency Append。形状检测曾要求「正文」；现写/生成的宾语是大纲即覆盖续写兜底。

- **验证**：`cargo test --lib` 1517 passed / 2 ignored（+2）。
- **契约**：`looks_like_write_subsequent_outlines_without_prose_keyword`；`write_subsequent_outlines_is_not_append_continue`。
- **未关闭**：须用原句再跑真机；不得宣称续写质量已修复。

### v0.53.3 - 按正文重写大纲在幕前弹出落库预览

真机「根据正文内容重新写后续的故事大纲，同时生成场景大纲」已写库，幕前只闪 toast。现把大纲正文弹「已按正文重写设定」；后续指令从章末往下写。

- **验证**：`cargo test --lib` 1515 passed / 2 ignored（+3）；`npx vitest run` 597 passed / 3 skipped。
- **契约**：`persist_summary_includes_written_outline_text`；`parse_targets_subsequent_story_and_generate_scene`；`refresh_prompt_asks_for_next_beat_not_opening_recap`；幕前弹窗含【故事大纲】/【场景大纲】，纸面不变。
- **未关闭**：须用原句再跑真机看弹窗；不得宣称续写质量已修复。

### v0.53.2 - 键值散文与场景大纲也能按正文重写

真机 Gemma 返回 `story_outline:… scene_outline：…` 无 JSON。现解析中英冒号键值；仅场景大纲也可 salvage。

- **验证**：`cargo test --lib` 1512 passed / 2 ignored（+7）。
- **契约**：真机两段原文可解析；「根据正文重新生成下一章」不是本作业。
- **未关闭**：须用换说法 / 场景大纲原句再跑真机；不得宣称续写质量已修复。

### v0.53.1 - 按正文重写不再因对象 JSON / 散文丢掉

真机 Gemma 4 `agency_producer` 1194 token / 82 字。`parse_refresh_payload` 接受对象大纲与中文键；仅故事大纲时 salvage 散文。

- **验证**：`cargo test --lib` 1505 passed / 2 ignored（+6）。
- **契约**：对象 `story_outline` 经 `normalize_outline`；短垃圾仍 `parse_fail` 不写库。
- **未关闭**：须用同一句再跑真机；不得宣称续写质量已修复。

### v0.53.0 - 按正文重写生产资产（不改正文）

幕前「将故事大纲按照现有正文重新写过」只更新 `story_outlines`，纸面不变。同一路由可按指令改角色 / 世界观 / 当前场大纲。`smart_execute` 在 Agency Append 之前走 `run_asset_refresh`（Producer 一次 JSON，Tool 档，无 writer）。

- **验证**：`cargo test --lib` 1499 passed / 2 ignored（+19）；`npx vitest run` 597 passed / 3 skipped（+1）。
- **契约**：`parse_asset_refresh_targets`；`apply_asset_refresh_override` 非续写/非 prose；persist 不碰 `scenes.content`；金敏秀不进表；`user_created` 情感核保留；ingest 可精炼；场景前缀 + `【当前场大纲】`；`result_kind=asset_refresh` 不进编辑器。
- **未关闭**：真机须用同一开头输入原句，看幕后大纲变、幕前字数不变；不得宣称续写质量/唱反调已修复。

### v0.52.0 - 续写拍级分层与准入轨迹（章末近文 1500）

对照 OpenViking 只引进分层加载与轨迹，不嵌其进程。章末近文窗口 500→1500，避免刚写完的债被漏掉；大纲/配额点名的角色仍准入；在场与冲突双方用全卡，其余准入半卡；故事大纲回流只留核心冲突+最近 5 个转折。日志写出「为什么是这几人」。

- **验证**：`cargo test --lib` 1480 passed / 2 ignored（+7）。
- **契约**：`prior_tail_for_cast` 1500；`mentioned_from_continue_tasks`；`format_admission_trace`；L1 无情感内核；`cap_story_outline_content` 最多 5 个转折点。
- **未关闭**：真机须用同一开头再点续写，看日志 `continue_assets: shot=1500`；不得宣称续写质量/唱反调已修复。

### v0.51.6 - 工具档/后台档不再被创作模型挤到同一台机

幕后已把主创/管理/回流分到 Qwren127、Gemma 4、Qwen 3.8，续写仍全打到 Qwren127。根因：`generate()` 在角色置顶之后又把当前活跃模型抬回链头。

- **验证**：`cargo test --lib apply_active_front` 3 passed。
- **契约**：工具档/后台档 `apply_active_model_front` 不得盖掉已指定模型；创作档仍置顶。
- **未关闭**：真机须用同一三档再点续写，看管理/回流是否打到 Gemma / Qwen 3.8；不得宣称续写质量已修复。

### v0.51.5 - 进行中的续写不再弹前台中断卡

底栏已在「Agency 续写中」时，二次点击/自动续写撞上同一 run，不再盖住纸面。等底栏结束或点底栏取消即可。

- **验证**：vitest 弹窗对 `active_run` 渲染空；FrontstageApp 不出现「需要您先处理」。
- **契约**：`isActiveCreativeRunConflict` 的弹窗路径 return null；二次点击不 `setShowInterruptionModal`。
- **未关闭**：真机须再点续写确认不再弹卡；不得宣称续写质量已修复。

### v0.51.4 - 「已有创作任务」不再打发去设置

二次续写撞上正在进行的 Agency run 时，弹窗曾写「需要您先处理 / 前往设置」。现改为「正在续写中」：等底栏结束，或取消当前续写。二次点击不再清掉真正那次生成状态。

- **验证**：`npx vitest run` 596 passed / 3 skipped（+3）；`cargo test --lib` 1470 passed / 2 ignored。
- **契约**：`AgentInterruptionModal` 不渲染「前往设置」；`isActiveCreativeRunConflict` 认 `active_run` 字段。
- **未关闭**：真机须再点续写确认弹窗文案；不得宣称续写质量已修复。

### v0.51.3 - 续写规划污染不再把 600 秒耗尽

v0.51.2 清空节拍卡泄露后立刻重试。Qwren127 用 390s 写出大纲归纳规划，净化后再开一轮，前端 600s 取消，网关还点下一候选。本版：大纲注入前切断规划；剩余 <90s 不重试；取消即停候选链。

- **验证**：`cargo test --lib` 1470 passed / 2 ignored（+4）。
- **契约**：`condense_outline_strips_planning_dump_glued_to_turning_point`；`writer_retry_has_time_requires_ninety_seconds`；`candidate_chain_stops_on_cancellation_not_timeout`；`test_detect_and_strip_bare_cot_outline_planning_dump_returns_empty`。
- **未关闭**：真机须用同一开头再点续写；不得宣称续写质量/唱反调已修复。库内旧规划大纲靠读取时切断，不自动改表。

### v0.51.2 - 续写切断节拍卡/约束规划泄露

本地 Qwen 主创把节拍卡、状态网、约束清单写进正文（有时接在复述已有场面之后）。`detect_and_strip_bare_cot` 增加节拍卡行话全文切断；规划在文首则清空触发重试。续写 system 禁止输出任务分析。

- **验证**：`cargo test --lib` 1466 passed / 2 ignored（+3）；既有 DeepSeek CoT 用例仍绿。
- **契约**：`test_detect_and_strip_bare_cot_qwen_beat_card_dump_returns_empty`；`test_detect_and_strip_bare_cot_strips_beat_card_after_prose`；`test_sanitize_novel_output_drops_qwen_beat_card_leak`。
- **未关闭**：真机须用同一开头再点续写；不得宣称续写质量/唱反调已修复。GitNexus 索引在 StoryForge→StoryMoss 改名后需 `node .gitnexus/run.cjs analyze`。

### v0.51.1 - 幕前取消键去掉系统原生凸起

生成中底栏 X 被 macOS WKWebView 画成 Aqua 灰凸块。取消键与发射键加 `appearance-none border-0 shadow-none`；取消键透明底 + 顶栏同款陶土 hover；flush 路径 CSS 双杀 UA 按钮。

- **验证**：`npx vitest run` 取消键 / 发射键 `appearance-none` 契约。无 Rust 逻辑变更。
- **契约**：`FrontstageBottomBar` 取消生成无脉冲红块且 `appearance-none`；`AiPromptBar` 发射键去掉系统原生按钮外观。

### v0.51.0 - 手写/粘贴正文触发三角色观察

幕前写满一段并停手 30 秒后，代理工作室会出现「观察」run：管理回流资产、主创编译当前场大纲与下一拍（不改正文）、编辑审查只写审查区。与自动分章同一空闲窗口；该场多出 ≥200 字才重跑；创世/续写进行中则让路只做 Ingest。观察不用 `running`，以免挡住点续写。

- **验证**：`cargo test --lib` 1463 passed / 2 ignored（+9）；`npx vitest run` 592 passed / 3 skipped（+1）。
- **契约**：`should_observe_requires_two_hundred_char_growth`；`apply_observe_writer_writes_outline_not_prose_or_beats`；`has_blocking_creative_run_ignores_observe`；`observe_editor_label_is_silent_background`；工作室「观察中」轨迹。
- **未关闭**：真机须粘贴 ≥200 字停手 30s 核对时间线；整章替换未多 200 字不重跑；分章新章等下次保存。

### v0.50.2 - 自动分章后章节名跟随章号，续写不再把切走的正文写回

下拉出现「第7章」后面还挂「第6章」，第 3–6 章开头同一段验尸。根因：重排只改 `chapter_number`；续写客户端旧全文比截断后的 DB 长，被当成底稿写回。本版派生标题跟随新号；幕前按章号显示；Append 在 DB 为前缀且多出 ≥200 字时用 DB；V130 修存量标题。

- **验证**：`cargo test --lib` 1454 passed / 2 ignored（+4）；`npx vitest run` 591 passed / 3 skipped（+1）。
- **契约**：`generic_chapter_title_matches_arabic_and_chinese_numerals`；中间章切开旧章标题变 `第4章`；`append_base_prefers_db_when_client_is_pre_split_superset`；`v130_retitles_stale_generic_and_keeps_custom`；`displayChapterTitle` 第6章@8 → 第8章。
- **未关闭**：已重复的第 3–6 章正文不自动删。真机须用同一开头再点续写。

### v0.50.1 - 自动分章后续写不再误报「请先打开一个章节」

第 6 章已打开、文思活跃立刻续写，却弹出 `VALIDATION_FAILED`「请先打开一个章节」。根因：`SCENES_PAGE_SIZE=5`，分章切到第 6 章后分页首页没有新章 scene，`sceneId` 回落 `chapter.id`，Agency Append 在 scenes 表找不到。本版分章补拉 `get_chapter_scenes`；后端 `resolve_append_scene_id` 把 chapter id 解析成关联 scene。

- **验证**：`cargo test --lib` 1450 passed / 2 ignored（+1）；`npx vitest run` 590 passed / 3 skipped。
- **契约**：`append_chapter_id_resolves_to_linked_scene`；分章自动切换 `sceneId === 'scene-2'` 且调用 `get_chapter_scenes`。
- **未关闭**：真机须用同一开头再点续写；不得宣称唱反调已修复。角色表脏行不自动删除；ContextPrioritizer 未接 Agency。

### v0.50.0 - 续写三角色闭环（资产可见 / 当场大纲 / 审查进下一拍）

幕后工作室续写后资产栏空、管理只有 start 没有 done、编辑 revise 不进下一拍。本版：后台 spawn 落活动日志且必有 done；本拍阵容与当前场大纲投影到当前 run 资产栏（可点）；Append 写入 `【当前场大纲】`；未解决 revise 最多 2 条进入下一拍节拍卡。

- **验证**：`cargo test --lib` 1449 passed / 2 ignored（+13）；`npx vitest run` 590 passed / 3 skipped（+2）。
- **契约**：`bg_done_detail_covers_all_exits` / `persist_activity_writes_start_and_done`；`production_change_projects_to_current_run_asset_zone`；`append_writeback` 含 `【当前场大纲】`；`compile_beat_card_injects_prior_run_revise_issues`；工作室资产卡点击 `setCurrentView`。
- **未关闭**：真机须用同一开头再点续写；不得宣称唱反调已修复。角色表脏行不自动删除；ContextPrioritizer 未接 Agency。

### v0.49.1 - 卸掉幕前划词润色浮条

划词弹出的「润色 / 扩写 / 指令」条挡住打字、选区也塌不下去。v0.48.1 改成够长才出仍无用。本版从幕前编辑器卸掉，并删除 `AiSelectionActions`。改写走底部指令栏。

- **验证**：`npx vitest run` 588 passed / 3 skipped（−17）；`tsc` / `format:check` / `architecture_guard.py` 全绿。无 Rust 逻辑变更。
- **契约**：划选长句不渲染 `ai-selection-actions`，也没有润色/扩写/指令按钮。

### v0.49.0 - 续写大纲以正文为真相源

有章节正文时禁止按书名发明角色/大纲；空方法论落库场景结构规范；管理 Agent 熔断改为 salvage + 后台续跑，不挡住续写。未接地书大纲不当下一节点；探针拦截场外开篇。

- **验证**：`cargo test --lib` 1436 passed / 2 ignored（+18）；`npx vitest run` 605 passed / 3 skipped；`tsc` / `format:check` / `architecture_guard.py` / `cargo +nightly fmt` 全绿。
- **契约（设计 §11）**：`title_inventions_dropped_when_absent_from_prose` / `test_materialize_drops_names_absent_from_prose`；`ferdinand_outline_is_not_grounded` / `compile_next_node_ignores_ungrounded_book_outline`；`test_ensure_methodology_default_writes_scene_structure_when_empty`；`test_ensure_assets_with_prose_does_not_require_producer_loop`；`probe_rejects_offshot_pov_opening`。
- **未关闭**：真机须用同一开头再点续写；**不得宣称「唱反调」已修复**。角色表脏行不自动删除；ContextPrioritizer 未接 Agency。

### v0.48.1 - 划词浮条不再挡住手工写作

v0.39.0 划词 AI 操作条（润色/扩写/改写）在任意选区、拖选过程中弹出，自带输入框抢焦点，mouseup 落在浮条上后选区塌不下去。改为 ≥4 字且鼠标松开后才出现；idle 默认无输入框；Esc 收起。

- **验证**：`npx vitest run` 605 passed / 3 skipped（+4）；`tsc` / `format:check` / `architecture_guard.py` 全绿。无 Rust 逻辑变更。

### v0.48.0 - 续写按镜头在场、禁止旧快照覆盖、未确认幽灵先写入

对照 v0.47.0 真机失败（《帝国的烟火》2026-08-16 晨）：在场窗口过大、角色表补位、书大纲未落地当前席、连续续写丢幽灵并用旧快照覆盖。P0：镜头 500 字阵容、去掉补位、下一节点须点名本拍在场者、落库取更长底稿、末句禁复述已完成动作、NewScene 不罚丢人、连续续写先 `appendAiContent` 未确认幽灵。

- **验证**：`cargo test --lib` 1418 passed / 2 ignored（+5）；`npx vitest run` 601 passed / 3 skipped；`tsc` / `format:check` / `architecture_guard.py` / `cargo +nightly fmt` 全绿。
- **未关闭**：真机 8 次须在 0.48.0 上重跑；v0.47.0 真机已失败，不得宣称五症状已修复。角色表脏行不清理；ContextPrioritizer 未接 Agency。

### v0.47.0 - 续写质量闭合（债务/节点/阵容/状态网）

对照 `docs/plans/2026-08-15-continue-quality-closure-design.md` 落地 P0–P3：债务只在正文兑现时刷新；下一节点不回绕；冲突看本拍阵容；写回事实出场与别名点名；续写过短回退保留节拍卡；拍级状态网 + 一次探针重试；CI `eight_beat_append_quality_contract`。NextChapter 装配先算 SceneUpdate 再开写事务，避免 SQLite unlock_notify 死锁。

- **验证**：`cargo test --lib` 1413 passed / 2 ignored（+22）；`tsc` / `format:check` / `architecture_guard.py` / `cargo +nightly fmt` 全绿。无前端逻辑变更。
- **未关闭**：真机 8 次幕前续写未跑，不得宣称三症状已修复；ContextPrioritizer 未接 Agency；不叠更早几章全文。

### v0.46.0 - 传统色主题（纸帘印 + 幕前幕后分选）

十二套写作向传统色替换旧四套。幕前顶栏色点只改纸面；设置页两列分选幕前/幕后。旧 id 自动迁到朱红/群青/藤黄/黛紫。印色即锚色，`--ai-accent-tint` 跟随当前窗。

- **验证**：`npx vitest run` 601 passed / 3 skipped（+11）；`tsc` / `format:check` / `architecture_guard.py` 全绿。无 Rust 逻辑变更。
- **未关闭**：空资产 trim 金标；ToolLoop head 双构造；v0.42.0 §8 真机探针；全界面截图回归。

### v0.45.1 - 续写前文改为开篇+近文双窗

长章续写只带末 800 字，模型看不见本章前面的情节。改为短章全文、长章开篇 600 + 近文 1800；先剥 HTML；「末段已在场」只看近文；预算裁前文时保住章末。

- **验证**：`cargo test --lib` 1391 passed / 2 ignored（+6）；无前端逻辑变更。
- **未关闭**：仍不叠更早几章全文；空资产 trim 金标；ToolLoop head 双构造；v0.42.0 §8 真机探针。

### v0.45.0 - 提示词运行时组装（创世/续写/ToolLoop）

创世首章、续写 `write_beat_once`、ToolLoop 头部改走 `prompts/assembly.rs` 哑拼接器。幕后提示词页场景预览改报 Agency 续写/创世。`board_read`/`board_write`/`asset_query` 自带用法行。内置 agency/writer/scene_outline 残留 `{{ident}}` CI fail-closed。

- **验证**：`cargo test --lib` 1385 passed / 2 ignored（+18）；`npx vitest run` 590 passed / 3 skipped；`tsc` / `format:check` / `architecture_guard.py` 全绿。
- **未关闭**：空资产/空末句 trim 金标未锁；ToolLoop head 双构造路径；v0.42.0 §8 真机探针仍未跑；P3 producer/concept_pack 未接线。

### v0.44.1 - 幕前输入框去掉系统原生描边

v0.44.0 拆掉底栏卡片后，macOS WKWebView 仍给 `<textarea>` 画原生 inset 边，用户看见输入区一圈细线。`AiPromptBar` 加上 `appearance-none border-0 shadow-none`；flush 路径 CSS 双杀 UA 样式。契约测试锁 class，避免再只查外壳。

- **验证**：`npx vitest run` 590 passed / 3 skipped（+1）；`tsc` / `format:check` 全绿。无 Rust 逻辑变更。
- **未关闭**：全界面截图回归未做；v0.42.0 §8 真机探针仍未跑。

### v0.44.0 - 墨纸 / 机械视觉定向进化补齐

对照 `docs/plans/2026-08-14-ink-paper-mechanical-deepen-design.md` 补齐 v0.43.0 未做满的缺口：幕前输入无框、Medium 分文件、纸 chroma / 选区 22%、顶栏 press、暖金内芯同色相、Panel 高光 + 弹簧 500ms、侧栏去金框。

- **验证**：`npx vitest run` 589 passed / 3 skipped（+11）；`tsc` / `format:check` / `architecture_guard.py` 全绿。无 Rust 变更。
- **未关闭**：设计全界面人工目视清单未做成截图回归；v0.42.0 §8 真机探针仍未跑。霞鹜 v1.250 无 Medium 文件，本版用同 tag Bold 映射为 CSS 500（README 已写明）。

### v0.43.0 - 墨纸 / 机械视觉定向进化

幕前输入条 `flush`：一层纸面、陶土淡彩发射、取消去 pulse。霞鹜文楷本地 woff2。幕后色板/阴影收软、press 曲线、EmptyHint。

- **验证**：`npx vitest run` 578 passed / 3 skipped（+22）；`tsc` / `format:check` / `architecture_guard.py` 全绿。无 Rust 变更。
- **未关闭**：设计全界面人工目视清单未做成截图回归；v0.42.0 §8 真机探针仍未跑。

### v0.42.0 - 续写按拍选取创作资产

长篇续写提示词不再全表倾倒。节拍卡当准入名单（≤8 人完整卡）；其余本故事相关角色一行名单；脏名不进；大纲去重；前文只留当前章末 800 字。`to_prompt()` 不动。

- **验证**：`cargo test --lib` 1367 passed / 2 ignored（+13）；无前端逻辑变更。
- **未关闭**：规格 §8 真机探针未跑；`story_outlines` 表内无界追加；ContextPrioritizer 未接 Agency。

### v0.41.2 - 续写 600s 超时：跳过超窗候选、散文失败不再进 tool_loop

推理模型空正文后，网关按窗口跳过装不下的本地模型（如 Gemma 8k）；单章 `write_beat_once` 散文回退失败直接报错，不再 `write_chapter` tool_loop 重烧候选链至前端看门狗。

- **验证**：`cargo test --lib` 1354 passed / 2 ignored（+4）；无前端逻辑变更。
- **未关闭**：本地连接超时仍可能 60s×2；`story_outlines` 膨胀与跨故事角色串入；设计 §13 真机探针未跑。

### v0.41.1 - Agency 续写上线核验加固

对照设计核验 v0.41.0 后补齐：主创 `sanitize_novel_output` + 8% 自重复重试；改写永不选 TimeSliced/TriShot；划词不走 Append；测试环境跳过 finalize LLM 摘要；文思活跃断言 `scene_id`。

- **验证**：`cargo test --lib` 1350 passed / 2 ignored（+5）；`npx vitest run` 556 passed / 3 skipped；`tsc` / `format:check` / `architecture_guard.py` 全绿。
- **未关闭**：设计 §13 连续 8 次幕前续写真机探针（需 LLM），不得宣称四症状已修复。

### v0.41.0 - Agency 唯一续写路径 + 幕前同章追加

创世与幕前/幕后续写只走 Agency 三角色；幕前续写/文思活跃为同章追加（`PersistMode::Append`），划词改写仍走 PlanExecutor。SceneBeatCard 把资产编译成这一拍的硬任务；落库写回出场/冲突/地点；债务按拍计数。切断 TimeSliced/TriShot 续写路由。

- **PersistMode**：Append 接到当前章（需 `scene_id`，增量 ≥200 才落库，返回 `increment`）；幕后「续写一章」仍 NextChapter。LeadWriter 默认单次 `complete()`，Editor `spawn_editor_qc` 后台，装配后立即 `finish_run`。
- **资产强关联**：Bundle 情感四元组+关系；SceneBeatCard 0 LLM Rust 编译；双锚点 writer prompt；写回 `characters_present` / `character_conflicts` / `setting_location`；`BeatCounters` 在 expansion 层（避免 `creative_engine → agency`）。
- **切断旧路由**：`smart_execute` 续写 → Agency Append；`execute_writer` 遇续写/创世 Err；设置 `generation_mode` 仅管改写（auto/fast/full），UI 去掉 time_sliced/tri_shot。
- **验证**：`cargo test --lib` 1345 passed / 2 ignored（+17）；`npx vitest run` 556 passed / 3 skipped（不变）；`tsc` / `format:check` / `architecture_guard.py` 全绿。
- **已知债务**：ContextPrioritizer 未接 Agency 热路径；`characters_present` id/名字混杂旧数据。

### v0.40.0 - AI 原生组件库 P3（数据展示六件套）+ P4（项目收尾）

**beautifului AI 原生组件库收官（P3 数据展示 + P4 收尾）**：6 个数据展示组件适配为受控组件入库 `src-frontend/src/components/ui/ai/` 并替换幕后落点；P4 完成替换残留清理、视觉修正与浅色页令牌化。只引用 `--ai-*` 语义令牌（契约扩至 17 变量），不引新依赖，纯前端无后端改动。

- **P3 数据展示**：AiSearchList（PromptsPanel 搜索计数区）、AiCodeBlock（六文件七处裸 pre/JSON 块）、AiDiffTable（AgencyEval 检查点对比，metrics_json 解析补基准/对比列）、AiFilterTable（UsageStats 分组筛选 + 最近调用表；AiFilterChipsBar 可选接 Logs 级别筛选）、AiRecordsTable（PromptsPanel 分组列表 + AgencyEval 双表）、AiInsightCards（UsageStats/AgencyEval 统计卡，内嵌 MiniLineChart）；AiChat 经勘察关闭（ChatComposer 为 AiPromptBar 严格子集）。
- **P4 清理**：P1-P3 替换残留 TS 13 处 + frontstage 死 CSS 约 40 类；历史死件 8 件（AiSuggestionBubble/AiHintOverlay/HelpPanel/ZenModeExit/useLlmStream/useStudioConfig/hetiAddon/Toggle）。
- **P4 修正与令牌化**：新增 `--ai-on-accent` 令牌替换四组件 text-white 直写；`/N` 透明度修饰符失效 13 处改 color-mix；Tasks 裸 pre → AiCodeBlock；AiDiffTable testid 改 per-row key；AgencyEval/AgencyStudio/AgencyLearning 浅色页令牌化（AgencyLearning 裸表 → AiRecordsTable）。
- **验证**：`npx vitest run` 556 passed / 3 skipped（P3 564 − 死件自带测试 8）；`cargo test --lib` 1328 passed / 2 ignored（无 Rust 改动）；`tsc` / `format:check` / `architecture_guard.py` 全绿。

### v0.39.0 - AI 原生组件库 P1+P2（共 10 组件）+ 保存 UNIQUE 修复

**beautifului AI 原生组件库（P1 生成体验 + P2 代理与任务）**：10 个组件适配为受控组件入库 `src-frontend/src/components/ui/ai/`，逐点接入幕后/幕前落点；只引用 `--ai-*` 语义令牌（双窗口各自定义），不引新依赖，纯前端无后端改动。

- **P1 生成体验**：令牌桥（16 个 `--ai-*` 变量 + tailwind ai 色组/9 动画工具）；AiLoading（幕后 3 处加载指示）、AiThinking（AgencyStudio 当前执行轨迹）、AiStreamingText（幕前幽灵续写，Intl.Segmenter 中文词级分词）、AiPromptBar（幕前底部指令条 + / 命令菜单）、AiApprovalCard（创建向导四选项步骤）；删除幕前死代码 `StreamingText.tsx` + `useStreamingGeneration.ts`。
- **P2 代理与任务**：AiContextCards（PromptCoverageBar 槽位清单）、AiToolChips（Tasks/Skills 筛选条）、AiRecommendationCard（级联改写逐段确认卡）、AiTaskRows（Tasks 任务行外壳）、AiSelectionActions（幕前划词浮条，smartExecute + insertContentAt 选区替换）；frontstage.css 补 `--shadow-float`。
- **保存修复**：幕前保存 UNIQUE 失败（scenes.story_id, sequence_number）根因——自愈补建逻辑在章节已有关联 scene 时盲目 INSERT 重复行、序号被占时硬撞约束；`heal_missing_scene_in_tx` 改为重定向既有关联 scene + 序号被占取 MAX+1。
- **验证**：`cargo test --lib` 1328 passed / 2 ignored（+2）；`npx vitest run` 523 passed / 3 skipped（+68）；`tsc` / `format:check` / `architecture_guard.py` 全绿。

### v0.38.2 - 幕后深色调主题 + 代理工作室实时动态持久化

**幕后主题底座（beautifului AI 原生改造 P0）**：4 套深色调主题（暖金 warm/冷青 cool/琥珀 amber/靛紫 indigo），与幕前色调同 id、同 localStorage key（`storymoss-color-theme`）双向同步。

- **主题定义**：`src-frontend/src/styles/backstageThemes.ts`——`BACKSTAGE_THEME_VARS`（16 个 `--cinema-*`/`--status-*` 变量）+ `backstageThemes` + `applyBackstageTheme`（运行时重写 `documentElement` 同名变量，未知 id 回退 warm）；warm 与 tokens.css 现状值全量一致（零视觉回归）。
- **全局接线**：`src-frontend/src/hooks/useBackstageTheme.ts`——挂载即应用 + storage（跨窗口）/ Tauri `color-theme-changed`（同窗口）双通道；listen unlisten 竞态加 cancelled 标志。在 `App.tsx` 顶层调用一次。
- **设置页入口**：`GeneralSettings.tsx` 的 `ColorThemeSelector`（已 export）——幕前/幕后双预览色点（`theme-swatch-frontstage-/backstage-{id}`），选择即同时 `applyColorTheme` + `applyBackstageTheme`。
- **清理**：删除死代码 `frontstage/hooks/useWritingStyle.ts`（注意：`hooks/useWorldBuilding.ts` 有同名 hook，不受影响）。
- **验证**：vitest 新增 11 项，全套 455 passed / 3 skipped；tsc / prettier 全绿。

**代理工作室实时动态持久化**：v0.38.0 将 agency 事件监听提升到常驻 `App.tsx` 顶层 + 全局 `agencyActivityStore`，接线正确但用户仍看不到实时动态。根因：活动事件（`agency-agent-activity` / `agency-run-progress`）纯内存（Zustand store 无 persist），macOS 隐藏 WKWebView 窗口事件送达不可靠，事件丢失即永久丢失。本版将活动事件持久化到 DB + 前端 3s 轮询，使实时显示不再依赖 Tauri 事件到达隐藏窗口。

- **DB 持久化**：新增 `agency_activity_log` 表（V129 迁移），`emit_activity` / `emit_progress` 在 `app.emit()` 后 `tokio::task::spawn_blocking` fire-and-forget 写 DB（不阻塞创世流程，失败仅 `log::warn!` 不致命）；测试环境（`app_handle=None`）不进入此块。
- **后端命令**：新增 `agency_list_activities` Tauri 命令（`run_id` -> 按 `id ASC` 返回 `Vec<AgencyActivityLogEntry>`，limit 200）。
- **前端轮询**：`AgencyStudio.tsx` 新增 `useQuery(['agency-activities', activeRunId], listActivities, { refetchInterval: 3000 })`，DB 活动事件为主源，live store 事件补充轮询间隔内新事件（按业务键 `role|action|detail` / `phase|status|message` 去重）。
- **live 事件监听保留**：`App.tsx` 事件监听 + `agencyActivityStore` 不变，提供轮询间隔内的即时更新（双保险）。
- **验证**：`cargo test --lib` 1326 passed / 2 ignored（+1：`test_log_and_list_activities`）；`npx vitest run` 455 passed / 3 skipped（无前端测试变更）；`cargo +nightly fmt` / `tsc` / `format:check` 全绿。

### v0.38.1 - 修复续写伏笔账本多字节中文切片 panic（文思活跃模式）

用户报告文思活跃模式续写弹 Fatal：`[TimeSliced] bundle 加载任务失败: ... "end byte index 30 is not a char boundary; it is inside '指' (bytes 29..32)"`。根因：`foreshadowing_service.rs` 构造伏笔账本 title 预览用 `&content[..30]` 按字节切片，中文 content 的 byte 30 落在三字节字符「指」内部 -> Rust UTF-8 panic -> 续写 bundle 加载失败。文思活跃连续续写读伏笔账本（`load_write_time_bundle -> pending/overdue_foreshadowings`），每次必炸。

- **主修复·`foreshadowing_service.rs`**：title 截取从字节语义改字符语义（`chars().count() > 30` + `chars().take(30).collect()`）；伏笔 title 是用户预览，按 30 字符比 30 字节（10 汉字）更合理。
- **同类预防·`post_process.rs`**：两处 `&draft_content[..8000/6000]` 改 `floor_char_boundary`--保留字节预算（控制上下文长度），切点回退最近字符边界。
- **同类预防·`intent.rs`**：JSON 解析失败日志 `&content[..min(200)]` 改 `floor_char_boundary`。
- **回归测试**：`service_ledger_title_multibyte_no_panic`--用报错原文验证 `get_ledger` 不 panic。
- **验证**：`cargo test --lib` 1325 passed / 2 ignored（+1）；`cargo +nightly fmt` ✅。纯 Rust 修复。

### v0.38.0 - 代理工作室实时显示修复与三 Agent 完善

修复幕后代理工作室（AgencyStudio）未打开时创世/续写事件丢失、打开后空白等待的问题——事件监听此前挂在条件挂载的页面上，随卸载销毁。

- **实时显示修复**：agency 事件监听提升到常驻 `App.tsx` 顶层；新增全局 `src-frontend/src/stores/agencyActivityStore.ts`（activities/progress cap 200，对标 backendActivityStore 单例无 persist），页面未开不再丢实时动态，打开即见；跨故事切换时 activeRunId 按 storyId 校正。
- **三 Agent（主创/管理/编辑审计）事件信号补齐**（`agency/coordinator.rs`）：概念/资产/首章/资产补齐/装配的 start/done 全路径配对（含 legacy 与快速路径单点覆盖）；修复 legacy 概念完成信号角色标注（LeadWriter→Producer）；后台质检黑板写入实时推 `agency-board-changed`。
- **前端打磨**：幕前 DETAIL_VERB 补全（概念/装配/资产补齐/资产回流/第N章草稿/审查第N章等）；幕后时间线去重改业务键（role|action|detail|phase|status），同源重复事件不再显示两次。
- **续写熔断不丢稿（测试补齐）**：行为已由 65d90b5（v0.30.30）实现（草稿 ≥600 字符降级放行/<600 丢稿），本档补齐流程级测试。
- **验证**：`cargo test --lib` 1306 passed / 2 ignored（+5）；`npx vitest run` 421 passed / 3 skipped（+17）。

### v0.37.0 - 资产回流：后台资产 agent 对已生成正文生效

修复 IngestPipeline 从正文提取的角色/关系只写 kg 记忆层、续写 writer 只读生产资产表两不相通的问题（且提取 prompt 字段名与 schema 错配、新登场角色被丢弃、Agency 续写路径不跑提取）。

- **提取 prompt 写作级升级**（`resources/prompts/memory/memory_content_analysis.md`）：字段与 schema 严格对齐——角色画像（情感内核/触发/创伤/需求）、双向情感关系、世界观增量（规则/历史/文化）、场景大纲、故事增量（核心冲突/转折点）。
- **新增资产桥**（`src-tauri/src/memory/asset_bridge.rs`）：提取结果 upsert 进生产资产表（characters / character_relationships / world_buildings / scenes.outline_content / story_outlines），新角色自动注册；源感知合并——只精炼机器来源（ingest/agency/auto_placeholder），手工编辑（user_created/manual）永不覆盖。
- **Agency 续写接入**：每章正文落库后后台自动跑提取（`spawn_asset_ingest`，含 KG 持久化）；orchestrator/TriShot 路径经 `run_ingest` 自动生效。
- **并发安全**：per-story 进程内锁 + `BACKGROUND_LLM_SEMAPHORE` 后台串行化；失败不致命，绝不影响正文落库。
- **验证**：`cargo test --lib` 1301 passed / 2 ignored（+14）；`npx vitest run` 404 passed / 3 skipped（无前端逻辑变更）。
- **已知问题（backlog）**：story_outlines 无 source 列（机器提取追加进手写大纲、content 无界增长）；关系按有向去重（反向建第二行）；ingest tokens 不计入 AgencyBudget；agency 取消不传播给已 spawn 的 ingest。

### v0.30.48 - 创世持久化链路审计修复 + issue #13/#14/#15 批量修复

- **v0.30.46 创世正文未即时保存与资产缺失**：前端两条创世路径 `selectChapter(skipContent)` 后补 `setTimeout(flushSceneSave, 0)`（auto-accept 时 sceneId 未就绪导致 flush 被跳过）；`agency/coordinator.rs` 场景装配 create+update 合成单事务 + 空正文校验；`generate_chapter_outline` 写黑板身份 Producer→LeadWriter（修 `scenes.outline_content` 恒 None）；`orchestrator.rs` 创世成功臂回读空正文即报错；`scene_repository.rs` 空串 content 归一 None 防 COALESCE 覆盖；`scene_commands.rs` 吞错上抛；`agency/materialize.rs` 新增 foreshadowing 落库（纯文本/JSON 数组/对象三形态）+ item_type 别名归一化 + characters upsert。
- **v0.30.47 角色谱静默失败 + llm_calls 空表（issue #13/#14）**：`agents/novel_creation.rs` 角色谱/文风/首场景三路径改 `extract_and_sanitize_json` 健壮解析 + 去 unwrap + warn 日志；`llm/service.rs` `prompt[..200]` 字节切 UTF-8 panic（llm_calls 永不落库根因）改 `chars().take(200)`；创世向导三卡片加 `isGenerating` 防重入；`BookDeconstruction.tsx` 4 处 toast 改 `extractMessage`。
- **v0.30.48 向导策略加载误报 + 快速创作空输入（issue #15）**：策略推荐加载中误显「策略加载失败」改为转圈动画；快速创作简介为空时先确认"仅根据标题自由发挥"。

### v0.30.45 - 修复文思活跃模式续写提示词泄露（LLM 思维链泄露到正文）

用户报告"开启了文思活跃模式后，出现提示词泄露问题"--续写返回的不是小说正文，而是 LLM 的思维链（CoT）："这是一个小说续写任务，需要我以专业作者身份..."。四层防线全部失守导致 deepseek-v4 推理模型的 CoT 被当作正文返回。

- **根因 1·`resolve_content` 错误回退（`llm/openai.rs`）**：v0.30.25 假设推理模型可能把实际内容放在 reasoning_content，content 为空时回退。但 reasoning_content 是思维链不是正文。现移除回退，content 为空返回空 + warn。
- **根因 2·`max_tokens: 2048` 太小（`agents/orchestrator.rs`）**：推理模型 CoT 消耗 1500-2500 token，2048 留给正文预算为 0 -> content 空 -> 触发回退。三处 `Some(2048)` -> `Some(4096)`。
- **根因 3·裸 CoT 检测（`agents/orchestrator.rs` `sanitize_novel_output`）**：新增 `detect_and_strip_bare_cot` 纯函数--扫描前 2000 字符非空行，≥3 行命中 CoT 信号词（40+ 个）判定泄露，尝试提取正文起点，找不到返回空。作为 step 0e 插入。
- **根因 4·prompt 禁止输出思考过程（`resources/prompts/writer/`）**：`writer_system.md` + `orchestrator_timesliced_writer.md` 新增"不要输出思考过程/分析/规划"+"禁止以分析性语句开头"。
- **验证**：`cargo test --lib` 1091 passed / 2 ignored（+4）；`npx vitest run` 352 passed / 3 skipped；`cargo +nightly fmt` / `cargo clippy --lib`（539 零新增）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.44 - 修复文思活跃模式续写报"生成过程异常结束，未收到有效内容"

用户报告"开启了文思活跃模式后，出现了报错的诊断信息"。诊断数据显示 LLM（deepseek-v4）成功返回 2460 字符，但前端 `generatedText` 仅剩 3 字符（"正文续"），打字机动画显示 18 字符增长（12->15->18）后被中断，最终弹出"生成过程异常结束，未收到有效内容"。根因：`smartExecuteInFlightRef.current = false` 在 smartExecute resolve 后、内容处理前被提前清除--后台活动同步回调（100ms 防抖）在内容处理期间把 `isGenerating` 置 false，触发安全网 effect（`!isGenerating && smartExecuteNeedDiagnosticRef.current`）误报。`handleRequestGeneration` 的活跃模式分支还错误地走了打字机幽灵文本（3 字符/帧），而非直接 `appendAiContent` 追加到编辑器正文。

- **主修复·`handleRequestGeneration` 提前清除 flight 标志（`FrontstageApp.tsx`）**：移除 smartExecute resolve 后的 `smartExecuteInFlightRef.current = false`。改为在各退出路径统一清除：打字机完成时、displayText 空 bail、background bootstrap、genesis 首章、aborted、active mode 追加后。确保内容处理期间 `isGenerating` 不被后台活动同步干扰。
- **主修复·`handleSmartGeneration` 同类根因（`FrontstageApp.tsx`）**：移除 smartExecute resolve 后的 `smartExecuteInFlightRef.current = false`，与 `handleRequestGeneration` 同理。在各内容交付路径（aborted / isAlreadyPresent / isBootstrapCompleted&&delivered / active mode append / isFirstChapterReady / ghost text）统一清除 `smartExecuteInFlightRef` + `smartExecuteNeedDiagnosticRef`；`finally` 块在 `setIsGenerating(false)` 之后兜底清除 flight 标志防泄漏。
- **活跃模式直追（`FrontstageApp.tsx` `handleRequestGeneration`）**：在打字机之前新增活跃模式分支--`wensiModeRef.current === 'active'` 时直接 `appendAiContent(displayText, 'auto')` + 清除两标志 + `setIsGenerating(false)`，绕过打字机（与 `handleSmartGeneration` 活跃模式行为一致）。
- **回归测试（`FrontstageApp.wensi-active.test.tsx`）**：+2 测试。①活跃模式续写内容直接追加到编辑器正文，不走打字机幽灵文本；②`smartExecuteNeedDiagnosticRef` 被清除，不触发"生成过程异常结束"诊断。测试 mock 修复：RichTextEditor mock 的 `getHTML()` 此前返回 stale `props.content`，改为用 mutable ref 跟踪编辑器内部 HTML（对齐真实 TipTap `getHTML` 返回实时 DOM 行为）。
- **验证**：`npx tsc --noEmit` ✅；`npx vitest run` 352 passed / 3 skipped（+2）；`cargo +nightly fmt` / `cargo clippy --lib`（538 零新增）/ `architecture_guard` / `npm run format:check` 全绿。纯前端修复，无 Rust 变更。

### v0.30.43 - 修复续写内容丢失根因：flushSceneSave 读取滞后的 latestContentRef + onChapterUpdated 覆写未保存内容

v0.30.33/v0.30.34 的关闭前 flush + 序列化持久化仍未能完全解决续写内容丢失。深入诊断定位两个根因：①`flushSceneSave` 读取 `latestContentRef.current` 而非编辑器实际 HTML--RichTextEditor 的 `onChange` 有 200ms 防抖（`htmlDebounceRef`），`latestContentRef` 可能比编辑器实际内容滞后 200ms，关闭应用/切换章节时若读 `latestContentRef`，最后 200ms 内的输入会丢失；②`onChapterUpdated`（后台 auto_commit 触发）用 DB 旧内容 `setContent` 覆写编辑器但不更新 `latestContentRef`，若用户有尚未落库的输入（防抖窗口内），编辑器被 DB 旧内容覆写后用户再输入，旧输入从编辑器消失且 `latestContentRef` 被新输入覆盖，造成不可逆丢失。

- **主修复·flushSceneSave 直接读编辑器（`FrontstageApp.tsx`）**：`flushSceneSave` 从 `editorRef.current?.getHTML()` 读取编辑器实际 HTML，`editorRef` 不可用时回退 `latestContentRef.current`；读后回写 `latestContentRef.current = content` 保持一致。覆盖关闭前 flush（`frontstage-flush-requested` 事件）、章节切换（`selectChapter`）、AI 追加（`appendAiContent`）、修稿（`handlePipelineRefine`/`onReviseResult`）全部 flush 路径。消除 200ms HTML 防抖窗口导致的内容丢失。
- **Root Cause #2·onChapterUpdated 保护未保存内容 + 同步 latestContentRef（`FrontstageApp.tsx`）**：`onChapterUpdated` 在 `setContent(formatted)` 前新增守卫--若 `latestContentRef`（会被 flush 保存的内容）非空且与 DB 内容不同，说明用户有尚未落库的输入（200ms HTML 防抖窗口内或 2000ms 自动保存防抖未出火），此时绝不用 DB 旧内容覆写编辑器，直接 `return` 跳过；`setContent` 后补 `latestContentRef.current = formatted` 同步刷新后的内容，使后续 flush 保存 onChapterUpdated 刚加载的 DB 内容而非旧值。
- **附带·setContent('') 清空 latestContentRef（`FrontstageApp.tsx`）**：无章节时 `setContent('')` 后补 `latestContentRef.current = ''`，避免 flushSceneSave 保存已清空的旧内容。
- **验证**：`cargo test --lib` 1087 passed（无 Rust 变更）；`npx tsc --noEmit` ✅；`npx vitest run` 350 passed / 3 skipped（+1：close-flush 保存编辑器实际内容而非滞后 latestContentRef 回归测试）；`cargo +nightly fmt` / `cargo clippy --lib`（538 零新增）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.42 - 修复世界观生成失败（LLM 返回 markdown 代码块包裹的 JSON + 未转义引号 + 静默失败 + prompt 字段名不匹配）

issue #14 用户报告"世界观生成失败，请重试"，但日志显示 LLM API 调用成功返回内容（7636 字符），失败发生在下游 JSON 解析且完全无错误日志。根因三层：①模型将 JSON 包裹在 ` ```json ... ``` ` 代码块中、或在字符串值内直接换行/使用裸双引号，`serde_json::from_str` 静默失败；②`novel_creation.rs` 严格解析全量响应（含围栏）直接失败，`agency/coordinator.rs::parse_lenient` 用 `rfind('}')` 会被尾部杂散 `}` 误导且无法修复字符串内裸换行；③`novel_creation_world_options.md` prompt 要求"concepts 数组"但代码读 `parsed["world_buildings"]`，即使解析成功也找不到数组；prompt 缺少格式约束。

- **Fix 1·`parse_lenient` 复用健壮提取器（`agency/coordinator.rs`）**：`parse_lenient` 改为先调 `crate::narrative::extract_and_sanitize_json`（剥离 markdown 围栏 / 推理链、括号深度匹配跳过尾部杂散 `}`、修复字符串内未转义换行、移除 BOM / 注释 / 尾随逗号），失败再回退旧的首尾花括号截取。覆盖 agency 全部 JSON 解析路径（concept_pack / producer_depth_assets 世界观 / editor 裁决 / retrieval plan）。`extract_and_sanitize_json` 已存在于 `narrative` 且被 memory/analysis 等模块使用，`agents` 已有 `crate::narrative::strip_reasoning_blocks` 先例，无新跨层依赖。
- **Fix 2·`novel_creation.rs` 世界观选项解析健壮化**：提取 `parse_world_options_response` 纯函数（便于单测，无需 mock LlmService），先 `extract_and_sanitize_json` 剥离围栏再 `serde_json::from_str`；解析失败时 `log::warn!` 记录错误 + raw 长度 + 200 字片段（此前完全静默）；`world_buildings` 缺失时错误信息明确指出"缺少 world_buildings 数组"；元素反序列化 `unwrap` 改 `map_err` 不再 panic。
- **Fix 3·prompt 字段名修正 + 格式约束（`novel_creation_world_options.md` + `narrative_world_building_generate.md`）**：`novel_creation_world_options.md` "concepts 数组" -> `world_buildings`（与代码一致）并补全完整 schema 示例；两份 prompt 新增格式约束--禁止 markdown 代码块包裹、字符串值内引用用中文引号「」或转义 `\"`、禁止 JSON 外输出任何文字。
- **验证**：`cargo test --lib` 1087 passed / 2 ignored（+5：parse_lenient 剥围栏/修复裸换行 +2，novel_creation 解析 +3）；`npx tsc --noEmit` ✅；`npx vitest run` 349 passed / 3 skipped；`cargo +nightly fmt` / `cargo clippy --lib`（538 零新增）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.41 - 修复续写内容被假阳性去重静默丢弃（模型回显指令 + 短文本假阳性 + 内容丢失）

用户诊断报告显示续写生成时 LLM（deepseek-v4）成功返回 2511 字符，但前端仅显示 6 字符（"续写\n黑暗。"），随后报"生成过程异常结束，未收到有效内容"。根因链：①模型在生成内容开头回显用户指令"续写"（非正文）；②打字机动画首帧仅 3 字符（"续写\n"），归一化后 2 字符"续写"几乎必然出现在 9656 字已有正文中；③`isTextDuplicate` 假阳性返回 true，`setGeneratedText` 跳过赋值并 `markAccepted` 存入 2 字符指纹；④生成内容被静默丢弃。两层修复：

- **Fix 1·`isTextDuplicate` 最小长度守卫（`textCleanup.ts`）**：归一化后 < 30 字符的生成文本直接返回 false，不进行去重检查。打字机首帧（3 字符）、短回显前缀（2 字符）等短文本在长篇正文中几乎必然命中 `includes()` 造成假阳性；只有生成文本足够长（≥30 归一化字符）时才检查是否为已有内容的子串。全量内容（2511 字符）仍正常评估去重。
- **Fix 2·`stripInstructionEcho` 指令回显剥离（`textCleanup.ts` + `FrontstageApp.tsx`）**：新增 `stripInstructionEcho(generated, userInput)` --归一化比较生成文本开头与用户指令，若开头匹配则裁掉原始文本中对应前缀及紧随的分隔符（换行/冒号/逗号等），剩余内容过短（<10 字符）则保留原文防误剥。在 `handleRequestGeneration` 和 `handleSmartGeneration` 的 `sanitizeContinuationOutput` 后调用，覆盖打字机路径与 smart_execute 直接路径。
- **测试**：`isTextDuplicate.test.ts` +2 测试（短文本假阳性守卫 + 长文本真阳性）；`textCleanup.test.ts` +7 测试（`stripInstructionEcho` 7 场景）；更新 2 既有测试（前缀检测改用 ≥40 字符 + `isTextDuplicate` 用 ≥30 字符）。
- **验证**：`npx tsc --noEmit` ✅；`npx vitest run` 349 passed / 3 skipped（+13）；`npm run format:check` ✅；`architecture_guard` ✅。纯前端修复，无 Rust 变更（cargo 基线不变）。

### v0.30.40 - 修复代理工作室不显示活动记录数据（activeRunId 仅从事件捕获 + 无 list_runs 命令）

用户报告"前端后台的代理工作室，没有显示代理活动的记录数据"。根因：`AgencyStudio.tsx` 的 `activeRunId` **仅从实时事件捕获**（`agency-agent-activity` / `agency-run-progress` / `agency-board-changed` 三个 `listen`），IPC 查询 `getRun`/`listBoard` 的 `enabled: !!activeRunId`--如果用户在 run 启动后或完成后才打开代理工作室，没有事件到达，`activeRunId` 恒为 `null`，页面永远显示"暂无活动"。此外无 `agency_list_runs` 命令发现已有 run，activity/progress 事件 fire-and-forget 不持久化（时间线数据页面卸载即丢失）。

- **后端·新增 `agency_list_runs` 命令（`agency/repository.rs` + `agency/commands.rs` + `handlers.rs`）**：`AgencyRepository::list_runs_for_story(story_id, limit)` 按 `created_at DESC` 列出某 story 的全部 run（利用已有 `idx_agency_runs_story` 索引）；`agency_list_runs` Tauri 命令（limit=20）注册到 `handlers.rs`。前端可通过 IPC 发现已有 run，不依赖实时事件。
- **前端·activeRunId 水合（`AgencyStudio.tsx`）**：新增 `useQuery(['agency-runs', currentStory?.id], () => listRuns(currentStory.id))` 查询（10s 轮询）；`useEffect` 在 `runs` 数据到达且 `!activeRunId` 时取 `runs[0].id`（最新 run）水合。实时事件仍可覆盖（新 run 启动时事件到达，切到新 run）。
- **前端·历史时间线重建（`AgencyStudio.tsx`）**：时间线从仅 live 事件改为三源合并--①Live 事件（activities + progress）；②历史重建（board items 的 `created_at` + `producer` + `zone` + `key` + `summary` 生成时间线条目，如"管理 创建 资产：世界观 - 双星系统"）；③Run 生命周期（`created_at` 启动 + `updated_at` 终态）。合并后按 `(at, text)` 去重、时间倒序、截断 100 条。无需新表/迁移，从已持久化的 `agency_board_items` 重建。
- **前端·Run 选择器（`AgencyStudio.tsx`）**：标题栏右侧新增 `<select>` 下拉框，每个 option 显示 `[status] phase - premise 前30字 (时间)`，用户可切换浏览历史 run。切换时 `setActiveRunId` -> react-query 自动刷新 board + run 数据。
- **附带·clippy 冗余修复（`agency/repository.rs`）**：`list_runs_for_story` + `list_checkpoints` 的 `Ok(rows.collect::<Result<Vec<_>, _>>()?)` 改为 `rows.collect::<Result<Vec<_>, _>>()`（`Ok` + `?` 冗余，clippy `needless_question_mark`）。
- **验证**：`cargo test --lib` 1082 passed（+1：`test_list_runs_for_story`）；`cargo check` / `npx tsc --noEmit` / `npx vitest run`（339 passed / 3 skipped，+3：水合 / run 选择器 / 历史时间线）/ `cargo +nightly fmt` / `cargo clippy --lib`（538，baseline 540 零新增且 -2 修复既有）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.39 - 修复续写不按故事大纲推进剧情（TimeSliced 路径缺失 build_progression_anchor）

用户报告"续写和故事大纲仍然缺乏强关联"、"没有按照故事大纲来写剧情和推进剧情"。根因：v0.30.31 引入的 `build_progression_anchor`（确定性注入剧情推进方向锚点）**只在 TriShot 路径（`execute_trishot`）调用，从未移植到 TimeSliced 路径（`execute_time_sliced`）**。而 TimeSliced 是默认续写路径（`generation_mode = "auto"` 路由续写到 TimeSliced 而非 TriShot）。TimeSliced 的 writer 通过 `bundle.to_prompt()` 得到完整故事大纲，但缺少"已推进进度"指针（最近 3 章 `scenes.outline_content`），无法判断当前在故事大纲的哪个节点，因此无法按节点推进剧情，导致续写偏离大纲、原地踏步或仅复述设定。

- **根因·build_progression_anchor 未在 TimeSliced 调用（`agents/orchestrator.rs`）**：v0.30.31 新增 `build_progression_anchor` 函数，注入①本次创作指令（创作方向）；②故事大纲前1200字（硬约束）；③本章场景大纲前800字（硬约束）；④已推进进度（最近3章 outline_content，进度指针）；⑤世界观规则前600字（硬约束）；⑥显式调和指令（在硬约束内落实指令核心意图，推进到下一节点）。但该函数**仅在 `execute_trishot` 的 `!synthesis.is_fallback` 分支调用**（line ~1617），`execute_time_sliced`（默认续写路径，line 842-1068）从未调用。TimeSliced writer 只有 `bundle.to_prompt()`（含故事大纲）+ `build_continuation_context`（前文回顾）+ `build_ending_anchor`（末句硬锚点），**无进度指针、无显式调和指令**。
- **Fix·TimeSliced 路径注入 build_progression_anchor（`agents/orchestrator.rs` `execute_time_sliced`）**：在 prompt 模板渲染后、`ending_anchor` 注入前，插入 `build_progression_anchor(&bundle, pool.inner(), &task.context.story.story_id, chapter_number, &user_instruction)` 调用，与 TriShot 路径完全对齐。writer 现在收到完整的推进方向锚点：故事大纲硬约束 + 已推进进度指针 + 显式调和指令（"推进到故事大纲下一节点、承接已推进进度，不得原地踏步"）。注意 `story_id` 在 `spawn_blocking` 闭包中被 move，改用 `&task.context.story.story_id`。
- **验证**：`cargo test --lib` 1081 passed；`cargo check` / `npx tsc --noEmit` / `npx vitest run`（336 passed / 3 skipped）/ `cargo +nightly fmt` / `cargo clippy --lib`（539，baseline 540 零新增）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.38 - 修复续写输出被编辑器元评论污染（is_prose_request 被 serde 默认 false 导致 sanitize 跳过）

用户报告"第三次续写时出的错"--续写产出正文后紧接一段 AI 文学编辑元评论（"好的，作为一名专业的文学编辑，我将根据您提供的问题列表和总体评分，对您的文本进行深度重塑…请粘贴您的《永夜神骸》第一章内容"）。续写误路由 bug 第 6 次复发（v0.30.9-14 各堵一条路径，但分类层根因未修）。

- **根因（三层叠加）**：①分类提示词"继续写"示例省略 `is_prose`，LLM 若遵循示例返回合法 JSON 但缺该字段，`#[serde(default)]` 填 `is_prose_request=false`；②serde 默认值（false）与 LLM 失败兜底值（true）相反--partial-but-valid JSON 走 `parse_classification_json` 成功解析、`is_fallback=false` 被缓存，后续相同输入持续返回毒化 false；③`sanitize_plan_for_prose_request` 门控仅检查 `is_prose_request`，false 时跳过全部净化 -> SING 多步计划 `[writer, inspector, builtin.style_enhancer]` 未拦截 -> `final_content` = style_enhancer 元评论覆盖 writer 正文。
- **Fix 1·后置不变量（`intent.rs` `parse_classification_json`）**：成功反序列化后若 `is_continuation || is_new_novel` 但 `is_prose_request=false`，强制设 true（续写/创世本质是 prose，逻辑必然）。
- **Fix 2·提示词示例补全（`intent.rs` `build_classification_prompt`）**："继续写"示例补 `is_prose=true`，消除 LLM 省略该字段的源头。
- **Fix 3·sanitize 门控扩展（`planner/mod.rs` `sanitize_plan_for_prose_request`）**：门控从 `is_prose_request` 扩展为 `is_prose_request || is_continuation`--纵深防御，即使 Fix 1 未生效，`is_continuation=true` 也触发净化+塌缩。
- **验证**：`cargo test --lib` 1081 passed（+4 回归）；`cargo check`/`tsc`/`vitest`（336/3 skipped）/`fmt`/`clippy`（540->539 零新增）/`architecture_guard`/`format:check` 全绿。

### v0.30.37 - 修复创作生成失败时 toast 显示 "[object Object]"（issue #12）

用户反馈 issue #12：创作/生成失败时错误提示显示 `[object Object]`。根因与 issue #11（v0.30.31 修复的"获取模型列表"路径）同源：后端 `AppError` 自定义 `Serialize` 产出普通 JSON 对象 `{ code, message, severity, data? }`，Tauri v2.4 作为普通对象（非 JS `Error` 实例）投递到前端 catch 块；前端用 `String(err)` 或 `err instanceof Error ? err.message : String(err)` 转字符串，对普通对象产出 `[object Object]`，可读 `message` 被丢弃。v0.30.31 的 `extractMessage` helper 只覆盖"获取模型列表"，**创作/生成错误路径未迁移**--幕前 smart_execute、幕后快速创作/AI 向导、生成草稿/大纲、文思生成、管线修稿/审稿/定稿仍显示 `[object Object]`。

- **主修复·统一改用 `extractMessage`（10 个前端文件）**：所有创作/生成错误路径的 `String(err)` / `instanceof Error ? err.message : String(err)` / `err?.message || String(err)` 统一替换为 `extractMessage(err)`（`src/utils/errorHandler.ts`，依次尝试结构化 AppError 对象取 `.message` -> `Error.message` 内嵌 JSON 解析 -> 普通 Error `.message` -> 字符串 -> 带 `.message` 对象 -> 兜底 `'Unknown error'`）。
  - `FrontstageApp.tsx`（5 处）：smart_execute 主 catch（`structured?.message ?? extractMessage(error)`，复用已计算 `structured`）+ 第二 smart_execute catch + 修稿/审稿/定稿。
  - `SceneEditor.tsx`（2 处）：生成大纲/草稿失败。
  - `Stories.tsx`（4 处）：幕后快速创作/向导创作/风格混合保存/风格样本生成。
  - `RichTextEditor.tsx`（2 处）：文思内联建议生成/智能排版。
  - `WenSiPanel.tsx`（2 处）：自动续写/自动修改。
  - `usePipeline.ts`（6 处）：修稿/审稿/定稿/修复/合并/加载。
  - `CharacterStatePanel.tsx`（1 处）、`Skills.tsx`（7 处）、`PromptsPanel.tsx`（5 处）、`useUpdater.ts`（2 处）。
  - 不动 `main.tsx` / `ErrorBoundary.tsx`：已优先取 `.message`，`String()` 仅最后兜底，对带 `.message` 的 AppError 对象不会产出 `[object Object]`。
- **回归测试（`src/utils/__tests__/errorHandler.test.ts`，+8）**：AppError 普通对象提取 `message`（断言不等于 `[object Object]`）/ 带 `data` / `parseStructuredError` 识别 / `Error.message` 内嵌 JSON / 普通 Error / 字符串 / 带 `.message` 对象 / 兜底文案。
- **验证**：`npx tsc --noEmit` ✅；`npx vitest run` 336 passed / 3 skipped（+8）；`npm run format:check` ✅；`architecture_guard` ✅。纯前端，无 Rust 变更（cargo 基线 1077 不变）。

### v0.30.36 - 修复首次创世指令不保存到输入历史（按↑调取不到）

用户报告"输入框的历史输入内容也没有保存，按向上方位键调取不到历史输入"。根因：`handleInputSubmit` 保存输入历史时读取 `sid = currentStory?.id`，首次创世（无已有故事）时 `currentStory=null` -> `sid=undefined` -> `if (sid) saveInputHistory(...)` 跳过，创世指令从未持久化；随后 isBootstrap 分支 `setCurrentStory(null)` 触发 useEffect 清空 `inputHistory`，创世成功后 `setCurrentStory(新故事)` 再次触发 useEffect 从 localStorage 加载（空）。新故事输入历史始终为空，按↑无响应（无历史 + 无章节时 `fetchSmartHint` 也因 `!currentChapter` 提前返回）。v0.30.23 修复意图分类（创世不再被误判为续写）后，创世指令正确走 isBootstrap 路径，暴露了此前被续写误分类掩盖的首次创世不保存缺陷。

- **主修复·创世成功后补存创世指令（`FrontstageApp.tsx`）**：`handleSmartGeneration` 的 `story_created` 处理块（`setCurrentStory(targetStory)` 之后）新增同步写入--`loadInputHistory(storyId)` 读取新故事现有历史，若不含 `userInput` 则 `saveInputHistory(storyId, [userInput, ...existing].slice(0, MAX))` 持久化。关键时序：此写入在 `setCurrentStory` 触发 `useEffect[currentStory?.id]` 之前同步执行（同一同步块无 await），useEffect 随后 `loadInputHistory(storyId)` 即可读到创世指令。
- **不动 `handleRequestGeneration`**：文思活跃续写路径（`user_input: context || '续写'`），非用户创世指令，无有意义输入可存。
- **验证**：`npx tsc --noEmit` ✅；`npx vitest run` 328 passed / 3 skipped（+2：创世指令持久化 + 按↑召回）；`npm run format:check` ✅；`architecture_guard` ✅。纯前端，cargo 基线 1077 不变。

### v0.30.35 - editor 质检后台异步化：首章立即显示 + 后台质检 + toast 反馈

用户报告创世顶满 600s 超时无产出。根因：editor 质检（`review_and_assemble` 中的 `evaluate_gate`）在 Scene 装配落库**之前**同步执行，被 `tokio::time::timeout(600s)` 包裹。producer（深度资产 ~30-60s）+ writer（tool_loop ~4-5min）花约9分钟后 editor 只剩约1分钟，而 editor 的 `editor_verdict_prose_fallback` 用固定 300s timeout 发起 LLM 调用，34s 后被硬 600s 砍掉，既未完成质检也无法走 `salvage_failed_gate` 保产出，整 run 超时无任何首章返回。本版本把 editor 质检从同步硬阻塞改为后台异步 spawn：writer 完成首章 + 装配落库后立即返回前端显示首章（约5-6min 即可见），editor 在后台独立 spawn 质检（独立 300s deadline，不受 smart_execute 600s 限制），结果通过 `genesis-qc-result` 事件 + toast 通知用户。

- **后端·装配与质检分离（`coordinator.rs`）**：①新增 `assemble_only`（pub(crate)）-- 从 `review_and_assemble` 提取纯装配部分（`update_phase("assembly")` + `cleanup_prose_for_persist` 抗重复三件套 + `SceneRepository::create/update` 落库 + `emit_activity`），不含 editor 质检与修订，返回 `(BoardItem, scene_id)`。②新增 `spawn_editor_qc`--测试环境 `app_handle=None` 时 no-op；生产环境 `tokio::spawn` 后台任务，构造全新 `AgencyLlm(EditorAuditor)` / `AgencyBudget` / `BlackboardService` / `ToolRegistry`，用 `Some(Instant::now() + 300s)` 独立 deadline 调 `evaluate_gate_impl`，结果三态分支：`Passed` -> `{passed:true,salvaged:false}`；`RevisionRequired` -> `{passed:false,issues}`；`Failed` -> 先 `salvage_failed_gate`（草稿≥600字合成 pass 裁决保产出）-> 成功 `{passed:true,salvaged:true}` / 失败 `{passed:false,issues:[reason]}`；`Err` -> 降级放行 `{passed:true,salvaged:true}`。`emit_activity(EditorAuditor,"start"/"done","后台审查")` + emit `genesis-qc-result` 事件。③`genesis_fastpath` / `run_genesis_legacy_inner` Phase C 由 `review_and_assemble` 改为 `assemble_only` + `spawn_editor_qc`，返回 `revised:false, verdict:EditorVerdict::pending()`。④删除已无用的 `review_and_assemble` 方法（其 helper `build_revision_task`/`evaluate_gate` 仍被续写路径复用）。⑤`EditorVerdict` 新增 `pending()` 构造函数（verdict="pending"，comments="后台质检进行中"）。⑥新增事件常量 `EVENT_GENESIS_QC_RESULT = "genesis-qc-result"`。
- **前端·后台质检结果 toast（`FrontstageApp.tsx`）**：`setupEventListeners` 新增 `genesis-qc-result` 监听，三态反馈：质检通过（`passed && !salvaged`）-> `toast.success('编辑审计质检通过')`；降级放行（`passed && salvaged`，审计超时/失败但首章已保留）-> `toast.warning('质检降级放行（审计超时/失败，首章已保留）')`；不合格（`!passed`）-> `toast.warning('质检不合格，建议重新创世。问题：' + issues)`。后台 editor 不影响 `isGenerating`（agency 事件不进 `backendActivityStore`），用户可在质检期间继续写作；不自动重新创世，由用户手动决定。
- **producer 深度资产保持前台**：审计后发现 `producer_depth_assets` 已是单次 `complete_json` 调用（非 tool_loop，约30-60s），非瓶颈；且保障首章不脱节（v0.30.29 专门修复的"首章在无大纲/无世界观下写就脱节"问题）。主要瓶颈是 writer tool_loop（4-5min）+ editor tool_loop，移 editor 后台后用户在 writer 完成即可见首章。
- **验证**：`cargo test --lib` 1077 passed（+2：`test_editor_verdict_pending_defaults` / `test_assemble_only_persists_scene_without_qc`；移除 3 个已不适用的 genesis 同步质检测试，`test_editor_verdict_prose_fallback` 改为直接测 `evaluate_gate` 保留 prose-fallback 覆盖）；`cargo check` / `npx tsc --noEmit` / `npx vitest run`（326 passed / 3 skipped，+4：`genesis-qc-result` 注册 + passed/salvaged/failed 三态 toast）/ `cargo +nightly fmt` / `cargo clippy --lib`（539，baseline 540 零新增）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.34 - 修复续写内容丢失根因：序列化场景持久化 + 修稿 bypass 修复 + 关闭超时提升

v0.30.33 的关闭前 flush + AI 追加立即落库仍未能完全解决续写内容丢失。深入诊断定位三个收敛根因：①`flushSceneSave` 无序列化--文思活跃连续续写时多次 `void flushSceneSave()` 并发 fire-and-forget，`update_scene` 全量覆写在 `spawn_blocking` 线程池上 SQLite 写锁获取顺序非 FIFO，较早的小内容可能在较晚的大内容之后提交，静默覆写（编辑器显示正确但 DB 被回退，重启才发现）；②close-flush 3s 超时 < SQLite `busy_timeout` 5s，写锁竞争下 close-flush 的 `update_scene` 被 kill；③`handlePipelineRefine` 的 `setContent` / `onReviseResult` 的 `insertText` 绕过 `appendAiContent`，不更新 `latestContentRef`，关闭时 flush 保存旧内容。

- **主修复·序列化场景持久化（`FrontstageApp.tsx`）**：新增 `saveChainRef`（Promise 链）+ `persistSceneContent(sceneId, content, title)` -- 每次调用捕获快照后排队，`await prev` 等待前一次完成再 `update_scene`，`finally release()` 释放链。所有 `update_scene` 调用（`flushSceneSave` / `handleContentChange` 防抖 saveFn / 保护性保存）统一走此函数，保证串行提交、最后一次写总是最新内容。`flushSceneSave` 改为 `cancelAutoSave + persistSceneContent`。
- **Root Cause #2·关闭超时 3s -> 6s（`lib.rs`）**：`CloseRequested` 超时兜底线程从 `sleep(3s)` 提升到 `sleep(6s)`，超过 SQLite `busy_timeout=5s`，确保写锁竞争下 close-flush 的 `update_scene` 仍能提交。
- **Root Cause #3·修稿 bypass 修复（`FrontstageApp.tsx`）**：①`handlePipelineRefine` 的 `editorRef.setContent(refined_content)` 后补 `getHTML -> setContent(store) -> latestContentRef = html -> void flushSceneSave()`（`setContent` 抑制 `onChange` 不更新 ref，此前关闭时 flush 保存修稿前旧内容）；②`onReviseResult` 的 `editorRef.insertText(html)` 后同理补同步 + `void flushSceneSave()`。
- **保护性保存 + handleContentChange saveFn 统一（`FrontstageApp.tsx`）**：新建小说前保护性保存从手写 `cancelAutoSave + loggedInvoke` 改为 `await flushSceneSave()`（序列化）；`handleContentChange` 防抖 saveFn 从手写 `loggedInvoke('update_scene')` 改为 `await persistSceneContent(payload.sceneId, payload.content, payload.title)`（序列化）。消除所有非序列化的 `update_scene` 直调。
- **验证**：`cargo test --lib` 1078 passed；`npx tsc --noEmit` ✅；`npx vitest run` 322 passed / 3 skipped；`cargo +nightly fmt` / `cargo clippy --lib`（baseline 540 零新增）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.33 - 修复关闭应用时续写内容丢失（关闭前 flush + AI 追加立即落库 + 章节切换 flush）

用户报告"多次续写后关闭应用再重启，续写内容丢失，没有得到及时保存"。根因：幕前续写 `appendAiContent` 追加 AI 内容后仅调度 2000ms 防抖保存（`scheduleAutoSave(..., 2000)`），文思活跃连续续写时每次 `cancelAutoSave()` 重置定时器，间隔 <2s 则永不出火；关闭应用时后端 `CloseRequested` 直接 `graceful_shutdown -> std::process::exit(0)` 不给前端 flush 机会，防抖窗口内的内容随进程退出丢失。三层修复：

- **主修复·关闭前 flush 协调（`lib.rs` + `FrontstageApp.tsx`）**：后端 `CloseRequested` 由直接 `graceful_shutdown` 改为 `api.prevent_close()` + emit `frontstage-flush-requested` 事件 + 3s 超时兜底线程；前端 `useEffect` 监听该事件 -> `await flushSceneSaveRef.current()`（取消防抖 + 立即 `update_scene` 落库 `latestContentRef.current`）-> `invoke('graceful_quit')` 命令触发优雅关闭（WAL checkpoint 确保刚写入的数据落盘）。`graceful_shutdown` 加 `AtomicBool` 幂等守卫防 flush 完成与超时兜底竞争。3s 超时兜底覆盖前端无响应/已崩溃/flush 卡住。
- **纵深·AI 追加立即落库（`FrontstageApp.tsx` `appendAiContent`）**：`scheduleAutoSave(..., 2000)` 替换为 `void flushSceneSave()`（立即 fire-and-forget 落库）。AI 内容是离散完整块（非高频打字），立即落库合适；消除文思活跃连续续写 cancelAutoSave 反复重置定时器导致永不出火的丢失窗口；即使应用崩溃（非优雅关闭）内容也已落库。wordCount 已在上方 `setWordCount` 更新无需重复。
- **附带·章节切换前 flush（`FrontstageApp.tsx` `selectChapter`）**：`cancelAutoSave()` 替换为 `void flushSceneSaveRef.current()`，切换章节前将当前场景未保存内容落库，避免防抖窗口内的续写/编辑内容在切换章节时丢失（flush 内部已 cancelAutoSave 无需重复）。
- **提取 `flushSceneSave`（`FrontstageApp.tsx`）**：此前保护性保存（新建小说前 `cancelAutoSave + sync update_scene`，line 3888）与自动保存 saveFn 各自重复同一套 `update_scene` 逻辑；现提取为共享 `flushSceneSave` useCallback（cancelAutoSave -> 读 store sceneId/title + latestContentRef -> `loggedInvoke('update_scene')` -> setIsSaved + justSavedRef），通过 `flushSceneSaveRef` 暴露给 effect 监听器与 selectChapter。
- **验证**：`cargo test --lib` 1078 passed；`npx tsc --noEmit` ✅；`npx vitest run` 322 passed / 3 skipped；`cargo +nightly fmt` / `cargo clippy --lib`（baseline 540 零新增）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.32 - 增强性指令纳入世界观/故事大纲/场景大纲/上下文强关联

承接 v0.30.31 让世界观/故事大纲/场景大纲/进度彼此强关联后，用户指出增强性指令（logline 后缀）未被纳入这套强关联--增强后缀生成时不看世界观，进入管线后又与资产各居一隅、互不交叉引用，"失去了增强性指令的意义"。本版本补齐两个缺口：增强生成纳入世界观，指令与资产在 writer prompt 显式调和（资产=硬约束，指令=创作方向，在硬约束内落实指令核心意图，冲突时调整指令具体表现以符合约束但保留核心意图）。

- **P0-A·增强生成纳入世界观（`commands/orchestrator.rs` + `agency_logline_suffix_contextual.md`）**：`build_logline_context_sync` 此前只拉 story_outline/scene_outline/characters/current_content，**完全不读 `world_buildings`**，增强后缀可能在不知世界规则下提出违反世界观的设定。现 `LoglineContext` 新增 `world_setting` 字段，拉 `WorldBuildingRepository` 渲染 concept + rules 前3 + history（截断 1000），`build_contextual_logline_system` 注入 `world_setting` var；`agency_logline_suffix_contextual.md` 新增 `## 世界观设定` 段 + 输出要求"后缀须与世界观规则一致，不得提出违反世界观的设定或角色"。
- **P0-B·TriShot 指令纳入 `build_progression_anchor` + 显式调和（`agents/orchestrator.rs`）**：v0.30.31 的 `build_progression_anchor` 注入 story_outline/scene/progress/world 标记"最高优先级，不得偏离"，但**不接收指令参数、从不引用用户指令**；指令被 Call1 LLM 抽象进 `synthesized_prompt`，与资产各居一隅、无调和。现签名加 `user_instruction: &str`，指令非空时作为首个段【本次创作指令（你的创作方向，须与下方硬约束协调一致）】注入；收尾指令改为显式调和："本次创作指令是你的创作方向；故事大纲/场景大纲/世界观/已推进进度是硬约束。须在硬约束内落实指令核心意图--推进到故事大纲下一节点、遵循世界观规则、承接已推进进度。若指令与某硬约束冲突，调整指令的具体表现以符合约束，但保留指令核心意图；不得因约束丢弃指令，也不得因指令违反约束。"调用点传 `&task.input`（raw 指令，空则跳过指令段走原推进约束）；仅有指令无资产时输出指令段 + "推进剧情向前发展"。
- **P1-C·创世指令-资产调和（`agency/coordinator.rs`）**：`writer_first_chapter`/`writer_prose_fallback` 写作要求增"故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾"；`writer_prose_fallback` 补回"资产区为准"系统提示。
- **P1-D·TimeSliced 指令-资产调和（`orchestrator_timesliced_writer.md` + `agents/orchestrator.rs` fallback）**：要求段加"写作指令须与故事上下文中的世界观、故事大纲、场景大纲协调一致；若冲突，在遵循上下文硬约束的前提下落实指令核心意图"。
- **验证**：`cargo test --lib` 1078 passed（+1：`test_build_progression_anchor_directive_only_no_assets` 边界；现有 2 测试更新为断言指令段 + 调和约束）；`cargo check` / `tsc` / `vitest`（322 passed / 3 skipped）/ `cargo +nightly fmt` / `cargo clippy --lib`（baseline 540 零新增）/ `architecture_guard` / `format:check` 全绿。

### v0.30.31 - 续写链路修复：世界观/故事大纲/场景大纲注入与剧情推进方向

用户报告"世界观设定没有体现在续写中，世界观和故事大纲、场景大纲结合不紧密，续写内容剧情推进不够紧凑，迷失剧情推进方向"。全面审计定位五类根因，聚焦幕前续写实际路径（Legacy TriShot）+ 共享生成端/prompt 资产 + Agency 注入函数顺带修复。进度指针用现有 `scenes.outline_content` 回读最近 3 章，无 DB 迁移、无 schema 变更。

- **P0-A·Legacy TriShot 确定性注入（最关键）**：根因--TriShot `final_prompt = Call1 LLM 合成的 synthesized_prompt`，manifest 不含 story_outline、synthesizer 不透传 bundle_prompt 关键段，故事大纲/场景大纲 outline_content/world_buildings 三者均不到达 writer（v0.30.15 注释声称修了 TimeSliced/TriShot，实际只修了 TimeSliced）。①`write_time_bundle.rs` load_sync 读 world_buildings 表（concept+rules前5+history+cultures前3，截断 2000）为 `world_setting`；`domain/write_time_bundle.rs` WriteTimeBundle 新增 `world_setting: Option<String>`；`to_prompt` 增【世界观设定】段。②`manifest.rs` build 增加 story_outline+world_setting 清单项（hard_constraint），scene_outline one_line 纳入 outline_content。③`orchestrator.rs` 新增 `build_progression_anchor`，在 `final_prompt = synthesized_prompt` 后确定性注入【剧情推进方向（最高优先级）】段（故事大纲1200+场景大纲800+已推进进度+世界观600+推进约束），`!is_fallback` 时注入。
- **P0-B·writer prompt 推进约束**：`writer_system.md`/`orchestrator_timesliced_writer.md`/`trishot_synthesizer.md` 各加"剧情必须推进到下一节点，不得原地踏步、不得仅复述设定或复述前文"。
- **P0-C·scene_outline.md 修伪前提 + 加 world/progress 变量**：删"按序号定位节点"伪前提（故事大纲是散文无编号节点），改为"根据【已推进进度】定位"；Legacy `creation_commands.rs generate_scene_outline` 加载 world_buildings + 最近 3 章 outline_content 注入 task.parameters，`service.rs build_outline_prompt` 读取注入 vars；Agency `generate_chapter_outline` vars 同步注入。
- **P1-A·Agency build_continue_writer_context 修复（顺带修）**：世界观全字段（concept+rules前5+history+cultures前3），此前只 concept+history 且超 6000 整段丢弃，现超预算截断降级注入；前文阈值倒挂修复（>8000->>12000 且保底最近 1 场正文 1500 字）；新增【已推进进度】段；`write_chapter` 三分支加推进约束+点名世界观。
- **P1-C·world_buildings 生成端填全字段**：`ensure_world_building` concept 存全文（此前截 500）；prompt 增"正文末尾用【核心规则】列出 3-5 条世界规则"，best-effort 解析存 rules；history 不再冗余存储（concept 全文已含）。
- **P1-D·editor 质量门预注入参照资产**：`evaluate_gate_impl` editor task 预注入参照资产（世界观红线+世界观设定+故事大纲），使"合同兑现/连续性/世界观一致性/推进方向"维度可校验。
- **验证**：`cargo test --lib` 1077 passed（+2：build_progression_anchor 全段注入 + 空场景返回空）；`cargo check`/`npx tsc --noEmit`/`npx vitest run`（322/3 skipped）/`cargo +nightly fmt`/`cargo clippy --lib`（baseline 540 零新增）/`architecture_guard`/`npm run format:check` 全绿。

### v0.30.30 - Agency 创作链路结构性优化：抗重复闭环 + 质量门宽松度 + 熔断不丢稿

承接 v0.30.29 内容质量根因修复后显式推迟的 D/E 两类结构性优化。聚焦 Agency 创世/续写链路三个"产出被白白丢弃"的结构性缺口：①创世装配写 RAW 正文不经清理（续写在 v0.30.29 已接清理三件套，创世没有）；②质量门 model 分对 scoreless pass 兜底 0.85 过宽松；③editor 完全评不出裁决时整 run 失败、writer MaxTurns/Deadline 熔断直接丢稿。把"熔断不等于丢稿"哲学（v0.30.19 salvage + 散文回退）补齐到 writer 与 gate Failed 两个剩余缺口，并把创世装配纳入与续写一致的清理管线。

- **D1·抗重复提示词补齐 + 创世装配接入清理三件套（`coordinator.rs` + 两份 agency 提示词资产）**：①提取共享 helper `cleanup_prose_for_persist(raw, story_id)`（`spawn_blocking` 内 `trim_self_repetition` -> `strip_existing_overlap`（取最新场景全文，无则跳过）-> `trim_dangling_tail`，join 失败回退原文）；创世 `review_and_assemble` 装配 Scene 前调此 helper（此前写 RAW `draft.content`）；续写 `handle_gate` 内联清理块替换为调此 helper（行为等价去重）。②`agency_lead_writer_system.md` 创作红线新增"禁止重复输出"一条；`agency_editor_auditor_system.md` 审查维度新增第 6 维"重复与复述" + `dimension_scores` 模板加 `"repetition":1-5`。③内联 writer prompts（`writer_first_chapter`/`writer_prose_fallback`/`write_chapter` 三分支/`build_revision_task`）各加一句禁止重复指令。
- **D2·失效 prompt_id 核查（结论：by-design，仅加注释）**：`agency/roles.rs` 中 `Writer/Inspector/OutlinePlanner/StyleMimic` 引用 `agency_writer_system` 等占位 ID（无 bundled 文件），但运行时回退 `default_role_prompt`（`roles.rs` 注释已明确），且这些角色不在创世/续写主流程（只用 LeadWriter/Producer/EditorAuditor）。9 个"orphan" prompt 文件已被 WalkDir 注册供用户覆盖/未来用，非 bug。仅在 `writer.rs`/`inspector.rs`/`outline_planner.rs`/`style_mimic.rs` spec 处补注释说明占位 ID 回退，无功能改动。
- **E1·模型分宽松 - scoreless pass 兜底 0.85 -> 0.7（`coordinator.rs` `ModelGraderReport::from_verdict`）**：editor 不给数值分只给 `verdict:"pass"`（本地模型常见）时 `model_score` 由 0.85 降到 0.7。Gate v2 加权 `0.2*code+0.3*rule+0.5*model` 阈值 0.75：0.85 时单 model 项贡献 0.425，code+rule 仅需 65% 满分即过门（太宽松）；0.7 低于阈值，须 code+rule 达 80% 满分才放行，code/rule 满分时仍可过（0.85）不误伤优质稿。`"revise"=0.4`/兜底 `0.5` 不动。
- **E2·editor 连累整 run - GateOutcome::Failed 降级放行（`coordinator.rs`）**：新增 helper `salvage_failed_gate(draft, reason) -> Option<EditorVerdict>`：草稿 `chars().count() >= 600`（substantive）-> 合成 `verdict:"pass"` 裁决（`comments` 透明记录降级原因），`log::warn!` 标记；草稿过短返回 `None`（不救垃圾稿）。4 个 Failed arm（genesis 首门/复审、续写首门/复审）由直接 `return Err` 改为先尝试 salvage：救回则继续装配（仍走 D1/C3 清理三件套），救不回才 Err。对齐 v0.30.19 salvage 哲学"熔断不等于丢稿"。
- **E3·writer 熔断丢稿 - MaxTurns/Deadline 先取黑板草稿（`coordinator.rs`）**：genesis 与续写 writer abort 处理原先仅在 `reason == "连续解析失败"` 时触发 `writer_prose_fallback`，`MaxTurns`/`Deadline` 直接 `return Err`。但 MaxTurns/Deadline 熔断前 writer 可能在早期轮次已 `board_write` 产出草稿到黑板 Draft 区（`LoopResult.output` 是占位串不含正文，黑板里有）。现统一：MaxTurns/Deadline 先 `latest_draft`/`latest_draft_by_key` 取回已产出草稿（`>= 200` 字符才用），取不到/过短才落 `writer_prose_fallback` 散文回退，仍失败才 Err。连续解析失败路径行为不变（黑板通常无稿 -> 直接散文回退）。
- **验证**：`cargo test --lib` 1069 passed（+4：scoreless pass 阈值 / salvage_failed_gate 长短稿边界 / cleanup_prose_for_persist 自重复清理 / 续写 writer MaxTurns 黑板取回）；`cargo check` / `npx tsc --noEmit` / `npx vitest run`（322 passed / 3 skipped）/ `cargo +nightly fmt` / `cargo clippy --lib`（baseline 540 零新增）/ `architecture_guard` / `npm run format:check` 全绿。

### v0.30.29 - 内容质量根因修复：强模型结构化大纲不再被丢弃 + 大纲/世界观约束到生成链路

- **P0 根因·DepthAssets 结构化 outline（`coordinator.rs`）**：`outline: String` -> `serde_json::Value`（兼容 String/Object/Array）；新增 `normalize_outline` 将结构化对象（core_conflict/three_act_structure{act1,act2,act3}/turning_points）渲染为可读文本（【核心冲突】【三幕结构】【关键转折点】），未知字段 fallback `to_string()`；新增 `outline_value_is_empty` 判空；散文兜底 `outline` 改 `Value::Null`；内联 prompt 鼓励整书三幕+转折点大纲。**实证根因**：强模型返回结构化整书大纲对象时 serde `String` 类型不匹配 -> `parse_lenient` 返回 `None` -> 走散文兜底（`outline=空`）-> 大纲不写 `story_outlines` 表 -> 首章与续写都看不到大纲（模型越强、大纲越完整，越被丢弃）。下游零改动（`content` 仍 TEXT，消费者已当纯文本处理）。
- **P1·创世首章注入 world/outline（`coordinator.rs`）**：Phase B 编排由多模型 `tokio::join!(writer, producer)` 并行改串行 producer-first（producer 先写深度资产到黑板 Asset 区，writer 后读资产写首章）；新增 `build_assets_ctx_brief` helper（读 Asset 区 3000 字符预算）注入 `writer_first_chapter`/`writer_prose_fallback`，system prompt 增补"人设/世界观/伏笔以资产为准不得自相矛盾"。消除首章在无大纲/无世界观下写就的脱节；任一失败仍上抛回退 legacy。
- **C1·续写注入 MASTER_SETTING 红线（`coordinator.rs` `build_continue_writer_context`）**：上下文最前注入合同红线（`StoryContractRepository::get_by_type` + `extract_redline_text` 800 字截断），对齐 C 链路 `WriteTimeBundle.to_prompt` "红线最前最突出"不变量；Agency 续写此前完全绕过红线。
- **C3·续写落库前抗重复三件套（`coordinator.rs` `handle_gate`）**：装配 Scene 前对 `draft.content` 应用 `TextUtils::trim_self_repetition` -> `strip_existing_overlap`（取最新场景全文比对尾部 3000 字）-> `trim_dangling_tail`，与 C 链路 `orchestrator.rs` 同款；此前自重复/复述/截断半句直接入库回灌污染后续章节。
- **C4·章节大纲改用 scene_outline.md（`coordinator.rs` `generate_chapter_outline`）**：硬编码 prompt 替换为 DB-backed `resolve_prompt_with_vars(pool, "scene_outline", &vars)`（`spawn_blocking`，支持用户在提示词管理界面覆盖），vars 单独查库（story_outline/scene_number/characters/scene_info）；`unwrap_or_else` 保留硬编码 fallback。章节大纲受"禁止发明新角色、定位故事大纲节点"强约束。
- **验证**：`cargo test --lib` 1065 passed（+5：normalize_outline 对象/字符串/空/部分/未知 fallback；C1 红线注入扩展）；`cargo check`/`npx tsc --noEmit`/`npx vitest run`（322 passed / 3 skipped）/`cargo +nightly fmt`/`cargo clippy --lib`（baseline 540 零新增）/`architecture_guard`/`npm run format:check` 全绿。

### v0.30.28 - UI 双模式设计系统重塑；落地页下载自动同步；幕前交互打磨

- **双模式设计系统**：幕前「墨纸」（`--paper-*`/`--ink-*`/`--terracotta*`，无阴影扁平）与幕后「机械」（`--cinema-*`/`--cinema-gold*`，多层阴影仪表板）落地为统一 token；`tailwind.config.js` 暴露 `paper`/`ink`/`terracotta`/`cinema`/`status` 调色板与 `rounded-paper`/`rounded-panel`/`shadow-panel`。幕后仪表盘外壳、机械设置页、导航轨去重 + 无障碍、墨纸↔机械模式切换按 `docs/plans/2026-07-27-ui-redesign-design.md` 实现；硬编码颜色全量 token 化，`@apply` 不透明度改 `color-mix(in oklch,...)` 修 Vite 构建。
- **落地页自动同步**：`landing/src/hooks/useLatestRelease.ts` 运行时 fetch `latest.json` 拼下载链接（模块级 cache + in-flight promise + `FALLBACK_VERSION` 兜底），发版自动跟随无需重部署；AGENTS.md 新增用户级规则 #7（兜底版本随发版 bump、bundle 命名变更校验）。
- **幕前交互打磨**：恢复 UI 重塑误删的 `.frontstage-input-ghost*` CSS（`frontstage.css`），logline 后缀灰色 + 13px 小字号、前缀占位对齐消除层叠；`FrontstageBottomBar.tsx` 剥离冲突 Tailwind 工具类让 CSS 为唯一源。发射/取消按钮扁平化（`rounded-md` 淡彩底 + 陶土色图标，移除 `active:scale`，修无效 `text-cinema-50`）。移除 ghost-chrome 静止蒙版：删 `useGhostChrome` hook + 测试，`FrontstageHeader`/`FrontstageBottomBar` 不再鼠标静止 3s 淡出至 `opacity 0.08`，常驻完整不透明。
- **验证**：`cargo test -p storymoss` 1060 passed（无 Rust 变更）；`npx vitest run` 322 passed / 3 skipped；tsc / fmt / clippy（540 零新增）/ prettier / architecture_guard 全绿；landing 24 passed。

### v0.30.27 - 上下文感知 Logline 后缀；输入框自适应高度

- `generate_logline_hint` 在提供 `story_id` 时拉取故事大纲、当前章节大纲、角色列表与最近正文，渲染新 prompt 资产 `agency_logline_suffix_contextual`，生成贴合上下文的后缀；无上下文时回退原 `agency_logline_suffix`。
- `FrontstageBottomBar.tsx` 通过 `textareaRef` + `useEffect` 根据 `inputValue`/`ghostHint`/`loglineHint` 动态调整 textarea 高度（上限 200px），幽灵层不再固定 `max-height: 60px`。

### v0.30.26 - 统一 Logline 增强提示为内联幽灵文本；修复分时预检缺少角色

- **根因**：v0.30.24 的 logline 增强提示以独立 div 显示在输入栏下方，与统一的幽灵提示系统不一致；用户按 `->` 后直接用完整 logline 替换原输入，导致意图分类可能将增强后的长文本误判为续写，进而进入 TimeSliced 路径触发“缺少角色”预检失败。
- **Fix 1（UI 统一·FrontstageBottomBar.tsx + frontstage.css）**：移除 `.frontstage-logline-hint` 独立建议条；改为在 `frontstage-input-ghost-wrapper` 内渲染内联幽灵文本——前缀用 `visibility: hidden` 占位以与输入框文本对齐，后缀以灰色透明样式显示在已输入内容之后。loading 状态仍以内联形式提示“正在生成增强版指令…”。
- **Fix 2（交互·FrontstageApp.tsx）**：按 `->` 时执行 `setInputValue(inputValue + loglineHint)` 并清空 `loglineHint`，随后按 Enter 提交“原输入 + 增强后缀”组合文本。移除 `originalInputForLoglineRef` 与 `intentClassificationInput` 透传，`handleSmartGeneration` 恢复只接收 `userInput`，意图分类统一基于当前输入框文本。
- **Fix 3（Prompt 资产·orchestrator.rs）**：新增 `resources/prompts/agency/agency_logline_suffix.md`，要求 LLM 只输出应追加到原输入后的后缀；`generate_logline_hint` 改用该 prompt，返回后缀字符串。
- **Fix 4（分时预检缺少角色·intent.rs + preflight.rs + FrontstageApp.tsx）**：后端意图分类兜底路径增加按输入文本判断创世意图；`QuickPreflightChecker` 在角色表为空时自动创建占位主角（仅一次 DB 写入，不触发 LLM auto_contract），避免空角色表阻塞生成；前端接受 logline 提示后用原输入做意图分类。
- **验证**：`cargo test -p storymoss` 1060 passed；`npx vitest run` 310 passed / 3 skipped；fmt / clippy（baseline 549 零新增）/ tsc / prettier / architecture_guard 全绿。

### v0.30.25 - 修复续写 600s 超时（auto_contract 阻塞 + reasoning_content 丢失 + 无超时）

- **根因（三层叠加）**：用户输入"续写"后卡死 600s。①前端 `FrontstageApp.tsx` 在调用 `smart_execute` 前 `await autoCreateMissingContracts`，`auto_fill` 串行 4 次 LLM 调用（~6 分钟），v0.26.22 的 `is_silent_background` 只隐藏了 `isAnyBackendActive` 但 `await` 仍阻塞且 `isGenerating=true` 触发 600s 看门狗；后端 TimeSliced 续写路径本已跳过 auto_contract，但前端从未调用到。②DeepSeek 推理模型把思维链放在 `reasoning_content` 字段，`openai.rs` 的 `Message` 结构体不捕获该字段 -> serde 丢弃 -> `content=""`（0 字符）但 `tokens=2643`，auto_contract 收到空内容静默失败，合同永远补不齐。③`auto_contract.rs` 的 `auto_fill` 每个 `build_*` 调用无超时，单个慢模型调用阻塞数分钟。
- **Fix 1（主修复·`FrontstageApp.tsx`）**：续写请求不再阻塞 auto_contract。`handleSmartGeneration` 中 `classification.is_continuation` 时后台 fire-and-forget `autoCreateMissingContracts`（不 await，直接进入 smart_execute）；`handleRequestGeneration`（仅续写入口）同理。新增 `fireAutoContractInBackground` helper + `autoContractInProgressRef` 防并发 + 非阻塞 toast（成功/失败）。非续写请求（rewrite/audit）保持原有阻塞 await 行为。
- **Fix 2（次修复·`openai.rs`）**：`Message` 结构体加 `#[serde(skip_serializing, default)] reasoning_content: Option<String>`；`OpenAiDelta` 同理。非流式/流式提取在 `content` 为空时 fallback 到 `reasoning_content`，并 `log::warn!`。提取纯函数 `resolve_content` 供单测。
- **Fix 3（三修复·`auto_contract.rs`）**：`auto_fill` 的 4 个 `build_*` LLM 调用各包 `tokio::time::timeout(30s)`，超时与 Err 同处理（warn + 跳过 + 继续）。总上限 120s（4×30s），远低于 600s。
- **验证**：`cargo test --lib` 987 passed（+5：resolve_content fallback + Message 反序列化）；`npx vitest run` 311 passed；fmt / clippy（baseline 549）/ tsc / prettier / architecture_guard 全绿。

### v0.30.24 - Logline 幽灵提示（用户输入简单创世指令时实时生成增强版 logline）

- **功能**：用户在输入栏输入简单创世指令（如"写一部现代间谍的长篇小说"）后，后台用 v0.30.22 的 PROBLEM logline 生成功能产出一句话强力 logline，以幽灵提示形式显示在输入栏下方，用户按 `->` 即可用 logline 替换原始简单指令再执行。避免用户简单指令得不到好的生成结果而反复试，同时为用户提供故事创意的体验和技能锻炼。
- **后端（`commands/orchestrator.rs` + `handlers.rs`）**：新增 `generate_logline_hint` 命令--输入为空或 ≥ 100 字符返回 `None`（与 v0.30.22 `< 100 字符` 触发条件对齐）；复用 `agency_problem_logline` prompt 资产（用户在幕后编辑后自动生效）；`LlmService::generate_for_task_with_system_prompt` + 15s 超时；失败/超时静默返回 `None`。提取纯函数 `should_skip_logline_generation` / `is_valid_logline` 供单测。
- **前端状态（`FrontstageApp.tsx`）**：新增 `loglineHint` / `loglineHintLoading` state + `loglineHintTimerRef` / `loglineHintReqIdRef` 防抖 ref；`useEffect` 监听 `inputValue` 变化，1.5s 防抖后调 `generateLoglineHint`，请求 ID 防竞态（仅接受最新请求结果）；`handleInputKeyDown` 扩展 `->` 接受 logline（输入非空时，区别于 ghost hint 的空输入场景）+ `Esc` 清除；`handleInputSubmit` 清理 logline hint。
- **UI（`FrontstageBottomBar.tsx` + `frontstage.css`）**：输入框下方新增 `.frontstage-logline-hint` 建议条--loading 时显示旋转图标 + "正在生成增强版指令…"；就绪后显示 Lightbulb 图标 + logline 文本 + "按 -> 使用"提示；点击建议条也可接受（等同于 `->`）。CSS 含淡入动画 + hover 高亮。
- **不干扰现有幽灵提示系统**：现有 `ghostHint` 仅在输入为空时显示（placeholder 式），logline 提示在输入非空时显示（suggestion 式），是独立的 UI 层。两个 `->` 处理互斥（ghost hint 要求 `!inputValue`，logline hint 要求 `inputValue`）。
- **验证**：`cargo test --lib` 982 passed（+4：should_skip / is_valid 纯函数守卫）；`npx vitest run` 311 passed（+4：logline 渲染 / loading / 点击接受 / 空输入不渲染）；fmt / clippy（baseline 550，实际 549）/ tsc / prettier / architecture_guard 全绿。

### v0.30.23 - 意图分类 Bug 修复（LLM 分类去偏 + 失败兜底上下文化）

- **根因（LLM 分类本身被破坏）**：用户输入"写一部现代间谍的长篇小说"被分类为续写 -> `VALIDATION_FAILED: 请先在左侧选择或创建一个作品`。5 层防线失守：①提示词注入 `已有故事=true` 上下文偏差使 LLM 倾向续写；②`仅当"明确要求新开一部"` 过于保守；③无正例；④兜底 `conservative_fallback()` 恒返回 `is_new_novel=false` 无视 DB 状态；⑤失败结果被缓存。
- **Fix A（提示词去偏·主修复·intent.rs）**：`build_classification_prompt` 移除 `上下文：已有故事={story}` 上下文注入行（偏差来源）；移除 `仅当` 保守措辞；新增 3 个正例（"写一部科幻小说" -> is_new_novel=true）。LLM 不再受 DB 状态偏差，基于用户输入本身判定意图。
- **Fix B（上下文感知兜底·intent.rs）**：新增 `conservative_fallback_with_context(has_existing_story)`--LLM 失败时无故事返回创世（不可能续写不存在的作品），有故事返回续写。3 个兜底路径全部改用。原 `conservative_fallback()` 标记 `#[deprecated]`。
- **Fix C（不缓存失败·intent.rs）**：仅 LLM 成功解析的结果写入缓存，兜底结果不缓存。缓存键简化为仅 `user_input`（提示词不再使用上下文）。
- **Fix D（前端兜底上下文化·FrontstageApp.tsx）**：catch 块和 null 防御两处 LLM 失败兜底从硬编码 `is_new_novel: false` 改为 `is_new_novel: stories.length === 0`。`isBootstrap` 判定不变（尊重 LLM 结果）。
- **设计原则**：LLM 是意图判断的唯一权威；不回到硬编码关键词匹配；不用 `|| !has_existing_story` 覆盖 LLM 结果；DB 状态仅在 LLM 失败兜底时使用。
- **验证**：`cargo test --lib` 978 passed（+4）；`npx vitest run` 307 passed；fmt / clippy（baseline 550）/ tsc / prettier / architecture_guard 全绿。

### v0.30.22 - PROBLEM 七元素框架集成（Logline 生成 + 故事大纲增强）

- **背景**：用户输入简单指令（如"写一部科幻小说"）直接作为 `premise` 透传到 `concept_pack` -> `genesis_fastpath`，全程无方向约束。`concept_pack` 虽生成 `logline` 字段但从未使用，`ensure_story_outline` 提示词宽松，缺乏结构化创意质量检验。
- **核心**：将 Erik Bork 的 PROBLEM 七元素（Punishing/Relatable/Original/Believable/Life-Altering/Entertaining/Meaningful）编码为可编辑 prompt 资产，在创世和续写两个关键点注入。
- **Phase 1（Prompt 资产）**：新增 `agency_problem_logline.md` + `agency_problem_outline.md`，WalkDir 自动注册。
- **Phase 2（DB）**：V114 迁移 `stories ADD COLUMN logline TEXT`；`Story` model + `StoryRepository`（3 SELECT 加列 + `update_logline`）。
- **Phase 3（Logline 生成）**：`generate_logline` 单次 Producer LLM 调用；`run_genesis_inner` 在 `concept_pack` 前检测简单前提（< 100 字符）生成 logline 替换原 premise；genesis 成功后 `update_logline` 持久化。
- **Phase 4（大纲增强）**：`ensure_story_outline` system prompt 从 registry 加载 PROBLEM 大纲提示词 + logline 上下文；`producer_depth_assets` outline 字段增强 PROBLEM 指引。
- **Phase 5（Writer 上下文）**：`build_continue_writer_context` 追加 `【故事Logline】` 注入。
- **验证**：`cargo test --lib` 974 passed（+3：logline 生成 / 跳过 / 持久化）；fmt / clippy（baseline 550）/ tsc / prettier / architecture_guard 全绿。

### v0.30.21 - 续写资产层级生成（世界观 -> 故事大纲 -> 章节大纲 -> 正文）

- **根因**：续写路径 `ensure_assets`（`coordinator.rs`）仅检查 `characters` 表行数，角色存在即返回--不检查、不生成 world_buildings / story_outlines。`build_continue_writer_context` 不注入故事大纲，`write_chapter` task 仅"续写第N章"无方向约束，导致生成内容缺乏方向和情节推进。
- **Fix A（ensure_assets 扩展）**：角色检查后追加 world_buildings / story_outlines 检查；缺失时调 `ensure_world_building` / `ensure_story_outline` 单次 Producer LLM 调用（不跑 tool_loop，不抢主创 LLM）生成并落库。失败时 `log::warn` + `Ok(())` 不阻断续写。
- **Fix B（build_continue_writer_context 注入故事大纲）**：读 `story_outlines.content` 注入 writer task（4000 字符预算），为 writer 提供整体推进方向。
- **Fix C（generate_chapter_outline）**：writer tool_loop 前单次 Producer LLM 调用生成章节大纲（服从故事大纲），写入黑板 Draft 区 `key=outline-{chapter_key}`。无故事大纲时跳过（返回空串）。strict writer task 含故事大纲 + 本章大纲 + 写作要求（起伏/转折/冲突）。
- **Fix D（handle_gate 存储 outline_content）**：装配 Scene 时从黑板读取章节大纲，存入 `scenes.outline_content`。
- **层级约束**：世界观构建 -> 故事大纲 -> 章节大纲 -> 正文，每一层基于上一层生成，形成从世界观到正文的约束链。
- **验证**：`cargo test --lib` 971 passed（+4：ensure_world_building / ensure_story_outline / generate_chapter_outline / skip_without_story_outline）；fmt / clippy（baseline 550）/ tsc / architecture_guard 全绿。

### v0.30.20 - Agency 续写效率优化与质量门硬化

- **背景**：对照创世路径（`run_genesis`）审计续写路径（`run_continue` / `run_continue_batch`），发现三项结构性缺口（无 run 级 deadline / 无资产预注入 / 无散文回退）+ editor 质量门 deadline 仍为 None（v0.30.4 有意豁免，但 v0.30.19 的 salvage + prose_fallback 已使 deadline 安全可行）。
- **P0-1 续写 run_deadline**（`coordinator.rs`）：`run_continue` / `run_continue_batch` 入口调 `setup_run_deadline()`，与创世一致。`setup_run_deadline` 在 `app_handle=None`（测试环境）时 no-op。deadline 设置后 `run_role_with_llm_and_budget` 自动读取并传给 tool_loop（剩余 <30s 熔断保产出）。
- **P0-2 续写 writer 散文回退**（`coordinator.rs`）：参数化 `writer_prose_fallback` 新增 `chapter_key: &str`（替换硬编码 "第1章"），prompt "第一章正文" -> "章节正文"（generic）；`write_chapter` 熔断时 reason == "连续解析失败" 回退散文单调用（与 genesis legacy 同理），MaxTurns/Deadline 仍 Err。
- **P0-3 续写 writer 上下文预注入**（`coordinator.rs`）：新建 `build_continue_writer_context` 从 DB 读角色（`CharacterRepository`）/世界（`WorldBuildingRepository`）/最近 2 场景（`SceneRepository`），格式化截断 8000 字符注入 writer task，消除多轮 board_read/asset_query 轮询（tool_loop 从 3-7 轮降到 1-2 轮）。空则保留原 task（writer 自轮询）。
- **P1-1 Editor 质量门 deadline**（`coordinator.rs`）：`evaluate_gate_impl` 新增 `deadline: Option<Instant>` 参数替换原 `None`；`evaluate_gate` 方法传 `self.current_deadline()`；`GateRunner` 结构体新增 `deadline` 字段，`gate_runner()` 工厂设 `self.current_deadline()`，`GateRunner::evaluate` 传 `self.deadline`。v0.30.19 的 salvage + prose_fallback 使 deadline 安全（熔断后仍有两次兜底）。
- **P1-2 Editor 草稿预注入**（`coordinator.rs`）：`evaluate_gate_impl` editor task 从 "审查 draft 区的最新章节草稿" 改为注入 `draft.content`（截断 8000 字符），editor 无需 board_read 即可审查（tool_loop 从 2 轮降到 1 轮）。
- **P1-3 连接超时调优**（`config/settings.rs`）：`llm_connect_timeout_secs` 默认 60s -> 15s（TCP 连接建立超时，非响应超时；模型不可达时浪费从 240s 降到 60s）。
- **验证**：`cargo test --lib` 967 passed（+2：续写散文回退 + 上下文预注入）；fmt / architecture_guard 全绿；clippy 549（零新增）；tsc 通过。

### v0.30.19 - 质量门编辑审计 Agent 熔断修复（本地模型 JSON 不遵从，散文回退）

- **根因**：`evaluate_gate_impl`（`coordinator.rs`）中 editor_auditor 的 ReAct tool_loop 在本地模型（Qwen 3.6）不遵从 JSON action 格式时连续解析失败/达到最大轮数（6 轮）熔断。原实现 `if editor_out.aborted` 直接返回 `GateOutcome::Failed`，既不 salvage 末轮输出也不尝试替代路径，导致整 run 失败。与 v0.30.3 writer 熔断同类（本地模型 JSON 不遵从），但 editor 路径此前无散文回退。
- **Fix（两层兜底，`coordinator.rs`）**：①salvage--移除 `editor_out.aborted` 早返回，熔断时仍先 `parse_lenient::<EditorVerdict>` 尝试从末轮输出提取裁决 JSON；salvage 成功则用之，熔断则 break 进散文回退（同模型重试必同败，不重试）。②散文回退--新增 `editor_verdict_prose_fallback` 自由函数，单次 `llm.complete()` 直接请求裁决 JSON（不经 tool_loop/工具），复用 editor 系统提示词审查标准 + 追加「直接输出 JSON、不走工具循环」强约束。与 `writer_prose_fallback`（v0.30.3）同理。回退失败才降级 `Failed`。
- **验证**：`cargo test --lib` 965 passed（+1 `test_editor_verdict_prose_fallback` 正向回归；2 个现有熔断测试更新为显式验证回退也失败时 run 仍 failed）；fmt / clippy（baseline 550 零新增）/ architecture_guard 全绿。

### v0.30.18 - 修复幕前意图分类 null 崩溃（v0.30.16 CI E2E PAGEERROR 根因）

- **根因**：`handleSmartGeneration`（`FrontstageApp.tsx`）调用 `classifyIntent` 后直接读 `classification.is_new_novel`。`classifyIntent` 走 `loggedInvoke`（出错即 throw），但 **resolve 为 null 时不抛异常**--catch 块只拦抛出异常，无法拦截 null。E2E 环境 `e2e/mock-tauri.ts` 对未注册命令默认 `return null`（`classify_intent` 未 mock），导致 `classifyIntent` resolve 为 null -> `null.is_new_novel` -> `TypeError: Cannot read properties of null (reading 'is_new_novel')` PAGEERROR，幕前崩溃，连带 6 个 E2E（设置页/自动保存/创世重复）失败。v0.30.16 master 与 tag 两次 CI 均 hit；v0.30.15 未触发（非确定性）。真实用户若后端序列化异常返回 null 也会崩。
- **Fix（`FrontstageApp.tsx`）**：① catch 块之后新增 post-catch null 兜底--`if (!classification)` 时填充续写兜底对象（与 catch 同语义：`is_new_novel=false`，因误判续写为创世会启动 Agency 全流程覆盖工作，故默认偏向续写），避免 `null.is_new_novel` 崩溃；② 不再缓存 null 结果（`if (classification)` 守卫 cache.set），避免缓存 null 导致每次重入重调。
- **macOS 构建失败（v0.30.16 tag）**：`Failed to create Info.plist: Io(code 5, "Input/output error")`--GitHub macOS runner 瞬时磁盘 I/O 错误，同代码 master 构建成功（39m18s）。属 flake，已 `gh run rerun --failed` 重建，非代码问题。
- **验证**：`npx tsc --noEmit` ✅；`npx vitest run` 307 passed / 3 skipped（含 genesis-duplicate 14 项）；`npm run format:check` ✅。纯前端，cargo 基线 964 不变。

### v0.30.17 - 幕前顶部创世状态显示三 Agent 动作/进度

- **背景**：用户反馈幕前顶部创世流程状态提示信息不足，看不出「主创在干嘛、做完了什么工作」。Agency 创世的后端 `agency-agent-activity` 事件（`coordinator.rs` emit_activity，role=lead_writer/producer/editor_auditor，action=start/done，detail=概念/首章/深度资产/审查/装配）早已存在，但此前仅幕后 `AgencyStudio` 消费，幕前未订阅。底部 LLM 连接状态本次不动。
- **新增 `useAgencyAgentActivity` hook（`src-frontend/src/frontstage/useAgencyAgentActivity.ts`）**：幕前订阅 `agency-agent-activity`，按 主创/管理/编辑审计 顺序聚合各角色最新一条活动，产出 `{ text, done }[]` 文案（进行中「主创正在写第一章」，已完成「管理已完成深度资产」）；订阅 `agency-run-progress`，run 结束（completed/failed/cancelled/error）时清空，避免创世结束后残留陈旧进度。
- **接线 `FrontstageHeader.tsx`**：顶部状态栏在 `orchestratorStatus` 之后渲染各 Agent 进度条目（进行中琥珀 `saving` 态、已完成绿色 `saved` 态，title 标注「创世多 Agent 进度（主创 / 管理 / 编辑审计）」），无活动时不占位。
- **附带（用户级永久指令）**：`AGENTS.md` 强制构建规则 #2 改为「本地构建仅在用户明确要求时执行」--推送后由 GitHub Actions 负责全平台构建，本地仅跑 `cargo test` / `tsc` / `vitest` 等验证命令，省略耗时 `cargo tauri build` 打包。
- **验证**：`npx tsc --noEmit` ✅；`npx vitest run` 307 passed / 3 skipped（+2：三 Agent 进度渲染 + run 结束清空）；`npm run format:check` ✅。纯前端，无 Rust 变更（cargo 基线 964 不变）。

### v0.30.16 - 故事资产手动编辑（补齐编辑缺口）

- **背景**：审计后台发现 故事大纲/故事摘要 只读展示（`useUpdateStoryOutline`/`useUpdateStorySummary` hook 零调用），伏笔无内容编辑+删除，角色关系无编辑。角色/世界构建/场景已有完整编辑，无需改动。
- **Gap 1 故事大纲编辑（`pages/Stories.tsx`）**：只读 `<p>` 改为 查看/编辑 切换（textarea + 保存/取消），保存调 `useUpdateStoryOutline`（后端命令已就绪）。
- **Gap 2 故事摘要编辑（`pages/KnowledgeGraph.tsx`）**：抽取 `SummaryCard` 组件，查看/编辑 切换 + 保存，调 `useUpdateStorySummary`。
- **Gap 3 伏笔内容编辑+删除（后端+前端）**：`ForeshadowingTracker` 新增 `update_foreshadowing`/`delete_foreshadowing` 方法；新增 `update_foreshadowing`/`delete_foreshadowing` Tauri 命令并注册（`handlers.rs`）；前端新增 `useUpdateForeshadowing`/`useDeleteForeshadowing` hook，`Foreshadowing.tsx` 卡片加 编辑表单（内容/重要性/设置场景）+ 删除按钮。
- **Gap 4 角色关系编辑（前端）**：新增 `useUpdateCharacterRelationship` hook（后端 `update_character_relationship` 已存在），`Characters.tsx` 的 `RelationshipCard` 加 编辑表单（关系类型/描述）。
- **验证**：`cargo test --lib` 964 passed；`npx vitest run` 305 passed；tsc / `cargo +nightly fmt` / clippy（零新增，baseline 550）/ architecture_guard 全绿。

### v0.30.15 - 场景围绕故事大纲生成（创作原则加固）

- **创作原则**：有故事大纲时，场景必须围绕故事大纲展开。用户报告续写内容与故事大纲"两张皮"（场景大纲写"金敏秀"，续写跑偏到核电站，与故事大纲"韩雪/李明在首尔"脱节）。
- **根因 A（场景大纲生成用错提示词）**：`generate_scene_outline` 复用故事级 `outline_planner.md`（要求三幕式/章节划分/角色弧线），`task.input` 几乎为空且**不注入 story_outlines.content** -> 模型幻觉新角色"金敏秀"（不在角色卡），场景大纲与故事大纲冲突。
- **根因 B（writer 看不到故事大纲）**：续写走 TimeSliced/TriShot，prompt 只用 `WriteTimeBundle.to_prompt()`；故事大纲只在 Full/Fast 路径计算，**从未到达 writer** -> 内容偏离大纲。
- **Fix A（场景大纲生成锚定故事大纲，`creation_commands.rs` + `agents/service.rs` + 新提示词）**：新增场景级提示词 `resources/prompts/planner/scene_outline.md`（强制复用已登场角色、禁止发明新角色、围绕故事大纲对应节点展开）；`generate_scene_outline` 加载 `story_outlines.content` + 场景序号注入 `task.parameters`；`build_outline_prompt` 分流（场景模式用 `scene_outline`，workflow 故事级仍用 `outline_planner`）。
- **Fix B（writer 锚定故事大纲，`domain/write_time_bundle.rs` + `creative_engine/write_time_bundle.rs`）**：WriteTimeBundle 新增 `story_outline` 字段，`load_sync` 加载 `story_outlines.content`，`to_prompt()` 在世界观红线**之后**插入权威段【故事大纲（本场景必须围绕此大纲展开，禁止偏离）】（保持红线第一不变量）；冲突时以故事大纲为准并使用已登场角色。一处覆盖 TimeSliced+TriShot。
- **验证**：`cargo test --lib` 964 passed（+4）；fmt / architecture_guard 全绿；clippy 零新增（baseline 550）。
- **注意**：现有"金敏秀"场景大纲需用户重新点"生成大纲"覆盖（Fix A 修生成器）；Fix B 让 writer 即使面对旧毒大纲也锚定故事大纲。

### v0.30.14 - 续写返回风格增强模板修复（多步 plan 尾部非 writer 覆盖正文）

- **根因（结构性）**：`execute_plan`（`planner/executor.rs:685-687`）用**最后产出 `content` 的步骤**作为 `final_content` 返回用户。force-correction（防线 2）只修正**首步**，无法拦截多步 plan **尾部**的 `style_enhancer`/`inspector`--尾部非 writer 的模板/报告会覆盖 writer 已产出的正文。用户报告"增强第二章"得到 `[inspector, style_enhancer]` 多步 plan，style_enhancer 收到 inspector 报告后抱怨"这是一份质量检查报告而非章节原文"。这是该误路由 bug **第 5 次复发**（v0.30.10/11/12/13 各堵一条路径：模板重放/朴素子串/inspector 漏拦/SING 绕过，但多步尾部漏网）。
- **Fix（防线 3，`planner/mod.rs` + `planner/executor.rs`）**：新增 `PlanGenerator::sanitize_plan_for_prose_request`，在 plan 执行咽喉点 `execute_with_context`（force-correction 之后）对所有 `is_prose_request` plan 统一净化：①移除 `builtin.style_enhancer`/`text_formatter`/`character_voice`/`emotion_pacing` 等绝不产出正文的技能步骤；②续写（`is_continuation`）塌缩为单 writer 步；③其余 prose 请求弹出尾部非 writer 步骤，**保证末步为 writer**（`final_content` = 正文），保留 `[inspector, writer]` 等 Rule 9 合法流；④净化后空则补 writer 步。非 prose 请求（显式审查 `Audit`/`is_prose_request=false`）不净化。
- **验证**：`cargo test --lib` 960 passed（+12 sanitize 回归）；fmt / architecture_guard 全绿；clippy 零新增（baseline 550）。

### v0.30.13 - 续写返回风格增强模板修复（SING 路径绕过 force-correction）

- **根因**：planner force-correction（防线 2）只在 `PlanGenerator::generate_plan` 内施加，而 `PlanExecutor::execute_with_context` 的 SING（IntentionGraphPlanner）路径直接返回 plan（`planner/executor.rs:148-178`）、**完全绕过** `generate_plan`。当 SING 把续写路由到 `builtin.style_enhancer`（Skill 资产）作为首步时，force-correction 从不执行，style_enhancer 收到空 content 返回"请提供需要增强的原始文本"模板。v0.30.11 禁用模板重放消除了模板路径，但 SING 路径的绕过漏洞仍在。
- **Fix（结构修复，`planner/mod.rs` + `planner/executor.rs`）**：提取 `PlanGenerator::force_correct_first_step_to_writer` 为 `pub(crate)` 方法（封装 swap + understanding/purpose 标注），在 `generate_plan` 与 **plan 执行咽喉点** `execute_with_context`（所有 plan 来源 SING/PlanGenerator/fallback 必经，`execute_plan` 之前）**统一施加**。SING 路径产生的 `builtin.style_enhancer`/`inspector`/`outline_planner` 等首步经咽喉点修正为 `writer`。幂等：已为 writer 的首步不受影响，两处重复调用安全。
- **验证**：`cargo test --lib` 948 passed（+4 咽喉点回归）；fmt / architecture_guard 全绿；clippy 零新增（baseline 550 -> 549）。

### v0.30.12 - 续写返回审查报告修复（force-correction 漏拦 inspector）

- **根因**：planner force-correction（`planner/mod.rs` 防线 2）的"强制改 writer"capability 列表含 `outline_planner`/`style_mimic`/`plot_analyzer`/`builtin.*`，**漏掉 `inspector`**；提示词 Rule 9 允许"有内容时用 inspector 先审后写"、Rule 21 never-use 列表也漏 inspector。本地模型（Gemma-4-31B）把"继续写当前这部小说"误判为"审查/改进已有文本"路由到 `inspector` -> force-correction 不拦 -> inspector 运行 `inspector_system` 提示词 -> 产出"总体评分 0.85 / 具体问题清单"审查报告作为生成结果。
- **Fix A（force-correction 主修复）**：提取纯函数 `PlanGenerator::should_force_correct_to_writer`（可单测），将 `inspector` 纳入 swap-to-writer 列表，按 LLM 分类分流：续写（`is_continuation`）/ 创世 / 无分类 / 审查且 `is_prose_request=true`（分类矛盾兜底）强制 `writer`；仅纯审查（`Audit` 且非 prose）与改写润色（`Rewrite`，Rule 9 流，最终输出仍是 writer 正文）保留 `inspector`。
- **Fix B（提示词）**：Rule 9 澄清"继续写/续写/往下写"是续写而非 refine，必须直接 `writer`、绝不用 `inspector`；Rule 21 将 `inspector` 加入 prose 请求 never-use 列表。
- **验证**：`cargo test --lib` 944 passed（+8 force-correction 回归）；`npx vitest run` 305 passed；tsc / fmt / clippy / format:check 全绿。

### v0.30.11 - 全面整改：用 LLM 解析器替换朴素子串意图匹配

- **背景**：审计全项目发现 ~30 处 `.contains()`/`.includes()` 朴素子串匹配，其中 6 处高危直接在用户自然语言输入上做意图路由（`find_match`、`is_novel_creation_intent` 前后端、`from_instruction_and_context`、force-correction、`synthesize_query_rule_based`），是 v0.30.10 `PlanTemplateLibrary` bug 的同类。用户指示用 LLM 解析器替代。
- **核心：`IntentParser::classify_writing_intent`（intent.rs）**：一次 LLM 调用产出 `WritingIntentClassification`（is_new_novel / is_continuation / task_type / is_prose_request / input_clarity / detected_genre / confidence）。最快模型 + 8s 超时 + 保守兜底（is_new_novel=false=续写）+ 会话 LRU 缓存。误判代价不对称：误判续写为创世会启动 Agency 全流程覆盖工作（灾难），故默认偏向续写。
- **Site 4**：`smart_execute` 用 `classification.is_new_novel` 替代 `is_novel_creation_intent`；前端 `classify_intent` IPC 先行，payload 透传分类，后端信任不重复调用。
- **Site 1**：`find_template` 禁用（恒返回 None，patterns 来自 LLM understanding 切词噪声）；`find_match` 标 `#[allow(dead_code)]`。
- **Site 4b/5**：TriShot 守卫 + 续写绕过 + force-correction 读 `PlanContext.intent_classification`（无新 LLM 调用）。
- **Site 3**：`from_instruction_and_context` 修运算符优先级 bug + 移除单字 pattern + `hint` 参数经 `task.parameters["task_type_hint"]` 透传。
- **Site 8**：`build_writer_prompt` 题材优先 LLM `detected_genre` > `extract_genre`（加否定窗口 + 长度降序）> 故事 genre。
- **Site 7**：`detect_input_clarity` 移除单字信号；调用方读 `classification.input_clarity`。
- **Site 2**：`intention_graph::builder` LLM 主路径硬化（JSON 子串截取 + raw_input 推断）+ 规则兜底默认 `generate prose`。
- **前端**：新增 `classifyIntent` API；`handleSmartGeneration` 入口调分类（缓存 + 兜底）；删除 `isNovelCreationIntent`/`isContinuationIntent`。
- **字段名 bug**：prompt 指示返回 `"is_prose"` 但 struct 字段 `is_prose_request` 无 alias 致恒 false；加 `#[serde(alias = "is_prose")]` 修复（单测捕获）。
- **不适用 LLM（诚实标注）**：Site 9 `derive_model_role_from_label`（内部 label）、Site 10 `discover_from_outputs`（LLM 输出，需结构化 findings 改造）保留为后续。
- **验证**：`cargo test --lib` 936 passed（+1）；`npx vitest run` 305 passed；tsc / fmt / format:check / clippy / architecture_guard 全绿。

### v0.30.10 - 续写返回风格增强模板修复（模板匹配误路由 + content 空兜底）

- **根因**：`PlanTemplateLibrary::find_match` 用朴素 substring 匹配（`user_input.contains(pattern)`），之前记录的 style_enhancer 计划的触发词（如"这部小说"）会匹配"继续写当前这部小说"，导致续写请求**跳过 planner LLM 和所有安全规则**，直接重放 style_enhancer 计划。style_enhancer 收到空 content 后返回"在您提供文本后，我将从以下几个方面进行增强"模板而非续写正文。
- **Fix A（executor.rs 主修复）**：`execute_with_context` 在 `find_template` 前检测续写意图词（继续/续写/接着写/往下写/接下来/后续/接着），命中则跳过模板匹配，强制走 planner LLM 路径，确保续写请求由 Rules 8/19/21 正确路由到 writer。
- **Fix B（mod.rs 防线 2 扩展）**：force-correction 从仅捕获 `outline_planner` 扩展到 `style_mimic` / `plot_analyzer` / `builtin.style_enhancer` / `builtin.text_formatter` / `builtin.character_voice` / `builtin.emotion_pacing`，当首步为这些 capability 且输入含写作/续写关键词时强制改为 `writer`。
- **Fix C（executor.rs content 兜底）**：新增 `inject_content_fallback` 静态方法，为 `style_mimic` / `plot_analyzer` / `builtin.*` 技能在 content 为空时按 depends_on -> step_outputs -> plan_context.current_content_preview 顺序注入文本，与 v0.30.9 inspector draft 兜底同理。
- **Fix D（mod.rs Rule 21 强化）**：Rule 21 新增"继续"/"续写"关键词和"这部"/"当前"故事相关主语，并明确禁止 `style_mimic` / `plot_analyzer` / `builtin.style_enhancer` 用于 prose 请求。
- **验证**：`cargo test --lib` 929 passed（+5：content 兜底注入 5 场景）；fmt / clippy 无新增告警。

### v0.30.9 - 续写返回 Inspector 审查模板修复（draft 空内容兜底注入）

- **根因**：legacy planner 的 LLM 生成的 ExecutionPlan 中 inspector 步骤常遗漏 `"draft": "{{step_N}}"` 参数。`execute_inspector` 仅从 `params["draft"]` 读取待检查正文，缺失时 `task.input` 为空串，`build_inspector_prompt` 渲染出"【待检查内容】部分为空"的模板文本，Inspector 直接将该模板作为"审查结果"返回，用户看到审查模板而非续写正文。
- **Fix A（主修复·executor.rs）**：`resolved_params` 块新增 inspector draft 兜底注入--当 `capability_id == "inspector"` 且 `draft` 为空时，按 `depends_on` 顺序查找 writer 步骤的 `step_outputs["content"]`，找不到则扫描全部 `step_outputs`，自动注入非空 content 作为 `draft`。提取为可测静态方法 `inject_inspector_draft_fallback`。
- **Fix B（提示词·mod.rs）**：planner 提示词 Rule 9 强化--明确要求 inspector 必须使用 `"draft": "{{step_id}}"` 传参，否则 inspector 收到空内容只返回请求输入的模板；JSON 示例增加 inspector 步骤示范 `"draft": "{{step_1}}"` + `depends_on: ["step_1"]`。
- **验证**：`cargo test --lib` 924 passed（+5：inspector draft 兜底注入 5 场景）；fmt / clippy 无新增告警。

### v0.30.8 - 全面修复 nullable 列读取（Invalid column type Null 系列）

- **根因**：`world_buildings.cultures`（index 5）和 `rules`（index 3）在基础 schema 为 nullable TEXT，旧数据该列为 NULL，repository 用 `row.get(N)?` 读非空 `String` 即报 `Invalid column type Null`。与 v0.30.6 `dynamic_traits` NULL 同类。
- **全面排查**：系统性审查全部 27 个 repository 文件，发现并修复所有 nullable 列被当作非空 `String` 读取的问题（共 8 个文件、31 处）：`world_building_repository`（cultures/rules）、`scene_repository`（characters_present/character_conflicts × 4 方法）、`scene_version_repository`（同上 × 2 方法）、`studio_config_repository`（llm_config/ui_config/agent_bots）、`writing_style_repository`（custom_rules）、`knowledge_graph_repository`（attributes × 4 / evidence × 2）、`user_preference_repository`（6 列 × 2 方法）。全部改为 `Option<String>` + `unwrap_or_default`/`unwrap_or_else` 兜底。
- **迁移**：V112 回填 `world_buildings.cultures/rules`；V113 全面回填 scenes/scene_versions/studio_configs/writing_styles/kg_entities/kg_relations/user_preferences 的所有 nullable JSON/TEXT 列。
- **验证**：`cargo test --lib` 919 passed（+2：world_buildings NULL 兜底 + 合法 JSON 解析）；fmt / architecture_guard 全绿。

### v0.30.7 - 计划执行失败修复（LLM 在 depends_on 写入上下文名）

- **根因**：LLM 生成的 ExecutionPlan 在 `depends_on` 中混入上下文名（如 `"Story Context"`、`"writer"`）而非 plan 内 step_id。`topological_sort`（swarm.rs）已正确跳过非 step_id 依赖，但 `PlanExecutor::execute` 的依赖校验未对齐--遇到非 step_id 依赖直接判 `not found`，导致 step_1 被跳过 -> step_2 依赖 step_1 也 not found -> step_3 链式失败，整 plan 崩溃。
- **Fix（executor.rs）**：依赖校验前收集 `plan_step_ids` 集合，对不在集合中的依赖（非 step_id）跳过校验并 `log::warn`，与 `topological_sort` 行为一致；仅校验真实 step_id 依赖是否已产出。参数引用 `{{step_id}}` 由 `resolve_parameters` 兜底处理缺失键。
- **Fix（mod.rs）**：Rule 3 强化--明确 `depends_on` MUST ONLY contain step_id values of OTHER steps in this same plan，NEVER put context names / capability names / free text，并举例 `"Story Context"` / `"writer"` 为错误值。
- **验证**：`cargo test --lib` 917 passed（+2：topological_sort 非 step_id 依赖跳过 + 混合依赖排序）；fmt / tsc / architecture_guard 全绿。

### v0.30.6 - 获取角色失败修复（dynamic_traits 列 NULL）

- **根因**：`characters.dynamic_traits` 列在基础 schema 为 nullable TEXT（无 `NOT NULL`/`DEFAULT`），StoryForge 数据迁移导入的旧角色行该列为 NULL。`get_by_story`/`get_by_id` 用 `row.get::<_, String>(9)` 读非空类型，遇 NULL 即报 `Invalid column type Null at index: 9, name: dynamic_traits`，续写/创世获取角色失败弹 Fatal 诊断卡片。
- **修复（双层）**：读取层 `get_by_story`/`get_by_id` 改读 `Option<String>` 兜底 `"[]"`（NULL 行返回空 `dynamic_traits`）；数据层 V111 迁移回填 `characters.dynamic_traits NULL -> '[]'` 保证一致。
- **验证**：`cargo test --lib` 915 passed（+2：NULL 兜底 + 合法 JSON 解析回归）；fmt/clippy 无新增告警。

### v0.30.4 - 幕前输入历史持久化（按故事隔离）

- **功能**：幕前底部输入框已输入内容现长久保留，关闭窗口/重启后不丢失，与编码工具一致。每条提交按故事 ID 隔离存入 `localStorage`（`frontstage:inputHistory:<storyId>`，最近 20 条），切换故事自动加载该故事的历史。
- **UX**：保留既有 ghost-hint 交互（↑/↓ 切换 LLM 建议 <-> 历史记录，-> 确认填充），持久化对导航无侵入。localStorage 不可用时静默降级为内存态。
- **实现**：`src-frontend/src/frontstage/FrontstageApp.tsx`（模块级 `loadInputHistory`/`saveInputHistory` + `useEffect` 加载 + `handleInputSubmit` 同步持久化）。
- **验证**：`npx vitest run` 297 passed（+2：持久化写入 + 重载召回）；tsc / prettier 通过。纯前端，无 Rust 变更。

### v0.30.5 - 创世流程严重超时修复（600s 顶满 + 前端先杀后端）

- **根因（对照 `creative_workflow.log` 2026-07-20 08:37–08:47）**：Agency 创世 5 阶段慢，producer tool_loop 5.5min + writer tool_loop 4.5min（含本地模型连接超时 60s×4 候选=240s）顶满 600s；前端 `Promise.race` 600s 到了先 `llm_cancel_all_generations` 杀掉后端，创世被 CANCELLATION 砍掉无产出 + 僵尸 run 卡死故事续写；writer 在 tool_loop 中盲目 board_read 轮询 7-10 轮。
- **Fix 1**：`config/commands.rs` 放开 `smart_execute_total_timeout_secs` / `frontend_timeout_secs` clamp 上限 600->1800（默认仍 600s）；`GeneralSettings.tsx` 输入框 max 同步到 1800。
- **Fix 2**：`FrontstageApp.tsx` 创世路径前端超时 = 后端 + 30s 缓冲（主超时 + 看门狗 + 诊断卡片三处统一）；提取纯函数 `utils/genesisTimeout.ts`。
- **Fix 3（核心）**：`coordinator.rs` 新增 `asset_retrieval_plan`--writer 前置单次 LLM 调用从资产 catalog 选出必需 key（30s 超时 + 失败兜底全量 + `RetrievalPlan` 别名兼容），消除 writer 多轮 board_read 轮询。
- **Fix 4**：`coordinator.rs` 新增 `build_writer_assets_context`--检索规划后按 key 过滤资产全文预注入 writer task（8000 字符预算截断），tool_loop 轮次从 7-10 降到 1-2。
- **Fix 5**：`tool_loop.rs` 新增 run 级 deadline 感知（`with_deadline` + 每轮检查，剩余 <30s 熔断保产出）；新增 `LoopAbortReason` 枚举，`circuit_break_reason` 识别 deadline 熔断返回"剩余时间不足"，coordinator writer 路径据此快速失败而非回退 legacy（避免 legacy 又跑一遍超时）。
- **验证**：`cargo test --lib` 913 passed（+14）；`npx vitest run` 305 passed（+8：genesisTimeout 纯函数）；tsc / fmt / clippy / format:check / architecture_guard 全绿。

### v0.30.3 - 创世主创 Agent 熔断修复（本地模型 JSON 不遵从）

- **根因**：本地模型（Qwen/Gemma）对 `producer_depth_assets` 的 `complete_json` 返回散文而非 JSON -> 快速路径失败回退 legacy -> legacy writer tool_loop 要求 JSON action 而模型写散文 -> 连续 3 轮解析失败熔断，首章未完成。
- **Fix A（主修复）**：`producer_depth_assets` 在 `parse_lenient` 失败时兜底 salvage 散文为 world 资产，快速路径继续，避免回退 legacy。
- **Fix B（可诊断性）**：`tool_loop.rs` 此前零条日志，解析失败 raw 响应只存在内存 run 结束即丢弃。现每轮解析失败 + 熔断点 + max-turns 均 `log::warn!`（含 role、轮次、截断 raw 500 字）。
- **Fix C（纵深防御）**：legacy writer "连续解析失败"熔断时回退自由体散文单调用（新 `writer_prose_fallback`），"达到最大轮数"仍直接 Err。
- **验证**：`cargo test --lib` 899 passed（+2 新测试）；fmt/clippy 通过。

### Agency 多代理创作框架 P1 — 创世 2.0 骨架（串行端到端）

- **新模块**：`src-tauri/src/agency/`（board 黑板 / tool_loop ReAct 工具循环 / roles 三角色 / coordinator 协调器（P2 起含并行稳态循环 gate(n-1)∥writer(n)）/ repository+models 持久化 / bus 消息总线（P2 已接线：修订提案 proposal）/ budget 角色预算 / commands IPC）。
- **IPC**：`agency_start_genesis` / `agency_get_run` / `agency_list_board` / `agency_cancel_run` / `agency_continue_chapter` / `agency_continue_batch`。
- **提示词目录**：`resources/prompts/agency/`。
- **依赖边界**：agency 允许依赖 db/llm/router/prompts，不允许被反向依赖。
- **教训**：迁移文件必须与引用它的代码同一 commit 提交（P3 T5 教训：V109 与测试被并行 CI 提交拆散导致断档）。
- 设计：`docs/plans/2026-07-17-agency-multi-agent-framework-design.md`（P1-P3 已完成，除真机验收外）。

### v0.26.59 — StoryForge → StoryMoss 品牌收尾，官网落地页上线

- **品牌重命名**：完成仓库文档、CI、Tauri 配置与 GitHub Release 的 StoryForge → StoryMoss 全局替换。
- **官网落地页**：`landing/` 站点部署至 `https://ai.91z.net`，重写产品介绍并加入 Logo；下载按钮按平台自动匹配安装包。
- **验证**：landing 19 tests passed。

### v0.26.58 — 修复 OpenAI/Deepseek 模型因 top_p=0 健康检测失败

- **根因**：OpenAI 兼容 API（含 Deepseek）不接受 `top_p = 0.0`，会返回 `Invalid top_p value`。
- **修复**：`OpenAiAdapter` 在序列化前过滤 `top_p`，仅保留 `(0, 1.0]` 的合法值；非法值自动省略，让服务端使用默认值。
- **验证**：新增 `llm::openai` 单元测试；`cargo test --lib` 770 passed。

### v0.26.57 — 自动划分章节、本地导出保存与提示词目录

- **自动划分章节**：后台设置新增「按字数 / 按情节」分章策略；字数上限留空默认 3000 字；场景保存空闲 30s 后仅对最新章自动切分。
- **本地导出保存**：导出结果通过系统保存对话框落盘；文本写 UTF-8，二进制（pdf/epub）复制后端临时文件；取消时不关闭弹窗。
- **提示词目录**：提示词注册表新增「打开目录」按钮，直接用系统文件管理器打开 bundled prompts 目录；编辑器改用原生 textarea 避免 CSP 导致 Loading。
- **验证**：`cargo test --lib` 769 passed；`npx vitest run` 292 passed；tsc / fmt / format:check 全绿。

### v0.26.56 — 网关契约测试串行化（mock app_data_dir）

- **修复**：写 AppConfig 的 executor 契约测试加进程锁，避免并行污染导致 `creative_x_overrides` 偶发失败。
- **验证**：`creative_x_overrides` / `demoted_degraded` / `sticky_unhealthy` / `disabled_model` 并行 `--test-threads=8` 通过。

### v0.26.55 — 幕后模型列表开启/关闭开关

- **UI**：模型卡片「开启/关闭」；仅轮询已启用模型。
- **运行时**：复用 v0.26.54 fail-closed；`is_promotable` 要求仍在注册表。
- **验证**：ModelCard vitest + disable/probe Rust 契约。

### v0.26.54 — 修复创作模型被粘性降级绕过

- **根因**：Deepseek 已是创作/活跃，但连续失败 demotion 让 `resolve_role_model` 丢弃显式角色，Call3 长期用 MN-Oblivion。
- **修复**：显式角色不受粘性 demotion；Unhealthy 在 resolve 清一次再探；`set_active_model`/`save_settings` 清 demotion；`generate()` 用 `is_promotable`；禁用模型 fail-closed（持久化 enabled、不探测、活跃自动回退）。
- **验证**：gateway/health/commands 契约通过；architecture_guard。

### v0.26.53 — 故事名取消单击回幕后（双击改名可用）

- **修复**：故事名不再单击打开幕后（与双击改名冲突）；回幕后走设置按钮（禅模式也保留）。
- **验证**：Header 单击不调 `onOpenBackstage`；设置按钮可回幕后；双击仍进编辑。

### v0.26.52 — 修复模型新增与默认创作模型即时生效

- **幕前连接状态**：`model_config`/`app_settings` 刷新同步失效 `gateway-status`；状态栏含 `Unknown`。
- **创作模型**：用户显式角色允许 Unknown 置顶；`set_active_model(creative)` / `save_settings` 同步 `active_llm_profile`。
- **验证**：Rust 4 + vitest 5；tsc/fmt/architecture_guard。

### v0.26.51 — 幕前故事名与章节名内联改名

- **故事名**：草苔/未命名展示；有正文自动建「未命名」故事；双击改名。
- **章节名**：编辑器上方 + 顶栏状态统一双击改名；空标题 `第N章`；`update_scene` 持久化。
- **验证**：displayStoryTitle/ChapterTitle + Header/EditableChapterTitle 相关测试；tsc/format/architecture_guard。

### v0.26.50 — 修复打字触发后台运行与深度思考假超时

- **AutoIngest 防抖**：打字自动保存不再立刻抢本地模型（30s + BACKGROUND_LLM_SEMAPHORE）。
- **合同补齐静默**：不再用 `contract-auto-progress` 拉高 `isGenerating`。
- **活动同步**：后台活动不得单独禁用输入栏；`isGenerating` 超时看门狗强制弹诊断。
- **验证**：scene_service 6；contract gate 2。

### v0.26.49 — 修复续写与正文脱节（末句硬锚点）

- Call3/TimeSliced 在 prompt 最末尾注入末 2 句硬锚点，覆盖「开场」类大纲指令；抗 Lost-in-the-Middle。
- **验证**：ending_anchor 相关 3 passed。

### v0.26.48 — 修复自动更新（GitHub Releases + latest.json）

- 开启 `createUpdaterArtifacts`；CI 产出签名更新包与 `latest.json`；Linux AppImage；下载进度累加与 404 提示。
- **验证**：`cargo test --lib updater::` 2 passed。

### v0.26.47 — CI 热修复（Rust fmt）

- `cargo +nightly fmt` 修复 v0.26.46 rust-check 失败；无逻辑变更。

### v0.26.46 — 创世方法论全链路、题材 match-or-create 与拆书持久化

- **方法论**：5 个 background 模板恢复 `strategy_section`；Genesis 分步注入 + `methodology_step` 推进；ID 归一化；Selector 预填 `recommended_methodology_id`。
- **题材**：`EnsureGenreProfileStep` match-or-create；概念保真硬化。
- **拆书**：StoryArc/作者/伏笔落库；分块超时与并发止血；前端按书过滤进度。
- **验证**：genesis/methodology/prompt 契约 20+ passed。

### v0.26.45 — Genesis 人物卡强制落地（姓名 + 欲望/阻力）

- **人物卡**：merge/render/probe 纯函数；first_scene + Call3 双重注入；真名≥80%、欲/阻信号探针；软重试 fail-open。
- **验证**：narrative 61；protagonist_card 6。

### v0.26.44 — Genesis 首章质量：开篇骨架与提示词加厚

- **开篇骨架**：quick_phase 四步（概念→策略→骨架→开篇）；10s 超时 fail-open；概念字段规则映射降级。
- **提示词**：概念加厚（主角/冲突/世界锚点）；strategy_selector 中文化；first_scene 纪律单源化。
- **四元组 + 占位角色**：Genesis 接入 `infer_narrative_quartet`；TriShot 占位用骨架主角，去掉「异星末世」硬编码。
- **验证**：`narrative::genesis` 12 passed；骨架解析契约 +1。

### v0.26.43 — 修复底部状态栏 emoji 显示为方框

- **根因**：阶段文案嵌入 emoji + 解析正则拆碎中文/代理对；WebView 缺字显示 □□。
- **修复**：纯文案 + `StatusIcon`（Lucide）；解析前剥 emoji。
- **验证**：StatusIcon / FrontstageBottomBar 相关 18 passed。

### v0.26.42 — 修复续写 Tab 提示可见但无幽灵文本

- **根因**：Tab 接受后 30s `hideGhostUntil` / `postAcceptHideUntilRef` 未在新续写时清零；幽灵树仍渲染 Tab 条，幽灵段落被压住。
- **修复**：续写入口与 `setGeneratedText` 清零父级锁；RichTextEditor 新幽灵到达时清零本地锁（接受中不解除）。
- **验证**：`RichTextEditor.duplicate.test.tsx` 6 passed（+1）。

### v0.26.41 — 记忆统一读模型与 Finalize scene_id 根治

- **Finalize**：`scene_id` 贯穿 drafts/IPC/UI；直写编辑场景。
- **记忆**：`story_memory_facts` VIEW + `kg_entity_id` 链接；`list_unified_facts`；表不 DROP。
- **验证**：cargo 701；facade 7；finalize 3；vitest 261。

### v0.26.40 — 幕后资产闭环 P0–P3

- **P0**：侧栏热/温/冷/配徽章；合同/KG 生成影响说明；诊断组默认折叠。
- **P1**：SceneEditor 管线轨；KG 摘要进 WriteTimeBundle；MCP→设置扩展；Wizard 幂等+KG（既有）。
- **P2**：MemoryFacade；quality_gate 永不热路径 LLM。
- **P3**：TracingPanel 资产→prompt 覆盖率。

### v0.26.39 — 幕后信息架构全面重排

- **侧栏五组**：创作 / 故事资产 / 创作工具 / 洞察与运维 / 系统；中文重命名。
- **数据洞察**：合并用量/写作/功能使用；设置七 Tab 重组；拆书设置就近；账号死链修复。
- **验证**：`npx vitest run` 249 passed / 3 skipped；tsc/format 通过。

### v0.26.38 — 提示词面板修复与组合智能化

- **修复 Loading / 打开目录 / 导出**：Monaco CDN → textarea；`open_prompts_directory` 原生打开；dialog+fs 导出覆盖/完整包。
- **接通 FrameworkSelections**：methodology + contextual_injectors 确定性回灌 Call 3（0 额外 LLM）。
- **场景组合预览**：`preview_prompt_composition` + 面板分层跳转。
- **验证**：`cargo test --lib` 690 passed；`npx vitest run` 244 passed / 3 skipped；fmt、format、architecture_guard 均通过。

### v0.26.37 — 修复幕前「保存中」常亮与字数不更新

- **根因**：`update_scene` IPC 参数形状错误 + `appendAiContent` 不刷新字数/不调度保存。
- **修复**：`buildUpdateSceneIpcArgs`；追加后同步 `wordCount` + `scheduleAutoSave`。
- **验证**：vitest 242 passed / 3 skipped。

### v0.26.36 — 后台配置变更即时生效（超时/字体/主题）

- **超时热重载**：`save_settings` → `reload_config` + `app_settings` sync；幕前立刻用新超时。
- **首字节超时**：`llm_first_chunk_timeout_secs` 接入三适配器。
- **字体/主题跨窗口**：Tauri 事件 `editor-config-changed` / `color-theme-changed`。
- **验证**：cargo test 685；vitest 240 passed / 3 skipped。

### v0.26.35 — 全面落地幕后工作室审计残留 R1–R11

- **R1**：`list_stories` 返回真实 `scene_count`；Dashboard「场景」统计对齐。
- **R2**：CreationPathGuide 快速创作绑定 `runCreationWorkflow`；`App` 导航统一 `appStore.currentView`。
- **R3**：后端 `apply_wizard_to_story`（角色去重、首场景 upsert、KG 摄取）；前端单 IPC。
- **R4**：幕后 `App`/`GenesisPanel` 监听 `genesis-warnings`。
- **R5/R6**：PipelinePanel / SceneEditor 标注场景序号语义。
- **R7–R11**：世界构建文风 Tab、UsageStats 启发式、伏笔 Kanban、角色→场景跳转、拆书转故事导航。
- **验证**：`cargo test --lib` 685 passed；`npx vitest run` 237 passed / 3 skipped；fmt、format、architecture_guard、tsc 均通过。

### v0.26.34 — 修复提示词导入参数并新增「打开本地目录」功能

- **修复批量导入静默失败**：`PromptsPanel.handleImportAll` 调用 `save_prompt_override` 时参数键由错误的 `promptId` 修正为 `prompt_id`，与后端 `rename_all = "snake_case"` 对齐。
- **新增「打开目录」功能**：后端新增 `get_prompts_directory` 命令暴露当前 prompts 资源目录；前端标题栏新增「打开目录」按钮，使用系统文件管理器打开目录。
- **新增「刷新」功能**：支持重新加载提示词列表与目录路径。
- **改善错误展示**：加载失败时在页面上方显示具体错误信息。
- **导出/导入按钮归位**：将「导出」「导入」按钮从「全部重置」确认弹窗移至页面标题栏。
- **验证**：`cargo test --lib` 685 passed；`npx vitest run` 237 passed / 3 skipped；fmt、format、architecture_guard 均通过。

### v0.26.24 — 修复续写重复、截断与跨内容复述（5 项根因）

对照 `creative_workflow.log` 2026-07-07 08:44–09:05 续写会话（新写 → 多次续写）：

- **散布式句子块重复**：新增 `trimInterspersedRepeatedBlocks`（Rust + TS 对齐，golden 双跑），处理单次生成内意象循环重复。
- **跨内容重叠复述**：新增 `stripExistingOverlap`，剥离 Writer 复述已有正文段落（`startsWith` / `isTextDuplicate` 无法拦截的路径）。
- **截断末句污染**：新增 `trimDanglingTail`，裁掉 60s 超时硬截断留下的极短半句。
- **续写 8% 重试闸门**：TriShot 续写路径补齐 anti-repeat 重试（对齐 Genesis）。
- **前端管线统一**：`sanitizeContinuationOutput` 覆盖 smart_execute / appendAiContent / handleRequestGeneration。

### v0.26.23 — 修复续写卡死与幽灵文本混乱（4 项根因）

对照 `creative_workflow.log` 2026-07-07 续写会话时间线定位 4 个根因：

- **Bug B（卡死主因）**：`auto_contract` 4 个 LLM label（master_setting/chapter/scene_outline/default_character）加入 `is_silent_background`，后台补齐合同不再阻塞 `isAnyBackendActive`（原 6 分钟阻塞续写）。
- **Bug D（混乱主因）**：`handleSmartGeneration` 入口加重入守卫——存在未接受幽灵时先丢弃并提示，避免新旧续写结果竞争。
- **Bug A**：`RichTextEditor` 新增 `bodyForceHideGhost` state 镜像 `force-hide-ghost` 类，移除类时触发重渲染，消除幽灵 10s 渲染延迟。
- **Bug C**：续写（非创世首章）call3 超时上限 120s→60s，慢模型 fail-fast 回退到快模型（Gemma4 10s vs MN-Oblivion 198s）。

### v0.26.21 — 修复 Windows MSI 构建（迁移文件名重命名）

- v0.26.17 起打包 `src/db/migrations/` 为 Tauri resource，但 24 个迁移文件名含中文/全角逗号/破折号且最长 102 字符，导致 WiX `light.exe` 标识符生成失败（v0.26.14/v0.26.16 resources 引入前 Windows MSI 曾成功）。
- 重命名 24 个迁移文件为 ASCII 短名（保留 `V###` 前缀与排序）。`schema_migrations` 按 version 跟踪，已应用迁移不受影响；`parse_filename` 仅解析 `V###` 前缀，无逻辑变更。
- v0.26.20 尝试的 `wix.language: zh-CN` 无效（问题在标识符生成而非代码页）。

### v0.26.20 — 修复 v0.26.19 CI 格式检查失败

- `ParallelWorldOutlineCharacterStep` doc 注释超 `max_width=100`，`cargo +nightly fmt` 自动换行。仅注释格式变更。
- macOS 公证随 Apple Developer 协议续签恢复成功。

### v0.26.19 — Genesis 创世流程全面审计与测试加固

对照项目文档对「智能创作流程-创世」全面审计，分 Phase 1–4 执行：

- **Phase 1（P0 竞态与契约）**：
  - **Gap B**：`isFirstChapterReady` 路径在 `finalContent` 为空时不锁 `delivered`，避免编辑器永久空白。
  - **P0-2 角色世界观上下文**：`ParallelWorldOutlineCharacterStep` 中 character 提示词读取 `bundle.world_building` 恒为空（闭包捕获竞态），改为先 await world 拿真实 `world_concept` 再构造 character；提取 `world_concept_for_character_prompt` 纯函数 + 单测。
  - **P0-3 ChapterSwitch delivered 时序**：`selectChapter` 懒加载失败时不标记 `delivered`（`markDeliveredOnLoad` 仅在 `setContent` 成功后标记）。
- **Phase 2（P1 架构对齐）**：后台错误可观测性（`GenesisContext.errors` → `genesis_runs.steps_json` + `genesis-warnings` 事件 → 前端 toast）；mutex 中毒锁加固；策略移入 quick phase 经评估暂缓（记录为债务）；`window/mod.rs` 与 `FrontstageEvent.ts` 注释对齐 auto-accept 真实路径。
- **Phase 3（测试加固）**：8% 重试闸门 + ChapterSwitch payload 提取纯函数 + 契约测试；前端 Gap C + 状态机端点测试；**跨层共享 trim golden fixture**（`tests/fixtures/trim_golden.json`，Rust + TS 双跑锁定 `trim_self_repetition` 跨层一致性）。
- **Phase 4（代码整洁）**：`*_future` → `*_gen` 重命名；`AppConfig::load` 去重；`appendAiContent` skip 路径不 `markAccepted`；Gap C 重复入站也跳过 setContent；`isGenesisSettingUpRef` 合并评估——不合并（覆盖窗口不同）。
- **验证**：`cargo test --lib` 655 passed（+10）；`npx vitest run` 183 passed（+17）；`npx tsc --noEmit` 零错误；fmt 通过。

### v0.26.18 — Genesis 第一章重复：竞态路径加固

- **Gap A**：ChapterSwitch `auto_accept=true` 但 content 为空时 `skipContent=true` 且不标记 `delivered`，让 smart_execute 投递。
- **Gap B**：`isFirstChapterReady` 路径仅在已 append 或编辑器已有内容时标记 `delivered`。
- **Gap C**：`selectChapter` 咽喉点新增 `delivered` + 编辑器已有内容守卫，跳过 `setContent`。
- **回归测试**：新增 Gap A 竞态路径单测，vitest 167 passed。

### v0.26.17 — Issue #4 启动加固：打包 SQL 迁移与 init_db 诊断增强

- **打包 SQL 迁移**：Release 安装包包含 `$RESOURCE/db/migrations/`。
- **init_db 加固**：启动前确保 app data 目录；失败日志含 DB 路径；新增 fresh init 回归测试。

### v0.26.16 — 根治 Genesis 第一章重复、Issue #4 启动稳定性与代码格式修复

- **根治 Genesis 第一章内容重复**：替代 v0.26.7–v0.26.14 的散布布尔守卫补丁模式，从两个独立根因进行结构性修复。
  - **R2 生成侧验证闸门（`src-tauri/src/narrative/genesis.rs`）**：检测 LLM 输出自重复比例，≥8% 时用更强 anti-repeat 指令重试一次；prompt 模板新增「结构纪律」段，明确禁止首尾回环与整章重复。
  - **R1 前端单写者状态机（`src-frontend/src/frontstage/FrontstageApp.tsx`）**：将 `genesisAutoAcceptedRef` 布尔替换为 `idle → generating → delivered` 三态状态机；`generating` 态阻塞 `onChapterUpdated` 与 `loadStories` 自动选择；`delivered` 态阻塞 `setGeneratedText` 幽灵文本恢复。
  - `textCleanup` 提升到 `src-frontend/src/utils` 共享；Rust `trim_self_repetition` 对齐前端 KMP 最长 border 检测；全路径统一调用 `trimSelfRepetition`。
- **修复 Issue #4：init_db 失败时启动 panic/Windows 闪退**：`GatewayExecutor::new` 改为显式接收 `pool`，`setup` 仅在 pool 可用时初始化网关执行器，避免 `state::<DbPool>()` 在启动时 panic；新增不可写目录回归测试。
- **修复 CI 格式检查失败**：`cargo +nightly fmt -- --check` 与 `npm run format:check` 现已通过。

### v0.26.14 — 修复 Genesis 第一章模型输出自重复与幕前诊断日志过载

- **修复 v0.26.13 仍被用户感知的「新写小说第一章内容重复」**：通过分析 `creative_workflow.log` 中 13:43 的完整链路，确认前端 `append_ai_done` 只触发一次、`append_text_check.occurrences=1`，**不是前端把内容追加了两次**；重复来自 LLM 生成的 613 字正文自身首尾段落重复。
- 新增 `trimSelfRepetition` 工具（`src/frontstage/utils/trimSelfRepetition.ts`）：
  - 段落级：检测「后半段 == 前半段」或「末段 == 首段」并裁剪。
  - 字符级：对归一化文本做 KMP 最长 border 检测，保守阈值（重复长度 ≥30 字符且 ≥ 全文 8%）下裁掉尾部重复前缀。
- 在 `FrontstageApp` 的 `appendAiContent` 以及 `smart_execute` 返回的 `finalContent` 进入编辑器/幽灵文本前调用自重复清理，覆盖 Genesis 自动接受、Tab 接受、ContentUpdate/AppendContent 等全部路径。
- **缓解「写完后过会儿页面崩溃」**：`RichTextEditor` 的 `frontstage:rich_editor_diag` 渲染诊断日志从每帧都记改为仅前 20 次渲染 + 幽灵文本/隐藏锁状态变化时记录，并将 IPC 日志节流从 50ms 收紧到 200ms，降低长时间写作或文思活跃模式下的 IPC 与日志压力。
- 新增 `trimSelfRepetition` 单元测试，覆盖首尾段落重复、整章重复、单段内 suffix 重复、短文本不裁剪等场景。

### v0.26.13 — 修复 Genesis 第一章渲染层视觉重复（幽灵容器残留）

- 修复 v0.26.12 仍偶发的「新写小说第一章内容重复」视觉问题：数据层只写一次，重复来自渲染层幽灵文本/空幽灵容器与正文同框。
- `RichTextEditor` 的 `shouldShowGhostTree` 改为仅在 `generatedText` 非空时渲染，避免生成中状态的空幽灵容器残留或复用旧内容。
- `FrontstageApp` Genesis 自动接受路径先 `setIsGenerating(false)`，确保幽灵树条件立即失效。
- 增强 `frontstage:rich_editor_diag` 诊断字段：`isGenerating`、`isHidingGhost`、`bodyHidingGhost`、`generatedTextLen`。
- 增强 Playwright E2E 回归测试，新增自动接受后 `ghost-paragraph` 必须隐藏的断言。

### v0.26.12 — 修复角色列表为空/未加载时的幕前崩溃与订阅状态空值

- 修复 `useCharacters` 返回 `null` 或未加载完成时，`RichTextEditor`「角色名点击」effect 访问 `characters.length` 导致白屏崩溃的问题。
- `useSubscription` 增加空值防护，避免 `getSubscriptionStatus()` 返回 `null` 时产生错误日志。
- 新增 Playwright E2E 回归测试 `e2e/genesis-duplicate.spec.ts`，覆盖「已有故事 + 新写末世小说」完整流程。
- `frontstage/main.tsx` 与 `ErrorBoundary` 增强崩溃诊断输出。

### v0.26.11 — 修复 Genesis 第一章 store-editor 失步与崩溃隐患

- 修复 Genesis 自动接受第一章后，store 依赖 200ms onChange debounce 回写导致的 store-editor 失步。
- `appendAiContent` 追加后立即用 `editorRef.getHTML()` 同步 store 与 `latestContentRef`。
- `RichTextEditor.appendText` 空文档分支标记外部同步并更新 `lastExternalContentRef`，防止 content prop 被再次 setContent。
- `RichTextEditorRef` 新增 `getHTML()` 方法。
- 确认 `tauri.conf.json` `devUrl` 指向 dev server，避免开发时加载陈旧 dist 崩溃。

### v0.26.10 — 强化 Genesis 第一章重复防护（双重基准与追加最终防线）

- 修复 v0.26.9 单一 `latestContentRef` 基准与编辑器 DOM 短暂失步时，重复检测仍可能失效的问题。
- `isTextAlreadyInEditor`、`appendAiContent` 采用 `latestContentRef` + `editorRef.getText()` 双重基准。
- `appendAiContent` 增加正文前缀剥离安全网，并在追加后用 DOM 文本校准 `latestContentRef`。
- `RichTextEditor.appendText` 增加最终防线：编辑器尾部已包含待追加内容则直接跳过。

### v0.26.9 — 根治 Genesis 第一章重复（DOM 竞态与追加去重）

- 修复 TipTap DOM 状态滞后于 React `content` prop 时，重复检测依赖 `editorRef.getText()` 导致失效的问题。
- `isTextAlreadyInEditor`、`handleRequestGeneration`、`handleSmartGeneration`、`appendAiContent` 统一改用 `latestContentRef` 作为内容基准。
- `appendAiContent` 追加后立即同步 `latestContentRef`，避免 onChange debounce 窗口期内重复追加。
- `RichTextEditor` 幽灵文本直接包含检测剥离 HTML 标签，覆盖 ContentUpdate/AppendContent 路径。
- 新增 DOM 滞后竞态单元测试。

### v0.26.8 — 彻底修复 Genesis 第一章重复（竞态路径覆盖）

- 修复 `genesisAutoAcceptedRef` 无法覆盖 pipeline-complete 先加载 DB 正文竞态的问题。
- 新增 `isTextDuplicate` 归一化去重工具与 `isTextAlreadyInEditor` helper。
- `handleRequestGeneration` / `handleSmartGeneration` 设置幽灵文本前检测编辑器是否已包含生成内容。
- `pipeline-complete` 加载正文后标记 Genesis 已自动接受。

---

_最后更新: 2026-08-25 - v0.54.0_

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **StoryMoss** (22262 symbols, 46734 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> Index stale? Run `node .gitnexus/run.cjs analyze` from the project root — it auto-selects an available runner. No `.gitnexus/run.cjs` yet? `npx gitnexus analyze` (npm 11 crash → `npm i -g gitnexus`; #1939).

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows. For regression review, compare against the default branch: `detect_changes({scope: "compare", base_ref: "master"})`.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `query({search_query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `context({name: "symbolName"})`.
- For security review, `explain({target: "fileOrSymbol"})` lists taint findings (source→sink flows; needs `analyze --pdg`).

## Never Do

- NEVER edit a function, class, or method without first running `impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit changes without running `detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/StoryMoss/context` | Codebase overview, check index freshness |
| `gitnexus://repo/StoryMoss/clusters` | All functional areas |
| `gitnexus://repo/StoryMoss/processes` | All execution flows |
| `gitnexus://repo/StoryMoss/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
