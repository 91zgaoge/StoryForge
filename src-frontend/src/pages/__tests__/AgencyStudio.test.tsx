import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const { setCurrentView } = vi.hoisted(() => ({ setCurrentView: vi.fn() }));

vi.mock('@/services/api/agency', () => ({
  listBoard: vi.fn().mockResolvedValue([]),
  getRun: vi.fn().mockResolvedValue(null),
  listRuns: vi.fn().mockResolvedValue([]),
}));
vi.mock('@/stores/appStore', () => ({
  useAppStore: (sel: (s: Record<string, unknown>) => unknown) =>
    sel({
      currentStory: { id: 's1', title: '工作室书' },
      setCurrentView,
    }),
}));

import AgencyStudio from '../AgencyStudio';
import { listBoard, getRun, listRuns } from '@/services/api/agency';
import { useAgencyActivityStore } from '@/stores/agencyActivityStore';

const RUN_1 = {
  id: 'run-1',
  story_id: 's1',
  premise: '一个故事',
  status: 'completed',
  phase: 'assembly',
  result_json: null,
  error_message: null,
  created_at: '2026-07-29T10:00:00+08:00',
  updated_at: '2026-07-29T10:05:00+08:00',
};

const BOARD_ITEM_1 = {
  id: 'b1',
  run_id: 'run-1',
  story_id: 's1',
  zone: 'asset' as const,
  item_type: 'world',
  key: '世界观',
  content: '内容',
  summary: '双星系统',
  version: 1,
  producer: 'producer',
  status: 'active',
  created_at: '2026-07-29T10:01:00+08:00',
  updated_at: '2026-07-29T10:01:00+08:00',
};

function renderStudio() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <AgencyStudio />
    </QueryClientProvider>
  );
}

describe('AgencyStudio', () => {
  beforeEach(() => {
    // 重置全局事件 store（真实 store，无 persist/副作用，测试间需清理）
    useAgencyActivityStore.setState({ activities: [], progress: [], activeRunId: null });
    vi.mocked(listRuns).mockResolvedValue([]);
    vi.mocked(listBoard).mockResolvedValue([]);
    vi.mocked(getRun).mockResolvedValue(null);
  });

  it('渲染三角色状态卡与黑板空态', async () => {
    renderStudio();
    expect(await screen.findByText('主创')).toBeInTheDocument();
    expect(await screen.findByText('管理')).toBeInTheDocument();
    expect(await screen.findByText('编辑审计')).toBeInTheDocument();
    expect(await screen.findByText(/暂无活动/)).toBeInTheDocument();
  });

  it('有历史 run 时水合 activeRunId 并显示黑板', async () => {
    vi.mocked(listRuns).mockResolvedValue([RUN_1]);
    vi.mocked(listBoard).mockResolvedValue([BOARD_ITEM_1]);
    vi.mocked(getRun).mockResolvedValue(RUN_1);

    renderStudio();
    // 黑板标题应出现（activeRunId 水合后）
    expect(await screen.findByText('黑板')).toBeInTheDocument();
    // 黑板条目应出现
    expect(await screen.findByText('世界观')).toBeInTheDocument();
    // 不应显示"暂无活动"
    expect(screen.queryByText(/暂无活动/)).not.toBeInTheDocument();
  });

  it('run 选择器渲染多个 option', async () => {
    const RUN_2 = {
      ...RUN_1,
      id: 'run-2',
      premise: '故事二',
      status: 'running',
      phase: 'writing',
      created_at: '2026-07-29T11:00:00+08:00',
      updated_at: '2026-07-29T11:05:00+08:00',
    };
    vi.mocked(listRuns).mockResolvedValue([RUN_2, RUN_1]);

    renderStudio();
    const select = await screen.findByRole('combobox');
    expect(select).toBeInTheDocument();
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(2);
  });

  it('历史时间线从 board items 重建', async () => {
    vi.mocked(listRuns).mockResolvedValue([RUN_1]);
    vi.mocked(listBoard).mockResolvedValue([BOARD_ITEM_1]);
    vi.mocked(getRun).mockResolvedValue(RUN_1);

    renderStudio();
    // 时间线应包含 board item 重建的条目
    expect(await screen.findByText(/管理 创建 资产：世界观/)).toBeInTheDocument();
    // 时间线应包含 run 生命周期条目
    expect(await screen.findByText(/运行启动/)).toBeInTheDocument();
    expect(await screen.findByText(/运行完成/)).toBeInTheDocument();
  });

  it('页面后开（无实时事件）时角色卡从 board items 重建三代理最近动作', async () => {
    const BOARD_WRITER = {
      ...BOARD_ITEM_1,
      id: 'b2',
      zone: 'draft' as const,
      key: '首章',
      producer: 'lead_writer',
      created_at: '2026-07-29T10:02:00+08:00',
    };
    const BOARD_EDITOR = {
      ...BOARD_ITEM_1,
      id: 'b3',
      zone: 'review' as const,
      key: '质检结论',
      producer: 'editor_auditor',
      created_at: '2026-07-29T10:03:00+08:00',
    };
    vi.mocked(listRuns).mockResolvedValue([RUN_1]);
    vi.mocked(listBoard).mockResolvedValue([BOARD_ITEM_1, BOARD_WRITER, BOARD_EDITOR]);
    vi.mocked(getRun).mockResolvedValue(RUN_1);

    renderStudio();
    // 三张角色卡分别显示各自角色的历史最近动作
    expect(await screen.findByText(/最近动作：创建 资产：世界观/)).toBeInTheDocument();
    expect(await screen.findByText(/最近动作：创建 草稿：首章/)).toBeInTheDocument();
    expect(await screen.findByText(/最近动作：创建 审查：质检结论/)).toBeInTheDocument();
    // run 状态使用本地化文案（completed -> 完成）
    expect((await screen.findAllByText(/run 状态：assembly · 完成/)).length).toBe(3);
  });

  it('listRuns 失败时显示错误提示而非静默空态', async () => {
    vi.mocked(listRuns).mockRejectedValue(new Error('IPC boom'));

    renderStudio();
    expect(await screen.findByText(/代理状态获取失败/)).toBeInTheDocument();
    expect(await screen.findByText(/IPC boom/)).toBeInTheDocument();
  });

  it('同一业务事件重复发（时间戳不同）时时间线只显示一次（业务键去重）', async () => {
    vi.mocked(listRuns).mockResolvedValue([RUN_1]);
    vi.mocked(getRun).mockResolvedValue(RUN_1);
    // 快速路径失败回退 legacy 时会重复发同一 done/概念事件：
    // live 条目的 at 是 Date.now，两次发的 at 必不同，旧 at|text key 去重失效。
    useAgencyActivityStore.setState({
      storyId: 's1',
      activeRunId: 'run-1',
      activities: [
        { run_id: 'run-1', role: 'producer', action: 'done', detail: '概念', at: 1000 },
        { run_id: 'run-1', role: 'producer', action: 'done', detail: '概念', at: 2000 },
        // 业务键不同（action 不同）的条目应保留
        { run_id: 'run-1', role: 'producer', action: 'start', detail: '概念', at: 1500 },
      ],
      progress: [
        {
          run_id: 'run-1',
          phase: 'concept',
          status: 'running',
          message: '正在构思故事概念',
          at: 1100,
        },
        {
          run_id: 'run-1',
          phase: 'concept',
          status: 'running',
          message: '正在构思故事概念',
          at: 1300,
        },
      ],
    });

    renderStudio();
    expect((await screen.findAllByText('管理 done 概念')).length).toBe(1);
    expect((await screen.findAllByText('管理 start 概念')).length).toBe(1);
    expect((await screen.findAllByText('concept running 正在构思故事概念')).length).toBe(1);
  });

  it('观察 run 显示观察中且轨迹为工作中', async () => {
    const OBSERVE_RUN = {
      ...RUN_1,
      id: 'observe-s1',
      premise: '观察',
      status: 'observing',
      phase: 'observe',
    };
    vi.mocked(listRuns).mockResolvedValue([OBSERVE_RUN]);
    vi.mocked(getRun).mockResolvedValue(OBSERVE_RUN);
    useAgencyActivityStore.setState({
      storyId: 's1',
      activeRunId: 'observe-s1',
      activities: [
        { run_id: 'observe-s1', role: 'producer', action: 'start', detail: '资产回流', at: 1000 },
        {
          run_id: 'observe-s1',
          role: 'lead_writer',
          action: 'start',
          detail: '编译节拍',
          at: 1100,
        },
        {
          run_id: 'observe-s1',
          role: 'editor_auditor',
          action: 'start',
          detail: '后台审查',
          at: 1200,
        },
      ],
      progress: [],
    });

    renderStudio();
    expect((await screen.findAllByText(/观察中/)).length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('当前执行轨迹')).toBeInTheDocument();
    expect(screen.getByTestId('ai-thinking-spinner')).toBeInTheDocument();
    expect(screen.getByText('管理 start 资产回流')).toBeInTheDocument();
    expect(screen.getByText('主创 start 编译节拍')).toBeInTheDocument();
    expect(screen.getByText('编辑审计 start 后台审查')).toBeInTheDocument();
  });

  it('点击资产卡跳到对应幕后页', async () => {
    const CHAR_ITEM = {
      ...BOARD_ITEM_1,
      id: 'b-char',
      item_type: 'character',
      key: 'character:苏会山',
      summary: '苏会山',
    };
    vi.mocked(listRuns).mockResolvedValue([RUN_1]);
    vi.mocked(listBoard).mockResolvedValue([CHAR_ITEM]);
    vi.mocked(getRun).mockResolvedValue(RUN_1);
    setCurrentView.mockClear();

    renderStudio();
    fireEvent.click(await screen.findByText('character:苏会山'));
    expect(setCurrentView).toHaveBeenCalledWith('characters');
  });
});
