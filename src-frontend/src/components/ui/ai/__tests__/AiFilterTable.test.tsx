import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiFilterTable, AiFilterChipsBar, type AiFilterColumn } from '../AiFilterTable';

interface Row {
  id: number;
  name: string;
  status: string;
}

const chips = [
  { key: 'all', label: '全部', count: 3 },
  { key: 'todo', label: '待办', dot: 'var(--ai-orange)', count: 2 },
  { key: 'done', label: '完成', mono: true, count: 1 },
];

const columns: AiFilterColumn<Row>[] = [
  { key: 'name', label: '名称', width: '2fr', render: r => r.name },
  { key: 'status', label: '状态', align: 'center', render: r => <span>{r.status}</span> },
];

const rows: Row[] = [
  { id: 1, name: '条目一', status: 'todo' },
  { id: 2, name: '条目二', status: 'done' },
];

describe('AiFilterChipsBar', () => {
  it('渲染全部 chips 与计数徽章，dot 颜色透传 style', () => {
    render(<AiFilterChipsBar items={chips} activeKey="all" onSelect={() => {}} />);
    expect(screen.getByText('全部')).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
    // 按钮内第一个 span 即 dot（getByText 命中的是 chip 按钮本体，故在按钮内部查）
    const dot = screen.getByRole('button', { name: /待办/ }).querySelector('span')!;
    expect(dot.style.background).toContain('var(--ai-orange)');
  });

  it('activeKey 对应 chip aria-pressed=true，点击调用 onSelect(key)', () => {
    const onSelect = vi.fn();
    render(<AiFilterChipsBar items={chips} activeKey="all" onSelect={onSelect} />);
    expect(screen.getByRole('button', { name: /全部/ })).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: /待办/ }));
    expect(onSelect).toHaveBeenCalledWith('todo');
  });

  it('active chip 实心（bg-ai-surface + border-ai-line），mono chip 带 font-mono', () => {
    render(<AiFilterChipsBar items={chips} activeKey="all" onSelect={() => {}} />);
    expect(screen.getByRole('button', { name: /全部/ }).className).toContain('bg-ai-surface');
    expect(screen.getByRole('button', { name: /完成/ }).className).toContain('font-mono');
  });
});

describe('AiFilterTable', () => {
  it('渲染 chips + 表头 + 行（column.render 生效）', () => {
    render(
      <AiFilterTable
        chips={chips}
        activeChip="all"
        onChipSelect={() => {}}
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
      />
    );
    expect(screen.getByText('名称')).toBeInTheDocument();
    expect(screen.getByText('条目一')).toBeInTheDocument();
    expect(screen.getByText('done')).toBeInTheDocument();
  });

  it('无 chips props 时只渲染表格（表格单用场景）', () => {
    render(<AiFilterTable columns={columns} rows={rows} rowKey={r => r.id} />);
    expect(screen.queryByTestId('ai-filter-chips')).not.toBeInTheDocument();
    expect(screen.getByText('条目二')).toBeInTheDocument();
  });

  it('空行渲染 emptyText；align=right 列带 text-right', () => {
    render(
      <AiFilterTable
        columns={[{ key: 'n', label: '数值', align: 'right', render: (r: Row) => r.name }]}
        rows={[]}
        rowKey={(r: Row) => r.id}
        emptyText="暂无 LLM 调用记录"
      />
    );
    expect(screen.getByText('暂无 LLM 调用记录')).toBeInTheDocument();
    expect(screen.getByText('数值').className).toContain('text-right');
  });

  it('行错峰 animationDelay 递增（封顶 12 行）', () => {
    const many = Array.from({ length: 15 }, (_, i) => ({ id: i, name: `r${i}`, status: 'x' }));
    render(<AiFilterTable columns={columns} rows={many} rowKey={r => r.id} />);
    const first = screen.getByText('r0').closest('.grid')! as HTMLElement;
    const last = screen.getByText('r14').closest('.grid')! as HTMLElement;
    expect(first.style.animationDelay).toBe('0ms');
    expect(last.style.animationDelay).toBe('480ms');
  });
});
