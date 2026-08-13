/**
 * AiSearchList — 搜索框 + 结果计数/空态（适配自 beautifului SearchList）
 *
 * 受控约定：value/onChange/resultCount 全部由调用方提供，组件不含演示数据；
 * 剥离参考实现：ITEMS 演示数据、结果下拉列表与点击回填（集成点的结果集是
 * 宿主主列表，下拉语义不符）、min-h/max-w-72 演示尺寸。
 * 移植说明：内联 animation: fade-in 裸 keyframes → animate-fade-in 类
 * （tailwind.config.js L90 已注册 fadeIn）；size-5.5（Tailwind v4 动态间距）
 * → size-[22px]；rounded-card/shadow-raised/rounded-control/shadow-hairline →
 * rounded-[12px]/border-ai-line/rounded-[8px]；var(--ink-3) 等直引改 ai-* 令牌类。
 */
import { Search, X } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiSearchListProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  ariaLabel?: string;
  /** 有搜索词且提供时渲染计数行；为 0 时渲染空态卡 */
  resultCount?: number;
  emptyText?: string;
  emptyHint?: string;
  className?: string;
}

export function AiSearchList({
  value,
  onChange,
  placeholder = '搜索…',
  ariaLabel = '搜索',
  resultCount,
  emptyText = '未找到匹配结果',
  emptyHint = '尝试调整搜索关键词',
  className,
}: AiSearchListProps) {
  const hasQuery = value.trim().length > 0;
  const empty = hasQuery && resultCount === 0;

  return (
    <div className={cn('flex w-full flex-col gap-2', className)} data-testid="ai-search-list">
      <div className="flex h-10 items-center gap-2 rounded-[12px] border border-ai-line bg-ai-surface px-3 transition-colors duration-100 focus-within:border-ai-line-strong hover:bg-ai-hover">
        <Search size={14} strokeWidth={2} aria-hidden className="shrink-0 text-ai-ink-3" />
        <input
          value={value}
          onChange={e => onChange(e.target.value)}
          placeholder={placeholder}
          aria-label={ariaLabel}
          className="min-w-0 flex-1 bg-transparent text-[13px] text-ai-ink outline-none placeholder:text-ai-ink-3"
        />
        {hasQuery && (
          <button
            type="button"
            aria-label="清除搜索"
            onClick={() => onChange('')}
            className="animate-fade-in flex size-[22px] shrink-0 items-center justify-center rounded-full text-ai-ink-3 transition-colors duration-100 hover:bg-[color-mix(in_srgb,var(--ai-line)_70%,transparent)] hover:text-ai-ink"
          >
            <X size={11} strokeWidth={2.2} aria-hidden />
          </button>
        )}
      </div>

      {hasQuery && typeof resultCount === 'number' && !empty && (
        <p
          className="animate-fade-in px-0.5 text-[12.5px] text-ai-ink-2"
          data-testid="ai-search-count"
        >
          搜索 “{value}” 找到 <span className="tabular-nums">{resultCount}</span> 条结果
        </p>
      )}

      {empty && (
        <div
          className="animate-fade-in flex flex-col items-center justify-center gap-1 rounded-[12px] border border-ai-line bg-ai-surface px-4 py-8"
          data-testid="ai-search-empty"
        >
          <span className="mb-1.5 flex size-8 items-center justify-center rounded-[8px] border border-ai-line bg-ai-inset text-ai-ink-3">
            <Search size={15} strokeWidth={1.8} aria-hidden />
          </span>
          <span className="text-[13px] font-medium text-ai-ink">{emptyText}</span>
          <span className="text-[12px] text-ai-ink-3">{emptyHint}</span>
        </div>
      )}
    </div>
  );
}

export default AiSearchList;
