import { create } from 'zustand';

/**
 * Agency 三 agent 事件流全局缓存（对标 backendActivityStore：单例、无 persist）。
 *
 * P0 修复：AgencyStudio 仅在 currentView==='agency-studio' 时挂载，组件内的事件
 * 监听随卸载销毁，创世/续写期间（页面未开）事件全丢。监听器提升到常驻 App.tsx
 * 顶层后写入本 store，页面打开即可见实时动态。
 */

/** 角色事件流中的 role 值（AgentRole::as_str）-> 显示名 */
export const AGENCY_ROLES: { key: string; name: string }[] = [
  { key: 'lead_writer', name: '主创' },
  { key: 'producer', name: '管理' },
  { key: 'editor_auditor', name: '编辑审计' },
];

/** agency-agent-activity 事件载荷（at 为前端接收时刻，后端不携带） */
export interface AgentActivityEvent {
  run_id: string;
  role: string;
  action: string; // "start" | "done"
  detail: string; // 概念/首章/深度资产/审查/装配
  at: number;
}

/** agency-run-progress 事件载荷（at 为前端接收时刻） */
export interface AgentProgressEvent {
  run_id: string;
  phase: string;
  status: string;
  message: string;
  at: number;
}

/** 事件流缓存上限：超出丢弃最旧，防长会话内存膨胀 */
const EVENT_CAP = 200;

interface AgencyActivityState {
  activities: AgentActivityEvent[];
  progress: AgentProgressEvent[];
  activeRunId: string | null;

  /** 追加 agent 活动事件；内部自动将 activeRunId 切到该事件的 run */
  appendActivity: (e: Omit<AgentActivityEvent, 'at'>) => void;
  /** 追加 run 进度事件；内部自动将 activeRunId 切到该事件的 run */
  appendProgress: (e: Omit<AgentProgressEvent, 'at'>) => void;
  setActiveRunId: (runId: string) => void;
  /** 页面打开时从 DB run 列表水合 activeRunId；已有 activeRunId（实时事件先行）时不覆盖 */
  hydrateFromRuns: (runs: { id: string }[]) => void;
}

export const useAgencyActivityStore = create<AgencyActivityState>(set => ({
  activities: [],
  progress: [],
  activeRunId: null,

  appendActivity: e => {
    set(state => ({
      activities: [...state.activities.slice(-(EVENT_CAP - 1)), { ...e, at: Date.now() }],
      activeRunId: e.run_id,
    }));
  },

  appendProgress: e => {
    set(state => ({
      progress: [...state.progress.slice(-(EVENT_CAP - 1)), { ...e, at: Date.now() }],
      activeRunId: e.run_id,
    }));
  },

  setActiveRunId: runId => set({ activeRunId: runId }),

  hydrateFromRuns: runs => {
    set(state => {
      if (!state.activeRunId && runs.length > 0) return { activeRunId: runs[0].id };
      return state;
    });
  },
}));
