import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { smartExecute } from '@/services/tauri';

// 划词浮条宿主逻辑（P2 Task5 评审整改 I2）：
// 选区变化/塌陷时重置 result/thinking 状态 + accept 前校验选文快照
let fakeSelection = { from: 0, to: 0, empty: true };
let fakeText = '';
let selectionUpdateHandler: (() => void) | null = null;

function createFakeEditor() {
  const chainable = {
    focus: () => chainable,
    insertContent: () => chainable,
    insertContentAt: () => chainable,
    setTextSelection: () => chainable,
    run: () => true,
  };
  return {
    getHTML: () => '<p>initial</p>',
    getText: () => 'initial',
    isFocused: false,
    isEmpty: false,
    commands: {
      setContent: vi.fn(),
      insertContent: vi.fn(),
      insertContentAt: vi.fn(),
    },
    chain: () => chainable,
    on: (event: string, cb: () => void) => {
      if (event === 'selectionUpdate') selectionUpdateHandler = cb;
    },
    off: vi.fn(),
    state: {
      get selection() {
        return fakeSelection;
      },
      doc: {
        content: { size: 1000 },
        textBetween: (_from: number, _to: number, _sep?: string) => fakeText,
      },
    },
  };
}

let fakeEditor = createFakeEditor();

vi.mock('@tiptap/react', () => ({
  useEditor: () => fakeEditor,
  EditorContent: function MockEditorContent() {
    return <div data-testid="editor-content" />;
  },
}));

vi.mock('@tiptap/starter-kit', () => ({
  default: { configure: () => ({ name: 'starter-kit' }) },
}));
vi.mock('@tiptap/extension-placeholder', () => ({
  default: { configure: () => ({ name: 'placeholder' }) },
}));
vi.mock('@tiptap/extension-underline', () => ({
  default: { configure: () => ({ name: 'underline' }) },
}));
vi.mock('@tiptap/extension-highlight', () => ({
  default: { configure: () => ({ name: 'highlight' }) },
}));

vi.mock('../tiptap/AiSuggestionNode', () => ({ AiSuggestionNode: {} }));
vi.mock('@/frontstage/extensions/SceneDividerNode', () => ({ SceneDividerNode: {} }));

vi.mock('@/utils/cn', () => ({
  cn: (...classes: (string | false | undefined)[]) => classes.filter(Boolean).join(' '),
}));
vi.mock('@/stores/appStore', () => ({
  useAppStore: (selector: (state: { editorConfig: unknown }) => unknown) =>
    selector({ editorConfig: null }),
}));
vi.mock('@/services/tauri', () => ({
  getCharacterByName: vi.fn(),
  smartExecute: vi.fn(),
  formatText: vi.fn(),
}));
vi.mock('./CharacterCardPopup', () => ({ CharacterCardPopup: () => null }));
vi.mock('./CharacterPeekCard', () => ({ CharacterPeekCard: () => null }));
vi.mock('./EditorContextMenu', () => ({ EditorContextMenu: () => null }));
vi.mock('@/frontstage/config/writingStyles', () => ({ defaultStyle: {} }));
vi.mock('@/frontstage/config/colorThemes', () => ({ getCurrentEditorColors: () => ({}) }));
vi.mock('@/hooks/useSubscription', () => ({ useSubscription: () => ({ isPro: false }) }));
vi.mock('@/utils/logger', () => ({ createLogger: () => ({ error: vi.fn() }) }));
vi.mock('lucide-react', () => ({
  Sparkles: () => null,
  X: () => null,
  Check: () => null,
  Type: () => null,
  Scissors: () => null,
  ArrowUp: () => null,
  ChevronRight: () => null,
  RefreshCw: () => null,
}));

// 必须在 mock 之后动态导入被测组件，确保 mock 生效
let RichTextEditor: typeof import('../RichTextEditor').default;

const smartExecuteMock = vi.mocked(smartExecute);

function makeResult(finalContent: string) {
  return { success: true, steps_completed: 1, messages: [], final_content: finalContent };
}

function fireSelection(from: number, to: number, text: string) {
  fakeSelection = { from, to, empty: from === to };
  fakeText = text;
  act(() => {
    selectionUpdateHandler?.();
  });
}

describe('RichTextEditor 划词浮条状态重置（P2 Task5 I2）', () => {
  beforeEach(async () => {
    // AiSelectionActions 在 jsdom 下需要的最小环境 stub
    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe() {}
        unobserve() {}
        disconnect() {}
      }
    );
    if (!Element.prototype.animate) {
      Element.prototype.animate = (() => ({
        cancel: () => {},
        onfinish: null,
        playState: 'finished',
      })) as unknown as typeof Element.prototype.animate;
    }
    fakeSelection = { from: 0, to: 0, empty: true };
    fakeText = '';
    selectionUpdateHandler = null;
    fakeEditor = createFakeEditor();
    const mod = await import('../RichTextEditor');
    RichTextEditor = mod.default;
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.unstubAllGlobals();
  });

  it('选区改变后旧 result 不复活；在飞 smartExecute 迟到结果被丢弃', async () => {
    let resolveExecute: ((value: ReturnType<typeof makeResult>) => void) | null = null;
    smartExecuteMock.mockImplementation(() => new Promise(resolve => (resolveExecute = resolve)));
    render(<RichTextEditor content="<p>initial</p>" onChange={() => {}} />);

    // 划词 → 浮条出现 → 点「润色」进入 thinking
    fireSelection(1, 5, '被选文字');
    fireEvent.click(screen.getByRole('button', { name: /润色/ }));
    expect(screen.getByTestId('ai-selection-busy')).toBeInTheDocument();

    // smartExecute 未返回时用户改选另一段文字 → 状态应重置回 idle
    fireSelection(10, 14, '另一段文字');
    expect(screen.queryByTestId('ai-selection-busy')).toBeNull();
    expect(screen.getByRole('button', { name: /润色/ })).toBeInTheDocument();

    // 迟到的 smartExecute 结果不得让旧 result 复活
    await act(async () => {
      resolveExecute?.(makeResult('改写后的文字'));
    });
    expect(screen.queryByTestId('ai-selection-stream')).toBeNull();
    expect(screen.getByRole('button', { name: /润色/ })).toBeInTheDocument();
  });

  it('保留前校验选文快照：文档已变化时放弃替换并提示', async () => {
    const onShowStatus = vi.fn();
    smartExecuteMock.mockResolvedValue(makeResult('改写后的文字'));
    render(
      <RichTextEditor content="<p>initial</p>" onChange={() => {}} onShowStatus={onShowStatus} />
    );

    fireSelection(1, 5, '被选文字');
    fireEvent.click(screen.getByRole('button', { name: /润色/ }));
    await act(async () => {});
    expect(screen.getByTestId('ai-selection-stream')).toBeInTheDocument();

    // 结果就绪后文档对应范围内容被改动（快照不符）
    fakeText = '被用户改过的文字';
    fireEvent.click(screen.getByRole('button', { name: /保留/ }));

    expect(fakeEditor.commands.insertContentAt).not.toHaveBeenCalled();
    expect(onShowStatus).toHaveBeenCalledWith(expect.stringContaining('放弃替换'));
    // 状态回到 idle
    expect(screen.getByRole('button', { name: /润色/ })).toBeInTheDocument();
  });

  it('快照相符时保留正常替换选区', async () => {
    const onShowStatus = vi.fn();
    smartExecuteMock.mockResolvedValue(makeResult('改写后的文字'));
    render(
      <RichTextEditor content="<p>initial</p>" onChange={() => {}} onShowStatus={onShowStatus} />
    );

    fireSelection(1, 5, '被选文字');
    fireEvent.click(screen.getByRole('button', { name: /润色/ }));
    await act(async () => {});
    fireEvent.click(screen.getByRole('button', { name: /保留/ }));

    expect(fakeEditor.commands.insertContentAt).toHaveBeenCalledWith(
      { from: 1, to: 5 },
      '改写后的文字'
    );
    expect(onShowStatus).toHaveBeenCalledWith('已替换为改写内容');
  });
});
