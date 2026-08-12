import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { createRef } from 'react';
import { AiSelectionActions, type AiSelectionActionsProps } from '../AiSelectionActions';

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
  it('selectedText 为空字符串时不渲染', () => {
    const { container } = renderBar({ selectedText: '' });
    expect(container.querySelector('[data-testid="ai-selection-actions"]')).toBeNull();
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
    const input = screen.getByLabelText('描述修改要求');
    fireEvent.change(input, { target: { value: '改成古文腔' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onRun).toHaveBeenCalledWith('custom', '改成古文腔');
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
    fireEvent.click(screen.getByRole('button', { name: /保留/ }));
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
});
