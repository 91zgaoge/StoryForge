import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiRecordsTable, type AiRecordsColumn } from '../AiRecordsTable';

interface Row {
  id: string;
  name: string;
  tokens: number;
}

const rows: Row[] = [
  { id: 'r1', name: '条目一', tokens: 8000 },
  { id: 'r2', name: '条目二', tokens: 3000 },
];

const columns: AiRecordsColumn<Row>[] = [
  { key: 'name', label: '名称', render: r => <span>{r.name}</span> },
  {
    key: 'tokens',
    label: '总 tokens',
    align: 'right',
    sortable: true,
    render: r => <span>{r.tokens}</span>,
  },
];

describe('AiRecordsTable', () => {
  it('渲染表头与行（column.render 生效），空行渲染 emptyText', () => {
    const { rerender } = render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id} />
    );
    expect(screen.getByText('名称')).toBeInTheDocument();
    expect(screen.getByText('条目一')).toBeInTheDocument();
    rerender(
      <AiRecordsTable columns={columns} rows={[]} rowKey={r => r.id} emptyText="暂无判定记录" />
    );
    expect(screen.getByText('暂无判定记录')).toBeInTheDocument();
  });

  it('onRowToggle：点击行与 chevron 各触发一次（chevron 不双触发）', () => {
    const onToggle = vi.fn();
    render(
      <AiRecordsTable
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
        onRowToggle={onToggle}
        renderDetail={r => <div>详情-{r.name}</div>}
      />
    );
    fireEvent.click(screen.getByText('条目一'));
    expect(onToggle).toHaveBeenCalledWith('r1');
    fireEvent.click(screen.getAllByRole('button', { name: '展开' })[1]);
    expect(onToggle).toHaveBeenCalledWith('r2');
    expect(onToggle).toHaveBeenCalledTimes(2);
  });

  it('expandedKey 行挂载 renderDetail，chevron aria-expanded=true；未展开行不挂载详情', () => {
    render(
      <AiRecordsTable
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
        expandedKey="r2"
        onRowToggle={() => {}}
        renderDetail={r => <div>详情-{r.name}</div>}
      />
    );
    expect(screen.getByText('详情-条目二')).toBeInTheDocument();
    expect(screen.queryByText('详情-条目一')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '收起' })).toHaveAttribute('aria-expanded', 'true');
  });

  it('rowKeyAttribute 输出 data-{attr}={key} 到行 tr', () => {
    const { container } = render(
      <AiRecordsTable
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
        rowKeyAttribute="prompt-id"
      />
    );
    expect(container.querySelector('tr[data-prompt-id="r1"]')).toBeTruthy();
  });

  it('selectable：行勾选与全选走 onSelectionChange，部分选中时全选框 indeterminate', () => {
    const onSelectionChange = vi.fn();
    render(
      <AiRecordsTable
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
        selectable
        selectedKeys={new Set(['r1'])}
        onSelectionChange={onSelectionChange}
      />
    );
    const allBox = screen.getByLabelText('全选') as HTMLInputElement;
    expect(allBox.indeterminate).toBe(true);
    fireEvent.click(allBox);
    expect(onSelectionChange).toHaveBeenCalledWith(new Set(['r1', 'r2']));
    fireEvent.click(screen.getByLabelText('选择 r2'));
    expect(onSelectionChange).toHaveBeenCalledWith(new Set(['r1', 'r2']));
  });

  it('全选态下点击全选框清空选择', () => {
    const onSelectionChange = vi.fn();
    render(
      <AiRecordsTable
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
        selectable
        selectedKeys={new Set(['r1', 'r2'])}
        onSelectionChange={onSelectionChange}
      />
    );
    fireEvent.click(screen.getByLabelText('全选'));
    expect(onSelectionChange).toHaveBeenCalledWith(new Set());
  });

  it('sortable 列表头点击调用 onSortChange（同 key 翻转 dir），箭头仅 active 列可见', () => {
    const onSortChange = vi.fn();
    const { container } = render(
      <AiRecordsTable
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
        sort={{ key: 'tokens', dir: 1 }}
        onSortChange={onSortChange}
      />
    );
    fireEvent.click(screen.getByRole('button', { name: /总 tokens/ }));
    expect(onSortChange).toHaveBeenCalledWith({ key: 'tokens', dir: -1 });
    // SVG 的 className 是 SVGAnimatedString 而非字符串，须用 getAttribute('class') 断言
    expect(
      container.querySelector('[data-testid="ai-records-sort-tokens"]')!.getAttribute('class')
    ).toContain('opacity-100');
  });

  it('th 输出 aria-sort；footer 插槽渲染在 tfoot', () => {
    render(
      <AiRecordsTable
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
        sort={{ key: 'tokens', dir: -1 }}
        onSortChange={() => {}}
        footer={<span>合计 11000 tokens</span>}
      />
    );
    expect(screen.getByRole('columnheader', { name: /总 tokens/ })).toHaveAttribute(
      'aria-sort',
      'descending'
    );
    expect(screen.getByText('合计 11000 tokens')).toBeInTheDocument();
  });

  it('非 sortable 列表头不渲染按钮', () => {
    render(
      <AiRecordsTable
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
        sort={null}
        onSortChange={() => {}}
      />
    );
    expect(screen.queryByRole('button', { name: /名称/ })).not.toBeInTheDocument();
  });
});
