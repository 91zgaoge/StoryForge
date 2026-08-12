import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import { AiLoading } from '../AiLoading';

describe('AiLoading', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('渲染 label、9 格点与计时器', () => {
    const { container } = render(<AiLoading label="正在生成世界观" />);
    expect(screen.getByText('正在生成世界观')).toBeInTheDocument();
    expect(container.querySelectorAll('[aria-hidden] > span')).toHaveLength(9);
    expect(screen.getByTestId('ai-loading-elapsed').textContent).toMatch(/s$/);
  });

  it('elapsed 从 startedAt 起算', () => {
    vi.useFakeTimers();
    vi.setSystemTime(1_000_000);
    render(<AiLoading label="x" startedAt={995_000} />);
    expect(screen.getByTestId('ai-loading-elapsed').textContent).toBe('5.0s');
  });

  it('计时随时间推进（100ms 粒度）', () => {
    vi.useFakeTimers();
    render(<AiLoading label="x" />);
    act(() => {
      vi.advanceTimersByTime(2100);
    });
    expect(screen.getByTestId('ai-loading-elapsed').textContent).toBe('2.1s');
  });

  it('超过 60s 显示 m+s；orbit 变体中心格不点亮', () => {
    vi.useFakeTimers();
    vi.setSystemTime(200_000);
    const { container } = render(<AiLoading label="x" variant="orbit" startedAt={125_000} />);
    expect(screen.getByTestId('ai-loading-elapsed').textContent).toBe('1m 15.0s');
    // orbit 模式中心格（index 4）无动画
    const cells = container.querySelectorAll('[aria-hidden] > span');
    expect(cells[4].className).not.toContain('animate-pixel-on');
    expect(cells[0].className).toContain('animate-pixel-on');
  });
});
