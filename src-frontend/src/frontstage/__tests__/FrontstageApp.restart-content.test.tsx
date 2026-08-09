import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import FrontstageApp from '../FrontstageApp';
import { useFrontstageStore } from '../store/frontstageStore';
import { loggedInvoke } from '@/services/tauri';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

const CHAPTER_TEXT =
  '空气是粘稠的，带着一种金属锈蚀和腐败的甜腥味。\n\n凯尔的呼吸声在头盔内部被放大成粗重的喘息。';

const { listenCallbacks, captured } = vi.hoisted(() => ({
  listenCallbacks: {} as Record<string, (e: { payload: unknown }) => void>,
  captured: { content: '', editorHtml: '' },
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, cb: (e: { payload: unknown }) => void) => {
    listenCallbacks[event] = cb;
    return Promise.resolve(() => {});
  }),
  emit: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve(undefined)),
}));

vi.mock('@/services/tauri', () => ({
  loggedInvoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
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
      // B2 分页：不返回 content，模拟真实后端
      return Promise.resolve([
        {
          id: 'ch-1',
          story_id: 'story-1',
          chapter_number: 1,
          title: '第一章',
          content: null,
        },
      ]);
    }
    if (cmd === 'get_story_scenes_paged') {
      return Promise.resolve([]);
    }
    if (cmd === 'get_chapter') {
      // Phase 4: chapters 表已剥离 content 字段，get_chapter 不再携带正文
      return Promise.resolve({
        id: 'ch-1',
        story_id: 'story-1',
        chapter_number: 1,
        title: '第一章',
        content: null,
      });
    }
    if (cmd === 'get_chapter_aggregated_content') {
      // Scene 为唯一内容真相源
      return Promise.resolve(CHAPTER_TEXT);
    }
    if (cmd === 'get_story_word_count') {
      return Promise.resolve({ total_chars: CHAPTER_TEXT.length });
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
      // v0.30.43: getHTML 返回 captured.editorHtml（模拟编辑器实际内容），
      // 若未设置则回退 props.content。测试通过设置 editorHtml 模拟 200ms 防抖
      // 窗口内编辑器有新内容但 latestContentRef 尚未更新的场景。
      getHTML: () => captured.editorHtml || props.content,
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
  parseStructuredError: vi.fn((e: unknown) => e),
}));

describe('Bug: 应用重启后应正确加载上次生成的正文', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    for (const k of Object.keys(listenCallbacks)) delete listenCallbacks[k];
    captured.content = '';
    captured.editorHtml = '';
    useFrontstageStore.getState().setContent('');
    useFrontstageStore.getState().setSceneInfo('', '', undefined);
  });

  it('get_chapter 不携带 content 时，应通过 get_chapter_aggregated_content 加载正文', async () => {
    render(<FrontstageApp />, { wrapper });

    // 等待启动流程：loadStories -> selectStory -> selectChapter -> lazy load
    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'));

    // autoFormatText 会把纯文本包装成 <p> 段落，因此断言 HTML 中是否包含正文
    const textOnly = captured.content.replace(/<[^>]+>/g, '');
    expect(textOnly).toContain('空气是粘稠的，带着一种金属锈蚀和腐败的甜腥味。');
    expect(textOnly).toContain('凯尔的呼吸声在头盔内部被放大成粗重的喘息。');
  });

  it('close-flush 保存编辑器实际内容，而非滞后的 latestContentRef（200ms 防抖窗口）', async () => {
    render(<FrontstageApp />, { wrapper });

    // 等待章节加载完成
    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'));

    // 模拟 200ms HTML 防抖窗口：编辑器已有新内容，但 onChange 尚未触发、
    // latestContentRef 仍停留在章节加载时的旧内容。
    const UNSAVED_TYPING = '<p>空气是粘稠的……用户刚刚输入的新段落。</p>';
    captured.editorHtml = UNSAVED_TYPING;

    // 触发关闭前 flush 事件
    await act(async () => {
      listenCallbacks['frontstage-flush-requested']({ payload: undefined });
    });

    // 验证 update_scene 被调用，且 content 是编辑器实际内容（含未保存输入）
    await waitFor(() => {
      const updateCalls = vi
        .mocked(loggedInvoke)
        .mock.calls.filter(([cmd]) => cmd === 'update_scene');
      expect(updateCalls.length).toBeGreaterThan(0);
      const lastCallArgs = updateCalls[updateCalls.length - 1][1] as {
        updates: { content?: string };
      };
      expect(lastCallArgs.updates.content).toContain('用户刚刚输入的新段落');
    });
  });
});
