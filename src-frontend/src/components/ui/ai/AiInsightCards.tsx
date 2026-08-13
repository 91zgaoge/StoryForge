/**
 * AiInsightCards — 统计洞察卡片组（适配自 beautifului InsightCards）
 *
 * 受控约定：items/columns 全部由调用方提供（label/value/sub/icon/tone/series
 * 均为 props）；剥离参考实现的 CompareCard/AnomalyCard/AllocationCard 三演示卡、
 * makePoints 演示数据、PAGES/autoplay/「Insights N ‹ ›」分页壳、blur crossfade
 * 占位（L436-439 写死 opacity:1/blur(0)，随分页壳一并剥离）与 pill CTA。
 * liveline 依赖 → 内嵌私有 MiniLineChart（SVG polyline 静态快照：无 hover
 * scrub、无 tooltip、无实时动效；insight-chart-* 全局类随之剥离）；
 * useDarkMode MutationObserver 删除（双窗口固定主题由 --ai-* 接管）；
 * 序列色 hex 映射 ai-orange/ai-accent/ai-red（tone 映射，宿主不感知 hex）。
 */
import { useId } from 'react';
import { cn } from '@/utils/cn';

export interface AiInsightCardItem {
  key: string;
  label: string;
  value: string;
  sub?: string;
  icon?: React.ReactNode;
  /** 标签/icon/sub/折线的语义色（默认 neutral） */
  tone?: 'accent' | 'green' | 'orange' | 'red' | 'neutral';
  /** 静态快照迷你折线（时间正序数值序列；不提供则不渲染图表） */
  series?: number[];
  seriesLabel?: string;
}

export interface AiInsightCardsProps {
  items: AiInsightCardItem[];
  columns?: 2 | 3 | 4;
  className?: string;
}

const TONE_COLOR: Record<NonNullable<AiInsightCardItem['tone']>, string> = {
  accent: 'var(--ai-accent)',
  green: 'var(--ai-green)',
  orange: 'var(--ai-orange)',
  red: 'var(--ai-red)',
  neutral: 'var(--ai-ink-3)',
};

/** liveline 静态快照替代：SVG polyline + 面积渐变 + 末点（无实时动效） */
function MiniLineChart({
  values,
  color,
  label,
}: {
  values: number[];
  color: string;
  label?: string;
}) {
  const gradientId = useId();
  const w = 260;
  const h = 64;
  const pad = 6;
  if (values.length === 0) return null;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const x = (i: number) => pad + (i / Math.max(values.length - 1, 1)) * (w - pad * 2);
  const y = (v: number) => h - pad - ((v - min) / span) * (h - pad * 2);
  const d = values
    .map((v, i) => `${i === 0 ? 'M' : 'L'}${x(i).toFixed(1)},${y(v).toFixed(1)}`)
    .join(' ');
  const area = `${d} L${x(values.length - 1).toFixed(1)},${h - pad} L${x(0).toFixed(1)},${h - pad} Z`;
  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      className="h-16 w-full"
      role="img"
      aria-label={label ?? '趋势快照'}
      data-testid="ai-insight-chart"
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.25" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <path d={area} fill={`url(#${gradientId})`} />
      <path
        d={d}
        fill="none"
        stroke={color}
        strokeWidth="2"
        strokeLinejoin="round"
        strokeLinecap="round"
      />
      <circle cx={x(values.length - 1)} cy={y(values[values.length - 1])} r="3" fill={color} />
    </svg>
  );
}

const COLUMN_CLASS: Record<NonNullable<AiInsightCardsProps['columns']>, string> = {
  2: 'md:grid-cols-2',
  3: 'md:grid-cols-3',
  4: 'md:grid-cols-2 lg:grid-cols-4',
};

export function AiInsightCards({ items, columns = 4, className }: AiInsightCardsProps) {
  return (
    <div
      className={cn('grid grid-cols-1 gap-3', COLUMN_CLASS[columns], className)}
      data-testid="ai-insight-cards"
    >
      {items.map((item, i) => {
        const tone = TONE_COLOR[item.tone ?? 'neutral'];
        return (
          <div
            key={item.key}
            className="animate-ai-fade-up rounded-[12px] border border-ai-line bg-ai-surface p-4"
            style={{ animationDelay: `${i * 80}ms` }}
          >
            <div className="flex items-center justify-between gap-2">
              <span className="text-[11px] font-medium uppercase tracking-wider text-ai-ink-3">
                {item.label}
              </span>
              {item.icon && (
                <span className="shrink-0 opacity-40" style={{ color: tone }} aria-hidden>
                  {item.icon}
                </span>
              )}
            </div>
            <div className="mt-1 text-[22px] font-semibold tracking-[-0.01em] text-ai-ink tabular-nums">
              {item.value}
            </div>
            {item.sub && (
              <div className="mt-1 text-[11.5px]" style={{ color: tone }}>
                {item.sub}
              </div>
            )}
            {item.series && item.series.length > 0 && (
              <div className="mt-2 overflow-hidden rounded-[8px] border border-ai-line bg-ai-inset p-1.5">
                <MiniLineChart values={item.series} color={tone} label={item.seriesLabel} />
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

export default AiInsightCards;
