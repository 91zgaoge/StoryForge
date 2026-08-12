import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiToolChips } from '../AiToolChips';

const items = [
  { key: 'all', label: '全部' },
  { key: 'running', label: '执行中', count: 3 },
  { key: 'failed', label: '失败', mono: true },
];

describe('AiToolChips', () => {
  it('渲染全部 chips，radiogroup 可访问名生效', () => {
    render(
      <AiToolChips ariaLabel="任务状态筛选" items={items} activeKey="all" onSelect={() => {}} />
    );
    expect(screen.getByRole('radiogroup', { name: '任务状态筛选' })).toBeInTheDocument();
    expect(screen.getAllByRole('radio')).toHaveLength(3);
  });

  it('activeKey 对应 chip aria-checked=true，其余 false', () => {
    render(<AiToolChips items={items} activeKey="running" onSelect={() => {}} />);
    expect(screen.getByRole('radio', { name: /执行中/ })).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('radio', { name: '全部' })).toHaveAttribute('aria-checked', 'false');
  });

  it('点击调用 onSelect(key)', () => {
    const onSelect = vi.fn();
    render(<AiToolChips items={items} activeKey="all" onSelect={onSelect} />);
    fireEvent.click(screen.getByRole('radio', { name: /失败/ }));
    expect(onSelect).toHaveBeenCalledWith('failed');
  });

  it('count 以 tabular-nums 徽章渲染', () => {
    render(<AiToolChips items={items} activeKey="all" onSelect={() => {}} />);
    const badge = screen.getByText('3');
    expect(badge.className).toContain('tabular-nums');
  });

  it('active 为实心反白（bg-ai-ink），inactive 带 border-ai-line', () => {
    render(<AiToolChips items={items} activeKey="all" onSelect={() => {}} />);
    expect(screen.getByRole('radio', { name: '全部' }).className).toContain('bg-ai-ink');
    expect(screen.getByRole('radio', { name: /失败/ }).className).toContain('border-ai-line');
  });
});
