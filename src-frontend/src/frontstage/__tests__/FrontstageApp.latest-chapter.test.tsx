import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import FrontstageApp from '../FrontstageApp';
import { loggedInvoke } from '@/services/tauri';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

const CH1_TEXT = '第一章正文：临江城入了夜，雨便下个不停。';
const CH3_TEXT = '第三章正文：沈砚握紧罗盘，踏入江心雾霭。';

const CHAPTERS = [
  { id: 'ch-1', story_id: 'story-1', chapter_number: 1, title: '第一章', content: null },
  { id: 'ch-2', story_id: 'story-1', chapter_number: 2, title: '第2章', content: null },
  { id: 'ch-3', story_id: 'story-1', chapter_number: 3, title: '第3章', content: null },
];

const { captured, invokeCalls } = vi.hoisted(() => ({
  captured: { content: '' },
  invokeCalls: [] as Array<{ cmd: string; args?: Record<string, unknown> }>,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve(undefined)),
}));

vi.mock('@/services/tauri', () => ({
  loggedInvoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    invokeCalls.push({ cmd, args });
    if (cmd === 'get_gateway_status') {
      return Promise.resolve({
        last_probe_at: undefined,
        primary_model_id: undefined,
        models: [],
        is_probing: false,
      });
    }
    if (cmd === 'list_stories') {
      return Promise.resolve([{ id: 'story-1', title: '测试小说' }]);
    }
    if (cmd === 'get_story_chapters' || cmd === 'get_story_chapters_paged') {
      return Promise.resolve(CHAPTERS);
    }
    if (cmd === 'get_story_scenes_paged') {
      // 场景分页首页只含第一章场景，最新章场景需 get_chapter_scenes 补拉
      return Promise.resolve([
        {
          id: 'scene-ch1',
          story_id: 'story-1',
          chapter_id: 'ch-1',
          sequence_number: 1,
          title: '第一章',
          content: CH1_TEXT,
        },
      ]);
    }
    if (cmd === 'get_chapter_scenes') {
      return Promise.resolve([
        {
          id: 'scene-ch3',
          story_id: 'story-1',
          chapter_id: 'ch-3',
          sequence_number: 3,
          title: '第3章',
          content: CH3_TEXT,
        },
      ]);
    }
    if (cmd === 'get_chapter') {
      const id = args?.id as string;
      return Promise.resolve(CHAPTERS.find(c => c.id === id) ?? null);
    }
    if (cmd === 'get_chapter_aggregated_content') {
      return Promise.resolve(args?.chapter_id === 'ch-3' ? CH3_TEXT : CH1_TEXT);
    }
    if (cmd === 'get_story_word_count') {
      return Promise.resolve({ total_chars: CH1_TEXT.length + CH3_TEXT.length });
    }
    return Promise.resolve(undefined);
  }),
  recordFeedback: vi.fn(),
  smartExecute: vi.fn(),
  getInputHint: vi.fn(),
  runRefine: vi.fn(),
  runReview: vi.fn(),
  runFinalize: vi.fn(),
  getPipelineActiveDraft: vi.fn(),
}));

vi.mock('../components/RichTextEditor', () => ({
  __esModule: true,
  default: React.forwardRef(function MockRichTextEditor(
    props: { content: string; onChange?: (content: string) => void },
    ref: React.ForwardedRef<{ getText: () => string; getHTML: () => string }>
  ) {
    captured.content = props.content;
    React.useImperativeHandle(ref, () => ({
      getText: () => props.content.replace(/<[^>]+>/g, ''),
      getHTML: () => props.content,
    }));
    return React.createElement('div', { 'data-testid': 'rich-text-editor' }, props.content);
  }),
}));

vi.mock('../components/IngestHealthIndicator', () => ({
  IngestHealthIndicator: () => null,
}));

vi.mock('@/hooks/useSubscription', () => ({ useSubscription: () => ({ isPro: false }) }));
vi.mock('@/hooks/useSyncStore', () => ({ useSyncStore: () => {} }));
vi.mock('@/hooks/usePipelineProgress', () => ({
  usePipelineProgress: () => ({ data: null }),
  usePipelineComplete: () => null,
}));
vi.mock('@/hooks/useCharacters', () => ({ useCharacters: () => ({ data: [] }) }));
vi.mock('@/hooks/useSettings', () => ({
  useSettings: () => ({ data: null }),
  useModels: () => ({ data: [] }),
}));
vi.mock('@/stores/modelConnectionStore', () => ({
  useModelConnectionStore: () => ({ states: {} }),
}));
vi.mock('react-hot-toast', () => ({ default: { success: vi.fn(), error: vi.fn() } }));
vi.mock('@/utils/errorHandler', () => ({
  handleAsyncError: vi.fn(),
  showErrorToast: vi.fn(),
  logError: vi.fn(),
}));

describe('启动定位最新章节（v0.33.7）', () => {
  beforeEach(() => {
    captured.content = '';
    invokeCalls.length = 0;
  });

  it('selectStory 应选中 chapter_number 最大的章节并加载其正文', async () => {
    render(<FrontstageApp />, { wrapper });

    await waitFor(() => {
      expect(captured.content).toContain('第三章正文');
    });
    expect(captured.content).not.toContain('第一章正文');

    // 章节列表一次性全量拉取（get_story_chapters）
    expect(invokeCalls.some(c => c.cmd === 'get_story_chapters')).toBe(true);
    // get_chapter 懒加载的是最新章而非第一章
    const getChapterCalls = invokeCalls.filter(c => c.cmd === 'get_chapter');
    expect(getChapterCalls.length).toBeGreaterThan(0);
    expect(getChapterCalls[0].args?.id).toBe('ch-3');
  });

  it('最新章场景不在分页首页时应通过 get_chapter_scenes 补拉', async () => {
    render(<FrontstageApp />, { wrapper });

    await waitFor(() => {
      expect(captured.content).toContain('第三章正文');
    });
    expect(
      invokeCalls.some(c => c.cmd === 'get_chapter_scenes' && c.args?.chapter_id === 'ch-3')
    ).toBe(true);
  });
});
