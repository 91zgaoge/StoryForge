/**
 * AiRecordsTable — 记录表格（适配自 beautifului RecordsTable）
 *
 * 受控约定：columns/rows/selection/sort/expanded 全部由调用方提供（排序与过滤在
 * 宿主完成）；剥离参考实现的 INITIAL_ROWS/TAG_COLORS/STRENGTH 演示数据与内部
 * useState。records-* 约 25 个站点全局类（payload 未含定义）全部 Tailwind 自研。
 * 裁剪：sticky 首列、真实链接列、tag 列、「Add calculation」交互（P3 宿主无
 * 对应场景，tag/link 可由 column.render 自行给出）。
 * 新增（相对参考）：
 * - 受控展开行 expandedKey/onRowToggle/renderDetail（PromptsPanel 展开编辑器），
 *   详情仅 open 时挂载（animate-ai-fade-up 入场），避免折叠态常驻重型编辑器 DOM；
 * - rowKeyAttribute：tr 输出 data-{attr}={key}，兼容宿主既有测试选择器。
 * Checkbox 自研含 mixed（indeterminate 经 ref 设置）；tfoot 计算行 → 可选 footer 插槽。
 */
import { Fragment } from 'react';
import { ArrowDown, Check, ChevronDown } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiRecordsColumn<T> {
  key: string;
  label: React.ReactNode;
  icon?: React.ReactNode;
  align?: 'left' | 'right' | 'center';
  /** col 宽度（固定值如 '120px'；不设则均分） */
  width?: string;
  sortable?: boolean;
  render: (row: T) => React.ReactNode;
}

export interface AiRecordsSort {
  key: string;
  dir: 1 | -1;
}

export interface AiRecordsTableProps<T> {
  columns: AiRecordsColumn<T>[];
  rows: T[];
  rowKey: (row: T) => string;
  /** 三件套同时提供才出现勾选列（selection 全受控） */
  selectable?: boolean;
  selectedKeys?: ReadonlySet<string>;
  onSelectionChange?: (next: Set<string>) => void;
  /** 受控排序：组件只渲染表头 UI 并回调，排序本身在宿主 */
  sort?: AiRecordsSort | null;
  onSortChange?: (sort: AiRecordsSort) => void;
  /** 受控展开行（onRowToggle + renderDetail 同时提供才出现展开列） */
  expandedKey?: string | null;
  onRowToggle?: (key: string) => void;
  renderDetail?: (row: T) => React.ReactNode;
  /** tr 输出 data-{rowKeyAttribute}={key}（宿主既有测试选择器兼容） */
  rowKeyAttribute?: string;
  /** tfoot 计算行的受控化：可选 footer 插槽 */
  footer?: React.ReactNode;
  emptyText?: string;
  ariaLabel?: string;
  className?: string;
}

function Checkbox({
  checked,
  mixed = false,
  onChange,
  label,
}: {
  checked: boolean;
  mixed?: boolean;
  onChange: () => void;
  label: string;
}) {
  return (
    <label
      className="inline-flex shrink-0 cursor-pointer items-center"
      title={label}
      onClick={e => e.stopPropagation()}
    >
      <input
        type="checkbox"
        className="sr-only"
        checked={checked}
        ref={el => {
          if (el) el.indeterminate = mixed;
        }}
        onChange={onChange}
        aria-label={label}
      />
      <span
        aria-hidden
        className={cn(
          'flex size-4 items-center justify-center rounded-[5px] border transition-colors duration-150',
          checked || mixed
            ? 'border-ai-ink bg-ai-ink text-ai-surface'
            : 'border-ai-line-strong bg-ai-surface'
        )}
      >
        {mixed ? (
          <span className="h-[2px] w-2 rounded-full bg-current" />
        ) : checked ? (
          <Check size={12} strokeWidth={3} />
        ) : null}
      </span>
    </label>
  );
}

export function AiRecordsTable<T>({
  columns,
  rows,
  rowKey,
  selectable = false,
  selectedKeys,
  onSelectionChange,
  sort = null,
  onSortChange,
  expandedKey = null,
  onRowToggle,
  renderDetail,
  rowKeyAttribute,
  footer,
  emptyText = '暂无数据',
  ariaLabel = '记录表格（可滚动查看全部列与记录）',
  className,
}: AiRecordsTableProps<T>) {
  const showSelection = selectable && !!onSelectionChange;
  const expandable = Boolean(onRowToggle && renderDetail);
  const colCount = columns.length + (showSelection ? 1 : 0) + (expandable ? 1 : 0);

  const selected = selectedKeys ?? new Set<string>();
  const allSelected = rows.length > 0 && rows.every(r => selected.has(rowKey(r)));
  const partiallySelected = !allSelected && rows.some(r => selected.has(rowKey(r)));

  const toggleRow = (key: string) => {
    if (!onSelectionChange) return;
    const next = new Set(selected);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    onSelectionChange(next);
  };
  const toggleAll = () => {
    if (!onSelectionChange) return;
    const next = new Set(selected);
    if (allSelected) rows.forEach(r => next.delete(rowKey(r)));
    else rows.forEach(r => next.add(rowKey(r)));
    onSelectionChange(next);
  };
  const clickSort = (key: string) => {
    if (!onSortChange) return;
    onSortChange(
      sort && sort.key === key ? { key, dir: (sort.dir * -1) as 1 | -1 } : { key, dir: 1 }
    );
  };

  return (
    <div
      className={cn(
        'w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface',
        className
      )}
      data-testid="ai-records-table"
    >
      <div className="overflow-auto" tabIndex={0} aria-label={ariaLabel}>
        <table className="w-full border-collapse text-left">
          <colgroup>
            {showSelection && <col className="w-8" />}
            {columns.map(c => (
              <col key={c.key} style={c.width ? { width: c.width } : undefined} />
            ))}
            {expandable && <col className="w-10" />}
          </colgroup>
          <thead>
            <tr className="border-b border-ai-line">
              {showSelection && (
                <th className="px-3 py-2">
                  <Checkbox
                    checked={allSelected}
                    mixed={partiallySelected}
                    onChange={toggleAll}
                    label="全选"
                  />
                </th>
              )}
              {columns.map(c => {
                const active = sort?.key === c.key;
                return (
                  <th
                    key={c.key}
                    className={cn(
                      'px-3 py-2 text-[11.5px] font-medium text-ai-ink-3',
                      c.align === 'right' && 'text-right',
                      c.align === 'center' && 'text-center'
                    )}
                    aria-sort={active ? (sort!.dir === 1 ? 'ascending' : 'descending') : undefined}
                  >
                    {c.sortable && onSortChange ? (
                      <button
                        type="button"
                        onClick={() => clickSort(c.key)}
                        className={cn(
                          'inline-flex items-center gap-1 transition-colors hover:text-ai-ink',
                          c.align === 'right' && 'flex-row-reverse'
                        )}
                      >
                        {c.icon}
                        <span className="truncate">{c.label}</span>
                        <ArrowDown
                          size={12}
                          strokeWidth={2.2}
                          aria-hidden
                          data-testid={`ai-records-sort-${c.key}`}
                          className={cn(
                            'transition-[transform,opacity] duration-200',
                            active ? 'opacity-100' : 'opacity-0'
                          )}
                          style={{
                            transform: active && sort!.dir === -1 ? 'rotate(180deg)' : undefined,
                          }}
                        />
                      </button>
                    ) : (
                      <span className="inline-flex items-center gap-1">
                        {c.icon}
                        <span className="truncate">{c.label}</span>
                      </span>
                    )}
                  </th>
                );
              })}
              {expandable && <th className="px-2 py-2" aria-label="展开列" />}
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 && (
              <tr>
                <td
                  colSpan={colCount}
                  className="px-3 py-8 text-center text-[12.5px] text-ai-ink-3"
                >
                  {emptyText}
                </td>
              </tr>
            )}
            {rows.map(row => {
              const key = rowKey(row);
              const open = expandedKey === key;
              const isSelected = selected.has(key);
              const keyAttr = rowKeyAttribute ? { [`data-${rowKeyAttribute}`]: key } : {};
              return (
                <Fragment key={key}>
                  <tr
                    {...keyAttr}
                    onClick={onRowToggle ? () => onRowToggle(key) : undefined}
                    className={cn(
                      'border-b border-ai-line text-[12.5px] transition-colors duration-100',
                      onRowToggle && 'cursor-pointer',
                      isSelected ? 'bg-ai-accent-tint' : 'hover:bg-ai-hover'
                    )}
                  >
                    {showSelection && (
                      <td className="px-3 py-2">
                        <Checkbox
                          checked={isSelected}
                          onChange={() => toggleRow(key)}
                          label={`选择 ${key}`}
                        />
                      </td>
                    )}
                    {columns.map(c => (
                      <td
                        key={c.key}
                        className={cn(
                          'px-3 py-2 text-ai-ink-2',
                          c.align === 'right' && 'text-right',
                          c.align === 'center' && 'text-center'
                        )}
                      >
                        {c.render(row)}
                      </td>
                    ))}
                    {expandable && (
                      <td className="px-2 py-2">
                        <button
                          type="button"
                          aria-label={open ? '收起' : '展开'}
                          aria-expanded={open}
                          onClick={e => {
                            e.stopPropagation();
                            onRowToggle!(key);
                          }}
                          className="flex size-6 items-center justify-center rounded-full text-ai-ink-3 transition-colors hover:bg-ai-hover hover:text-ai-ink"
                        >
                          <ChevronDown
                            size={14}
                            strokeWidth={2.2}
                            aria-hidden
                            className="transition-transform duration-300"
                            style={{ transform: open ? 'rotate(180deg)' : undefined }}
                          />
                        </button>
                      </td>
                    )}
                  </tr>
                  {expandable && open && (
                    <tr>
                      <td colSpan={colCount} className="border-b border-ai-line p-0">
                        <div className="animate-ai-fade-up" data-testid="ai-records-detail">
                          {renderDetail!(row)}
                        </div>
                      </td>
                    </tr>
                  )}
                </Fragment>
              );
            })}
          </tbody>
          {footer && (
            <tfoot>
              <tr className="border-t border-ai-line bg-ai-inset">
                <td colSpan={colCount} className="px-3 py-2">
                  {footer}
                </td>
              </tr>
            </tfoot>
          )}
        </table>
      </div>
    </div>
  );
}

export default AiRecordsTable;
