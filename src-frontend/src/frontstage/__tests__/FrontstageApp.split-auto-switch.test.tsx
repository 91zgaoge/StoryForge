import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import FrontstageApp from '../FrontstageApp';
import { useFrontstageStore } from '../store/frontstageStore';
import { loggedInvoke } from '@/services/tauri';
import { cancelAutoSave } from '../autoSave';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

// 分章前旧章全文（编辑器中持有的内容）：独有开头 + 溢出段落
const FULL_TEXT = '旧章独有开头段落，分章后仍留在旧章。\n\n溢出段落，分章后应出现在新章。';
// 新章（ch-2）内容 = 溢出部分
const OVERFLOW_TEXT = '溢出段落，分章后应出现在新章。';

const { listenCallbacks, captured, editorHtml, syncStoreOptions, splitState } = vi.hoisted(() => ({
  listenCallbacks: {} as Record<string, (e: { payload: unknown }) => void>,
  captured: { content: '', generatedText: '' },
  editorHtml: { current: '' },
  // 捕获 useSyncStore 的回调选项，测试中直接触发 onChapterCreated
  syncStoreOptions: { current: null as Record<string, (...args: any[]) => void> | null },
  splitState: { done: false },
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
      const chapters = [
        { id: 'ch-1', story_id: 'story-1', chapter_number: 1, title: '第一章', content: null },
      ];
      if (splitState.done) {
        chapters.push({
          id: 'ch-2',
          story_id: 'story-1',
          chapter_number: 2,
          title: '第二章',
          content: null,
        } as any);
      }
      return Promise.resolve(chapters);
    }
    if (cmd === 'get_chapter') {
      const id = (args as { id?: string } | undefined)?.id;
      if (id === 'ch-2') {
        return Promise.resolve({
          id: 'ch-2',
          story_id: 'story-1',
          chapter_number: 2,
          title: '第二章',
          content: OVERFLOW_TEXT,
          // 注意：不携带 scene_id，强制走 scenes 列表匹配路径（回归分章 sceneId 解析）
        });
      }
      return Promise.resolve({
        id: 'ch-1',
        story_id: 'story-1',
        chapter_number: 1,
        title: '第一章',
        content: null,
      });
    }
    if (cmd === 'get_chapter_aggregated_content') {
      return Promise.resolve(FULL_TEXT);
    }
    if (cmd === 'get_story_scenes' || cmd === 'get_story_scenes_paged') {
      // 分章后新章的 scene 已落库，分页可拉到
      if (splitState.done) {
        return Promise.resolve([
          {
            id: 'scene-2',
            story_id: 'story-1',
            sequence_number: 2,
            title: '第二章',
            chapter_id: 'ch-2',
            content: OVERFLOW_TEXT,
            characters_present: [],
            character_conflicts: [],
          },
        ]);
      }
      return Promise.resolve([]);
    }
    if (cmd === 'get_story_word_count') {
      return Promise.resolve({ total_chars: FULL_TEXT.length });
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
  classifyIntent: vi.fn(),
  checkPreflight: vi.fn(),
  generateLoglineHint: vi.fn().mockResolvedValue(null),
}));

vi.mock('../autoSave', () => ({
  scheduleAutoSave: vi.fn(),
  cancelAutoSave: vi.fn(),
}));

vi.mock('../components/RichTextEditor', () => ({
  __esModule: true,
  default: React.forwardRef(function MockRichTextEditor(
    props: {
      content: string;
      onChange?: (content: string) => void;
      generatedText?: string;
    },
    ref: React.ForwardedRef<{
      getText: () => string;
      getHTML: () => string;
      appendText: (html: string) => void;
      setContent: (html: string) => void;
    }>
  ) {
    React.useEffect(() => {
      if (props.content !== editorHtml.current) {
        editorHtml.current = props.content;
      }
    }, [props.content]);
    captured.content = editorHtml.current;
    captured.generatedText = props.generatedText ?? '';
    React.useImperativeHandle(ref, () => ({
      getText: () => editorHtml.current.replace(/<[^>]+>/g, ''),
      getHTML: () => editorHtml.current,
      appendText: (html: string) => {
        editorHtml.current = (editorHtml.current || '') + html;
        captured.content = editorHtml.current;
        props.onChange?.(editorHtml.current);
      },
      setContent: (html: string) => {
        editorHtml.current = html;
        captured.content = editorHtml.current;
        props.onChange?.(html);
      },
    }));
    return React.createElement('div', { 'data-testid': 'rich-text-editor' }, editorHtml.current);
  }),
}));

vi.mock('../components/IngestHealthIndicator', () => ({
  IngestHealthIndicator: () => null,
}));

vi.mock('@/hooks/useSubscription', () => ({ useSubscription: () => ({ isPro: false }) }));
vi.mock('@/hooks/useSyncStore', () => ({
  useSyncStore: (opts: Record<string, (...args: any[]) => void>) => {
    syncStoreOptions.current = opts;
  },
}));
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
vi.mock('@/services/modelService', () => ({
  modelService: { checkModelStatus: vi.fn().mockResolvedValue(undefined) },
}));

const mockLoggedInvoke = vi.mocked(loggedInvoke);
const mockCancelAutoSave = vi.mocked(cancelAutoSave);

const updateSceneCalls = () => mockLoggedInvoke.mock.calls.filter(c => c[0] === 'update_scene');
const getChapterCallsFor = (id: string) =>
  mockLoggedInvoke.mock.calls.filter(
    c => c[0] === 'get_chapter' && (c[1] as { id?: string })?.id === id
  );
const chapterListReloadCalls = () =>
  mockLoggedInvoke.mock.calls.filter(
    c => c[0] === 'get_story_chapters_paged' || c[0] === 'get_story_chapters'
  );

describe('自动分章：chapterCreated(split_from_chapter_id) 命中当前编辑章时自动切换到新章', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    for (const k of Object.keys(listenCallbacks)) delete listenCallbacks[k];
    captured.content = '';
    captured.generatedText = '';
    editorHtml.current = '';
    syncStoreOptions.current = null;
    splitState.done = false;
    useFrontstageStore.getState().setContent('');
    useFrontstageStore.getState().setSceneInfo('', '', undefined);
  });

  it('分章命中当前章：重载章节列表 + 切换到新章 + 旧全文不回写旧 scene', async () => {
    render(<FrontstageApp />, { wrapper });

    // 等待旧章全文加载进编辑器
    await waitFor(() => expect(captured.content).toContain('旧章独有开头段落'));
    expect(captured.content).toContain('溢出段落');
    expect(syncStoreOptions.current?.onChapterCreated).toBeTypeOf('function');

    // 分章发生：章节列表此后包含新章
    splitState.done = true;
    mockLoggedInvoke.mockClear();
    mockCancelAutoSave.mockClear();

    await act(async () => {
      syncStoreOptions.current!.onChapterCreated!('story-1', 'ch-2', '第二章', 'ch-1');
    });

    // 编辑器切换到新章：显示溢出内容，不再显示旧章独有开头
    await waitFor(() => expect(captured.content).toContain('溢出段落'));
    await waitFor(() => expect(captured.content).not.toContain('旧章独有开头段落'));

    // 章节列表已重载，新章被拉取
    expect(chapterListReloadCalls().length).toBeGreaterThan(0);
    expect(getChapterCallsFor('ch-2').length).toBeGreaterThan(0);

    // 关键安全断言：旧全文绝未通过 update_scene 回写到旧 scene
    expect(updateSceneCalls()).toHaveLength(0);
    // 待执行的防抖保存被取消
    expect(mockCancelAutoSave).toHaveBeenCalled();
  });

  it('分章自动切换：新章 sceneId 解析为 scene.id（非 chapter.id 回落），避免重复 scene heal', async () => {
    render(<FrontstageApp />, { wrapper });

    // 等待旧章全文加载进编辑器
    await waitFor(() => expect(captured.content).toContain('旧章独有开头段落'));
    expect(syncStoreOptions.current?.onChapterCreated).toBeTypeOf('function');

    // 分章发生：此后 get_story_scenes_paged 返回新章的 scene-2（不携带 scene_id 的
    // get_chapter 强制走 scenes 列表匹配路径）
    splitState.done = true;
    mockLoggedInvoke.mockClear();

    await act(async () => {
      syncStoreOptions.current!.onChapterCreated!('story-1', 'ch-2', '第二章', 'ch-1');
    });

    // 切换到新章后 sceneId 必须是 scene id。修复前：split 分支未拉取 scenes，
    // selectChapter 读到 stale 空数组，sceneId 回落 chapter.id（'ch-2'），后续
    // update_scene 走后端 heal 建出 id=chapter.id 的重复 scene，正文被拆到两个 scene。
    await waitFor(() => expect(useFrontstageStore.getState().chapterId).toBe('ch-2'));
    await waitFor(() => expect(useFrontstageStore.getState().sceneId).toBe('scene-2'));
    // scenes 分页接口已被拉取，为 sceneId 解析提供新鲜列表
    expect(
      mockLoggedInvoke.mock.calls.filter(c => c[0] === 'get_story_scenes_paged').length
    ).toBeGreaterThan(0);
  });

  it('非分章的 chapterCreated（无 split_from_chapter_id）：只重载列表，不切换章节', async () => {
    render(<FrontstageApp />, { wrapper });

    await waitFor(() => expect(captured.content).toContain('旧章独有开头段落'));

    splitState.done = true;
    mockLoggedInvoke.mockClear();
    mockCancelAutoSave.mockClear();

    await act(async () => {
      syncStoreOptions.current!.onChapterCreated!('story-1', 'ch-2', '第二章');
    });

    // 列表重载，但未拉取/切换到新章，编辑器内容不变
    await waitFor(() => expect(chapterListReloadCalls().length).toBeGreaterThan(0));
    expect(getChapterCallsFor('ch-2')).toHaveLength(0);
    expect(captured.content).toContain('旧章独有开头段落');
    expect(mockCancelAutoSave).not.toHaveBeenCalled();
  });

  it('split_from_chapter_id 不是当前章：不切换章节', async () => {
    render(<FrontstageApp />, { wrapper });

    await waitFor(() => expect(captured.content).toContain('旧章独有开头段落'));

    splitState.done = true;
    mockLoggedInvoke.mockClear();
    mockCancelAutoSave.mockClear();

    await act(async () => {
      // 分章来源是别的章节（ch-other），与当前编辑的 ch-1 无关
      syncStoreOptions.current!.onChapterCreated!('story-1', 'ch-2', '第二章', 'ch-other');
    });

    await waitFor(() => expect(chapterListReloadCalls().length).toBeGreaterThan(0));
    expect(getChapterCallsFor('ch-2')).toHaveLength(0);
    expect(captured.content).toContain('旧章独有开头段落');
    expect(mockCancelAutoSave).not.toHaveBeenCalled();
  });
});
