import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { AiCodeBlock } from '../AiCodeBlock';

const writeText = vi.fn().mockResolvedValue(undefined);

beforeEach(() => {
  vi.useFakeTimers();
  Object.assign(navigator, { clipboard: { writeText } });
  writeText.mockClear();
});

afterEach(() => {
  vi.useRealTimers();
});

const CODE = '{\n  "a": 1,\n  "b": 2\n}';

describe('AiCodeBlock', () => {
  it('渲染代码全部行', () => {
    render(<AiCodeBlock code={CODE} />);
    expect(screen.getByTestId('ai-code-block')).toHaveTextContent('"a": 1');
    expect(screen.getByTestId('ai-code-block')).toHaveTextContent('"b": 2');
  });

  it('渲染 title 与 language', () => {
    render(<AiCodeBlock code={CODE} title="结果" language="JSON" />);
    expect(screen.getByText('结果')).toBeInTheDocument();
    expect(screen.getByText('JSON')).toBeInTheDocument();
  });

  it('lineNumbers 时渲染行号 1-4', () => {
    render(<AiCodeBlock code={CODE} lineNumbers />);
    const block = screen.getByTestId('ai-code-block');
    for (const n of ['1', '2', '3', '4']) {
      expect(block.querySelector(`[data-line-no="${n}"]`)).toBeTruthy();
    }
  });

  it('点击复制写剪贴板并翻转已复制，1500ms 后恢复', async () => {
    render(<AiCodeBlock code={CODE} />);
    await act(async () => {
      fireEvent.click(screen.getByRole('button', { name: '复制' }));
    });
    expect(writeText).toHaveBeenCalledWith(CODE);
    expect(screen.getByRole('button', { name: '已复制' })).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(1600);
    });
    expect(screen.getByRole('button', { name: '复制' })).toBeInTheDocument();
  });

  it('copyable=false 时不渲染复制按钮', () => {
    render(<AiCodeBlock code={CODE} copyable={false} />);
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('maxHeight 应用到 pre 样式', () => {
    render(<AiCodeBlock code={CODE} maxHeight={192} />);
    const pre = screen.getByTestId('ai-code-block').querySelector('pre')!;
    expect(pre.style.maxHeight).toBe('192px');
  });
});
