/**
 * AiFilterTable — 筛选 chips + 数据表（适配自 beautifului FilterTable）
 *
 * 受控约定：chips/activeChip/onChipSelect 与 columns/rows 全部由调用方提供，
 * 行过滤在宿主完成（参考实现内部 filter state 剥离）；剥离 FILTERS/ROWS 演示
 * 数据、filter-status-* 全局类（pill 经 column.render 插槽由调用方给出，同
 * P2 AiTaskRows pill 插槽思路）；chips 圆点硬编码 hex 收进 props（dot 为 CSS
 * 颜色值，宿主传 var(--ai-*)）。
 * 移植说明：行显隐 grid 0fr/1fr 动画 → animate-ai-fade-up 错峰入场（封顶 12 行）；
 * h-6.5 → h-[26px]；shadow-btn/shadow-card/rounded-card → border-ai-line/
 * rounded-[12px]；scrollbarWidth 内联 → [scrollbar-width:none]。
 * AiFilterChipsBar 为 chips 条的独立命名导出（仅需 chips 的场景复用，如 Logs 级别筛选）。
 */
import { cn } from '@/utils/cn';

export interface AiFilterChipItem {
  key: string;
  label: string;
  count?: number;
  /** CSS 颜色值（建议 var(--ai-*)） */
  dot?: string;
  mono?: boolean;
}

export interface AiFilterChipsBarProps {
  items: AiFilterChipItem[];
  activeKey: string;
  onSelect: (key: string) => void;
  ariaLabel?: string;
  className?: string;
}

export function AiFilterChipsBar({
  items,
  activeKey,
  onSelect,
  ariaLabel,
  className,
}: AiFilterChipsBarProps) {
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      className={cn(
        '-mx-1 flex items-center gap-1 overflow-x-auto px-1 py-1 [scrollbar-width:none]',
        className
      )}
      data-testid="ai-filter-chips"
    >
      {items.map(item => {
        const active = item.key === activeKey;
        return (
          <button
            key={item.key}
            type="button"
            aria-pressed={active}
            onClick={() => onSelect(item.key)}
            className={cn(
              'flex h-[26px] shrink-0 items-center gap-1.5 rounded-full px-2.5 text-[12px] font-medium transition-[background-color,color] duration-200',
              item.mono && 'font-mono',
              active
                ? 'border border-ai-line bg-ai-surface text-ai-ink'
                : 'text-ai-ink-2 hover:bg-ai-hover'
            )}
          >
            {item.dot && (
              <span
                className="size-1.5 rounded-full"
                style={{ background: item.dot }}
                aria-hidden
              />
            )}
            {item.label}
            {typeof item.count === 'number' && (
              <span
                className={cn(
                  'rounded-[4px] px-1 text-[10.5px] tabular-nums',
                  active ? 'bg-ai-field text-ai-ink-2' : 'text-ai-ink-3'
                )}
              >
                {item.count}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export interface AiFilterColumn<T> {
  key: string;
  label: React.ReactNode;
  align?: 'left' | 'right' | 'center';
  /** grid 列宽（fr 或固定值），默认 1fr */
  width?: string;
  render: (row: T) => React.ReactNode;
}

export interface AiFilterTableProps<T> {
  /** chips 三件套同时提供时才渲染筛选条（表格可单用） */
  chips?: AiFilterChipItem[];
  activeChip?: string;
  onChipSelect?: (key: string) => void;
  chipsAriaLabel?: string;
  columns: AiFilterColumn<T>[];
  rows: T[];
  rowKey: (row: T) => React.Key;
  emptyText?: string;
  minWidth?: number;
  className?: string;
}

const ALIGN = { left: 'text-left', right: 'text-right', center: 'text-center' } as const;

export function AiFilterTable<T>({
  chips,
  activeChip,
  onChipSelect,
  chipsAriaLabel,
  columns,
  rows,
  rowKey,
  emptyText = '暂无数据',
  minWidth = 420,
  className,
}: AiFilterTableProps<T>) {
  const template = columns.map(c => c.width ?? '1fr').join(' ');
  return (
    <div className={cn('w-full', className)} data-testid="ai-filter-table">
      {chips && activeChip !== undefined && onChipSelect && (
        <AiFilterChipsBar
          items={chips}
          activeKey={activeChip}
          onSelect={onChipSelect}
          ariaLabel={chipsAriaLabel}
          className="mb-1"
        />
      )}
      <div
        role="region"
        aria-label="数据表（可横向滚动）"
        tabIndex={0}
        className="overflow-x-auto rounded-[12px] border border-ai-line bg-ai-surface [scrollbar-width:none]"
      >
        <div style={{ minWidth }}>
          <div
            className="grid border-b border-ai-line px-3 py-2 text-[11.5px] font-medium text-ai-ink-3"
            style={{ gridTemplateColumns: template }}
          >
            {columns.map(c => (
              <span key={c.key} className={ALIGN[c.align ?? 'left']}>
                {c.label}
              </span>
            ))}
          </div>
          {rows.length === 0 ? (
            <div className="px-3 py-8 text-center text-[12.5px] text-ai-ink-3">{emptyText}</div>
          ) : (
            rows.map((row, i) => (
              <div
                key={rowKey(row)}
                className="animate-ai-fade-up grid items-center border-b border-ai-line px-3 py-2 text-[12px] transition-colors duration-100 last:border-0 hover:bg-ai-hover"
                style={{
                  gridTemplateColumns: template,
                  animationDelay: `${Math.min(i, 12) * 40}ms`,
                }}
              >
                {columns.map(c => (
                  <span key={c.key} className={cn('min-w-0', ALIGN[c.align ?? 'left'])}>
                    {c.render(row)}
                  </span>
                ))}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

export default AiFilterTable;
