# 代理工作室实时显示修复与三 Agent 完善——实施计划

> 来源：用户提供的完整计划（2026-08-11），核心论断已抽查属实（AgencyStudio 条件挂载 App.tsx:315、本地监听器随卸载销毁 AgencyStudio.tsx:85-107、App.tsx 常驻且 listen 静态导入、emit_activity coordinator.rs:1326、backendActivityStore 模式存在）。

## 总体策略

只改幕后：幕前幕后是不同 webview 进程，Zustand store 无法跨窗口共享。幕前 useAgencyAgentActivity 挂在常驻 FrontstageApp，工作正常不动。修复核心 = 把幕后 agency 事件监听从条件挂载的 AgencyStudio 提升到常驻 App.tsx 顶层 + 全局 store 缓存事件流。

## 阶段一（P0）：幕后全局 agency 事件捕获——前端

1. 新建 `src-frontend/src/stores/agencyActivityStore.ts`（对标 backendActivityStore 单例无 persist）：
   - export 类型 `AgentActivityEvent`/`AgentProgressEvent`（含 at: number）与常量 `AGENCY_ROLES`（lead_writer 主创/producer 管理/editor_auditor 编辑审计）
   - state：activities/progress（cap 200）、activeRunId
   - actions：appendActivity/appendProgress（内部自动 setActiveRunId(e.run_id)）、setActiveRunId、hydrateFromRuns（if(!activeRunId && runs.length) 守卫）
   - 不缓存 BoardItem、不做终态清空
2. `src-frontend/src/App.tsx` 顶层新增 useEffect（复用 backstage-shown 的 listen 惯例）：agency-agent-activity→appendActivity、agency-run-progress→appendProgress、agency-board-changed→setActiveRunId+invalidateQueries(['agency-board',runId])
3. 重构 `src-frontend/src/pages/AgencyStudio.tsx` 订阅 store：删本地 useState/监听器/类型重复定义；lastAction 与 liveTimeline 按 activeRunId 过滤；水合 effect 改调 hydrateFromRuns；onChange 用 store action；其余（三个 useQuery/byZone/historicalTimeline/JSX/常量）不动
4.（可选）useAgencyAgentActivity.ts 类型从 store import 收敛；其监听逻辑不动
5. 测试：新建 `stores/__tests__/agencyActivityStore.test.ts`（自动 setActiveRunId、cap 200、hydrate 守卫、按 run 过滤）；更新 AgencyStudio.test.tsx（删 event mock、mock store 注入，保持现有 6 个历史重建测试断言）

验收：创世期间（AgencyStudio 未挂载）幕后持续捕获事件；打开 AgencyStudio 立即见实时动态。

## 阶段二（P1）：后端事件信号补齐——coordinator.rs

AgentRole 已 glob 导入、emit_activity 为 &self 方法，无需新增导入。行号以内容锚定（近期有漂移）：

- B-1 concept_pack 调用前：Producer/start/概念（覆盖快速路径+legacy）
- B-2 角色改写：LeadWriter→Producer，Producer/done/概念（修角色标注 BUG）
- B-3 run_role_with_llm_and_budget 调用前：Producer/start/资产
- B-4 check_cancel 后 assemble_only 前：LeadWriter/done/首章（补 legacy 的 done）
- B-5 ensure_assets 内：Producer/start+done/资产补齐（覆盖单章+批量续写）
- B-6 handle_gate 内：Producer/start+done/装配（覆盖单章+批量）
- B-7 spawn_editor_qc：BlackboardService::new→with_events(pool, &app)，后台质检 Review 区写入推 agency-board-changed

测试：coordinator 单测扩展（for_test app_handle=None 时 emit 静默，验证配对逻辑）；B-7 手动/集成验证。

## 阶段三（P2）：健壮性

- C-1 续写「熔断不丢稿」（**行为改变，需用户确认**）：handle_gate 两处 salvage_failed_gate 返回 None 时 return Err → 草稿 ≥600 字符改为降级 EditorVerdict 放行装配落库；<600 保留 Err
- C-2 DETAIL_VERB 补全（useAgencyAgentActivity.ts）：第N章/第N章草稿/审查第N章/资产补齐/后台审查/资产
- C-3 timeline 去重 key 改业务键 ${role}|${action}|${detail}|${phase}|${status}
- C-4 幕前终态清空按 run_id 过滤（可选，**默认跳过**）

## 阶段四：验证

全绿清单：cargo test --lib / npx tsc --noEmit / npx vitest run / cargo +nightly fmt / cargo clippy --lib / npm run format:check / architecture_guard.py。
手动验证：创世期间切幕后即见实时动态；角色卡不永停「正在写」；续写资产补齐/装配可见；后台质检结论实时出现。

## 不改动（边界）

- 幕前 useAgencyAgentActivity 监听逻辑；后端编排/tool_loop/gate 评分；react-query 10s 轮询（兜底保留）；DB schema/迁移

## 基线

rust 1301+2ignored；vitest 404+3skipped
