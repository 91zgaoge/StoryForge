/**
 * AiThinking — 可展开的执行轨迹（适配自 beautifului ThinkingState）
 *
 * 受控组件：rows 由调用方数据驱动，无任何内部演示时序（STAGES/useSequence 已剥离）。
 * 标题按钮展开/收起（grid-template-rows 0fr/1fr）；行以 fade-up 交错入场；
 * 左侧竖线随内容高度生长；working=true 时标题 shimmer、末行显示旋转指示。
 */
import { useLayoutEffect, useRef, useState } from 'react';

export interface AiThinkingRow {
  /** 稳定 key（可选）：数据源的稳定标识。不传时回退 `primary-index`，
   *  滚动窗口（如 slice(-N)）场景必须传，否则索引移位导致全部行重放动画。 */
  id?: string;
  primary: string;
  secondary?: string;
  mono?: boolean;
  add?: number;
  del?: number;
  href?: string;
}

export interface AiThinkingProps {
  title: string;
  doneTitle?: string;
  working: boolean;
  rows: AiThinkingRow[];
  defaultExpanded?: boolean;
}

export function AiThinking({
  title,
  doneTitle,
  working,
  rows,
  defaultExpanded = false,
}: AiThinkingProps) {
  const [manualExpanded, setManualExpanded] = useState<boolean | null>(null);
  const expanded = manualExpanded ?? defaultExpanded;
  const traceRef = useRef<HTMLDivElement>(null);
  const [lineHeight, setLineHeight] = useState(0);

  // 竖线随内容高度生长（行数增减/展开收起后重测）
  useLayoutEffect(() => {
    if (traceRef.current) setLineHeight(traceRef.current.offsetHeight);
  }, [rows.length, expanded]);

  return (
    <div className="flex w-full flex-col" data-testid="ai-thinking">
      {/* header */}
      <button
        type="button"
        aria-expanded={expanded}
        onClick={() => setManualExpanded(!expanded)}
        className="-mx-1.5 flex w-fit items-center gap-2 rounded px-1.5 py-1 transition-colors duration-100 hover:bg-ai-hover-2"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill={working ? 'var(--ai-ink-2)' : 'var(--ai-ink-3)'}
          aria-hidden
        >
          <path d="M12 2l2.4 7.2L22 12l-7.6 2.8L12 22l-2.4-7.2L2 12l7.6-2.8z" />
        </svg>
        {working ? (
          <span
            className="animate-shimmer-text bg-clip-text text-[13px] font-medium whitespace-nowrap text-transparent"
            style={{
              backgroundImage:
                'linear-gradient(90deg, var(--ai-ink-3) 35%, var(--ai-ink) 50%, var(--ai-ink-3) 65%)',
              backgroundSize: '200% 100%',
            }}
          >
            {title}
          </span>
        ) : (
          <span className="text-[13px] font-medium whitespace-nowrap text-ai-ink-2">
            {doneTitle ?? title}
          </span>
        )}
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="var(--ai-ink-3)"
          strokeWidth="2.2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="transition-transform duration-300"
          style={{ transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)' }}
          aria-hidden
        >
          <path d="M6 9l6 6 6-6" />
        </svg>
      </button>

      {/* expandable trace */}
      <div
        data-testid="ai-thinking-trace"
        className="grid transition-[grid-template-rows,opacity] duration-300"
        style={{
          gridTemplateRows: expanded ? '1fr' : '0fr',
          opacity: expanded ? 1 : 0,
          transitionTimingFunction: 'cubic-bezier(0.23, 1, 0.32, 1)',
        }}
      >
        <div className="overflow-hidden">
          <div className="relative mt-1 ml-[5px] pl-4">
            <span
              aria-hidden
              className="absolute left-[3px] w-px bg-ai-line"
              style={{
                top: -8,
                height: lineHeight ? lineHeight - 2 : 0,
                transition: 'height 500ms cubic-bezier(0.23,1,0.32,1)',
              }}
            />
            <div ref={traceRef} className="flex flex-col gap-1 py-1">
              {rows.map((row, i) => {
                const isLast = i === rows.length - 1;
                const content = (
                  <>
                    {working && isLast ? (
                      <span
                        data-testid="ai-thinking-spinner"
                        className="animate-ai-spin size-3 shrink-0 rounded-full border-[1.5px] border-ai-line-strong border-t-ai-ink-2"
                      />
                    ) : (
                      <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="var(--ai-ink-3)"
                        strokeWidth="2.5"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        className="shrink-0"
                        aria-hidden
                      >
                        <path d="M20 6L9 17l-5-5" />
                      </svg>
                    )}
                    <span
                      className={`min-w-0 truncate text-[12.5px] font-medium text-ai-ink ${
                        row.href ? 'underline decoration-ai-line-strong underline-offset-2' : ''
                      }`}
                    >
                      {row.primary}
                    </span>
                    {row.secondary && (
                      <span
                        className={`shrink-0 text-[11.5px] text-ai-ink-3 ${
                          row.mono ? 'font-mono' : ''
                        }`}
                      >
                        {row.secondary}
                      </span>
                    )}
                    {row.add !== undefined && (
                      <span className="shrink-0 font-mono text-[11px] tabular-nums">
                        <span className="text-ai-green">+{row.add}</span>{' '}
                        <span className="text-ai-red">−{row.del ?? 0}</span>
                      </span>
                    )}
                  </>
                );
                const rowClass =
                  'animate-ai-fade-up flex min-h-7 w-full items-center gap-2 rounded-[6px] px-1.5 py-0.5 text-left';
                // 交错入场：index 封顶 8 档，避免长列表尾部行延迟过大
                const style = { animationDelay: `${Math.min(i, 8) * 80}ms` };

                const rowKey = row.id ?? `${row.primary}-${i}`;
                if (row.href) {
                  return (
                    <a
                      key={rowKey}
                      href={row.href}
                      target="_blank"
                      rel="noreferrer"
                      className={`${rowClass} transition-colors duration-150 hover:bg-ai-hover`}
                      style={style}
                    >
                      {content}
                    </a>
                  );
                }
                return (
                  <div key={rowKey} className={rowClass} style={style}>
                    {content}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default AiThinking;
