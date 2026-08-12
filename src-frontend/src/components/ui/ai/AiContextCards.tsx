/**
 * AiContextCards — 检索上下文卡片列表（适配自 beautifului ContextCards）
 *
 * 受控约定：title/count/items 全部由调用方提供，组件不含演示数据；
 * 剥离参考实现：CHUNKS 演示数据、chipsShown 700ms 定时器
 * （source chip 入场改为纯 CSS animate-pop-in + 内联 animationDelay 错峰）、
 * max-w-95 演示宽度限制；内联 SVG 图标改 lucide-react。
 * 站点私有类已替换：rounded-card → rounded-[12px]、shadow-card → border-ai-line、
 * primitive-card-bar → px-3 py-2、shadow-btn → border-ai-line。
 */
import { AlignLeft, ArrowUpRight } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiContextCardSource {
  label: string;
  badge: string;
  tone?: 'green' | 'red' | 'orange' | 'accent' | 'neutral';
}

export interface AiContextCardItem {
  key: string;
  title: string;
  meta?: string;
  body?: string;
  source?: AiContextCardSource;
}

export interface AiContextCardsProps {
  title: string;
  count?: number;
  items: AiContextCardItem[];
  className?: string;
}

const TONE_BG: Record<NonNullable<AiContextCardSource['tone']>, string> = {
  green: 'var(--ai-green)',
  red: 'var(--ai-red)',
  orange: 'var(--ai-orange)',
  accent: 'var(--ai-accent)',
  neutral: 'var(--ai-ink-3)',
};

export function AiContextCards({ title, count, items, className }: AiContextCardsProps) {
  return (
    <div className={cn('flex w-full flex-col gap-2', className)} data-testid="ai-context-cards">
      <div className="animate-fade-in flex items-center gap-2 px-0.5">
        <span className="text-[13px] font-semibold text-ai-ink">{title}</span>
        {typeof count === 'number' && (
          <span className="inline-flex h-5 items-center rounded-md border border-ai-line bg-ai-inset px-1.5 text-[11.5px] font-medium text-ai-ink-2 tabular-nums">
            {count}
          </span>
        )}
      </div>

      {items.map((item, i) => (
        <div
          key={item.key}
          className="animate-ai-fade-up overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface"
          style={{ animationDelay: `${i * 100}ms` }}
        >
          <div className="flex items-center gap-2.5 border-b border-ai-line px-3 py-2">
            <span className="flex min-w-0 items-center gap-1.5 text-[13px] font-medium text-ai-ink">
              <AlignLeft size={11} strokeWidth={2.5} aria-hidden className="shrink-0" />
              <span className="truncate">{item.title}</span>
            </span>
            {item.meta && (
              <span className="ml-auto shrink-0 text-[12px] text-ai-ink-3 tabular-nums">
                {item.meta}
              </span>
            )}
          </div>
          {item.body && (
            <p className="px-3 pt-2 pb-1 text-[12.5px] leading-relaxed text-ai-ink-2">
              {item.body}
            </p>
          )}
          {item.source && (
            <div className={cn('px-3 pb-3', item.body ? 'pt-1' : 'pt-2')}>
              <span
                className="animate-pop-in inline-flex h-6 items-center gap-1.5 rounded-full border border-ai-line bg-ai-inset px-2 text-[12px] font-medium text-ai-ink-2 transition-colors duration-300 hover:bg-ai-hover"
                style={{ animationDelay: `${400 + i * 80}ms` }}
              >
                <span
                  className="flex size-3.5 items-center justify-center rounded-[4px] text-[7px] font-bold text-white"
                  style={{ background: TONE_BG[item.source.tone ?? 'neutral'] }}
                >
                  {item.source.badge}
                </span>
                {item.source.label}
                <ArrowUpRight size={9} strokeWidth={2.5} aria-hidden />
              </span>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

export default AiContextCards;
