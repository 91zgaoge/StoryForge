/**
 * AiRecommendationCard — AI 建议确认卡（适配自 beautifului RecommendationCard）
 *
 * 受控约定：title/options/status/回调全部由调用方提供；剥离参考实现的
 * OPTIONS 演示数据与内部 accepted state（改为 status prop：pending/accepted/rejected）。
 * selected/open 为纯视图状态保留内部（同 P1 AiThinking manualExpanded 先例）。
 * 移植说明：tone/ctaStyle 字符串改为按 signal 推导（3→--ai-green，1-2→--ai-orange，
 * 0→--ai-ink-3）；primitive-card-pad/footer → p-3 / px-3 py-2；shadow-card →
 * border-ai-line；shadow-btn 删除（用 border 替代）；缓动 cubic-bezier 保留内联。
 * 拒绝按钮为本组件相对参考实现的新增（级联改写场景需要 接受/拒绝 双动作）。
 */
import { useState } from 'react';
import { cn } from '@/utils/cn';

export interface AiRecommendationOption {
  key: string;
  body: React.ReactNode;
  short: string;
  signal: 0 | 1 | 2 | 3;
  label: string;
}

export interface AiRecommendationCardProps {
  title: string;
  options: AiRecommendationOption[];
  status?: 'pending' | 'accepted' | 'rejected';
  acceptLabel?: string;
  rejectLabel?: string;
  alternativesLabel?: string;
  onAccept: (key: string) => void;
  onReject?: (key: string) => void;
  className?: string;
}

function signalTone(signal: number): string {
  if (signal >= 3) return 'var(--ai-green)';
  if (signal >= 1) return 'var(--ai-orange)';
  return 'var(--ai-ink-3)';
}

function Meter({ signal }: { signal: number }) {
  return (
    <span className="flex items-end gap-0.5" data-testid="ai-rec-meter" aria-hidden>
      {[0, 1, 2].map(bar => (
        <span
          key={bar}
          className="w-1 rounded-full transition-colors duration-300"
          style={{
            height: 10,
            background: bar < signal ? signalTone(signal) : 'var(--ai-line-strong)',
          }}
        />
      ))}
    </span>
  );
}

export function AiRecommendationCard({
  title,
  options,
  status = 'pending',
  acceptLabel = '接受',
  rejectLabel = '拒绝',
  alternativesLabel = '备选',
  onAccept,
  onReject,
  className,
}: AiRecommendationCardProps) {
  const [selected, setSelected] = useState(0);
  const [open, setOpen] = useState(false);

  const active = options[Math.min(selected, options.length - 1)];
  const others = options.map((o, i) => ({ o, i })).filter(({ i }) => i !== selected);

  return (
    <div
      className={cn(
        'w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface',
        className
      )}
      data-testid="ai-recommendation-card"
    >
      <div className="p-3">
        <span className="text-[13px] font-semibold text-ai-ink">{title}</span>
        <div
          key={active.key}
          className="animate-fade-in mt-1.5 text-[13px] leading-relaxed text-ai-ink-2"
        >
          {active.body}
        </div>
      </div>

      {/* alternatives 抽屉（仅多选项时出现） */}
      {options.length > 1 && (
        <div
          className="grid transition-[grid-template-rows,opacity] duration-300"
          style={{
            gridTemplateRows: open ? '1fr' : '0fr',
            opacity: open ? 1 : 0,
            transitionTimingFunction: 'cubic-bezier(0.16, 1, 0.3, 1)',
          }}
        >
          <div className="overflow-hidden">
            <div className="border-t border-ai-line bg-ai-inset px-2 py-2">
              <p className="px-1.5 pb-1 text-[11px] font-medium text-ai-ink-3">其他选项</p>
              {open &&
                others.map(({ o, i }) => (
                  <button
                    key={o.key}
                    type="button"
                    onClick={() => {
                      setSelected(i);
                      setOpen(false);
                    }}
                    className="flex w-full items-center gap-2.5 rounded-[8px] px-1.5 py-1.5 text-left transition-colors duration-100 hover:bg-ai-hover"
                  >
                    <Meter signal={o.signal} />
                    <span className="min-w-0 flex-1 truncate text-[12.5px] text-ai-ink">
                      {o.short}
                    </span>
                    <span className="shrink-0 text-[11px] text-ai-ink-3">{o.label}</span>
                  </button>
                ))}
            </div>
          </div>
        </div>
      )}

      <div className="flex items-center justify-between gap-3 border-t border-ai-line bg-ai-inset px-3 py-2">
        <span className="flex items-center gap-2">
          <Meter signal={active.signal} />
          <span className="text-[12.5px] font-medium text-ai-ink-2">{active.label}</span>
        </span>

        <span className="-mr-0.5 flex items-center gap-2">
          {status === 'pending' && options.length > 1 && (
            <button
              type="button"
              aria-expanded={open}
              onClick={() => setOpen(current => !current)}
              className={cn(
                'h-7 rounded-[8px] px-2.5 text-[12.5px] font-medium transition-[background-color,transform] duration-100 active:scale-[0.96]',
                open ? 'bg-ai-hover text-ai-ink' : 'bg-ai-surface text-ai-ink hover:bg-ai-hover'
              )}
            >
              {alternativesLabel}
            </button>
          )}
          {status === 'pending' && onReject && (
            <button
              type="button"
              onClick={() => onReject(active.key)}
              className="h-7 rounded-[8px] border border-ai-line bg-ai-surface px-2.5 text-[12.5px] font-medium text-ai-ink-2 transition-[background-color,transform] duration-100 hover:bg-ai-hover active:scale-[0.96]"
            >
              {rejectLabel}
            </button>
          )}
          {status === 'rejected' && (
            <span className="text-[12.5px] font-medium text-ai-ink-3">已拒绝</span>
          )}
          {status !== 'rejected' && (
            <button
              type="button"
              disabled={status !== 'pending'}
              onClick={() => onAccept(active.key)}
              className={cn(
                'h-7 rounded-[8px] px-3 text-[12.5px] font-medium transition-[background-color,transform] duration-150 active:scale-[0.96] disabled:cursor-default',
                status === 'accepted'
                  ? 'bg-ai-green text-white'
                  : 'bg-ai-ink text-ai-surface hover:opacity-90'
              )}
            >
              {status === 'accepted' ? '已接受' : acceptLabel}
            </button>
          )}
        </span>
      </div>
    </div>
  );
}

export default AiRecommendationCard;
