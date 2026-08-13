/**
 * AiApprovalCard — 人工审批/选项卡（适配自 beautifului ApprovalCard）
 *
 * 受控组件：questions 由调用方提供。一次一个问题；ring-dot 分页器 +
 * 上一题/下一题；radio 单选 480ms 后自动前进（最后一题自动提交）；
 * allowCustom 时提供「自定义回答…」输入；提交后显示绿色对勾「已提交」。
 * answers 以 question.key -> option.key[] 上报（自定义回答以文本为数组元素）。
 */
import { useEffect, useRef, useState } from 'react';
import { ArrowUp, Check, ChevronLeft, ChevronRight, X } from 'lucide-react';

export interface AiApprovalOption {
  key: string;
  label: string;
  description?: string;
}

export interface AiApprovalQuestion {
  key: string;
  title: string;
  type: 'radio' | 'check';
  options: AiApprovalOption[];
  allowCustom?: boolean;
}

export interface AiApprovalCardProps {
  questions: AiApprovalQuestion[];
  onSubmit: (answers: Record<string, string[]>) => void;
  onDismiss?: () => void;
  submitLabel?: string;
}

export function AiApprovalCard({
  questions,
  onSubmit,
  onDismiss,
  submitLabel = '提交',
}: AiApprovalCardProps) {
  const [qi, setQi] = useState(0);
  const [answers, setAnswers] = useState<Record<string, string[]>>({});
  const [custom, setCustom] = useState<Record<string, string>>({});
  const [sent, setSent] = useState(false);
  // 480ms 自动前进/提交定时器：卸载或再次点击前取消，防止卸载后提交与双击重复提交
  const autoTimerRef = useRef<number | null>(null);
  useEffect(
    () => () => {
      if (autoTimerRef.current !== null) window.clearTimeout(autoTimerRef.current);
    },
    []
  );
  const question = questions[qi];
  const last = qi === questions.length - 1;
  const selected = answers[question.key] ?? [];
  const hasAnswer = selected.length > 0 || Boolean(custom[question.key]?.trim());

  /* 汇总答案：每题优先 option key 列表，无选中则取自定义文本 */
  const buildAnswers = (extra?: Record<string, string[]>) => {
    const merged = { ...answers, ...extra };
    const out: Record<string, string[]> = {};
    for (const q of questions) {
      const picked = merged[q.key] ?? [];
      const text = custom[q.key]?.trim();
      if (picked.length > 0) out[q.key] = picked;
      else if (text) out[q.key] = [text];
    }
    return out;
  };

  const submit = (finalAnswers?: Record<string, string[]>) => {
    if (sent) return; // 防重复提交（480ms 自动提交与手动点击竞争）
    setSent(true);
    onSubmit(finalAnswers ?? buildAnswers());
  };

  /* 手动翻题：取消待触发的自动前进/提交，避免与手动导航竞争 */
  const gotoQuestion = (next: number | ((c: number) => number)) => {
    if (autoTimerRef.current !== null) {
      window.clearTimeout(autoTimerRef.current);
      autoTimerRef.current = null;
    }
    setQi(next);
  };

  const toggle = (optionKey: string) => {
    if (question.type === 'radio') {
      if (autoTimerRef.current !== null) return; // 已有待触发的自动前进/提交——忽略连击
      const next = { ...answers, [question.key]: [optionKey] };
      setAnswers(next);
      setCustom(c => ({ ...c, [question.key]: '' }));
      // 单选自动前进；最后一题自动提交（extra 透传避免读到旧 state）
      autoTimerRef.current = window.setTimeout(() => {
        autoTimerRef.current = null;
        if (last) submit(buildAnswers({ [question.key]: [optionKey] }));
        else setQi(c => Math.min(questions.length - 1, c + 1));
      }, 480);
    } else {
      setAnswers(c => {
        const picked = c[question.key] ?? [];
        return {
          ...c,
          [question.key]: picked.includes(optionKey)
            ? picked.filter(k => k !== optionKey)
            : [...picked, optionKey],
        };
      });
    }
  };

  return (
    <div
      className="w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface shadow-float"
      data-testid="ai-approval-card"
    >
      {sent ? (
        <div className="flex h-36 flex-col items-center justify-center gap-2">
          <span className="animate-pop-in flex size-6 items-center justify-center rounded-full bg-ai-green text-ai-on-accent">
            <Check className="size-3.5" strokeWidth={3} />
          </span>
          <span className="animate-ai-fade-up text-[13px] font-medium text-ai-ink">已提交</span>
        </div>
      ) : (
        <div key={question.key} className="animate-ai-fade-up p-4">
          <div className="flex items-start justify-between gap-3">
            <span className="text-[13px] font-medium text-ai-ink">{question.title}</span>
            {onDismiss && (
              <button
                type="button"
                aria-label="关闭"
                onClick={onDismiss}
                className="flex size-5 shrink-0 items-center justify-center rounded-[5px] text-ai-ink-3 transition-colors duration-100 hover:bg-ai-hover hover:text-ai-ink"
              >
                <X className="size-3.5" />
              </button>
            )}
          </div>
          <div className="mt-2 flex flex-col gap-0.5">
            {question.options.map(option => {
              const on = selected.includes(option.key);
              return (
                <button
                  key={option.key}
                  type="button"
                  aria-pressed={on}
                  onClick={() => toggle(option.key)}
                  className="-mx-1.5 flex items-center gap-2 rounded-[8px] px-1.5 py-1.5 text-left transition-colors duration-100 hover:bg-ai-hover"
                >
                  <span
                    className={`flex size-4 shrink-0 items-center justify-center transition-colors duration-200 ${
                      question.type === 'radio' ? 'rounded-full' : 'rounded-[5px]'
                    } ${
                      on
                        ? 'bg-ai-ink text-ai-surface'
                        : 'shadow-[inset_0_0_0_1.5px_var(--ai-line-strong)] text-transparent'
                    }`}
                  >
                    {question.type === 'radio' ? (
                      <span
                        className="size-1.5 rounded-full bg-ai-surface transition-transform duration-200"
                        style={{ transform: on ? 'scale(1)' : 'scale(0)' }}
                      />
                    ) : (
                      <Check className="size-3" strokeWidth={3} />
                    )}
                  </span>
                  <span className="min-w-0">
                    <span
                      className={`block text-[13px] transition-colors duration-200 ${
                        on ? 'text-ai-ink' : 'text-ai-ink-2'
                      }`}
                    >
                      {option.label}
                    </span>
                    {option.description && (
                      <span className="block truncate text-[11.5px] text-ai-ink-3">
                        {option.description}
                      </span>
                    )}
                  </span>
                </button>
              );
            })}
            {question.allowCustom && (
              <label className="-mx-1.5 flex items-center gap-2 rounded-[8px] px-1.5 py-1 transition-colors duration-100 focus-within:bg-ai-hover hover:bg-ai-hover">
                <span aria-hidden className="size-4 shrink-0" />
                <input
                  value={custom[question.key] ?? ''}
                  onChange={e => {
                    setCustom(c => ({ ...c, [question.key]: e.target.value }));
                    if (question.type === 'radio') {
                      setAnswers(c => ({ ...c, [question.key]: [] }));
                    }
                  }}
                  placeholder="自定义回答…"
                  aria-label="自定义回答"
                  className="min-w-0 flex-1 bg-transparent text-[13px] text-ai-ink outline-none placeholder:text-ai-ink-3"
                />
              </label>
            )}
          </div>
        </div>
      )}

      {/* footer — ring-dot 分页器 + 提交箭头 */}
      <div className="flex items-center justify-between border-t border-ai-line px-4 py-2">
        <span className="flex items-center gap-2">
          <button
            type="button"
            aria-label="上一题"
            disabled={qi === 0 || sent}
            onClick={() => gotoQuestion(c => Math.max(0, c - 1))}
            className="flex size-6 items-center justify-center rounded-[5px] text-ai-ink-3 transition-colors duration-100 enabled:hover:bg-ai-hover enabled:hover:text-ai-ink-2 disabled:opacity-35"
          >
            <ChevronLeft className="size-3.5" />
          </button>
          <span className="flex items-center gap-1">
            {questions.map((q, i) => (
              <button
                key={q.key}
                type="button"
                aria-label={`第 ${i + 1} 题`}
                aria-current={i === qi && !sent ? 'step' : undefined}
                disabled={sent}
                onClick={() => gotoQuestion(i)}
                className="rounded-full transition-all duration-300 disabled:cursor-default"
                style={
                  i === qi && !sent
                    ? { width: 9, height: 9, border: '2.5px solid var(--ai-ink)' }
                    : sent || i < qi
                      ? { width: 7, height: 7, background: 'var(--ai-ink-3)' }
                      : { width: 7, height: 7, border: '1.5px solid var(--ai-ink-3)' }
                }
              />
            ))}
          </span>
          <button
            type="button"
            aria-label="下一题"
            disabled={last || sent}
            onClick={() => gotoQuestion(c => Math.min(questions.length - 1, c + 1))}
            className="flex size-6 items-center justify-center rounded-[5px] text-ai-ink-3 transition-colors duration-100 enabled:hover:bg-ai-hover enabled:hover:text-ai-ink-2 disabled:opacity-35"
          >
            <ChevronRight className="size-3.5" />
          </button>
        </span>
        {!sent && (
          <button
            type="button"
            aria-label={last ? submitLabel : '继续'}
            disabled={!hasAnswer}
            onClick={() => (last ? submit() : gotoQuestion(c => c + 1))}
            className="-mr-0.5 flex size-7 items-center justify-center rounded-[8px] transition-[background-color,color,transform] duration-200 enabled:active:scale-[0.96] disabled:cursor-not-allowed"
            style={{
              background: hasAnswer ? 'var(--ai-ink)' : 'var(--ai-field)',
              color: hasAnswer ? 'var(--ai-surface)' : 'var(--ai-ink-3)',
            }}
          >
            <ArrowUp className="size-3.5" strokeWidth={2.5} />
          </button>
        )}
      </div>
    </div>
  );
}

export default AiApprovalCard;
