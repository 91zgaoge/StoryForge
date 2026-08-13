/**
 * AiCodeBlock — 只读代码块（适配自 beautifului CodeBlock）
 *
 * 受控约定：code/language/title 全部由调用方提供；剥离参考实现的
 * LINE_MS/HOLD_MS 逐行流式演示循环、LINES/RAW 演示数据与 Tok 语法着色
 * （本批集成点均为 JSON/文本 dump，无着色需求）；复制反馈 copied 1500ms
 * 翻转为交互式 UI 状态保留（非自运行演示），带卸载清理。
 * 移植说明：rounded-card → rounded-[12px]、shadow-card → border-ai-line、
 * primitive-card-bar → px-3 py-2、fade-up 裸 keyframes → animate-ai-fade-up
 * （逐行错峰 animationDelay，封顶 20 行防大日志块动画节点爆炸）；
 * max-w-95 演示宽度限制删除（宽度由宿主决定）。
 */
import { useEffect, useRef, useState } from 'react';
import { Check, Copy } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiCodeBlockProps {
  code: string;
  language?: string;
  title?: string;
  /** 行号（默认关） */
  lineNumbers?: boolean;
  /** px；超出滚动 */
  maxHeight?: number;
  copyable?: boolean;
  className?: string;
}

export function AiCodeBlock({
  code,
  language,
  title,
  lineNumbers = false,
  maxHeight,
  copyable = true,
  className,
}: AiCodeBlockProps) {
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    },
    []
  );

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => setCopied(false), 1500);
    } catch {
      // 剪贴板不可用（权限/非安全上下文）时静默
    }
  };

  const lines = code.split('\n');

  return (
    <div
      className={cn(
        'w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface',
        className
      )}
      data-testid="ai-code-block"
    >
      <div className="flex items-center justify-between gap-2 border-b border-ai-line px-3 py-2">
        <span className="flex min-w-0 items-baseline gap-2">
          {title && (
            <span className="truncate font-mono text-[12px] font-medium text-ai-ink">{title}</span>
          )}
          {language && <span className="shrink-0 text-[11.5px] text-ai-ink-3">{language}</span>}
        </span>
        {copyable && (
          <button
            type="button"
            aria-label={copied ? '已复制' : '复制'}
            onClick={copy}
            className={cn(
              'flex h-6 shrink-0 items-center gap-1 rounded-[6px] px-1.5 text-[11.5px] font-medium transition-colors duration-100 hover:bg-ai-hover',
              copied ? 'text-ai-green' : 'text-ai-ink-3 hover:text-ai-ink'
            )}
          >
            {copied ? (
              <Check size={10} strokeWidth={3} aria-hidden />
            ) : (
              <Copy size={10} strokeWidth={2} aria-hidden />
            )}
            {copied ? '已复制' : '复制'}
          </button>
        )}
      </div>

      <pre
        className="overflow-auto bg-ai-inset px-3 py-2.5 font-mono text-[11.5px] leading-[1.7] text-ai-ink-2"
        style={maxHeight ? { maxHeight } : undefined}
      >
        {lineNumbers ? (
          lines.map((line, i) => (
            <div
              key={i}
              className="animate-ai-fade-up flex"
              style={{ animationDelay: `${Math.min(i, 20) * 25}ms` }}
            >
              <span
                data-line-no={i + 1}
                className="w-8 shrink-0 pr-2.5 text-right text-[10.5px] leading-[1.86] text-ai-ink-3/60 select-none"
              >
                {i + 1}
              </span>
              <span className="min-w-0 whitespace-pre-wrap break-all">{line}</span>
            </div>
          ))
        ) : (
          <code className="whitespace-pre-wrap break-all">{code}</code>
        )}
      </pre>
    </div>
  );
}

export default AiCodeBlock;
