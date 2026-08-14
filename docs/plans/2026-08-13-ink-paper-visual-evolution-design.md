# 墨纸 / 机械：安静贵气定向进化

日期：2026-08-13
状态：已发版 v0.43.0
决策来源：用户要求「Polished, calm, expensive UI with softer contrast, whitespace, premium fonts, spring motion」；taste-skill（`high-end-visual-design` + `redesign-existing-projects`）对照现码审核；用户随即以幕前输入框与发射键为第一刀。

---

## 1. 问题与裁定

### 1.1 症状

幕前已接近「宣纸上写字」，但局部控件仍是聊天产品：双层边框、炭黑方块发射键、生成中红色脉冲取消。P1 把 `AiPromptBar` 嵌进墨纸底栏后，v0.30.28 已拍扁的发射键被 ChatGPT 式实心墨块盖回去。字体声明了霞鹜文楷却未 `@font-face`，纸感在多数机器上塌成苹方/雅黑。幕后底近纯黑、阴影用纯黑、velvet 是典型 AI 紫，和「安静贵气」打架。`--transition-spring` 已定义从未接到交互。

### 1.2 裁定（已确认）

| 项 | 决定 |
|---|---|
| 品牌 | **保留**双界面：幕前墨纸、幕后机械。不换成玻璃黑 SaaS，不推倒另起语言。 |
| 方向 | 在现有品牌上做**定向进化**：对比收软、字先于框、动效有质量、无装饰性脉冲。 |
| 第一刀 | 幕前输入条 + 发射/取消。该落点是后续所有控件的样板，不是一次性美化。 |
| 信息架构 | 双窗、侧栏分组、路由、logo、IPC **不动**。 |
| 依赖 | 不迁 Lucide、不引入 GSAP / 新图标包 / 新字体 CDN 作为运行时硬依赖（字体可本地打包）。 |

异议已记录一次：`high-end-visual-design` 的岛式顶栏、`py-24`、Bento、滚动入场模糊会毁掉写作桌面应用。原则留下（软对比、弹簧、贵字体、双包边），版式按产品改：幕前是稿纸，幕后是仪表。

### 1.3 与既有设计文档的关系

| 文档 | 本设计如何对待 |
|---|---|
| `docs/plans/2026-07-27-ui-redesign-design.md` | **基座仍有效**（墨纸/机械 token、无阴影稿纸、机械金强调）。下列条目**作废**：§4.1 Ghost Chrome（v0.30.28 已删，禁止复活）；§6 幕后纯黑 `rgba(0,0,0,0.4/0.5/0.6)` 阴影（改为同色相扩散影）；§4.2 按钮 `active:scale-95` 过冲（改为 §6 的 press 契约）。 |
| `docs/plans/2026-08-12-beautifului-ai-native-design.md` | `--ai-*` 17 令牌契约与受控组件库 **不动**。幕前消费 AI 组件时必须走墨纸外壳，禁止再套一层聊天 chrome。 |
| 本文件 | 视觉进化的**现行契约**。与上表冲突时以本文件为准。 |

---

## 2. 目标与非目标

**目标**

- 幕前：打开就像在一张呼吸更慢的宣纸上写；Chrome 轻于正文；输入/发射与顶栏、光标、陶土强调同一家族。
- 幕后：一块被哑光金属托住的仪器；金是唯一强调；面板有内外半径级差；阴影不脏。
- 字体真正加载，不再靠系统回退假装纸感。
- 动效只出现在「有质量的一次」：press、展开、焦点；循环 pulse 退出主路径。

**非目标**

- 不重做落地页（`landing/` 独立站点，不在本设计范围）。
- 不改生成链路、Agency 路由、PersistMode、IPC。
- 不换图标库、不引入 GSAP / Framer 驱动主界面。
- 不恢复「鼠标静止 3 秒淡出 Chrome」。
- 不把幕前做成营销站，不把幕后做成浅色 paper 后台。

---

## 3. Visual Thesis

**一张呼吸更慢的宣纸，和一块被哑光金属托住的仪器；对比收软，字先于框，动效有质量，没有装饰性脉冲。**

手感目标（相对现状）：

| 面 | 现状约 | 目标 |
|---|---|---|
| 幕前 | variance 3 / motion 2 / density 3 | **4 / 3 / 2** |
| 幕后 | variance 4 / motion 3 / density 8 | **5 / 4 / 6** |

---

## 4. 不变量（禁止破坏）

1. 两个 Tauri 窗口、两个 HTML 入口；幕前不承担 CRUD，幕后不承担正文编辑。
2. `--ai-*` 17 个语义变量：幕后 `tokens.css` 与幕前 `frontstage.css` **各自定义、同名不同值**。AI 组件只引用 `--ai-*`，不写死 hex。
3. 色调 id `warm / cool / amber / indigo` 与 `localStorage` key `storymoss-color-theme` 双向同步。
4. 图标继续 `lucide-react`。新控件 `strokeWidth` 默认 **1.75**（装饰性可到 1.5）；禁止第二套图标包。
5. `prefers-reduced-motion` 已冻结 `--transition-*` 与 AI keyframes，**必须保留**。新弹簧同样走该媒体查询。
6. 不引入运行时字体 CDN 作为唯一来源（离线桌面应用）。字体文件进 `src-frontend` 或 Tauri resources，用 `@font-face`。

---

## 5. 明确拒绝（营销技能不得照搬）

| 拒绝 | 原因 |
|---|---|
| 岛式顶栏、汉堡变 X、全屏玻璃菜单 | 幕前顶栏是写作状态条，不是落地页导航 |
| 区块 `py-24`–`py-40` | 桌面应用视口被工具栏吃掉；留白用在正文柱与控件内边距 |
| Bento / 不对称 masonry | 幕后是可扫描的工作室，不是作品集 |
| 禁 Lucide、改 Phosphor | 全应用已用 Lucide；迁库成本大于观感收益 |
| GSAP / 滚动入场 blur | 写作时滚动是高频操作；blur 会拖 WebView |
| 填充炭黑方块 CTA（ChatGPT 发射键） | 与墨纸陶土家族冲突；已造成一次回归 |
| 嵌套双框（外纸面 + 内 AI 卡片边框） | 同一控件出现两套材质 |
| 循环 `animate-pulse` 作为空闲装饰 | 安静界面里显得廉价。健康态用静色；仅「正在探测 / 正在生成且无更好指示」可用一次短促动效 |
| 复活 Ghost Chrome | 用户已否；Chrome 常驻、改轻，不靠消失减噪 |

---

## 6. 运动与交互契约

两条曲线，不许混用：

| 名 | 值 | 用途 |
|---|---|---|
| **press** | `300ms cubic-bezier(0.32, 0.72, 0, 1)` | 按钮、焦点边、颜色过渡。无过冲。 |
| **spring** | 已有 `--transition-spring`：`300ms cubic-bezier(0.34, 1.56, 0.64, 1)` | 面板展开、开关滑块、菜单从底边长出。有轻微过冲。 |

Press 位移：`enabled:active:scale-[0.98]`。禁止 `scale-95` / `scale-[0.94]`（聊天产品手感）。

只允许这些动效出现在主路径：

- 光标与选区
- 幽灵续写（词级出现，已有 `AiStreamingText`）
- 按钮 press
- 菜单/面板展开（spring）
- 焦点边框色过渡（press 曲线）

禁止：空闲控件上的无限 pulse、无限 ping、装饰性扫光循环。`ai-sweep` 仅允许模型切换那一次。

---

## 7. 字体

### 7.1 缺口（已核对）

幕前 `--font-serif` 第一位是 `'LXGW WenKai'`，应用内**没有** `@font-face`。落地页才从 jsDelivr 拉。未装字体的机器落到苹方 / 微软雅黑，纸感不成立。

### 7.2 契约

| 角色 | 字体 | 加载 |
|---|---|---|
| 幕前正文、章节名、输入条文本 | 霞鹜文楷 Regular + Medium | 本地 `@font-face`，`font-display: swap` |
| 幕前 UI 标签（顶栏、提示、按钮旁注） | 现有 `--font-sans`（系统栈可保留） | 不强制新无衬线包 |
| 幕后界面 | 系统栈；数字用 `tabular-nums` | 不在本设计引入 Cinzel 作正文 |
| 字数、耗时、TTFB | `tabular-nums` | 全产品数字列 |

字重层次：正文 Regular；UI 强调 Medium / Semibold；禁止处处 Bold。

字体文件许可与体积在实施计划里单列（霞鹜文楷 SIL / OFL；只打 Regular+Medium 两档，避免把整包打进安装包）。

---

## 8. 颜色与材质

### 8.1 幕前（进化，不换种）

现有 OKLCH 纸/墨/陶土保留为真源。允许的微调（实施时用变量，禁止散落 hex）：

- 墨色略抬：`--ink` 从约 `oklch(25% …)` 提到约 `32%`，降低「印刷黑压在暖纸上」的硬对比。
- 陶土继续只做 10% 强调（光标、焦点发丝、**有内容时的发射键淡彩**）。禁止陶土实心大按钮。
- 纸面仍**无投影**。层次只靠 `--paper-50/100/200/300`。
- `--shadow-float` 仅用于浮层（命令菜单、划词条），且应为暖色相低透明度，不是冷黑。

### 8.2 幕后

- `--cinema-950` 从 `#050508` 改为带色偏的木炭（随四套色调走，禁止纯 OLED 黑）。
- 阴影从 `rgba(0,0,0,0.4)` 改为 **同色相扩散影**（暖金主题偏暖黑，冷青主题偏蓝黑）。`--shadow-panel` / `--shadow-float` 是单一改点。
- `--cinema-gold` 仍是**唯一强调**。`--cinema-velvet` 降饱和后**只允许**出现在 Agency 轨迹等非主路径点缀，禁止铺面板、禁止当第二 CTA。
- 状态色从 Tailwind 默认 500（`#22c55e` / `#ef4444` / `#facc15`）收到不超过 `cinema-gold` 的饱和度；`--ai-green/red/orange` 跟着变，不另开一套状态语义。

### 8.3 半径级差

| 层 | 半径 |
|---|---|
| 幕前纸面控件（输入条外壳） | `--radius-sm`（`rounded-paper`，2px）——稿纸，不是胶囊 |
| 幕前内按钮（发射/取消） | `rounded-md`（6–8px），**不是** `rounded-full` 实心圆 |
| 幕后外壳 | `--radius-md`（8px） |
| 幕后内芯 | `calc(var(--radius-md) - 4px)`，嵌在 `p-1` 外槽里 |

幕后面板允许 double-bezel：外浅槽 + 发丝 ring + 内芯 inset highlight。幕前**禁止** bezel（稿纸无金属槽）。

---

## 9. 控件样板：幕前输入条（P0）

这是本设计的**规范实例**。后续幕前按钮、取消、次级动作必须抄这套，不许再发明一套聊天样式。

### 9.1 已落地的契约（工作区，对照代码）

| 规则 | 实现 |
|---|---|
| 一层纸面 | 外层 `bg-paper-50 border-paper-300 rounded-paper`；`AiPromptBar variant="flush"` 去掉内层 `border-ai-line bg-ai-surface` |
| 焦点 | 仅外壳 `focus-within:border-terracotta/50`，无 ring 光晕 |
| 发射（空） | 透明底，`--ai-ink-3` 箭头，`disabled:opacity-40` |
| 发射（有内容） | `color-mix(in oklch, var(--ai-accent) 18%, transparent)` 底 + `--ai-accent-ink` 图标 |
| 取消生成 | 同尺寸 `size-7 rounded-md`；墨色，**无** `status-danger`、**无** `animate-pulse` |
| Press | `duration-300` + §6 press 曲线 + `active:scale-[0.98]` |
| 图标 | Lucide `strokeWidth={1.75}` |
| 幽灵提示 | `.frontstage-input-ghost` 对齐 flush 后的 textarea：`top: 5px; left: 4px` |
| 行为不变 | Enter 发送、IME `isComposing`、`/` 命令、logline 后缀、`scene_id` / smart_execute |

`AiPromptBar` 默认 `variant="card"` 留给可能的幕后/浮层；**幕前底部栏必须 `flush`**。禁止再把 card 嵌进纸面。

### 9.2 禁止回归

- 炭黑填充发射键：`background: var(--ai-ink)` + `color: var(--ai-surface)`。
- 外框 + 内框双 chrome。
- 取消键红色脉冲方块。
- 为对齐幽灵提示而把内层 padding 改回去却不改 CSS 偏移。

### 9.3 验收（用户可感知）

- 输入条看起来是稿纸上的一条栏，不是聊天输入气泡。
- 空输入时发射键几乎隐进纸面；有字时陶土淡彩，不是黑块。
- 生成中取消键与发射键同一脚印，不叫。
- 幽灵提示 / logline 后缀与输入文字基线对齐（不偏上、不压进发射键）。

---

## 10. 幕前其余落点（P0 之后）

按「是否还在用聊天材质」排序，不按页面清单扫一遍。

| 落点 | 现状问题 | 目标 |
|---|---|---|
| 底栏整体 | 底栏 `bg-paper-100/90` + 顶边可保留；信号条 `animate-pulse`（降级/未知） | 健康态静色；探测中才允许短动效；竖条与输入用发丝分隔（P0 已加） |
| 生成状态条 | `shadow-sm`、心跳 `animate-ping` | 去硬阴影；本地生成用陶土静色或一次 spring，不用 ping 环 |
| 顶栏按钮 | 需对照 Header 是否还有实心块 | 与发射键同一 press / 淡彩规则 |
| 划词浮条 `AiSelectionActions` | `--shadow-float` 冷黑 | 暖色相浮层影；动作钮抄发射键淡彩，不抄炭黑 |
| 命令菜单 | `rounded-[10px] border-ai-line shadow-float` | 可保留一层浮层；圆角与纸面级差 8px 外壳 / 6px 行 |
| 编辑器正文柱 | 宽约 900px、行高 1.8 | 保持；字体真正加载后重核字号，不靠加粗制造「贵」 |

幕前 Chrome 策略：常驻、减重、减对比。不靠消失。

---

## 11. 幕后落点（后于幕前主路径）

| 落点 | 目标 |
|---|---|
| 色板 | §8.2：木炭底、同色相影、velvet 降权、状态色收浊 |
| 面板 | double-bezel；内外半径级差 |
| 侧栏 | 分组间距加大；热/温/冷徽章改为低对比文字，不用四色胶囊抢金强调 |
| 主按钮 | 深底 + 金边或金淡彩，禁止高饱和填充；press 曲线同 §6 |
| 输入 | 聚焦金发丝 + 低透明度外光，已有方向保留，去掉刺眼 2px 纯金 glow |

幕后密度目标是 8→6，不是做成幕前那样的疏。工作室要可扫描。

---

## 12. 阶段

每阶段独立可审、可回退；前一阶段未对照 §13 验收不得开下一阶段。

| 阶段 | 内容 | 状态 |
|---|---|---|
| **P0** | 幕前输入条一层纸面 + 陶土淡彩发射 + 安静取消 + 幽灵偏移 | 已实施（implementation plan Task 1） |
| **P1** | 霞鹜文楷 Regular+Medium 本地 `@font-face`；数字 `tabular-nums` | 已实施（Task 2–3）。字体为 Regular woff2 映射 `font-weight: 400 500`（npm pack 仅为子集，改用官方 TTF→woff2） |
| **P2** | 幕前墨色抬升；幕后 cinema-950 / 阴影 / velvet / 状态色 | 已实施（Task 4–5） |
| **P3** | 把 press/spring 接到按钮与面板；主路径删除装饰 pulse/ping | 已实施（Task 6–7） |
| **P4** | 幕后 bezel + 侧栏减噪；幕前剩余实心块抄 P0 | 已实施（Task 8–9；划词 idle 发送钮已补 P0 淡彩） |
| **P5** | 空态 / 加载骨架统一（最后） | 已实施（Task 10） |

杠杆顺序来自 redesign 技能：字体 > 色板阴影 > 动效 > 间距 bezel > 空态。P0 提前插入是因为用户点名的控件已经在破坏墨纸，且它同时锁定「以后不许再嵌聊天条」。

---

## 13. 验证

### 13.1 契约测试（代码）

- `AiPromptBar`：`flush` 无内层 `border-ai-line`；Enter / IME / 发送禁用；`trailingAction` 替换发射键。
- `FrontstageBottomBar`：`data-variant="flush"`；取消无 `animate-pulse` / `status-danger`；placeholder、发送、取消、幽灵/logline 既有测试保持。
- 字体落地后：存在 `@font-face` 声明霞鹜文楷的探针（读 CSS 文件即可，不必真渲染字形）。
- 令牌：既有 `aiTokens.test.ts` 17 变量两窗口仍全绿；P2 若改 `--ai-green/red/orange` 映射，测试只锁「变量存在」，不锁旧饱和 hex。

### 13.2 用户可感知（设计验收，缺一不得称完成）

- 幕前底部：一层纸、陶土淡发射、取消不叫、幽灵对齐。
- 有内容 / 空内容 / 生成中 三态切换无跳布局（发射与取消同脚印）。
- `prefers-reduced-motion` 下发射键无 scale、无扫光。
- P1 之后：未装系统霞鹜的机器上正文仍是楷体纸感（打包字体生效）。

### 13.3 不准用的「完成」定义

- 只改了 className、未对照三态看过。
- 字体 stack 写了名字但安装包里没有文件。
- 为对齐某一像素把双框加回去。

准入线与仓库一致：`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check`；根目录 `python3 scripts/architecture_guard.py`。本设计无 Rust 变更则不跑 `cargo test` 作为视觉验收。

---

## 14. 文件地图

| 文件 | 角色 |
|---|---|
| `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx` | 幕前纸面外壳、flush、取消键 |
| `src-frontend/src/components/ui/ai/AiPromptBar.tsx` | `variant` + 发射键材质 |
| `src-frontend/src/frontstage/styles/frontstage.css` | 幕前 token、幽灵偏移、日后 `@font-face` |
| `src-frontend/src/styles/tokens.css` | 幕后 token、spring、阴影 |
| `src-frontend/src/styles/backstageThemes.ts` | 四套色调；P2 改 950/阴影/velvet |
| `src-frontend/tailwind.config.js` | `paper`/`ai`/`ease-spring`；不在此引入营销字体 |

---

## 15. Prompt Guide（给后续实现者）

描述任一控件时必须回答：

1. **模式**：墨纸还是机械？
2. **层数**：几个边框？幕前答案必须是 0 或 1。
3. **强调色**：陶土淡彩还是金？禁止第三色当主 CTA。
4. **曲线**：press 还是 spring？
5. **静止时动不动**：空闲必须静。

反例（禁止再写）：

> 输入条用 AiPromptBar 默认卡片，发射键 `--ai-ink` 实心，生成中红色 pulse。

正例：

> 墨纸。一层 `--paper-50` 外壳，内 `AiPromptBar flush`。有内容时发射键陶土 18% tint + `--ai-accent-ink` 箭头，press 曲线，`scale-[0.98]`。取消同脚印、无 pulse。
