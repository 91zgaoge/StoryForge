import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { createRef } from 'react';
import {
  AiSelectionActions,
  shouldOfferSelectionActions,
  type AiSelectionActionsProps,
} from '../AiSelectionActions';

// jsdom 无真实选区矩形：stub getSelection 与 rAF，让 place() 能算出锚点
const rect = {
  left: 100,
  top: 100,
  right: 200,
  bottom: 120,
  width: 100,
  height: 20,
  x: 100,
  y: 100,
  toJSON: () => ({}),
};

beforeEach(() => {
  // jsdom 无 ResizeObserver / Web Animations API：补最小 stub
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
  vi.stubGlobal('getSelection', () => ({
    rangeCount: 1,
    getRangeAt: () => ({
      getBoundingClientRect: () => rect,
      getClientRects: () => [rect],
    }),
  }));
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => setTimeout(cb, 0));
  vi.stubGlobal('cancelAnimationFrame', (id: number) => clearTimeout(id));
});

async function flushPlace() {
  await act(async () => {
    await new Promise(r => setTimeout(r, 0));
  });
}

function renderBar(props: Partial<AiSelectionActionsProps> = {}) {
  const containerRef = createRef<HTMLElement>();
  const utils = render(
    <div ref={containerRef as React.RefObject<HTMLDivElement>}>
      <AiSelectionActions
        containerRef={containerRef as React.RefObject<HTMLElement>}
        selectedText="被选中的文字"
        phase="idle"
        onRun={() => {}}
        onAccept={() => {}}
        onDiscard={() => {}}
        {...props}
      />
    </div>
  );
  return utils;
}

describe('AiSelectionActions', () => {
  async function openCustom() {
    fireEvent.click(screen.getByRole('button', { name: '自定义指令' }));
    await flushPlace();
  }

  it('selectedText 为空字符串时不渲染', () => {
    const { container } = renderBar({ selectedText: '' });
    expect(container.querySelector('[data-testid="ai-selection-actions"]')).toBeNull();
  });

  it('短于 4 字不渲染，避免点选误拖挡住正文', () => {
    const { container } = renderBar({ selectedText: '被' });
    expect(container.querySelector('[data-testid="ai-selection-actions"]')).toBeNull();
  });

  it('idle 默认不渲染自定义输入框，只给动作钮', async () => {
    renderBar();
    await flushPlace();
    expect(screen.queryByLabelText('描述修改要求')).toBeNull();
    expect(screen.getByRole('button', { name: /润色/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '自定义指令' })).toBeInTheDocument();
  });

  it('shouldOfferSelectionActions 以 4 字为下限', () => {
    expect(shouldOfferSelectionActions('')).toBe(false);
    expect(shouldOfferSelectionActions('被')).toBe(false);
    expect(shouldOfferSelectionActions('被选文')).toBe(false);
    expect(shouldOfferSelectionActions('被选文字')).toBe(true);
  });

  it('idle 渲染润色/扩写动作，点击调用 onRun(action)', async () => {
    const onRun = vi.fn();
    renderBar({ onRun });
    await flushPlace();
    fireEvent.click(screen.getByRole('button', { name: /润色/ }));
    expect(onRun).toHaveBeenCalledWith('polish', undefined);
  });

  it('展开 chevron 后出现改写动作', async () => {
    const onRun = vi.fn();
    renderBar({ onRun });
    await flushPlace();
    fireEvent.click(screen.getByRole('button', { name: '展开更多操作' }));
    fireEvent.click(screen.getByRole('button', { name: /改写/ }));
    expect(onRun).toHaveBeenCalledWith('rewrite', undefined);
  });

  it('自定义指令输入后回车调用 onRun(custom, 文本)', async () => {
    const onRun = vi.fn();
    renderBar({ onRun });
    await flushPlace();
    await openCustom();
    const input = screen.getByLabelText('描述修改要求');
    fireEvent.change(input, { target: { value: '改成古文腔' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onRun).toHaveBeenCalledWith('custom', '改成古文腔');
  });

  it('idle 自定义指令发送钮用 accent tint 而非 charcoal bg-ai-ink', async () => {
    renderBar();
    await flushPlace();
    await openCustom();
    fireEvent.change(screen.getByLabelText('描述修改要求'), { target: { value: '改成古文腔' } });
    const send = screen.getByRole('button', { name: /发送修改指令/ });
    expect(send.className).not.toMatch(/bg-ai-ink/);
    expect(send.getAttribute('style') ?? send.className).toMatch(/ai-accent/);
  });

  it('thinking 阶段显示 shimmer 忙碌标签', () => {
    renderBar({ phase: 'thinking' });
    const busy = screen.getByTestId('ai-selection-busy');
    expect(busy.className).toContain('animate-shimmer-text');
  });

  it('result 阶段渲染结果分词与 保留/放弃/重试，回调正确', () => {
    const onAccept = vi.fn();
    const onDiscard = vi.fn();
    renderBar({ phase: 'result', resultText: '改写后的文字', onAccept, onDiscard });
    expect(screen.getByTestId('ai-selection-stream')).toBeInTheDocument();
    const keep = screen.getByRole('button', { name: /保留/ });
    expect(keep.className).not.toMatch(/bg-ai-ink/);
    expect(keep.className).toMatch(/ai-accent/);
    fireEvent.click(keep);
    expect(onAccept).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: /放弃/ }));
    expect(onDiscard).toHaveBeenCalled();
  });

  it('浮条 mousedown 被 preventDefault（防选区塌陷）', () => {
    renderBar();
    const bar = screen.getByTestId('ai-selection-actions');
    const event = new MouseEvent('mousedown', { bubbles: true, cancelable: true });
    bar.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });

  it('mousedown 落在自定义指令 input 上时手动恢复焦点（C1：preventDefault 不阻断聚焦）', async () => {
    renderBar();
    await flushPlace();
    fireEvent.click(screen.getByRole('button', { name: '自定义指令' }));
    await flushPlace();
    const input = screen.getByLabelText('描述修改要求');
    const event = new MouseEvent('mousedown', { bubbles: true, cancelable: true });
    input.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true); // 防选区塌陷仍生效
    expect(input).toHaveFocus(); // 但焦点被手动恢复
  });

  it('IME 组词中按 Enter 不提交自定义指令（I3：isComposing 守卫）', async () => {
    const onRun = vi.fn();
    renderBar({ onRun });
    await flushPlace();
    fireEvent.click(screen.getByRole('button', { name: '自定义指令' }));
    await flushPlace();
    const input = screen.getByLabelText('描述修改要求');
    fireEvent.change(input, { target: { value: '改成古文腔' } });
    fireEvent.keyDown(input, { key: 'Enter', isComposing: true });
    expect(onRun).not.toHaveBeenCalled();
  });

  it('自定义动作后点重试携带上次自定义指令（M2）', async () => {
    const onRun = vi.fn();
    const containerRef = createRef<HTMLElement>();
    const renderWith = (phase: 'idle' | 'result', resultText?: string) => (
      <div ref={containerRef as React.RefObject<HTMLDivElement>}>
        <AiSelectionActions
          containerRef={containerRef as React.RefObject<HTMLElement>}
          selectedText="被选中的文字"
          phase={phase}
          resultText={resultText}
          onRun={onRun}
          onAccept={() => {}}
          onDiscard={() => {}}
        />
      </div>
    );
    const { rerender } = render(renderWith('idle'));
    await flushPlace();
    fireEvent.click(screen.getByRole('button', { name: '自定义指令' }));
    await flushPlace();
    const input = screen.getByLabelText('描述修改要求');
    fireEvent.change(input, { target: { value: '改成古文腔' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onRun).toHaveBeenCalledWith('custom', '改成古文腔');
    rerender(renderWith('result', '改写后的文字'));
    fireEvent.click(screen.getByRole('button', { name: '重试' }));
    expect(onRun).toHaveBeenLastCalledWith('custom', '改成古文腔');
  });
});
