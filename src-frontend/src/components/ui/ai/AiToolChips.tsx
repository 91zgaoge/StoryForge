/**
 * AiToolChips — 单选筛选 chips 组（提取自 beautifului ToolChips 的 chip 视觉语法）
 *
 * 受控约定：items/activeKey/onSelect 全部由调用方提供；剥离参考实现的
 * ROWS/DIFFS 演示数据、STEP_MS 自运行步进、工具调用行展开明细（与本批
 * 筛选条集成点语义不符，轨迹场景已由 P1 AiThinking 覆盖）。
 * active 实心反白取自参考 primary 样式（bg-ink text-canvas → bg-ai-ink text-ai-surface）。
 */
import { cn } from '@/utils/cn';

export interface AiToolChipItem {
  key: string;
  label: string;
  count?: number;
  mono?: boolean;
}

export interface AiToolChipsProps {
  items: AiToolChipItem[];
  activeKey: string;
  onSelect: (key: string) => void;
  ariaLabel?: string;
  className?: string;
}

export function AiToolChips({
  items,
  activeKey,
  onSelect,
  ariaLabel,
  className,
}: AiToolChipsProps) {
  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className={cn('flex flex-wrap gap-1.5', className)}
      data-testid="ai-tool-chips"
    >
      {items.map((item, i) => {
        const active = item.key === activeKey;
        return (
          <button
            key={item.key}
            type="button"
            role="radio"
            aria-checked={active}
            onClick={() => onSelect(item.key)}
            className={cn(
              'animate-pop-in inline-flex h-7 items-center gap-1 rounded-full px-2.5 text-[12px] font-medium transition-[background-color,color,transform] duration-150 active:scale-[0.96]',
              item.mono && 'font-mono',
              active
                ? 'bg-ai-ink text-ai-surface'
                : 'border border-ai-line bg-ai-surface text-ai-ink-2 hover:bg-ai-hover'
            )}
            style={{ animationDelay: `${i * 60}ms` }}
          >
            {item.label}
            {typeof item.count === 'number' && (
              <span className="text-[11px] tabular-nums opacity-70">{item.count}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export default AiToolChips;
