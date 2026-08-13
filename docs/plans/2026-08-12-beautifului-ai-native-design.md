# beautifului AI 原生组件替换 + 幕后可选样式 — 设计文档

日期：2026-08-12
状态：已获用户批准（对话确认）

## 1. 目标与范围

- **组件替换**：将 [beautifului.dev](https://www.beautifului.dev/)（copy-paste AI 原生组件库，浅色 paper 风格，源码内嵌于其页面 RSC payload，已验证可提取）全部 18 类组件中本应用能对应的 **15 类**，以其源码为蓝本适配进本项目（React + Tailwind 语法一致，设计令牌映射到本应用色系），逐点替换现有对应 UI。
- **幕后主题**：幕后（pages/ + components/，`com.storymoss.app` 后台窗口）从硬编码深色 cinema 系改为 **CSS 变量驱动 + 4 套深色调可选**。选项 id 与幕前色调主题一致（warm / cool / amber / indigo），各自映射一套深色配色；与幕前双向同步；入口并入幕后设置页外观区现有 ColorThemeSelector。
- **被否方案**：A「只换皮不换组件」达不到替换要求；B「按页面大爆炸重写」风险高、不可渐进验证。

## 2. 总体架构（两层）

```
┌─ 主题令牌层 ────────────────────────────────┐
│ tokens.css: cinema-* 色值全部改为 var() 引用 │
│ backstageThemes.ts: 4 套深色调定义 + apply   │
│ ai-native 组件只引用语义令牌，不写死颜色      │
└─────────────────────────────────────────────┘
┌─ AI 原生组件层 src/components/ai-native/ ───┐
│ beautifului 源码提取 → 适配 cn()/令牌/深色   │
│ → 15 个独立可组合组件                        │
└─────────────────────────────────────────────┘
        ▲ 各页面/幕前组件按映射表逐点替换
```

## 3. 组件映射与替换清单（15 类）

| beautifului 组件 | 新组件 | 替换落点 |
|---|---|---|
| Loading State | `AiLoading` | AnalysisProgress、DataLoader、各 Loader2 旋转点 |
| Thinking | `AiThinking` | 新增：agency 思考链展示（AgencyStudio / 幕前顶栏 expandable trace） |
| Streaming Text | `AiStreamingText` | 幕后 StreamOutput、幕前 StreamingText 统一 |
| Approval Card | `AiApprovalCard` | Tasks 级联改写审批、AgentInterruptionModal |
| Tool Chips | `AiToolChips` | AgencyStudio 活动流、TracingPanel 步骤 |
| Task Rows | `AiTaskRows` | Tasks 状态行、usePipelineProgress 面板 |
| Chat | ~~`AiChat`~~ | P3 勘察关闭：ChatComposer 为 AiPromptBar 严格子集、无多轮对话场景；差异特性「分节回复 + resolving 退焦」记录备选（见 §8 P3 注明） |
| Prompt Bar | `AiPromptBar` | FrontstageBottomBar（含 slash 命令机制新增） |
| Recommendation Card | `AiRecommendationCard` | AiSuggestionBubble / AiHintOverlay 建议卡 |
| Context Cards | `AiContextCards` | CharacterPeekCard / CharacterCardPopup、资产引用展示 |
| Diff Table + Records/Filter Table | `AiTable` 系列 | DiffViewer 表格化、Tasks / UsageStats / AgencyEval / AgencyLearning 原生 table |
| Search | `AiSearch` | VectorSearch |
| Insight Cards | `AiInsightCards` | Insights 页统计卡 |
| Code Block | `AiCodeBlock` | StreamOutput 内 code 渲染、PromptsPanel |
| Selection Actions | `AiSelectionActions` | RichTextEditor 划词浮条（tiptap BubbleMenu 依赖已装未用，正式启用） |

不纳入（YAGNI）：Fine-tune Card（属性检查器）、Sidebar Nav。

## 4. 幕后主题系统

- `tokens.css`：`--cinema-950…500 / gold / velvet` 等全部改为 `var(--bt-*)` 语义变量引用；`backstageThemes.ts` 定义 4 套深色调（每套给齐 cinema 阶梯 + 强调色 + status 色），`applyBackstageTheme(id)` 注入 `documentElement`。
- **同步**：复用幕前既有机制——同一 localStorage key `storymoss-color-theme` + `color-theme-changed` Tauri 事件 + `storage` 监听；幕前选 warm，幕后自动切到 warm 对应的深色调，反之亦然。
- **入口**：幕后设置页外观区现有 `ColorThemeSelector` 扩为「幕前色调 + 幕后色调」双预览（同一选项组，显示两侧效果），不单开入口。
- 顺带清理：`frontstage/hooks/useWritingStyle.ts` 死代码删除（真正生效路径是 `hooks/contracts/useEditorConfig.ts`）。

## 5. 现状关键事实（探索结论，实施依据）

- 幕前色调主题：`frontstage/config/colorThemes.ts`（4 组 OKLCH 派生 14 变量，`applyColorTheme` 注入，localStorage `storymoss-color-theme`，Tauri 事件 + storage 跨窗口同步）；设置 UI 在 `ColorThemeDot.tsx`（幕前顶栏）与 `GeneralSettings.tsx` 的 `ColorThemeSelector`（幕后设置页）。
- 写作风格：`frontstage/config/writingStyles.ts` + `hooks/contracts/useEditorConfig.ts`（localStorage `storymoss-editor-config`）→ `RichTextEditor.tsx:1287-1294` 注入 `--fs-*`。
- 幕后颜色：`src/styles/tokens.css:16-26` 定义 `--cinema-*` hex，`tailwind.config.js:24-44` 映射为 Tail 色；无 dark: 类、无切换机制；`index.css:11-13` body 硬编码深色。
- 两个窗口是独立入口：幕后 `main.tsx` 引 `index.css`→`tokens.css`；幕前 `frontstage/main.tsx` 只引 `frontstage.css`。`--terracotta/--parchment/--ink` 在两侧重复定义（hex vs OKLCH），各自窗口各用各的。
- 组件库现状：无 shadcn，`components/ui/` 仅 Button/Card/Panel/Toggle/StudioNavRail 五个自研件，`cn()` 在 `src/utils/cn.ts`；Tailwind v3.4.17 + @tailwindcss/typography。
- beautifului 源码提取：页面单 HTML（约 383KB）内嵌全部组件 React 源码（转义形态），提取后人工审查适配，不引入新依赖（图标继续用 lucide-react）。

## 6. 错误处理与回退

- 每个新组件替换采用「新组件 + 特性开关（localStorage flag）」先行，旧组件保留为 fallback；逐页面稳定后 P4 阶段删除旧件。
- 组件适配只引用语义令牌，主题缺失变量时回退默认深色调（warm 对应套）。

## 7. 测试策略

- 每个 ai-native 组件配 vitest + Testing Library 渲染测试（状态 / 交互）。
- 主题系统：`applyBackstageTheme` 单测（变量注入、持久化、事件）、4 套主题令牌完整性测试（每套必须定义全部必需变量）。
- 替换页面跑既有页面测试不回归；准入线：`npx tsc --noEmit`、`npx vitest run`、`npm run format:check`、`python3 scripts/architecture_guard.py` 全绿。

## 8. 阶段划分（每阶段独立提交 + 评审）

1. **P0 主题底座**：tokens 变量化 + backstageThemes + 设置页入口 + 双向同步
2. **P1 组件库第一批（生成体验）**：AiLoading / AiThinking / AiStreamingText / AiPromptBar / AiApprovalCard
3. **P2 组件库第二批（代理与任务）**：AiToolChips / AiTaskRows / AiRecommendationCard / AiContextCards / AiSelectionActions
4. **P3 组件库第三批（数据展示）**：AiTable 系列 / AiSearch / AiInsightCards / AiCodeBlock / AiChat（入库）（AiChat 经 P3 勘察关闭：ChatComposer 为 AiPromptBar 严格子集、无多轮对话场景，差异特性「分节回复 + resolving 退焦」记录备选，详见 P3 实施计划）
5. **P4 收尾**：旧组件删除、死代码清理、文档与发版（P4 已完成：替换残留与历史死件清理、`--ai-on-accent` 令牌与 /N 透明度修复、AgencyEval/AgencyStudio/AgencyLearning 浅色页令牌化；B2 死导出与 B3 历史 CSS 留作后续可选清理；发版推送不在 P4 范围，详见 P4 实施计划）
