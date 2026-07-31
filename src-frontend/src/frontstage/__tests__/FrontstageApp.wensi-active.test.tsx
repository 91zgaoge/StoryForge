import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, act } from '@testing-library/react';
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

const CONTINUATION_TEXT =
  '凯尔看着那颗悬浮的光球，脑海中闪过一丝不祥的预感。他伸出手，指尖微微颤抖，触碰到了那层薄薄的光膜。';

const { listenCallbacks, captured, mockSmartExecute, editorHtml } = vi.hoisted(() => ({
  listenCallbacks: {} as Record<string, (e: { payload: unknown }) => void>,
  captured: { content: '', generatedText: '' },
  mockSmartExecute: vi.fn(),
  // 模拟 TipTap 编辑器的实时内部 HTML（getHTML 返回此值，而非 stale props.content）。
  // appendText/setContent 更新它；外部 props.content 变更时同步它。
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
    // 外部 props.content 变更（章节切换/setContent）时同步到编辑器内部 HTML。
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

describe('Bug: 文思活跃模式续写内容丢失（smartExecuteInFlightRef 提前清除 + 打字机误用于 active 模式）', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    for (const k of Object.keys(listenCallbacks)) delete listenCallbacks[k];
    captured.content = '';
    captured.generatedText = '';
    editorHtml.current = '';
    useFrontstageStore.getState().setContent('');
    useFrontstageStore.getState().setSceneInfo('', '', undefined);
    // smartExecute: 第一次返回续写内容，后续返回空（停止 auto-continue 循环）
    let callCount = 0;
    mockSmartExecute.mockImplementation(() => {
      callCount++;
      if (callCount === 1) {
        return Promise.resolve({
          success: true,
          steps_completed: 1,
          final_content: CONTINUATION_TEXT,
          messages: [],
          error: null,
        });
      }
      // 后续调用返回空内容，让 handleRequestGeneration 走 displayText 空 bail 路径
      return Promise.resolve({
        success: true,
        steps_completed: 0,
        final_content: '',
        messages: [],
        error: null,
      });
    });
  });

  it('文思活跃模式续写：内容直接追加到编辑器正文，不走打字机幽灵文本', async () => {
    render(<FrontstageApp />, { wrapper });

    // 等待章节加载完成（编辑器有已有正文）
    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'), { timeout: 3000 });

    // 切换到文思活跃模式（默认 passive，点击一次 -> active）
    const wensiButton = screen.getByRole('button', { name: /文思/ });
    await act(async () => {
      await userEvent.click(wensiButton);
    });

    // 验证模式已切换到 active
    const wensiButtonAfter = screen.getByRole('button', { name: /文思/ });
    expect(wensiButtonAfter.getAttribute('aria-label')).toContain('文思活跃');

    // 输入续写指令并提交
    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '续写');
    await userEvent.keyboard('{Enter}');

    // 等待 smartExecute 被调用
    await waitFor(() => expect(mockSmartExecute).toHaveBeenCalled(), { timeout: 3000 });

    // 等待内容处理完成
    await waitFor(() => expect(captured.content).toContain('凯尔看着那颗悬浮的光球'), {
      timeout: 5000,
    });

    // 关键断言 1：续写内容已追加到编辑器正文（通过 appendAiContent）
    expect(captured.content.replace(/<[^>]+>/g, '')).toContain('凯尔看着那颗悬浮的光球');

    // 关键断言 2：续写内容不应出现在 generatedText（幽灵文本）中
    // active 模式应直接追加到编辑器，不走打字机 -> setGeneratedText
    expect(captured.generatedText).not.toContain('凯尔看着那颗悬浮的光球');
  });

  it('文思活跃模式续写：smartExecuteNeedDiagnosticRef 被清除，不触发"生成过程异常结束"诊断', async () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(<FrontstageApp />, { wrapper });

    // 等待章节加载完成
    await waitFor(() => expect(captured.content).toContain('空气是粘稠的'), { timeout: 3000 });

    // 切换到文思活跃模式
    const wensiButton = screen.getByRole('button', { name: /文思/ });
    await act(async () => {
      await userEvent.click(wensiButton);
    });

    // 输入续写指令并提交
    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await userEvent.type(input, '续写');
    await userEvent.keyboard('{Enter}');

    await waitFor(() => expect(mockSmartExecute).toHaveBeenCalled(), { timeout: 3000 });

    // 等待内容处理完成
    await waitFor(() => expect(captured.content).toContain('凯尔看着那颗悬浮的光球'), {
      timeout: 5000,
    });

    // 等待额外时间确保 safety-net effect 有机会触发（如果有 bug 的话）
    await new Promise(r => setTimeout(r, 500));

    // 关键断言：不应出现"生成过程异常结束，未收到有效内容"诊断
    // 如果 smartExecuteNeedDiagnosticRef 未被清除，safety-net effect 会在
    // isGenerating 变 false 时触发 captureDiagnosticInfo
    const diagnosticErrors = consoleErrorSpy.mock.calls.filter(
      call => typeof call[0] === 'string' && call[0].includes('生成过程异常结束')
    );
    expect(diagnosticErrors).toHaveLength(0);

    consoleErrorSpy.mockRestore();
  });
});
