# 续写导演锁：一人一号 / 近文亲缘 / 一拍一事

日期：2026-08-27
状态：已实现（**v0.56.0**；探针加宽 **v0.56.1**）
决策来源：真机续写《帝国的烟火》大婚行刺后，从「飞身扑上」往下写，人物拆成两人、亲缘写反。用户裁定 **路径 C 的实现取路径 1**：改续写流水线（导演编译 + 主创只写一件事），**不**把写正文的主创拉回 ToolLoop。

承接：

- `docs/plans/2026-08-13-agency-only-continuation-design.md`（Agency 唯一续写 + SceneBeatCard）
- `docs/plans/2026-08-15-continue-quality-closure-design.md`（拍级状态网 + 探针）
- `docs/plans/2026-08-25-grok-bot-control-plane-fusion-design.md`（续写 `complete()` 零工具；v0.55.0 短合同）
- v0.53.6 死人退场、禁止重演已写完的行刺（AGENTS.md / CHANGELOG；无独立设计文件）

本设计 **不改** PersistMode、`scenes.content` 真相源、三档路由、创世/管理/编辑 ToolLoop、幕前纸面。角色表脏行 **不**自动删除或合并落库。

---

## 1. 问题

### 1.1 用户可感知症状

真机增量（行刺已写完、末句「飞身扑上」之后）没有再演一遍刺杀，但：

| 症状 | 读者看到什么 |
|---|---|
| 同一人拆成两个 | 先写「曹元佩则彻底僵住了」，后又写「琬公主曹元佩则发出了一声…」「琬公主曹元佩则蜷缩在角落」——同一拍两个身体 |
| 亲缘写反 | 原文苏会山与曹元佩并坐（镇北王妃）；苏亦铁扑向「父亲」。续写把曹元佩写成对「侄子」的怜惜 |
| 点名式巡场 | 礼仪主持、景亲王、曹元佩、苏福贵、琬公主曹元佩各写一段反应，场面变成人物清单而不是下一拍 |

### 1.2 根因（代码核对）

身份错乱不是缺 `asset_read`，也不是主创没进 ToolLoop。

1. **别名合并不认称号+姓名。** `aliases_for` 只生成全名、`阿X`、末两字。`琬公主曹元佩` 与 `曹元佩` 不算同一人。角色表若同时有「琬公主」「曹元佩」两行，`match_character_names` 会两个都准入。
2. **节拍卡点名全场。** `present_in_text` 给每个在场者用途「末段已在场，承接行动」。状态网写「禁止忘掉已在场者」。
3. **探针强迫点名。** `probe_increment`：阵容 ≥2 则增量必须点名 ≥2 人；每个 `state.present` 未点名且未写「离开」就报 `丢掉已在场者：{name}`。重试等于命令模型点名全场。点完曹元佩还要再写一个反应，就造出「琬公主曹元佩」。
4. **亲缘没有从近文锁定。** 关系表与身份字段（`CoreCharacter.identity` 多从 `background` 来）可过时或幻觉；编译器不注入「曹元佩＝苏会山之妻，禁止写成苏亦铁的姑姑」。
5. **短合同只禁场外发明名，不禁把已准入的人拆成两个称呼。** `CONTINUE_BEAT_SYSTEM` 的 Right 还写「写在场活人的反应」，被理解成每人一段。

把主创拉进 ToolLoop 不能修 2–3：本地创作模型 JSON 熔断、空 CoT 重烧候选链、节拍卡泄进正文（v0.30.3 / v0.41.2 / v0.51.2）。v0.54–v0.55 已否决该路径。本版维持该否决。

### 1.3 已确认裁定

| 项 | 决定 |
|---|---|
| 路径 | **导演–主创两拍**。写正文仍一次 `complete()`，`tools = None` |
| 导演 | 管理档一次 JSON；失败 / 超时 / 测试环境 → 纯 Rust 锁，续写不停 |
| 角色表 | 写时合并身份；**不** UPDATE/DELETE `characters` 行 |
| 探针 | 去掉「每人必须点名」；加上拆人 / 亲缘颠倒 |
| 真机 | 须用同一「飞身扑上」开头再跑；过关前 **不得**宣称续写质量已修好 |

---

## 2. 目标与非目标

**目标：** 每一拍续写在发给主创之前，锁定「一人一号、近文亲缘、这一拍只写一件事」；增量若把同一人写成两个身体、或把锁里的父子写成叔侄，探针重试。用户贴出的婚礼后续不再出现两个曹元佩。

**非目标：**

- 不把 `write_beat_once` / `assemble_continue_beat` 改成 ToolLoop，不给主创 `asset_read`。
- 不自动合并或删除角色表脏行（可登记为后续债务）。
- 不改创世首章、划词改写、观察 run。
- 不宣称 ToolLoop JSON 熔断已修。
- 不把导演失败升级成整 run 失败。

---

## 3. 不变量（禁止破坏）

1. `scenes.content` 是唯一叙事真相源。导演锁是编译产物，不落库为第二真相源。
2. 续写主创热路径保持 `complete()`，请求不带 `tools`。`continue_beat_complete_does_not_require_tools` 仍绿。
3. Creative / Tool / Background 三档不变。导演走 **Tool** 档（`AgentRole::Producer`），主创仍 Creative。
4. `prompts` 不得依赖 `agency`。导演 JSON 的 system 字符串放在 `agency/` 或 `resources/prompts/agency/`，组装仍走现有 `assemble_continue_beat`（只改 user 内容与 CONTINUE_BEAT_SYSTEM 文案）。
5. 一次 `write_beat_once` 内冻结节拍卡 + 组装 user（v0.55）；本版冻结件 **增加**导演锁。函数返回后解冻。
6. 取消仍停候选链；剩余 <90s 不重试（`writer_retry_has_time`）。
7. 已完成死亡 / 行刺仍禁止重演（v0.53.6 探针保留）。

---

## 4. 模块切分

```
agency/continue_director.rs   新建。IdentityCluster、DirectorLock、
                              merge_identity_clusters、compile_director_lock_rust、
                              parse_director_json、subject_split_probe、kin_inversion_probe
agency/continue_assets.rs     aliases_for 不改语义；merge 用新函数，避免把「末两字」
                              误并成另一人
agency/beat_card.rs           阵容名改为规范名；用途改为亲缘或「可沉默」，
                              不再写「末段已在场，承接行动」作为全员用途
agency/beat_state.rs          删「丢掉已在场者」「点名不足 2 人」；
                              接入拆人 / 亲缘探针；状态网文案改为可沉默
agency/continue_freeze.rs     FrozenContinueShot 增加 lock
agency/coordinator.rs         write_beat_once：Rust 锁 → 可选导演 JSON → 注入 user →
                              冻结 → complete（仍零工具）
prompts/assembly.rs           CONTINUE_BEAT_SYSTEM 增加一人一号 / 禁止点名巡场范例
```

GitNexus：改 `write_beat_once`、`probe_increment`、`render_writer_user_prompt`、`aliases_for`、`CONTINUE_BEAT_SYSTEM` 前 `impact({target, direction:"upstream"})`。HIGH/CRITICAL 先告知。索引空则 grep 调用点并在实现记录里标明 inspected。

---

## 5. 数据流

`write_beat_once` 在现有「编译节拍卡 + admit + render assets」之后、`continue_freeze.pin` 之前：

1. `prior_tail_for_cast(current_content)`（已有 1500 字近文）。
2. `merge_identity_clusters(table_names, tail, admitted)` → 每簇一个 `canonical` + `aliases`。两表行若属同一簇，只保留一条准入（见 §6.3）。
3. `compile_director_lock_rust(clusters, relationships, card.dead, tail, last_sentence)` → `DirectorLock`。
4. 非测试且剩余时间足够：`Producer.complete_json`（Tool 档，超时 **20s**，`max_tokens` 小）。解析成功则 **只允许增补**别名与亲缘，不得拆开 Rust 已合并的簇，不得把死人标成活人。失败则用步骤 3。
5. `render_writer_user_prompt` 增加 `lock`：插入【本拍人物锁】【本拍只写】；节拍卡阵容用规范名；状态网用新文案。
6. `pin` 冻结 `card + admitted + user + lock`。自重复 / 过短 / 探针重试复用冻结件。
7. `assemble_continue_beat` + `LeadWriter.complete`（Creative，`tools=None`）。
8. `probe_increment(..., lock)`；拆人 / 亲缘颠倒 / 重演死亡 → 一次重试（仍用冻结 user，追加探针缺口一句）。

测试环境（`app_handle=None` 或现有 skip LLM 约定）：跳过步骤 4。

---

## 6. 身份合并（0 LLM，契约可单测）

### 6.1 同一人判定

`same_person(a, b)` 为真当且仅当（去空白后）：

- `a == b`；或
- 较长串以较短串为 **后缀**，短串 ≥2 字，短串 **不是** 纯称号词，且较长串去掉该后缀后的 **前缀含称号词**（见下表）。

称号词：`公主` `亲王` `王妃` `郡主` `太子` `娘娘` `皇上` `陛下` `钦差` `镇北王` `王`（仅当作为独立词出现在前缀中，如 `镇北王`；前缀仅为姓氏一字如 `苏` 不算）。

例：`琬公主曹元佩` 前缀 `琬公主` 含 `公主` → 与 `曹元佩` 同一人；`镇北王苏会山` 与 `苏会山` 同一人。`苏亦铁` 前缀 `苏` 无称号词 → **不**与 `亦铁` 合并。`明成公主` 与 `曹元佩` 不是同一人。

**不**用「末两字」做跨行合并。`aliases_for` 仍只用于正文点名匹配，不参与簇合并。

纯称号词（不可单独当规范名）：上表中无封号的词。`琬公主` 可当别名，不可当唯一规范名——簇内另有 ≥2 字非纯称号名时，规范名取后者。

### 6.2 从近文抽出称号+姓名

在近文中扫描「称号词紧挨已有角色名」（`琬公主`+`曹元佩`、`镇北王`+`苏会山`）。抽出的复合名加入该角色所在簇的 `aliases`，不新建簇。

### 6.3 两表行同一簇

若 `characters.name` 出现「琬公主」与「曹元佩」且 `same_person` 或近文把二者连成称号+名：

- 规范名：近文里作为完整姓名出现次数更高者；平局取较长非纯称号名。
- 准入名单只留规范名那一行的卡；另一行的字段若规范行对应槽为空，可拼进渲染（本版允许只渲染规范行，不强制拼字段）。
- **不写回数据库。**

### 6.4 死人

`dead_names_in_text` 之后把死名映射到规范名。簇内任一名在近文已死 → 整簇 `status=dead`。死人不得进行动阵容（v0.53.6 已有，保持）。

---

## 7. 导演锁

```rust
struct IdentityLock {
    canonical: String,
    aliases: Vec<String>,      // 含称号+姓名、封号；不含单字
    status: Living | Dead,
    kin: Option<String>,       // 一行中文，如「苏会山之妻；与苏会山并坐」
}

struct DirectorLock {
    identities: Vec<IdentityLock>,
    beat_move: String,         // 这一拍只写的一件事，一句
    forbidden: Vec<String>,    // 短禁令
}
```

### 7.1 Rust 兜底（必须单独可测）

- `identities`：§6 的簇；`kin` 从近文规则抽取，没有则 `None`。
- 近文亲缘规则（只认明确句式，不猜家谱）：
  - `X与Y一并坐下` / `X、Y夫妇` / `X与Y夫妇` → 互为配偶向 kin。
  - 近文出现 `父亲`/`父王`/`爹` 且死者规范名与扑向者规范名同场 → 死者为扑向者的父亲（婚礼原文：苏会山死、苏亦铁飞身扑上）。
  - 关系表边仅当两端都在簇内、且 **不与** 上述近文 kin 矛盾时写入（近文优先）。
- `beat_move`：`从「{末句动作}」之后写下一拍，禁止点名式每人一段。` 若有死人，前缀 ` {死人}已死。`
- `forbidden`：固定含 `禁止把同一人的两个称呼写成两个身体`；有死人则含 `禁止重演行刺或死亡`。

### 7.2 管理档 JSON（可选增补）

System 要求：只输出 JSON；`identities[].canonical` 必须是 Rust 锁已有规范名之一；可以往 `aliases`/`kin` 追加；禁止新增规范名；禁止把 `status=dead` 改成 living。

超时 20s 或 `parse_lenient` 失败 → 丢弃，用 Rust 锁。不重试导演（避免烧创作窗口）。

导演 **禁止** ToolLoop、禁止 `asset_read`。一次 `complete_json`。

### 7.3 注入主创 user

在节拍卡全文之后、状态网之前插入：

```
【本拍人物锁】
曹元佩＝琬公主＝镇北王妃。活人。苏会山之妻。禁止拆成两人。
苏会山＝镇北王。已死。苏亦铁之父。
苏亦铁。活人。苏会山之子。末句行动主体。
…
【本拍只写】苏亦铁扑向父亲尸体之后的乱局。禁止按在场名单每人写一段。
```

`render_full` 阵容改为规范名列表，每人用途取 `kin` 或「可沉默」。删除全员「末段已在场，承接行动」。

状态网「必须承接未决，禁止忘掉已在场者」改为：「在场者可以不出声。禁止写成他们不在场。禁止把同一人的别名写成另一个人。」

---

## 8. 探针

`probe_increment` 增加 `lock: &DirectorLock`（测试可构造最小锁）。

**删除（本版起不再作为缺口）：**

- `增量点名在场者不足 2 人`
- `丢掉已在场者：{name}`

**保留：** 重演已完成死亡/行刺；场外角色开篇；`NewScene` / `CharacterMove`（沉寂回归仍须点名该人）；`ConflictEscalation` 改为：有冲突动词 **或** 点名至少一名 **活人** 冲突方即可，不要求点名全部 `parties`（死人冲突方不必行动）。

**新增：**

1. **拆人。** 对每个 `IdentityLock`，若增量里 ≥2 个不同 `aliases∪{canonical}` **独立出现**（更长称呼盖住的短名不计），则 `gaps.push("同一人拆成两个身体：{canonical}")`。真机「琬公主曹元佩抱着曹元佩的衣角」必须命中。旧「则/蜷缩」主语模式是子集。

   「苏亦铁抱住父亲」里的「父亲」不是别名，不触发拆人。

2. **亲缘颠倒。** 若锁对 (A,B) 标明父子/父女/母子，增量在同一句或相邻句用 `侄子`/`侄女`/`姑姑`/`叔父` 描述该对，则缺口 `亲缘与人物锁相反`。不扫全书家谱，只扫锁里写明的 kin 行。

3. **死人行动。** 锁 `status=dead` 的人，点名后 80 字内出现眼睛/锁定/审视等活人行动，且窗口不是尸体/残骸，则 `gaps.push("{canonical}已死仍在行动")`。点名尸体本身不算。

探针缺口仍只触发 **一次** complete 重试（现有时间门）。重试 user 追加缺口原文，不重跑导演。

---

## 9. 续写 system 合同

`CONTINUE_BEAT_SYSTEM` 增加第 5 条与一对范例；非空行数允许 **11–16**（现测试锁 8–12，本版改测试）。

须含：

- `Wrong：曹元佩僵住了。琬公主曹元佩蜷缩在角落。`
- `Right：曹元佩是镇北王妃，一人。`
- 禁止点名式每人一段（措辞短）。

仍禁止：`asset_read`、`JSON action`、节拍卡原文当正文。`continue_beat_complete_does_not_require_tools` 仍断言 user/system 无工具协议。

---

## 10. 失败与降级

| 情况 | 行为 |
|---|---|
| 导演超时 / 非 JSON / canonical 不在 Rust 簇 | 用 Rust 锁，打 warn，继续写 |
| 导演把死人标成活人 | 丢弃该条 status，保留 Rust |
| 主创拆人 / 亲缘反 | 一次重试；仍失败则现网：有实质正文则 salvage 落库（不因身份探针丢稿，与过短 salvage 对齐）。日志 warn 身份缺口 |
| 取消 / 剩余 <90s | 不重试、不停在导演上 |

身份探针失败 **不**把整 run 标 failed（用户要的是下一拍正文，不是空白报错）。缺口写进活动日志，便于真机对照。

---

## 11. 测试与验收

全部为纯函数契约，不依赖真 LLM。婚礼原文（用户消息中大堂行刺段 + 失败续写段）进测试常量。

| 契约 | 失败时意味着 |
|---|---|
| `same_person_title_plus_given_name` | 琬公主曹元佩 与 曹元佩 未合并 |
| `same_person_does_not_merge_unrelated` | 明成公主 与 曹元佩 被误并 |
| `same_person_does_not_merge_name_without_title_prefix` | 苏亦铁 与 亦铁 被误并 |
| `two_table_rows_admit_one_canonical` | 表有琬公主+曹元佩时准入两个规范名 |
| `rust_lock_spouse_from_sat_together` | 「苏会山与曹元佩一并坐下」未写入配偶向 kin |
| `rust_lock_father_from_plunge` | 苏会山死 + 苏亦铁飞身扑上 未锁父子 |
| `probe_rejects_cao_split_bodies` | 用户失败续写未报拆人 |
| `probe_rejects_hugging_own_clothes_as_two_people` | 真机「抱着曹元佩的衣角」未报拆人 |
| `probe_rejects_dead_princess_living_gaze` | 明成公主已死仍用眼睛锁定未报 |
| `probe_does_not_gap_silent_present` | 只写苏亦铁扑尸、未点名曹元佩，仍报丢掉已在场者 |
| `probe_rejects_nephew_when_lock_is_father` | 锁为父子时写侄子不报亲缘反 |
| `probe_still_rejects_stab_replay` | v0.53.6 重演契约回归 |
| `continue_beat_complete_does_not_require_tools` | 主创又带上工具协议 |
| `continue_beat_system_has_identity_example` | 合同缺少拆人 Wrong/Right |
| `pin_keeps_director_lock` | 冻结件丢掉 lock |

既有：`catalog_for_role_is_name_and_one_line`、`continue_user_omits_asset_read`、死亡退场、冻结/解冻。

**设计验收探针（真机，本版不自动跑）：** 同一开头从「飞身扑上」续写一拍。通过 = 增量中曹元佩只有一个身体、苏亦铁不为曹元佩的侄子、不重演短刃扎进胸口。未跑通不得宣称症状已修复。

---

## 12. 版本与文档

实现随发版 **v0.56.0**。四源版本 + `landing/.../FALLBACK_VERSION` + docs of record 同步。CHANGELOG / AGENTS 写明：导演锁 + 探针去点名强迫；**未关闭**真机须再跑；不得宣称续写质量已修复。

ROADMAP 登记债务：角色表脏行写时合并、不落库；若用户要自动并表，另开设计（不可逆）。

---

## 13. 范围边界

一次实现计划覆盖 §5–§11。不拆第二个子系统。不包含：管理 Agent 清表、主创 ToolLoop、跨章家谱推理、前端 UI。
