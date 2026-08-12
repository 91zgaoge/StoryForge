-- 代理活动日志：持久化 agency-agent-activity / agency-run-progress 事件。
-- 幕后代理工作室 3s 轮询拉取，不再依赖 Tauri 实时事件到达隐藏窗口。
CREATE TABLE agency_activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    event_type TEXT NOT NULL,   -- 'activity' | 'progress'
    role TEXT,                  -- activity: lead_writer/producer/editor_auditor
    action TEXT,                -- activity: start/done
    detail TEXT,                -- activity: 概念/首章/深度资产/审查/装配
    phase TEXT,                 -- progress: concept/assets/writing/review/assembly
    status TEXT,                -- progress: running/completed/failed
    message TEXT,               -- progress: 描述文案
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_agency_activity_log_run ON agency_activity_log(run_id);
