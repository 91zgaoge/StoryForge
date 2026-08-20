import React from 'react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, act, fireEvent } from '@testing-library/react';
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

const CHAPTER_TEXT = '第一章的正文内容，用于保存链路测试。';

const { listenCallbacks, captured, updateSceneBehavior } = vi.hoisted(() => ({
  listenCallbacks: {} as Record<string, (e: { payload: unknown }) => void>,
  captured: { content: '', editorHtml: '' },
  // 'success' → update_scene 返回 1；'fail' → update_scene 拒绝
  updateSceneBehavior: { mode: 'success' as 'success' | 'fail' },
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
  loggedInvoke: vi.fn((cmd: string) => {
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
      return Promise.resolve([
        {
          id: 'ch-1',
          story_id: 'story-1',
          chapter_number: 1,
          title: '第一章',
          content: CHAPTER_TEXT,
          // 注意：不携带 scene_id，强制走 scenes 列表匹配路径
        },
      ]);
    }
    if (cmd === 'get_story_scenes_paged') {
      return Promise.resolve([
        {
          id: 'scene-1',
          story_id: 'story-1',
          sequence_number: 1,
          title: '第一章',
          chapter_id: 'ch-1',
          content: CHAPTER_TEXT,
          characters_present: [],
          character_conflicts: [],
        },
      ]);
    }
    if (cmd === 'get_story_scenes') {
      return Promise.resolve([]);
    }
    if (cmd === 'get_story_word_count') {
      return Promise.resolve({ total_chars: CHAPTER_TEXT.length });
    }
    if (cmd === 'update_scene') {
      if (updateSceneBehavior.mode === 'fail') {
        return Promise.reject(new Error('database is locked'));
      }
      return Promise.resolve(1);
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
  isActiveCreativeRunConflict: () => false,
}));

const updateSceneCalls = () =>
  vi.mocked(loggedInvoke).mock.calls.filter(([cmd]) => cmd === 'update_scene');

describe('保存链路加固（v0.33.x）', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    for (const k of Object.keys(listenCallbacks)) delete listenCallbacks[k];
    captured.content = '';
    captured.editorHtml = '';
    updateSceneBehavior.mode = 'success';
    useFrontstageStore.getState().setContent('');
    useFrontstageStore.getState().setSceneInfo('', '', undefined);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('selectStory 传入新鲜 scenesResult 时，sceneId 解析为 scene.id 而非 chapter.id（T1 回归）', async () => {
    render(<FrontstageApp />, { wrapper });

    // 等待启动流程：loadStories -> selectStory -> selectChapter
    await waitFor(() => expect(captured.content).toContain('第一章的正文内容'));

    // 修复前：冷启动 scenes state 为空，selectChapter 读到 stale 闭包，
    // sceneId 错误回落为 'ch-1'（chapter.id），update_scene 走后端 heal 建空 scene。
    await waitFor(() => expect(useFrontstageStore.getState().sceneId).toBe('scene-1'));
    expect(useFrontstageStore.getState().chapterId).toBe('ch-1');
  });

  it('persist 重试耗尽后顶栏显示「保存失败，点击重试」，点击后重新 flush 并恢复（T2/T3）', async () => {
    render(<FrontstageApp />, { wrapper });
    await waitFor(() => expect(useFrontstageStore.getState().sceneId).toBe('scene-1'));

    // 让 update_scene 开始失败，触发一次 flush（关闭前 flush 事件走同一链路）
    updateSceneBehavior.mode = 'fail';
    vi.useFakeTimers();
    await act(async () => {
      listenCallbacks['frontstage-flush-requested']({ payload: undefined });
      // 初次失败 + 2s/10s/30s 三次退避重试全部失败
      await vi.advanceTimersByTimeAsync(42000);
    });

    // 重试耗尽 → 可见错误态（此前永远停在「保存中...」）
    expect(screen.getByText('保存失败，点击重试')).toBeInTheDocument();
    expect(updateSceneCalls().length).toBe(4); // 1 次原始 + 3 次重试

    // 后端恢复后点击重试 → 重新 flush，错误态清除
    updateSceneBehavior.mode = 'success';
    await act(async () => {
      fireEvent.click(screen.getByText('保存失败，点击重试'));
      await vi.advanceTimersByTimeAsync(0);
    });

    expect(updateSceneCalls().length).toBe(5);
    expect(screen.queryByText('保存失败，点击重试')).not.toBeInTheDocument();
  });

  it('persist 重试出火时 sceneId 已切换则 no-op，不回写旧 scene（跨场景重试防护）', async () => {
    render(<FrontstageApp />, { wrapper });
    await waitFor(() => expect(useFrontstageStore.getState().sceneId).toBe('scene-1'));

    // update_scene 失败 → 排期 2s 重试（闭包持有 scene-1 的正文）
    updateSceneBehavior.mode = 'fail';
    vi.useFakeTimers();
    await act(async () => {
      listenCallbacks['frontstage-flush-requested']({ payload: undefined });
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(updateSceneCalls().length).toBe(1); // 初次失败，重试已排期

    // 模拟自动分章：store sceneId 在重试出火前已切到新 scene
    act(() => {
      useFrontstageStore.getState().setSceneInfo('scene-2', '第二章', 'ch-2');
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(42000);
    });

    // 持有旧全文的重试全部 no-op（不再调用 update_scene），也不会继续排期后续重试
    expect(updateSceneCalls().length).toBe(1);
    expect(updateSceneCalls().every(([, args]) => args?.scene_id === 'scene-1')).toBe(true);
  });
});
