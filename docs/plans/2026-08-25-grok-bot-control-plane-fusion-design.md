# grok-bot 控制面融合

日期：2026-08-25
状态：第一期已落地（v0.54.0）
决策来源：对照 [b-nnett/grok-bot-0.18-reconstructed](https://github.com/b-nnett/grok-bot-0.18-reconstructed) 的系统提示词装配、回合工具集、原生 function calling、技能目录预算；用户裁定 **Approach B（控制面融合）**，第一期做 **原生 tools（创世/管理/编辑 ToolLoop）**。

承接：v0.45.0 提示词运行时组装、v0.51.6 三档模型路由、v0.53.6 高潮不重演。本文件不改 PersistMode、`scenes.content` 真相源、幕前纸面、落地页。

---

## 1. 问题与裁定

### 1.1 学什么 / 不学什么

grok-bot 是桌面通用助手，不是写作引擎。可搬的是控制面纪律，不是产品形态。

| 可搬 | 不搬 |
|---|---|
| 原生 `tools[]` / `tool_calls[]`，文本 JSON 仅作回退 | Cursor / Claude / Codex 供应商 Router |
| 角色/回合级工具集（已有白名单，补原生通道） | SendMessage 作为唯一对用户通道 |
| 技能目录式渐进展开（第二期） | 电脑使用 / Docker 沙盒 / MCP 市场 |
| run 内冻结节拍卡（第三期） | JSX 提示词编译器 |
| 续写短操作合同 + 对错范例（第四期） | 把续写主创拉回 ToolLoop |

### 1.2 已确认裁定

| 项 | 决定 |
|---|---|
| 路径 | **B. 控制面融合**，四期可独立上线 |
| 第一期 | 原生 function calling 进 Agency ToolLoop；`complete()` 续写热路径 **不带** tools |
| JSON 回退 | 本地模型不返回 `tool_calls` 时，现有 `parse_action` 行为不变 |
| 路由 | Creative / Tool / Background 三档保持；禁止供应商级切换 |
| 组装器 | `prompts` 不得依赖 `agency` |

---

## 2. 第一期目标与非目标

**目标**

ToolLoop 调用 LLM 时携带当前角色白名单的 JSON Schema 工具定义。适配器优先解析原生 `tool_calls`；没有则回退文本 JSON action。续写 `write_beat_once` / `assemble_continue_beat` 的请求 `tools = None`。

**非目标（本版不做）**

- 资产渐进展开（第二期）
- run 内冻结节拍卡（第三期）
- 续写 system 对错范例重写（第四期）
- 流式生成带 tools
- PromptCache 按 tools 分键（ToolLoop 本就不走 cache 热路径；探测/续写仍 `tools=None`）

---

## 3. 不变量（禁止破坏）

1. `scenes.content` 是唯一叙事真相源。
2. 续写主创热路径保持单次 `complete()`，**不**把正文写作拉回 ToolLoop。
3. `GenerateRequest.tools == None` 时，三个适配器的请求体与现网字节级兼容（不序列化 `tools` 字段）。
4. `LoopLlm::complete` 签名不变；既有 mock 零改动。新增 `complete_turn`，默认实现忽略 tools、走 `complete`。
5. `architecture_guard`：`prompts` 不得依赖 `agency`。
6. 取消仍停候选链；超窗跳过仍成立。

---

## 4. 模块切分

```
llm/adapter.rs          ToolSpec / ToolCall / informal_args_to_json_schema /
                        openai_tools_payload / extract_openai_tool_calls /
                        extract_anthropic_tool_use / GenerateRequest.tools /
                        GenerateResponse.tool_calls
llm/openai.rs           chat/completions 带 tools；解析 message.tool_calls
llm/ollama.rs           tools 非空时改走 /api/chat；空则仍 /api/generate
llm/anthropic.rs        messages 带 tools；解析 content[].type=tool_use
ports/llm.rs            LlmPortRequest.tools
model_gateway/types.rs  GatewayRequest.tools
llm/service.rs          透传 tools → GenerateRequest
agency/tools.rs         tool_specs_for_role
agency/tool_loop.rs     complete_turn + resolve_loop_action
agency/coordinator.rs   AgencyLlm::complete_turn 把角色工具集送进网关
agency/budget.rs        BudgetedLlm 透传 complete_turn 并计量
```

---

## 5. 数据流

```
ToolLoop::run
  → registry.tool_specs_for_role(role)
  → llm.complete_turn(..., Some(&specs))
      AgencyLlm → LlmPortRequest.tools
        → GatewayRequest.tools
          → GenerateRequest.tools
            → OpenAI tools[] / Ollama /api/chat / Anthropic tools[]
  ← GenerateResponse { content, tool_calls }
  → resolve_loop_action(content, tool_calls)
      有 tool_calls → LoopAction::Tool（多枚只执行第一枚，同现网数组截断）
      无 → parse_action_full(content)   // 现网 JSON 回退
```

续写：`write_beat_once` → `complete_metered_with_format(..., tools=None)` → 适配器不发 tools。

---

## 6. 失败与降级

| 情况 | 行为 |
|---|---|
| 模型返回原生 tool_calls | 执行对应工具，observation 回灌 |
| 模型仍输出文本 JSON action | `parse_action` 与现网一致 |
| 模型两者都有 | **原生优先** |
| Ollama `/api/chat` 不支持 tools（旧版） | 适配器错误进入候选链；JSON 回退仅在「成功但无 tool_calls」时发生，不把 HTTP 4xx 假装成 JSON |
| Anthropic `tool_use` 块 `text` 缺省 | `text` 改为 Option，避免反序列化失败 |
| 续写 / 探测 / 流式 | `tools=None`，请求体不加 tools 字段 |

---

## 7. 验收（第一期）

契约测试必须保护用户结果：

1. `native_tool_calls_preferred_over_text_json`：原生 `board_read` 优先于正文里的 JSON。
2. `text_json_action_still_parses_when_tool_calls_empty`：无原生调用时现网 JSON 仍工作。
3. `generate_request_omits_tools_field_when_none`：`tools=None` 的 JSON 不含 `"tools"`。
4. `tool_specs_for_role_producer_is_json_schema`：白名单工具的 `parameters` 含 `"type":"object"`。
5. `continue_beat_complete_does_not_require_tools`：`assemble_continue_beat` 无 Tools 层；`GenerateRequest::default().tools` 为 None。
6. 既有 `cargo test --lib` agency tool_loop 用例全绿（mock 只实现 `complete`）。

未跑通上述探针，不得宣称「ToolLoop JSON 熔断已修复」。真机仍须用同一创世/资产路径看管理 Agent 是否少一轮解析失败。

---

## 8. 后续各期（本版不实施）

- **第二期**：准入角色半卡 + `asset_read`；Producer catalog 只注入名+一行。
- **第三期**：一次续写 run 冻结节拍卡/阵容（对照 grok-bot memory freeze）。
- **第四期**：`CONTINUE_BEAT_SYSTEM` 改为短操作合同 + 三条对错范例（重演行刺 / 泄露节拍卡 / 发明未出场角色）。
