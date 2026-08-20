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

const { listenCallbacks, captured, mockSmartExecute, editorHtml } = vi.hoisted(() => ({
  listenCallbacks: {} as Record<string, (e: { payload: unknown }) => void>,
  captured: { content: '', generatedText: '' },
  mockSmartExecute: vi.fn(),
  editorHtml: { current: '' as string },
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, cb: (e: { payload: unknown }) => void) => {
    listenCallbacks[event] = cb;
    return Promise.resolve(() => {});
  }),
  emit: vi.fn(),
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
          content: null,
          scene_id: 'scene-1',
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
        scene_id: 'scene-1',
      });
    }
    if (cmd === 'get_chapter_aggregated_content') {
      return Promise.resolve(CHAPTER_TEXT);
    }
    if (cmd === 'get_story_scenes' || cmd === 'get_story_scenes_paged') {
      return Promise.resolve([
        {
          id: 'scene-1',
          story_id: 'story-1',
          sequence_number: 1,
          title: '第一章',
          content: CHAPTER_TEXT,
          chapter_id: 'ch-1',
        },
      ]);
    }
    if (cmd === 'get_story_word_count') {
      return Promise.resolve({ total_chars: CHAPTER_TEXT.length });
    }
    return Promise.resolve(undefined);
  }),
  recordFeedback: vi.fn(),
  smartExecute: mockSmartExecute,
  getInputHint: vi.fn(),
  runRefine: vi.fn(),
  runReview: vi.fn(),
  runFinalize: vi.fn(),
  getPipelineActiveDraft: vi.fn(),
  classifyIntent: vi.fn().mockResolvedValue({
    is_new_novel: false,
    is_continuation: true,
    task_type: 'continuation',
    is_prose_request: true,
    input_clarity: 'concise',
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
vi.mock('@/utils/errorHandler', async importOriginal => {
  const actual = await importOriginal<typeof import('@/utils/errorHandler')>();
  return {
    ...actual,
    parseStructuredError: vi.fn((e: unknown) => e),
  };
});

describe('进行中的续写不得弹前台中断卡', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    for (const k of Object.keys(listenCallbacks)) delete listenCallbacks[k];
    captured.content = '';
    captured.generatedText = '';
    editorHtml.current = '';
    useFrontstageStore.getState().setContent('');
    useFrontstageStore.getState().setSceneInfo('', '', undefined);
    mockSmartExecute.mockRejectedValue({
      code: 'VALIDATION_FAILED',
      message: '该故事已有进行中的创作任务',
      severity: 'UserAction',
      data: { field: 'active_run' },
    });
  });

  it('smart_execute 撞上已有 run 时不渲染需要您先处理 / 前往设置', async () => {
    render(<FrontstageApp />, { wrapper });

    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'), { timeout: 3000 });

    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '续写');
    await userEvent.keyboard('{Enter}');

    await waitFor(() => expect(mockSmartExecute).toHaveBeenCalled(), { timeout: 3000 });
    await new Promise(r => setTimeout(r, 200));

    expect(screen.queryByText('需要您先处理')).not.toBeInTheDocument();
    expect(screen.queryByText('前往设置')).not.toBeInTheDocument();
    expect(screen.queryByText('正在续写中')).not.toBeInTheDocument();
  });
});
