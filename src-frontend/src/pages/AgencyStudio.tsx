import { useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useAppStore } from '@/stores/appStore';
import {
  AGENCY_ROLES,
  useAgencyActivityStore,
  type AgentActivityEvent,
  type AgentProgressEvent,
} from '@/stores/agencyActivityStore';
import { AiThinking } from '@/components/ui/ai/AiThinking';
import { AiContextCards } from '@/components/ui/ai/AiContextCards';
import { getRun, listActivities, listBoard, listRuns } from '@/services/api/agency';
import type { BoardItem } from '@/services/api/agency';

/** 角色事件流中的 role 值（AgentRole::as_str）-> 显示名 */
const ROLES = AGENCY_ROLES;

/** 扩展角色名映射（board items 的 producer 可能是 writer/inspector 等） */
const ROLE_NAMES: Record<string, string> = {
  lead_writer: '主创',
  producer: '管理',
  editor_auditor: '编辑审计',
  writer: '写手',
  inspector: '检查',
  outline_planner: '大纲',
  style_mimic: '风格',
};

const ZONES: { key: BoardItem['zone']; name: string }[] = [
  { key: 'asset', name: '资产' },
  { key: 'draft', name: '草稿' },
  { key: 'review', name: '审查' },
  { key: 'schedule', name: '计划' },
];

const ZONE_NAMES: Record<string, string> = Object.fromEntries(ZONES.map(z => [z.key, z.name]));

function hhmmss(at: number) {
  const d = new Date(at);
  const p = (n: number) => String(n).padStart(2, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

function roleName(key: string): string {
  return ROLE_NAMES[key] ?? key;
}

function runStatusLabel(status: string): string {
  switch (status) {
    case 'completed':
      return '完成';
    case 'failed':
      return '失败';
    case 'cancelled':
      return '取消';
    case 'pending':
      return '等待';
    case 'running':
      return '运行中';
    default:
      return status;
  }
}

/**
 * 时间线条目：text 为显示文案；role/action/detail/phase/status 为业务键字段
 * （来源没有的字段缺省，构造 key 时以空串占位）。
 */
interface TimelineEntry {
  at: number;
  text: string;
  role?: string;
  action?: string;
  detail?: string;
  phase?: string;
  status?: string;
}

/** 业务键：同一业务事件跨 live/historical 来源产生相同 key（不依赖 at） */
function timelineKey(t: TimelineEntry): string {
  return `${t.role ?? ''}|${t.action ?? ''}|${t.detail ?? ''}|${t.phase ?? ''}|${t.status ?? ''}`;
}

export default function AgencyStudio() {
  const currentStory = useAppStore(s => s.currentStory);
  // 实时事件由 App.tsx 常驻监听器写入 agencyActivityStore
  //（本组件条件挂载，组件内监听会随卸载销毁）。
  const activities = useAgencyActivityStore(s => s.activities);
  const progress = useAgencyActivityStore(s => s.progress);
  const activeRunId = useAgencyActivityStore(s => s.activeRunId);
  const hydrateFromRuns = useAgencyActivityStore(s => s.hydrateFromRuns);
  const setActiveRunId = useAgencyActivityStore(s => s.setActiveRunId);

  // Run 发现：页面打开时从 DB 水合最新 run（不依赖实时事件）。
  // 此前 activeRunId 仅从事件捕获 -> 页面后开时恒 null -> 空白。
  const runsQuery = useQuery({
    queryKey: ['agency-runs', currentStory?.id],
    queryFn: () => listRuns(currentStory!.id),
    enabled: !!currentStory,
    refetchInterval: 10_000,
  });
  const runs = runsQuery.data;

  // 水合：runs 数据到达时由 store 守卫取最新 run（同故事且 activeRunId 已存在
  // 则不覆盖；实时事件仍可覆盖——新 run 启动时事件到达，切到新 run；
  // 故事切换时 store 强制重置为当前故事最新 run）。
  useEffect(() => {
    if (runs && currentStory) hydrateFromRuns(runs, currentStory.id);
  }, [runs, currentStory, hydrateFromRuns]);

  const boardQuery = useQuery({
    queryKey: ['agency-board', activeRunId],
    queryFn: () => listBoard(activeRunId!),
    enabled: !!activeRunId,
    refetchInterval: 10_000,
  });
  const board = boardQuery.data;
  const runQuery = useQuery({
    queryKey: ['agency-run', activeRunId],
    queryFn: () => getRun(activeRunId!),
    enabled: !!activeRunId,
    refetchInterval: 10_000,
  });
  const run = runQuery.data;

  // DB 活动日志轮询（3s）：不依赖 Tauri 事件到达隐藏窗口。
  // live store 事件仍保留，补充轮询间隔内的即时更新。
  const activitiesQuery = useQuery({
    queryKey: ['agency-activities', activeRunId],
    queryFn: () => listActivities(activeRunId!),
    enabled: !!activeRunId,
    refetchInterval: 3_000,
  });
  const dbLogs = activitiesQuery.data ?? [];

  if (!currentStory) return <p className="p-6 text-ai-ink-3">请先选择一个故事</p>;

  // 查询失败时给出可见错误（此前 react-query 静默重试后失败，卡片恒为 "-"，
  // 用户无法区分"无数据"与"出错"）。
  const queryError = runsQuery.error ?? boardQuery.error ?? runQuery.error ?? activitiesQuery.error;

  // DB 活动事件转为 store 兼容格式
  const dbActivities: AgentActivityEvent[] = dbLogs
    .filter(a => a.event_type === 'activity')
    .map(a => ({
      run_id: a.run_id,
      role: a.role ?? '',
      action: a.action ?? '',
      detail: a.detail ?? '',
      at: new Date(a.created_at).getTime(),
    }));
  const dbProgress: AgentProgressEvent[] = dbLogs
    .filter(a => a.event_type === 'progress')
    .map(a => ({
      run_id: a.run_id,
      phase: a.phase ?? '',
      status: a.status ?? '',
      message: a.message ?? '',
      at: new Date(a.created_at).getTime(),
    }));

  // 合并去重：DB 事件为主，live store 事件补充轮询间隔内的新事件
  // （按业务键 role|action|detail / phase|status|message 去重，与时间线逻辑一致）
  const seenActKeys = new Set(dbActivities.map(a => `${a.role}|${a.action}|${a.detail}`));
  const liveActExtra = activeRunId
    ? activities.filter(a => {
        if (a.run_id !== activeRunId) return false;
        return !seenActKeys.has(`${a.role}|${a.action}|${a.detail}`);
      })
    : [];
  const runActivities = [...dbActivities, ...liveActExtra];

  const seenProgKeys = new Set(dbProgress.map(p => `${p.phase}|${p.status}|${p.message}`));
  const liveProgExtra = activeRunId
    ? progress.filter(p => {
        if (p.run_id !== activeRunId) return false;
        return !seenProgKeys.has(`${p.phase}|${p.status}|${p.message}`);
      })
    : [];
  const runProgress = [...dbProgress, ...liveProgExtra];

  const latestProgress = runProgress.length > 0 ? runProgress[runProgress.length - 1] : null;
  const runStatus = latestProgress
    ? `${latestProgress.phase} · ${runStatusLabel(latestProgress.status)}`
    : run
      ? `${run.phase} · ${runStatusLabel(run.status)}`
      : queryError
        ? '状态获取失败'
        : '-';
  const lastAction = (role: string) => {
    // 1. 实时事件优先（页面打开后收到的 agency-agent-activity）
    const a = [...runActivities].reverse().find(x => x.role === role);
    if (a) return `${a.action} ${a.detail}`;
    // 2. 历史重建：页面后开时从黑板条目推导该角色最近动作
    //    （与时间线同一数据源；board.producer 实测仅有三个主角色）
    const item = (board ?? [])
      .filter(i => i.producer === role)
      .sort((x, y) => new Date(y.created_at).getTime() - new Date(x.created_at).getTime())[0];
    if (item) return `创建 ${ZONE_NAMES[item.zone] ?? item.zone}：${item.key}`;
    // 3. 查询失败时明确提示，而非静默 "-"
    if (boardQuery.isError) return '状态获取失败';
    return '-';
  };
  const byZone = (zone: BoardItem['zone']) => (board ?? []).filter(i => i.zone === zone);

  // 时间线重建：三源合并
  // 1. Live 事件（activities + progress）--实时新事件
  // 2. 历史重建（board items 的 created_at + producer + zone + key + summary）
  // 3. Run 生命周期（created_at 启动 + updated_at 终态）
  const liveTimeline: TimelineEntry[] = [
    ...runActivities.map(a => ({
      at: a.at,
      text: `${roleName(a.role)} ${a.action} ${a.detail}`,
      role: a.role,
      action: a.action,
      detail: a.detail,
    })),
    ...runProgress.map(p => ({
      at: p.at,
      text: `${p.phase} ${p.status} ${p.message}`,
      detail: p.message,
      phase: p.phase,
      status: p.status,
    })),
  ];

  const historicalTimeline: TimelineEntry[] = [];
  if (board) {
    for (const item of board) {
      const ts = new Date(item.created_at).getTime();
      if (!isNaN(ts)) {
        historicalTimeline.push({
          at: ts,
          text: `${roleName(item.producer)} 创建 ${ZONE_NAMES[item.zone] ?? item.zone}：${item.key}${item.summary ? ' - ' + item.summary : ''}`,
          role: item.producer,
          action: '创建',
          detail: `${item.zone}:${item.key}`,
        });
      }
    }
  }
  if (run) {
    const startTs = new Date(run.created_at).getTime();
    if (!isNaN(startTs)) {
      const text = `运行启动 - ${run.premise.slice(0, 50)}`;
      historicalTimeline.push({ at: startTs, text, detail: text });
    }
    const endTs = new Date(run.updated_at).getTime();
    if (!isNaN(endTs) && run.status !== 'pending' && run.status !== 'running') {
      const text = `运行${runStatusLabel(run.status)} - ${run.phase}`;
      historicalTimeline.push({
        at: endTs,
        text,
        detail: text,
        phase: run.phase,
        status: run.status,
      });
    }
  }

  // 合并 + 去重 + 排序（最新在前）。
  // 去重用业务键而非 at|text：live 的 at 是 Date.now、historical 是 created_at，
  // 同一业务事件（如快速路径失败回退 legacy 重复发的 done/概念）时间戳不同，
  // 用 at 作 key 会显示两次；缺省字段以空串占位，同事件跨来源产生相同 key。
  const seen = new Set<string>();
  const timeline = [...liveTimeline, ...historicalTimeline]
    .filter(t => {
      const key = timelineKey(t);
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    })
    .sort((x, y) => y.at - x.at)
    .slice(0, 100);

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">代理工作室 · {currentStory.title}</h1>
        <div className="flex items-center gap-2">
          {runs && runs.length > 0 ? (
            <select
              className="rounded border border-ai-line bg-ai-field px-2 py-1 text-xs text-ai-ink"
              value={activeRunId ?? ''}
              onChange={e => setActiveRunId(e.target.value)}
            >
              {runs.map(r => (
                <option key={r.id} value={r.id}>
                  [{r.status}] {r.phase} - {r.premise.slice(0, 30)} (
                  {hhmmss(new Date(r.created_at).getTime())})
                </option>
              ))}
            </select>
          ) : (
            <span className="text-xs text-ai-ink-3">
              {runsQuery.isLoading
                ? '加载中…'
                : activeRunId
                  ? `run ${activeRunId.slice(0, 8)}`
                  : '等待事件'}
            </span>
          )}
        </div>
      </div>

      {queryError && (
        <p
          className="rounded border border-ai-line p-3 text-sm text-ai-red"
          style={{ background: 'color-mix(in srgb, var(--ai-red) 12%, transparent)' }}
        >
          代理状态获取失败：
          {queryError instanceof Error ? queryError.message : String(queryError)}
          （每 10 秒自动重试）
        </p>
      )}

      <section className="grid grid-cols-3 gap-4">
        {ROLES.map(r => (
          <div key={r.key} className="rounded border border-ai-line bg-ai-surface p-4">
            <div className="font-medium">{r.name}</div>
            <div className="mt-2 text-sm text-ai-ink-2">最近动作：{lastAction(r.key)}</div>
            <div className="mt-1 text-sm text-ai-ink-2">run 状态：{runStatus}</div>
          </div>
        ))}
      </section>

      {!activeRunId && (
        <p className="rounded border border-dashed p-4 text-sm text-ai-ink-3">
          暂无活动--启动创世或续写后，这里会实时显示代理动态。
        </p>
      )}

      {activeRunId && (
        <section>
          <h2 className="mb-2 font-medium">黑板</h2>
          <div className="grid grid-cols-4 gap-3">
            {ZONES.map(z => (
              <div key={z.key} className="rounded border border-ai-line bg-ai-surface p-3">
                {byZone(z.key).length === 0 ? (
                  <>
                    <div className="mb-2 text-sm font-medium text-ai-ink-3">{z.name}</div>
                    <p className="text-xs text-ai-ink-3">（空）</p>
                  </>
                ) : (
                  <AiContextCards
                    title={z.name}
                    count={byZone(z.key).length}
                    items={byZone(z.key).map(item => ({
                      key: item.id,
                      title: item.key,
                      meta: `v${item.version} · ${item.status}`,
                      body: item.summary,
                    }))}
                  />
                )}
              </div>
            ))}
          </div>
        </section>
      )}

      <section>
        <h2 className="mb-2 font-medium">时间线</h2>
        {runActivities.length > 0 && (
          <div className="mb-3">
            <AiThinking
              title="当前执行轨迹"
              doneTitle="执行轨迹（已结束）"
              working={run?.status === 'running'}
              rows={runActivities.slice(-12).map(a => ({
                id: `${a.role}|${a.action}|${a.detail}|${a.at}`,
                primary: `${roleName(a.role)} ${a.action}`,
                secondary: a.detail || undefined,
              }))}
              defaultExpanded={run?.status === 'running'}
            />
          </div>
        )}
        {timeline.length === 0 ? (
          <p className="text-sm text-ai-ink-3">暂无记录</p>
        ) : (
          <div className="space-y-1 text-sm">
            {timeline.map((t, idx) => (
              <div key={idx} className="flex gap-2">
                <span className="text-ai-ink-3">[{hhmmss(t.at)}]</span>
                <span>{t.text}</span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
