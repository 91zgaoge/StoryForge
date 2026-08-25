import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import FrontstageApp from '../FrontstageApp';
import { useFrontstageStore } from '../store/frontstageStore';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

const CHAPTER_TEXT =
  '空气是粘稠的，带着一种金属锈蚀和腐败的甜腥味。\n\n凯尔的呼吸声在头盔内部被放大成粗重的喘息。';

const { listenCallbacks, captured, editorHtml, mockSmartExecute, mockConfirmAssetRefresh } =
  vi.hoisted(() => ({
    listenCallbacks: {} as Record<string, (e: { payload: unknown }) => void>,
    captured: { content: '', generatedText: '' },
    editorHtml: { current: '' },
    mockSmartExecute: vi.fn(),
    mockConfirmAssetRefresh: vi.fn(),
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
  loggedInvoke: vi.fn((cmd: string, _args?: Record<string, unknown>) => {
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
          content: null,
        },
      ]);
    }
    if (cmd === 'get_chapter') {
      return Promise.resolve({
        id: 'ch-1',
        story_id: 'story-1',
        chapter_number: 1,
        title: '第一章',
        content: null,
      });
    }
    if (cmd === 'get_chapter_aggregated_content') {
      return Promise.resolve(CHAPTER_TEXT);
    }
    if (cmd === 'get_story_scenes' || cmd === 'get_story_scenes_paged') {
      return Promise.resolve([]);
    }
    if (cmd === 'get_story_word_count') {
      return Promise.resolve({ total_chars: CHAPTER_TEXT.length });
    }
    if (cmd === 'get_db_pool_status') {
      return Promise.resolve({
        max_size: 8,
        connections: 0,
        idle: 0,
        in_use: 0,
        connection_timeout_secs: 5,
      });
    }
    return Promise.resolve(undefined);
  }),
  recordFeedback: vi.fn(),
  smartExecute: mockSmartExecute,
  confirmAssetRefresh: mockConfirmAssetRefresh,
  getInputHint: vi.fn(),
  runRefine: vi.fn(),
  runReview: vi.fn(),
  runFinalize: vi.fn(),
  getPipelineActiveDraft: vi.fn(),
  classifyIntent: vi.fn().mockResolvedValue({
    is_new_novel: false,
    is_continuation: false,
    task_type: 'asset_refresh',
    is_prose_request: false,
    input_clarity: 'vague',
    detected_genre: null,
    confidence: 0.9,
  }),
  checkPreflight: vi.fn().mockResolvedValue({
    ready: true,
    missing_contracts: [],
    blocking_issues: [],
  }),
  generateLoglineHint: vi.fn().mockResolvedValue(null),
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
vi.mock('@/services/modelService', () => ({
  modelService: { checkModelStatus: vi.fn().mockResolvedValue(undefined) },
}));

describe('v0.53.5: 按正文重写大纲确认框', () => {
  const draftResult = {
    success: true,
    steps_completed: 1,
    final_content:
      '已按正文重写故事大纲、场景大纲。纸面未改。\n\n【故事大纲】\n韩雪在首尔雨夜对峙李明',
    messages: ['请确认大纲后再保存'],
    error: null,
    result_kind: 'asset_refresh',
    asset_refresh_draft: {
      story_id: 'story-1',
      scene_id: 'ch-1',
      overwrite_manual: false,
      instruction: '写后续的故事大纲，同时生成后续的场景大纲',
      story_outline: '韩雪在首尔雨夜对峙李明',
      scene_outline: '韩雪举枪，李明停在雨里。',
    },
  };

  beforeEach(() => {
    vi.clearAllMocks();
    for (const k of Object.keys(listenCallbacks)) delete listenCallbacks[k];
    captured.content = '';
    captured.generatedText = '';
    editorHtml.current = '';
    useFrontstageStore.getState().setContent('');
    useFrontstageStore.getState().setSceneInfo('', '', undefined);
    mockSmartExecute.mockResolvedValue(draftResult);
    mockConfirmAssetRefresh.mockResolvedValue('已按正文重写故事大纲、场景大纲。纸面未改。');
  });

  it('弹出可编辑对话框，不追加手稿、不生成幽灵文本', async () => {
    render(<FrontstageApp />, { wrapper });

    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'));

    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '写后续的故事大纲，同时生成后续的场景大纲');
    await userEvent.keyboard('{Enter}');

    await waitFor(() => expect(mockSmartExecute).toHaveBeenCalled());
    await screen.findByTestId('asset-refresh-confirm');
    expect(screen.getByText('确认大纲')).toBeTruthy();
    expect(screen.getByRole('button', { name: '确认' })).toBeTruthy();
    expect(screen.getByTestId('asset-refresh-cancel')).toBeTruthy();
    expect(screen.getByRole('button', { name: '重写' })).toBeTruthy();
    expect(
      (screen.getByTestId('asset-refresh-story-outline') as HTMLTextAreaElement).value
    ).toContain('韩雪');
    expect(
      (screen.getByTestId('asset-refresh-scene-outline') as HTMLTextAreaElement).value
    ).toContain('举枪');

    expect(captured.content).toContain('空气是粘稠的');
    expect(captured.content).not.toContain('已按正文重写故事大纲');
    expect(captured.content).not.toContain('韩雪在首尔雨夜对峙李明');
    expect(captured.generatedText).toBe('');
    expect(mockConfirmAssetRefresh).not.toHaveBeenCalled();
  });

  it('点确认后才保存用户改过的大纲', async () => {
    render(<FrontstageApp />, { wrapper });
    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'));
    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '写后续的故事大纲');
    await userEvent.keyboard('{Enter}');
    await screen.findByTestId('asset-refresh-confirm');
    const story = screen.getByTestId('asset-refresh-story-outline') as HTMLTextAreaElement;
    await userEvent.clear(story);
    await userEvent.type(story, '用户改过的后续大纲');
    await userEvent.click(screen.getByRole('button', { name: '确认' }));
    await waitFor(() => expect(mockConfirmAssetRefresh).toHaveBeenCalled());
    expect(mockConfirmAssetRefresh.mock.calls[0][0]).toMatchObject({
      storyId: 'story-1',
      storyOutline: '用户改过的后续大纲',
    });
    await waitFor(() => expect(screen.queryByTestId('asset-refresh-confirm')).toBeNull());
  });

  it('点取消废弃草稿，不保存', async () => {
    render(<FrontstageApp />, { wrapper });
    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'));
    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '写后续的故事大纲');
    await userEvent.keyboard('{Enter}');
    await screen.findByTestId('asset-refresh-confirm');
    await userEvent.click(screen.getByTestId('asset-refresh-cancel'));
    expect(mockConfirmAssetRefresh).not.toHaveBeenCalled();
    expect(screen.queryByTestId('asset-refresh-confirm')).toBeNull();
  });

  it('点重写再生成一轮，替换对话框内容且仍未保存', async () => {
    mockSmartExecute.mockResolvedValueOnce(draftResult).mockResolvedValueOnce({
      ...draftResult,
      asset_refresh_draft: {
        ...draftResult.asset_refresh_draft,
        story_outline: '重写后的故事大纲：李明先开口',
        scene_outline: '重写后的场景：雨巷里谁也不动。',
      },
    });
    render(<FrontstageApp />, { wrapper });
    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'));
    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '写后续的故事大纲');
    await userEvent.keyboard('{Enter}');
    await screen.findByTestId('asset-refresh-confirm');
    await userEvent.click(screen.getByRole('button', { name: '重写' }));
    await waitFor(() =>
      expect(
        (screen.getByTestId('asset-refresh-story-outline') as HTMLTextAreaElement).value
      ).toContain('李明先开口')
    );
    expect(mockSmartExecute).toHaveBeenCalledTimes(2);
    expect(mockConfirmAssetRefresh).not.toHaveBeenCalled();
    expect(screen.getByTestId('asset-refresh-confirm')).toBeTruthy();
  });
});
