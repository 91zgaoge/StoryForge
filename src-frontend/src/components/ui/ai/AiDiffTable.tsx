/**
 * AiDiffTable — 指标基准/对比/Δ 行式对比表（适配自 beautifului DiffTable）
 *
 * 受控约定：title/rows/baseLabel/compareLabel 全部由调用方提供；剥离参考实现的
 * useStage 三段时序演示（plain→red tint→added row）、ROWS/DOT 演示数据与
 * 「删除行/新增行」演示语法（集成点无行增删场景）。
 * Δ 着色语义为本组件新增：delta=0 → --ai-ink-3；非零按 betterWhen（默认 higher）
 * 判定改善（--ai-green）/恶化（--ai-red），箭头随 delta 正负。
 * 移植说明：red-tint/green-tint 无令牌 → color-mix(in srgb, … 12%, transparent)
 * 内联（零扩令牌，不动 16 变量契约）；primitive-card-bar/table-cell → tailwind
 * 数值；rounded-card/shadow-card → rounded-[12px]/border-ai-line。
 */
import { ArrowDownRight, ArrowUpRight, Minus } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiDiffRow {
  key: string;
  label: string;
  base: React.ReactNode;
  compare: React.ReactNode;
  /** 数值差（compare - base）；着色与箭头由 delta 正负 + betterWhen 决定 */
  delta: number;
  formatDelta?: (delta: number) => string;
  /** 默认 higher：delta>0 为改善；lower 反之（如 tokens 成本） */
  betterWhen?: 'higher' | 'lower';
}

export interface AiDiffTableProps {
  title?: string;
  rows: AiDiffRow[];
  baseLabel?: string;
  compareLabel?: string;
  className?: string;
}

const TONE_VAR = {
  good: 'var(--ai-green)',
  bad: 'var(--ai-red)',
  neutral: 'var(--ai-ink-3)',
} as const;

export function AiDiffTable({
  title,
  rows,
  baseLabel = '基准',
  compareLabel = '对比',
  className,
}: AiDiffTableProps) {
  return (
    <div
      className={cn(
        'w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface',
        className
      )}
      data-testid="ai-diff-table"
    >
      {title && (
        <div className="border-b border-ai-line px-3 py-2">
          <span className="text-[12.5px] font-medium text-ai-ink">{title}</span>
        </div>
      )}
      <table className="w-full table-fixed border-collapse text-left">
        <colgroup>
          <col className="w-[28%]" />
          <col className="w-[24%]" />
          <col className="w-[24%]" />
          <col className="w-[24%]" />
        </colgroup>
        <thead>
          <tr className="border-b border-ai-line">
            {['指标', baseLabel, compareLabel, 'Δ'].map(h => (
              <th key={h} className="px-3 py-2 text-[12px] font-medium text-ai-ink-3">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map(row => {
            const tone =
              row.delta === 0
                ? 'neutral'
                : (row.betterWhen ?? 'higher') === 'higher'
                  ? row.delta > 0
                    ? 'good'
                    : 'bad'
                  : row.delta < 0
                    ? 'good'
                    : 'bad';
            const color = TONE_VAR[tone];
            const Icon = row.delta === 0 ? Minus : row.delta > 0 ? ArrowUpRight : ArrowDownRight;
            const text = row.formatDelta
              ? row.formatDelta(row.delta)
              : `${row.delta >= 0 ? '+' : ''}${row.delta}`;
            return (
              <tr
                key={row.key}
                className="border-b border-ai-line transition-colors duration-150 last:border-0 hover:bg-ai-hover"
              >
                <td className="px-3 py-2 text-[13px] font-medium text-ai-ink">{row.label}</td>
                <td className="px-3 py-2 text-[12.5px] text-ai-ink-2 tabular-nums">{row.base}</td>
                <td className="px-3 py-2 text-[12.5px] text-ai-ink-2 tabular-nums">
                  {row.compare}
                </td>
                <td className="px-3 py-2">
                  <span
                    data-testid={`ai-diff-delta-${row.key}`}
                    className="inline-flex h-[22px] items-center gap-1 rounded-full px-2 text-[12px] font-medium tabular-nums"
                    style={{
                      color,
                      background:
                        tone === 'neutral'
                          ? 'var(--ai-inset)'
                          : `color-mix(in srgb, ${color} 12%, transparent)`,
                    }}
                  >
                    <Icon size={12} strokeWidth={2.5} aria-hidden />
                    {text}
                  </span>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export default AiDiffTable;
