import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiTaskRows, type AiTaskRowItem } from '../AiTaskRows';

const rows: AiTaskRowItem[] = [
  { key: 't1', status: 'completed', label: '拆书分析', meta: '一次性', pill: <span>已完成</span> },
  { key: 't2', status: 'running', progress: 45, label: '级联改写', meta: '每天' },
  { key: 't3', status: 'failed', label: '定时审稿', meta: 'cron', index: 3 },
];

describe('AiTaskRows', () => {
  it('渲染行标签 / meta / pill', () => {
    render(<AiTaskRows rows={rows} onToggle={() => {}} />);
    expect(screen.getByText('拆书分析')).toBeInTheDocument();
    expect(screen.getByText('一次性')).toBeInTheDocument();
    expect(screen.getByText('已完成')).toBeInTheDocument();
  });

  it('completed/failed 渲染对应徽章，running 渲染进度环（环内为百分比）', () => {
    const { container } = render(<AiTaskRows rows={rows} onToggle={() => {}} />);
    expect(container.querySelector('[data-testid="ai-task-badge-completed"]')).toBeTruthy();
    expect(container.querySelector('[data-testid="ai-task-badge-failed"]')).toBeTruthy();
    const ring = container.querySelector('[data-testid="ai-task-ring"]');
    expect(ring?.textContent).toBe('45');
    expect(ring?.querySelector('svg')?.classList.contains('animate-ai-spin')).toBe(true);
  });

  it('点击行调用 onToggle(key)，trailing 点击不触发行 toggle', () => {
    const onToggle = vi.fn();
    const onTrailing = vi.fn();
    render(
      <AiTaskRows
        rows={[
          {
            key: 't1',
            status: 'pending',
            index: 1,
            label: 'x',
            trailing: <button onClick={onTrailing}>执行</button>,
          },
        ]}
        onToggle={onToggle}
      />
    );
    fireEvent.click(screen.getByText('x'));
    expect(onToggle).toHaveBeenCalledWith('t1');
    fireEvent.click(screen.getByText('执行'));
    expect(onTrailing).toHaveBeenCalled();
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it('expandedKey 行展开 renderDetail 内容（grid 1fr），其余 0fr', () => {
    const { container } = render(
      <AiTaskRows
        rows={rows}
        expandedKey="t2"
        onToggle={() => {}}
        renderDetail={row => <div>详情-{row.label}</div>}
      />
    );
    expect(screen.getByText('详情-级联改写')).toBeInTheDocument();
    const expanded = screen
      .getByText('详情-级联改写')
      .closest('[data-testid="ai-task-detail"]')! as HTMLElement;
    expect(expanded.style.gridTemplateRows).toBe('1fr');
    expect(container.querySelectorAll('[data-testid="ai-task-detail"]').length).toBe(3);
  });

  it('details 数组在无 renderDetail 时作为默认展开内容', () => {
    render(
      <AiTaskRows
        rows={[
          {
            key: 't1',
            status: 'completed',
            label: 'x',
            details: [{ label: '匹配记录', meta: '12/12' }],
          },
        ]}
        expandedKey="t1"
        onToggle={() => {}}
      />
    );
    expect(screen.getByText('匹配记录')).toBeInTheDocument();
    expect(screen.getByText('12/12')).toBeInTheDocument();
  });

  it('list 变体行有 border-b，capsules 变体行为独立卡', () => {
    const { container, rerender } = render(
      <AiTaskRows rows={rows} variant="list" onToggle={() => {}} />
    );
    expect(container.querySelector('[data-testid="ai-task-rows"] .border-b')).toBeTruthy();
    rerender(<AiTaskRows rows={rows} variant="capsules" onToggle={() => {}} />);
    expect(
      container.querySelector(
        '[data-testid="ai-task-rows"] .rounded-\\[14px\\], [data-testid="ai-task-rows"] .rounded-\\[22px\\]'
      )
    ).toBeTruthy();
  });
});
