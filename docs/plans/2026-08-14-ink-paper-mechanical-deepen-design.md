# 墨纸 / 机械：定向进化补齐

日期：2026-08-14
状态：已发版 v0.44.0（全界面截图回归未做）
决策来源：taste-skill 全界面审核后的四节设计（品牌加深墨纸/机械、输入无框、字体与幕前 Chrome、幕后机械、纸色/脉冲/发版），用户逐节批准。

承接：`docs/plans/2026-08-13-ink-paper-visual-evolution-design.md`（已发版 v0.43.0）。本文件覆盖 **v0.43.0 未做满的缺口**，不另起品牌。与 08-13 冲突时，**未落地条目以本文件为准**；已落地且本文件未改写的契约（双窗、`--ai-*` 17 名、拒绝营销技能照搬、P0 发射/取消淡彩）继续有效。

---

## 1. 问题与裁定

### 1.1 症状

v0.43.0 修好了「字体没加载、发射键是炭黑块、取消键红色脉冲」。打开幕前，底部仍是一张浅卡纸贴在稿纸上：四边框、`rounded-paper`、独立 `bg-paper-50`，底栏还有一条 `border-t`。强调仍靠浏览器把 Regular 拉成 500。幕后暖金外壳是木炭，内芯仍是冷紫海军蓝；Panel 有外槽没有发丝与高光；侧栏选中描金框；弹簧 300ms，展开像点一下。选区声明了 `opacity`，WebKit 几乎不吃，划词仍是整块实心赭。

### 1.2 已确认裁定

| 项 | 决定 |
|---|---|
| 品牌 | **A. 加深墨纸 / 机械**。不统一双窗，不做玻璃黑 SaaS，不视觉重开。 |
| 输入 Chrome | **A. 无框**。字写在底栏纸面上。不要四边框、不要 `rounded-paper` 卡片、不要独立 `bg-paper-50`。 |
| 交付 | **A. 一份进化规格**。输入条是 P0 / 第一刀可独立上线，其余按 P1 / P2 切开，不等「全部做完再发」。 |
| 幕前 UI 标签字体 | 系统无衬线。**不**新打思源黑体。 |
| 正文强调字重 | 补真正的 Medium woff2，禁止 `font-weight: 400 500` 映射同一 Regular。 |
| 幕后密度 | 可扫描的工作室（约 6），不是幕前那种疏。 |

### 1.3 与既有文档

| 文档 | 本设计如何对待 |
|---|---|
| `2026-08-13-ink-paper-visual-evolution-design.md` | v0.43.0 已落地部分仍是现行样板（发射淡彩、取消不叫、本地 Regular、墨色 32%、同色相影、徽章低对比、Panel `p-1`、press 300ms）。本文件补齐其 §7.2 Medium、§8.1 纸暖、§8.2 内芯同色相、§8.3 发丝/高光、§6 spring 接到交互、§10 顶栏、选区。 |
| `2026-07-27-ui-redesign-design.md` | 基座仍有效。Ghost Chrome **禁止复活**。 |
| `2026-08-12-beautifului-ai-native-design.md` | `--ai-*` 17 令牌名与受控组件库不动。幕前消费 AI 组件必须 `flush` 或墨纸外壳，禁止再套聊天卡片。 |

---

## 2. 目标与非目标

**目标**

- 幕前打开是一张呼吸更慢的宣纸：输入无框、纸微暖、楷体有 Medium、选区是淡陶土、顶栏按钮与发射键同一家族。
- 幕后是一块被哑光金属托住的仪器：内外同色相、双包边、选中不描框、展开有 500ms 弹簧。
- 空闲界面静止。正在干活的指示可以动。

**非目标**

- 不改生成链路、Agency、PersistMode、IPC、侧栏信息架构、路由。
- 不重做 `landing/`。
- 不换 Lucide、不引入 GSAP / Framer、不引入运行时字体 CDN。
- 不把幕后做成浅色纸后台，不把幕前做成营销站。
- 不做全界面截图回归套件（仍记债务；本设计验收用契约测试 + 用户可感知清单）。
- 不宣称 v0.42.0 §8 续写真机探针已过。

---

## 3. 不变量（禁止破坏）

1. 两个 Tauri 窗口、两个 HTML 入口；幕前不承担 CRUD，幕后不承担正文编辑。
2. `--ai-*` 17 个语义变量：幕后 `tokens.css` 与幕前 `frontstage.css` 各自定义、**同名不同值**。只改值，不改名、不加第 18 个。
3. 色调 id `warm / cool / amber / indigo` 与 `localStorage` key `storymoss-color-theme` 双向同步。
4. 图标继续 `lucide-react`；新控件 `strokeWidth` 默认 1.75。
5. `prefers-reduced-motion` 冻结 `--transition-*` 与 AI keyframes，必须保留。新的 500ms spring 同样走该媒体查询 → `0.01s linear`。
6. 字体只本地 `@font-face`。许可 SIL OFL，文件进 `src-frontend/public/fonts/`。

---

## 4. 明确拒绝

沿用 08-13 §5，并加一条本轮容易再犯的：

| 拒绝 | 原因 |
|---|---|
| 去掉输入框后把 `border-t` 留在底栏顶 | 外框只是上移了一条线，不是无框 |
| 焦点 ring / `focus-within:border-*` | 无框契约：焦点给光标或发射键陶土，不给一圈线 |
| 幕前 bezel / 双包边 | 稿纸无金属槽 |
| 卡片悬停 `scale` 过冲 | 密集工作室会像玩具；悬停只变色，走 press |
| 空闲 `animate-pulse` / `animate-ping` | 安静界面里显得廉价 |
| 用 `::selection { opacity }` 当选区透明度 | 对 `::selection` 无效；必须写进 `background` 的 alpha |

---

## 5. 第一节 · 幕前输入无框（P0）

**规范实例。** 后续幕前按钮必须抄发射键，不许再发明聊天条。

### 5.1 现状（对照代码）

`FrontstageBottomBar`：栏 `bg-paper-100/90 backdrop-blur-sm border-t border-paper-300`；输入壳 `bg-paper-50 border border-paper-300 rounded-paper px-2.5 py-1.5 focus-within:border-terracotta/50`；内层已是 `AiPromptBar variant="flush"`。

### 5.2 契约

| 规则 | 落地 |
|---|---|
| 输入壳 | 去掉 `border`、`rounded-paper`、独立 `bg-paper-50`。textarea 直接坐在底栏纸面上。 |
| 底栏与正文 | 去掉 `border-t border-paper-300`，去掉 `backdrop-blur-sm`（那是玻璃，不是纸）。底栏用不透明的 `--parchment-dark`（或等价略深一档纸色），用色差贴住正文，不要描线。不要再用 `bg-paper-100/90`。 |
| 发射（空） | 近乎隐进纸面（`--ai-ink-3`，低 opacity）。 |
| 发射（有字） | 陶土淡彩底 + `--ai-accent-ink` 箭头。与 v0.43.0 相同，禁止退回炭黑块。 |
| 取消 | 同脚印 `size-7 rounded-md`，无 pulse、无 `status-danger`。 |
| 焦点 | **无** ring、**无** `focus-within` 边框。 |
| 幽灵 / logline | 按无框后的 padding 重核 `top` / `left`，基线与输入文字对齐。 |
| 行为 | Enter、IME `isComposing`、`/` 命令、logline 后缀、续写 `smart_execute` / `scene_id` **不变**。 |

`AiPromptBar` 默认 `variant="card"` 仍留给幕后/浮层。幕前底部栏必须 `flush`。

### 5.3 验收

- 输入区看起来是稿纸下缘，不是聊天气泡、也不是底栏里再嵌一张卡。
- 空 / 有字 / 生成中三态不跳布局。
- 既有 BottomBar 契约测试仍绿，并增加「外壳无 `border` / `rounded-paper` / 独立 `bg-paper-50`；栏无 `border-t`、无 `backdrop-blur-sm`」。

---

## 6. 第二节 · 字体与幕前其余 Chrome（P1）

### 6.1 字体

| 角色 | 字体 | 加载 |
|---|---|---|
| 正文、章节名、输入文字 | 霞鹜文楷 Regular + **Medium** | 本地 woff2。Regular `font-weight: 400`；Medium `font-weight: 500`。两档 **分文件**。 |
| 顶栏、字数、模型旁注 | 现有 `--font-sans` 系统栈 | 不打思源黑体、不引入 Geist / Inter / Cinzel 作 UI。 |
| 数字 | `tabular-nums` | 已有则保持。 |

Medium 来源与 Regular 同一官方仓库标签（`lxgw/LxgwWenKai`，与 `public/fonts/README.md` 已记的 v1.250 对齐）的 `LXGWWenKai-Medium.ttf`，fontTools 压 woff2。许可 OFL，更新 README：不再写「400 500 映射同一文件」。

体积：Medium 单档不得超过本仓库 Regular woff2 体积的 1.15 倍；超出则对 Medium 做与 Regular 相同的字形子集后再压，禁止打完整 TTF 进安装包。

### 6.2 顶栏与正文柱

- 顶栏图标按钮抄发射键的 **press 曲线、`scale-[0.98]`、陶土淡彩**。禁止实心块。不抄发射键的「空闲近隐」——设置/回幕后必须可发现，空闲保持 `--ai-ink-3` 实色图标，hover 才上淡彩。
- 正文柱宽约 900px、行高 1.8，不改。
- 不恢复鼠标静止淡出 Chrome。
- 幕前主路径动效只允许：光标、幽灵续写、按钮 press。展开类（若有）走 §7.4 spring。

### 6.3 选区

`::selection` 与 `.ProseMirror ::selection` 的 `background` 改为陶土 **22%** 透明，例如：

`color-mix(in oklch, var(--terracotta) 22%, transparent)`

删除无效的 `opacity`。文字色保持 `--ink`，保证对比可读。

---

## 7. 第三节 · 幕后机械（P2）

### 7.1 色：暖金内芯同色相

暖金主题 `--cinema-950` / `--cinema-900` 已是暖木炭。`--cinema-850` 到 `--cinema-500` 仍是冷紫海军蓝（`#0f0f16`、`#151520`、`#1e1e2e`、`#2a2a3c`、`#3a3a50`）。改为同一暖木炭家族、明度递增。目标值（warm only）：

| 变量 | 现 | 目标 |
|---|---|---|
| `--cinema-950` | `#0c0b09` | 不变 |
| `--cinema-900` | `#12110e` | 不变 |
| `--cinema-850` | `#0f0f16` | `#161310` |
| `--cinema-800` | `#151520` | `#1c1916` |
| `--cinema-700` | `#1e1e2e` | `#26211c` |
| `--cinema-600` | `#2a2a3c` | `#322c25` |
| `--cinema-500` | `#3a3a50` | `#423a31` |

写在 `backstageThemes.ts` 的 `warm.vars`，`tokens.css` 默认值跟 warm。cool / amber / indigo **不**跟这次拧（内部已自洽）。金仍是唯一强调；`--cinema-velvet` 只允许 Agency 轨迹等非主路径，禁止铺面板、禁止第二 CTA。

`BACKSTAGE_THEME_VARS` 名单不变。既有主题测试继续锁「每套主题给齐变量」，warm 断言改为上表目标 hex。

### 7.2 Bezel

`Panel` 已有外槽 `p-1`、内芯 `rounded-[calc(var(--radius-md)-4px)]`、`shadow-panel`。补：

- 发丝已由现有 `border-borderSubtle`（约 6% 白）承担，**不要再加第二圈 ring**。
- 内芯顶边高光：`box-shadow: inset 0 1px 0 color-mix(in oklch, white 6%, transparent)`。这是 P2 唯一要补的金属感。

幕前禁止 bezel。不要把这套阴影抄到 `FrontstageBottomBar`。

### 7.3 侧栏

分组、路由、热/温/冷文案 **不动**。选中项去掉 `border border-cinema-gold/20`，只留 `bg-cinema-gold/10 text-cinema-gold`。徽章保持低对比文字（已有 `impactBadgeClass`）。

### 7.4 弹簧

| 名 | 值 | 用途 |
|---|---|---|
| **press** | `300ms cubic-bezier(0.32, 0.72, 0, 1)` | 按钮、颜色过渡。无过冲。`enabled:active:scale-[0.98]`。 |
| **spring** | **`500ms`** `cubic-bezier(0.34, 1.56, 0.64, 1)` | 面板展开、开关滑块、页签指示。轻微过冲。 |

`--transition-spring` 从 `0.3s` 改为 `0.5s`，只改 `tokens.css`（幕前未声明该变量，不必为了对称在 `frontstage.css` 新增）。`prefers-reduced-motion` 仍冻结为 `0.01s linear`。

接到：`Panel` 展开（现 `duration-300 ease-spring` 改为吃 token 的 500ms）、开关、页签。卡片悬停 **只**走 press 变色，禁止 `scale` 弹跳。

改 `pressMotion.test.ts`（及任何锁 0.3s spring 的测试）时只锁 spring 时长为 `0.5s`，不要误改 press 的 `0.3s`。

---

## 8. 第四节 · 纸色、脉冲、发版

### 8.1 纸（P1，与字体同一刀）

仅 **暖赭默认** 纸：`--parchment` 从 `oklch(96.5% 0.008 95)` 改为 `oklch(96.5% 0.012 95)`。chroma +0.004，L 与 hue 不动。不到琥珀主题那档（`0.015` / hue 85）。

`colorThemes.ts` 的 `warm` 纸起点、以及 `tokens.css` 的 `--paper-50/100/200/300` 跟到同一暖纸家族（P1 仍改 hex，供残留 `paper-*` 工具类，避免两套纸色）。P0 底栏已经改用 `--parchment-dark`，P1 只改变量，底栏自动跟上。

冷青 / 琥珀 / 靛紫的纸 **不**跟暖赭一起改。

### 8.2 脉冲

空闲必须静。

**删：** `PromptsPanel` 空态 `FileText` 的 `animate-pulse`（装饰）。

**留（正在干活）：**

- `IntentionGraphDiagnostics` 加载骨架
- `ModelCard` 探测 step `running`
- `KnowledgeGraph` 归档 / 蒸馏 `isPending`
- `Stories` 创世进度 **当前步** 指示
- 幕前生成中的不确定进度条

其它新控件默认禁止无限 pulse / ping。健康态用静色。

### 8.3 发版切分

一份规格，三刀均可独立 tag（每刀对照本节验收后再开下一刀）：

| 刀 | 内容 | 用户可感知 |
|---|---|---|
| **P0** | §5 输入无框 | 底栏不再是卡片 |
| **P1** | §6 字体/顶栏/选区 + §8.1 纸 | 楷体有中重、纸微暖、划词不糊成实心赭 |
| **P2** | §7 幕后 + §8.2 脉冲 | 面板像金属槽、展开有弹簧、空态不跳 |

P0 不改纸 chroma，避免「无框」和「纸色」两个变量叠在同一刀里无法判断。P0 只负责拆框、对齐幽灵。

---

## 9. 文件地图

| 文件 | 角色 |
|---|---|
| `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx` | P0 无框；幽灵偏移 |
| `src-frontend/src/frontstage/styles/frontstage.css` | 选区 alpha、纸 chroma、`@font-face` Medium |
| `src-frontend/src/components/ui/ai/AiPromptBar.tsx` | 保持 flush；不要为对齐把 card 嵌回去 |
| `src-frontend/src/frontstage/components/FrontstageHeader.tsx` | 顶栏按钮抄发射键 |
| `src-frontend/public/fonts/` | 新增 Medium woff2 + README |
| `src-frontend/src/frontstage/config/colorThemes.ts` | warm 纸起点 |
| `src-frontend/src/styles/tokens.css` | paper hex 跟纸；`--transition-spring: 0.5s` |
| `src-frontend/src/styles/backstageThemes.ts` | warm 850–500 |
| `src-frontend/src/components/ui/Panel.tsx` | 内芯 inset 高光；展开 500ms |
| `src-frontend/src/components/Sidebar.tsx` | 选中去金框 |
| `src-frontend/src/pages/settings/PromptsPanel.tsx` | 删空态 pulse |
| `src-frontend/tailwind.config.js` | 不引入营销字体；spring 曲线已有，时长走 CSS 变量 |

改 `Button` / `Panel` 前必须跑 GitNexus `impact`（二者是枢纽，v0.43.0 已标 HIGH）。只改 className / 令牌值时仍要报爆炸半径，不得 silently 改 API。

---

## 10. 验证

### 10.1 契约测试

- BottomBar：无框 class 探针 + 取消无 pulse + flush + 既有 Enter/IME/logline。
- `@font-face` 同时声明 Regular 与 Medium，且 Regular 的 `font-weight` **不是** `400 500`。
- `public/fonts/lxgwwenkai-medium.woff2` 存在。
- `aiTokens.test.ts` 17 变量两窗口仍全绿。
- `backstageThemes` warm 850–500 等于 §7.1 表。
- Panel 内芯含 `inset 0 1px` 高光；外壳不新增第二圈 ring。Sidebar 选中 class 不含 `border-cinema-gold`。
- `--transition-spring` 为 `0.5s`；reduced-motion 冻结。
- PromptsPanel 空态图标无 `animate-pulse`。

### 10.2 用户可感知（缺一不得称该刀完成）

- P0：一层纸、无描边输入、发射淡彩、取消不叫、幽灵对齐。
- P1：未装系统霞鹜的机器上正文仍是楷体；字重 500 不是斜体伪粗；划词是淡陶土。
- P2：暖金幕后内芯不发紫；面板有浅槽+发丝；侧栏选中无金框；折叠面板展开能看出过冲而不是瞬切。

### 10.3 不准用的「完成」

- 只改了 className、未对照三态。
- Medium 只改了 CSS 映射、安装包里没有第二份文件。
- 为对齐某一像素把输入框加回去。
- 未跑 `cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check` 与根目录 `python3 scripts/architecture_guard.py`。本设计无 Rust 变更则不把 `cargo test` 当视觉验收。

---

## 11. Prompt Guide（给后续实现者）

描述任一控件必须回答：

1. **模式**：墨纸还是机械？
2. **层数**：几个边框？幕前答案必须是 0（输入）或最多 1（浮层）。幕后 Panel 允许外槽 + 发丝，不算第三套材质。
3. **强调色**：陶土淡彩还是金？禁止第三色当主 CTA。
4. **曲线**：press 还是 spring？
5. **静止时动不动**：空闲必须静。

反例：输入条 `bg-paper-50 border rounded-paper`，焦点 `border-terracotta`，发射键实心墨块。

正例：墨纸。字写在底栏 `--parchment-dark` 上，无边框。有字时发射键陶土 18% tint，press，`scale-[0.98]`。
