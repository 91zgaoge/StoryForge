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

const AUDIT_REPORT = '总体评分 0.85\n\n具体问题：\n1. 第二段动机铺垫不足\n2. 对话节奏偏快';

const { listenCallbacks, captured, editorHtml, mockSmartExecute } = vi.hoisted(() => ({
  listenCallbacks: {} as Record<string, (e: { payload: unknown }) => void>,
  captured: { content: '', generatedText: '' },
  editorHtml: { current: '' },
  mockSmartExecute: vi.fn(),
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
    is_continuation: false,
    task_type: 'audit',
    is_prose_request: false,
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
vi.mock('@/utils/errorHandler', () => ({
  parseStructuredError: vi.fn((e: unknown) => e),
}));
vi.mock('@/services/modelService', () => ({
  modelService: { checkModelStatus: vi.fn().mockResolvedValue(undefined) },
}));

describe('v0.31.x: 智能输入审计意图自动路由（result_kind=audit_report）', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    for (const k of Object.keys(listenCallbacks)) delete listenCallbacks[k];
    captured.content = '';
    captured.generatedText = '';
    editorHtml.current = '';
    useFrontstageStore.getState().setContent('');
    useFrontstageStore.getState().setSceneInfo('', '', undefined);
    mockSmartExecute.mockResolvedValue({
      success: true,
      steps_completed: 1,
      final_content: AUDIT_REPORT,
      messages: ['审计完成'],
      error: null,
      result_kind: 'audit_report',
    });
  });

  it('审计报告以弹窗展示，不追加手稿、不生成幽灵文本', async () => {
    render(<FrontstageApp />, { wrapper });

    // 等待章节加载完成
    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'));

    // 在智能输入框提交审计指令
    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '审计这一幕');
    await userEvent.keyboard('{Enter}');

    // smartExecute 被调用
    await waitFor(() => expect(mockSmartExecute).toHaveBeenCalled());

    // 报告弹窗出现，展示报告全文
    await screen.findByText('审计报告');
    await screen.findByText(/总体评分 0\.85/);

    // 报告未进入编辑器正文，也未生成幽灵文本
    expect(captured.content).not.toContain('总体评分');
    expect(captured.generatedText).toBe('');
  });

  it('关闭弹窗后报告被清除', async () => {
    render(<FrontstageApp />, { wrapper });

    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'));

    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '检查一下当前章节的问题');
    await userEvent.keyboard('{Enter}');

    await screen.findByText('审计报告');

    await userEvent.click(screen.getByText('关闭'));

    await waitFor(() => expect(screen.queryByText('审计报告')).toBeNull());
  });
});
