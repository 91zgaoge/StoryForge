/**
 * AiTaskRows — 任务行列表（适配自 beautifului TaskRows）
 *
 * 受控约定：rows/expandedKey/onToggle 全部由调用方提供；剥离参考实现的
 * TICKS/useTick 状态机演示（failed→done 自动翻转）与 manualOpen 内部展开态。
 * 展开内容两选一：details 简单明细，或 renderDetail 自定义（Tasks 页挂既有 TaskDetail）。
 * 移植说明：spin 内联裸 keyframe → animate-ai-spin（P1 已注册，reduced-motion 已冻结）；
 * green-tint/red-tint pill → pill/trailing 插槽由调用方传入；rounded-card/shadow-card →
 * rounded-[14px]/rounded-[22px] + border-ai-line；variant='List' → 'list'（小写）。
 */
import { Check, ChevronDown, X } from 'lucide-react';
import { cn } from '@/utils/cn';

export type AiTaskRowStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface AiTaskRowDetail {
  label: string;
  meta?: string;
}

export interface AiTaskRowItem<T = unknown> {
  key: string;
  status: AiTaskRowStatus;
  /** 0-100；running 时显示在进度环内 */
  progress?: number;
  /** 环内序号（pending 或无 progress 时） */
  index?: number;
  label: string;
  meta?: string;
  pill?: React.ReactNode;
  /** 行尾操作区（chevron 之前）；组件侧已统一 stopPropagation，点击/键盘均不触发行 toggle */
  trailing?: React.ReactNode;
  details?: AiTaskRowDetail[];
  payload?: T;
}

export interface AiTaskRowsProps<T = unknown> {
  rows: AiTaskRowItem<T>[];
  expandedKey?: string | null;
  onToggle: (key: string) => void;
  variant?: 'capsules' | 'list';
  renderDetail?: (row: AiTaskRowItem<T>) => React.ReactNode;
  className?: string;
}

function SpinnerRing({ active, children }: { active?: boolean; children?: React.ReactNode }) {
  const size = 24;
  const stroke = 2;
  const r = (size - stroke) / 2;
  const c = 2 * Math.PI * r;
  return (
    <span
      className="relative inline-flex shrink-0 items-center justify-center"
      style={{ width: size, height: size }}
      data-testid="ai-task-ring"
    >
      <svg
        width={size}
        height={size}
        className={cn('absolute inset-0', active && 'animate-ai-spin')}
        aria-hidden
      >
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke="var(--ai-line)"
          strokeWidth={stroke}
        />
        {active && (
          <circle
            cx={size / 2}
            cy={size / 2}
            r={r}
            fill="none"
            stroke="var(--ai-ink-3)"
            strokeWidth={stroke}
            strokeLinecap="round"
            strokeDasharray={`${c * 0.28} ${c * 0.72}`}
          />
        )}
      </svg>
      <span className="relative text-[10.5px] font-semibold text-ai-ink tabular-nums">
        {children}
      </span>
    </span>
  );
}

function StatusBadge({ status }: { status: AiTaskRowStatus }) {
  if (status === 'completed' || status === 'failed' || status === 'cancelled') {
    const bg =
      status === 'completed'
        ? 'var(--ai-green)'
        : status === 'failed'
          ? 'var(--ai-red)'
          : 'var(--ai-orange)';
    return (
      <span
        className="animate-pop-in flex size-[22px] shrink-0 items-center justify-center rounded-full text-white"
        style={{ background: bg }}
        data-testid={`ai-task-badge-${status}`}
        aria-hidden
      >
        {status === 'completed' ? (
          <Check size={13} strokeWidth={3.5} />
        ) : (
          <X size={12} strokeWidth={3.5} />
        )}
      </span>
    );
  }
  return null;
}

function RowBadge({ row }: { row: AiTaskRowItem }) {
  if (row.status === 'running') {
    return <SpinnerRing active>{row.progress ?? row.index ?? ''}</SpinnerRing>;
  }
  if (row.status === 'pending') {
    return <SpinnerRing>{row.index ?? ''}</SpinnerRing>;
  }
  return <StatusBadge status={row.status} />;
}

export function AiTaskRows<T = unknown>({
  rows,
  expandedKey = null,
  onToggle,
  variant = 'capsules',
  renderDetail,
  className,
}: AiTaskRowsProps<T>) {
  const list = variant === 'list';
  return (
    <div
      className={cn('flex w-full flex-col', list ? 'gap-0' : 'gap-2', className)}
      data-testid="ai-task-rows"
    >
      {rows.map((row, i) => {
        const open = expandedKey === row.key;
        return (
          <div
            key={row.key}
            className={cn(
              'animate-ai-fade-up self-stretch overflow-hidden bg-ai-surface transition-[border-radius] duration-300',
              list
                ? 'border-b border-ai-line last:border-b-0'
                : cn('border border-ai-line', open ? 'rounded-[14px]' : 'rounded-[22px]')
            )}
            style={{ animationDelay: `${i * 80}ms` }}
          >
            <div
              role="button"
              tabIndex={0}
              aria-expanded={open}
              onClick={() => onToggle(row.key)}
              onKeyDown={e => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  onToggle(row.key);
                }
              }}
              className="flex h-11 w-full cursor-pointer items-center gap-2.5 px-2.5 text-left transition-colors duration-100 hover:bg-ai-inset"
            >
              <span className="flex size-6 shrink-0 items-center justify-center">
                <RowBadge row={row} />
              </span>
              <span className="min-w-0 flex-1 truncate text-[13px] font-medium text-ai-ink">
                {row.label}
              </span>
              {row.meta && (
                <span className="shrink-0 text-[12.5px] text-ai-ink-2 tabular-nums">
                  {row.meta}
                </span>
              )}
              {row.pill}
              {row.trailing && (
                /* trailing 插槽点击/键盘均不触发行 toggle（组件侧统一拦截） */
                <span
                  className="flex shrink-0 items-center gap-1"
                  onClick={e => e.stopPropagation()}
                  onKeyDown={e => e.stopPropagation()}
                >
                  {row.trailing}
                </span>
              )}
              <span
                aria-hidden="true"
                className="-ml-1 flex size-7 shrink-0 items-center justify-center rounded-full text-ai-ink-3"
              >
                <ChevronDown
                  size={15}
                  strokeWidth={2.2}
                  className="transition-transform duration-300"
                  style={{ transform: open ? 'rotate(180deg)' : 'rotate(0deg)' }}
                />
              </span>
            </div>

            <div
              className="grid transition-[grid-template-rows,opacity] duration-300"
              style={{
                gridTemplateRows: open ? '1fr' : '0fr',
                opacity: open ? 1 : 0,
                transitionTimingFunction: 'cubic-bezier(0.23, 1, 0.32, 1)',
              }}
              data-testid="ai-task-detail"
            >
              <div className="min-h-0 overflow-hidden">
                {renderDetail ? (
                  renderDetail(row)
                ) : (
                  <div className="mb-2.5 grid grid-cols-[24px_1fr] gap-2.5 px-2.5">
                    <span aria-hidden className="mx-auto h-full w-px bg-ai-line" />
                    <div className="flex flex-col gap-1.5">
                      {(row.details ?? []).map((d, j) => (
                        <div
                          key={d.label}
                          className="flex items-center justify-between"
                          style={
                            open
                              ? {
                                  animation: `fade-up 300ms cubic-bezier(0.23,1,0.32,1) ${120 + j * 100}ms both`,
                                }
                              : undefined
                          }
                        >
                          <span className="text-[12px] text-ai-ink-2">{d.label}</span>
                          {d.meta && (
                            <span className="font-mono text-[11.5px] text-ai-ink-3 tabular-nums">
                              {d.meta}
                            </span>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default AiTaskRows;
