import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import FrontstageApp from '../FrontstageApp';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

// 捕获 listen 回调，以便测试中手动触发 ChapterSwitch 事件
const { listenCallbacks, mockSmartExecute, GENESIS_CMD, CHAPTER_TEXT, NEW_STORY_ID } = vi.hoisted(
  () => ({
    listenCallbacks: {} as Record<string, (e: { payload: unknown }) => void>,
    mockSmartExecute: vi.fn(),
    GENESIS_CMD: '写一部关于星际文明的科幻长篇小说',
    CHAPTER_TEXT:
      '星舰的引擎发出低沉的嗡鸣，像一头沉睡的巨兽。\n\n舱外的星光照亮了指挥台上的全息投影。',
    NEW_STORY_ID: 'genesis-story-1',
  })
);

const FRONTSTAGE_EVENT = 'frontstage-update';

// 首次创世：初始无故事，smartExecute 调用后才有新故事
let genesisStoryCreated = false;

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, cb: (e: { payload: unknown }) => void) => {
    listenCallbacks[event] = cb;
    return Promise.resolve(() => {});
  }),
  emit: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve()),
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
    // 首次创世：初始无故事 -> 返回 []；创世完成后返回新故事
    if (cmd === 'list_stories') {
      return Promise.resolve(genesisStoryCreated ? [{ id: NEW_STORY_ID, title: '星际文明' }] : []);
    }
    if (cmd === 'get_story_chapters') {
      return Promise.resolve([
        {
          id: 'ch-1',
          story_id: NEW_STORY_ID,
          chapter_number: 1,
          title: '第一章',
          content: CHAPTER_TEXT,
        },
      ]);
    }
    if (cmd === 'get_story_chapters' || cmd === 'get_story_chapters_paged') {
      return Promise.resolve(
        genesisStoryCreated
          ? [
              {
                id: 'ch-1',
                story_id: NEW_STORY_ID,
                chapter_number: 1,
                title: '第一章',
                content: CHAPTER_TEXT,
              },
            ]
          : []
      );
    }
    if (cmd === 'get_chapter') {
      return Promise.resolve({
        id: 'ch-1',
        story_id: NEW_STORY_ID,
        chapter_number: 1,
        title: '第一章',
        content: CHAPTER_TEXT,
      });
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
  // classifyIntent 不导出 -> undefined -> 调用时抛错 -> catch 兜底（无故事=创世）
  getInputHint: vi.fn(),
  generateLoglineHint: vi.fn(),
  runRefine: vi.fn(),
  runReview: vi.fn(),
  runFinalize: vi.fn(),
  getPipelineActiveDraft: vi.fn(),
}));

// 带 forwardRef 的编辑器 mock（支撑 appendAiContent / getText / setContent）
vi.mock('../components/RichTextEditor', () => ({
  __esModule: true,
  default: React.forwardRef(function MockRichTextEditor(
    props: { content: string; onChange?: (c: string) => void; generatedText?: string },
    ref: React.ForwardedRef<{
      getText: () => string;
      appendText: (html: string) => void;
      setContent: (html: string) => void;
      getHTML: () => string;
    }>
  ) {
    React.useImperativeHandle(ref, () => ({
      getText: () => props.content.replace(/<[^>]+>/g, ''),
      appendText: (html: string) => props.onChange?.((props.content || '') + html),
      setContent: (html: string) => props.onChange?.(html),
      getHTML: () => props.content,
    }));
    return React.createElement('div', { 'data-testid': 'rich-text-editor' }, props.content);
  }),
}));

vi.mock('../components/IngestHealthIndicator', () => ({ IngestHealthIndicator: () => null }));
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
vi.mock('react-hot-toast', () => ({
  default: { success: vi.fn(), error: vi.fn(), info: vi.fn(), warning: vi.fn(), loading: vi.fn() },
}));
vi.mock('@/utils/errorHandler', () => ({ parseStructuredError: vi.fn(() => null) }));

import { useGenerationStore } from '@/stores/generationStore';

describe('首次创世指令应保存到新故事输入历史', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    for (const k of Object.keys(listenCallbacks)) delete listenCallbacks[k];
    genesisStoryCreated = false;
    useGenerationStore.setState({ isGenerating: false });
    localStorage.removeItem(`frontstage:inputHistory:${NEW_STORY_ID}`);

    mockSmartExecute.mockImplementation(async () => {
      // smartExecute 返回后，list_stories 才返回新故事
      genesisStoryCreated = true;
      return {
        success: true,
        steps_completed: 1,
        final_content: CHAPTER_TEXT,
        messages: [
          `story_created:${NEW_STORY_ID}`,
          'session_id:ses-1',
          'novel_bootstrap_first_chapter_ready',
        ],
        error: null,
      };
    });
  });

  it('创世成功后，创世指令应持久化到新故事的 localStorage 历史', async () => {
    render(<FrontstageApp />, { wrapper });

    // 初始无故事，输入栏可用
    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await waitFor(() => expect(input).not.toBeDisabled());

    await userEvent.type(input, GENESIS_CMD);
    await userEvent.keyboard('{Enter}');

    // smartExecute 应被调用（创世指令已提交）
    await waitFor(() => expect(mockSmartExecute).toHaveBeenCalled());

    // 等待 story_created 块完成（新故事加载、setCurrentStory）
    await waitFor(
      () => {
        const raw = localStorage.getItem(`frontstage:inputHistory:${NEW_STORY_ID}`);
        expect(raw).toBeTruthy();
        expect(JSON.parse(raw!)).toContain(GENESIS_CMD);
      },
      { timeout: 5000 }
    );
  });

  it('创世成功后切换到新故事，按 ↑ 应召回创世指令', async () => {
    render(<FrontstageApp />, { wrapper });

    const input = screen.getByPlaceholderText('输入任意指令…') as HTMLTextAreaElement;
    await waitFor(() => expect(input).not.toBeDisabled());

    await userEvent.type(input, GENESIS_CMD);
    await userEvent.keyboard('{Enter}');

    await waitFor(() => expect(mockSmartExecute).toHaveBeenCalled());

    // 等待创世指令落库
    await waitFor(
      () => {
        const raw = localStorage.getItem(`frontstage:inputHistory:${NEW_STORY_ID}`);
        expect(raw).toBeTruthy();
      },
      { timeout: 5000 }
    );

    // 模拟 ChapterSwitch 事件（后端创世成功后会发射）
    await act(async () => {
      listenCallbacks[FRONTSTAGE_EVENT]?.({
        payload: {
          type: 'chapterSwitch',
          payload: {
            story_id: NEW_STORY_ID,
            chapter_id: 'ch-1',
            scene_id: null,
            title: '第一章',
            content: null,
            auto_accept: true,
          },
        },
      });
    });

    // 等待新故事选中 + 历史加载
    await new Promise(r => setTimeout(r, 300));

    // 清空输入框后按 ↑，应召回创世指令
    await userEvent.clear(input);
    input.focus();
    await userEvent.keyboard('{ArrowUp}');

    await waitFor(() => {
      expect(screen.getByText(new RegExp(GENESIS_CMD))).toBeInTheDocument();
    });
  });
});
