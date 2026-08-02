import { useEffect, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useAppStore } from '@/stores/appStore';
import { getRun, listBoard, listRuns } from '@/services/api/agency';
import type { BoardItem } from '@/services/api/agency';

/** 角色事件流中的 role 值（AgentRole::as_str）-> 显示名 */
const ROLES: { key: string; name: string }[] = [
  { key: 'lead_writer', name: '主创' },
  { key: 'producer', name: '管理' },
  { key: 'editor_auditor', name: '编辑审计' },
];

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

interface ActivityEvent {
  run_id: string;
  role: string;
  action: string;
  detail: string;
  at: number;
}

interface ProgressEvent {
  run_id: string;
  phase: string;
  status: string;
  message: string;
  at: number;
}

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

export default function AgencyStudio() {
  const currentStory = useAppStore(s => s.currentStory);
  const qc = useQueryClient();
  const [activities, setActivities] = useState<ActivityEvent[]>([]);
  const [progress, setProgress] = useState<ProgressEvent[]>([]);
  const [activeRunId, setActiveRunId] = useState<string | null>(null);

  // 事件接入：activity/progress 驱动时间线与角色卡；board-changed 失效黑板查询。
  useEffect(() => {
    let un1: (() => void) | undefined, un2: (() => void) | undefined, un3: (() => void) | undefined;
    (async () => {
      const { listen } = await import('@tauri-apps/api/event');
      un1 = await listen<Omit<ActivityEvent, 'at'>>('agency-agent-activity', e => {
        setActivities(prev => [...prev.slice(-99), { ...e.payload, at: Date.now() }]);
        setActiveRunId(e.payload.run_id);
      });
      un2 = await listen<Omit<ProgressEvent, 'at'>>('agency-run-progress', e => {
        setProgress(prev => [...prev.slice(-99), { ...e.payload, at: Date.now() }]);
        setActiveRunId(e.payload.run_id);
      });
      un3 = await listen<BoardItem>('agency-board-changed', e => {
        setActiveRunId(e.payload.run_id);
        qc.invalidateQueries({ queryKey: ['agency-board', e.payload.run_id] });
      });
    })();
    return () => {
      un1?.();
      un2?.();
      un3?.();
    };
  }, [qc]);

  // Run 发现：页面打开时从 DB 水合最新 run（不依赖实时事件）。
  // 此前 activeRunId 仅从事件捕获 -> 页面后开时恒 null -> 空白。
  const runsQuery = useQuery({
    queryKey: ['agency-runs', currentStory?.id],
    queryFn: () => listRuns(currentStory!.id),
    enabled: !!currentStory,
    refetchInterval: 10_000,
  });
  const runs = runsQuery.data;

  // 水合：runs 数据到达且当前无 activeRunId 时，取最新 run。
  // 实时事件仍可覆盖（新 run 启动时事件到达，切到新 run）。
  useEffect(() => {
    if (!activeRunId && runs && runs.length > 0) {
      setActiveRunId(runs[0].id);
    }
  }, [runs, activeRunId]);

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

  if (!currentStory) return <p className="p-6 text-gray-500">请先选择一个故事</p>;

  // 查询失败时给出可见错误（此前 react-query 静默重试后失败，卡片恒为 "-"，
  // 用户无法区分"无数据"与"出错"）。
  const queryError = runsQuery.error ?? boardQuery.error ?? runQuery.error;

  const latestProgress = progress.length > 0 ? progress[progress.length - 1] : null;
  const runStatus = latestProgress
    ? `${latestProgress.phase} · ${runStatusLabel(latestProgress.status)}`
    : run
      ? `${run.phase} · ${runStatusLabel(run.status)}`
      : queryError
        ? '状态获取失败'
        : '-';
  const lastAction = (role: string) => {
    // 1. 实时事件优先（页面打开后收到的 agency-agent-activity）
    const a = [...activities].reverse().find(x => x.role === role);
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
  const liveTimeline = [
    ...activities.map(a => ({
      at: a.at,
      text: `${roleName(a.role)} ${a.action} ${a.detail}`,
    })),
    ...progress.map(p => ({
      at: p.at,
      text: `${p.phase} ${p.status} ${p.message}`,
    })),
  ];

  const historicalTimeline: { at: number; text: string }[] = [];
  if (board) {
    for (const item of board) {
      const ts = new Date(item.created_at).getTime();
      if (!isNaN(ts)) {
        historicalTimeline.push({
          at: ts,
          text: `${roleName(item.producer)} 创建 ${ZONE_NAMES[item.zone] ?? item.zone}：${item.key}${item.summary ? ' - ' + item.summary : ''}`,
        });
      }
    }
  }
  if (run) {
    const startTs = new Date(run.created_at).getTime();
    if (!isNaN(startTs)) {
      historicalTimeline.push({
        at: startTs,
        text: `运行启动 - ${run.premise.slice(0, 50)}`,
      });
    }
    const endTs = new Date(run.updated_at).getTime();
    if (!isNaN(endTs) && run.status !== 'pending' && run.status !== 'running') {
      historicalTimeline.push({
        at: endTs,
        text: `运行${runStatusLabel(run.status)} - ${run.phase}`,
      });
    }
  }

  // 合并 + 去重 + 排序（最新在前）
  const seen = new Set<string>();
  const timeline = [...liveTimeline, ...historicalTimeline]
    .filter(t => {
      const key = `${t.at}|${t.text}`;
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
              className="rounded border bg-white px-2 py-1 text-xs text-gray-600"
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
            <span className="text-xs text-gray-400">
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
        <p className="rounded border border-red-200 bg-red-50 p-3 text-sm text-red-600">
          代理状态获取失败：
          {queryError instanceof Error ? queryError.message : String(queryError)}
          （每 10 秒自动重试）
        </p>
      )}

      <section className="grid grid-cols-3 gap-4">
        {ROLES.map(r => (
          <div key={r.key} className="rounded border p-4">
            <div className="font-medium">{r.name}</div>
            <div className="mt-2 text-sm text-gray-600">最近动作：{lastAction(r.key)}</div>
            <div className="mt-1 text-sm text-gray-600">run 状态：{runStatus}</div>
          </div>
        ))}
      </section>

      {!activeRunId && (
        <p className="rounded border border-dashed p-4 text-sm text-gray-500">
          暂无活动--启动创世或续写后，这里会实时显示代理动态。
        </p>
      )}

      {activeRunId && (
        <section>
          <h2 className="mb-2 font-medium">黑板</h2>
          <div className="grid grid-cols-4 gap-3">
            {ZONES.map(z => (
              <div key={z.key} className="rounded border p-3">
                <div className="mb-2 text-sm font-medium text-gray-500">{z.name}</div>
                {byZone(z.key).length === 0 && <p className="text-xs text-gray-400">（空）</p>}
                <div className="space-y-2">
                  {byZone(z.key).map(item => (
                    <div key={item.id} className="rounded bg-gray-50 p-2 text-sm">
                      <div className="flex items-center justify-between gap-2">
                        <span className="font-medium">{item.key}</span>
                        <span className="text-xs text-gray-400">
                          v{item.version} · {item.status}
                        </span>
                      </div>
                      <div className="truncate text-xs text-gray-500">{item.summary}</div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      <section>
        <h2 className="mb-2 font-medium">时间线</h2>
        {timeline.length === 0 ? (
          <p className="text-sm text-gray-400">暂无记录</p>
        ) : (
          <div className="space-y-1 text-sm">
            {timeline.map((t, idx) => (
              <div key={idx} className="flex gap-2">
                <span className="text-gray-400">[{hhmmss(t.at)}]</span>
                <span>{t.text}</span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
