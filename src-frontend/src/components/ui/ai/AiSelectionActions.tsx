/**
 * AiSelectionActions — 划词 AI 操作浮条（适配自 beautifului SelectionActions）
 *
 * 受控约定：selectedText/phase/resultText/回调全部由宿主（RichTextEditor）提供；
 * 剥离参考实现：演示文案 LEAD/PICKED/REWRITE、自运行 thinking→streaming 定时器、
 * iconoir-react 依赖（映射 lucide-react，见计划映射表）、Explain/Tone/Grammar 动作、
 * color-mix 选区高亮（编辑器原生选区自带高亮）。
 *
 * 移植说明：
 * - atoms/Shimmer 源码缺失 → 按 AiThinking.tsx:66-73 的 animate-shimmer-text 渐变模式复刻；
 * - atoms/StreamText 源码缺失 → 内嵌私有 SelectionStreamText（复用 segmentStreamText +
 *   animate-stream-in，错峰 animationDelay 模拟流式），不动 P1 受控版 AiStreamingText；
 * - 定位保留参考的 selection.getClientRects + ResizeObserver + rAF + 宽度动画（纯 DOM）；
 * - mousedown preventDefault 防选区塌陷（同 EditorContextMenu.tsx:95-98 既有模式）；
 * - shadow-overlay → shadow-float（需宿主窗口定义 --shadow-float，幕前见 frontstage.css）。
 */
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { ArrowUp, Check, ChevronRight, RefreshCw, Scissors, Sparkles, Type, X } from 'lucide-react';
import { segmentStreamText } from './AiStreamingText';
import { cn } from '@/utils/cn';

export type AiSelectionActionKey = 'polish' | 'expand' | 'rewrite' | 'custom';
export type AiSelectionPhase = 'idle' | 'thinking' | 'result';

export interface AiSelectionActionsProps {
  /** 定位宿主（相对定位的编辑器容器）；浮条坐标相对它计算 */
  containerRef: React.RefObject<HTMLElement | null>;
  /** 空字符串 = 不渲染 */
  selectedText: string;
  phase: AiSelectionPhase;
  resultText?: string;
  onRun: (action: AiSelectionActionKey, customInstruction?: string) => void;
  onAccept: () => void;
  onDiscard: () => void;
}

const ACTIONS: {
  key: Exclude<AiSelectionActionKey, 'custom'>;
  label: string;
  icon: typeof Sparkles;
}[] = [
  { key: 'polish', label: '润色', icon: Sparkles },
  { key: 'expand', label: '扩写', icon: Type },
  { key: 'rewrite', label: '改写', icon: Scissors },
];

const BUSY_LABEL: Record<AiSelectionActionKey, string> = {
  polish: '润色中',
  expand: '扩写中',
  rewrite: '改写中',
  custom: '处理中',
};

const control =
  'inline-flex h-7 shrink-0 items-center gap-1 rounded-full px-2.5 text-[12px] font-normal text-ai-ink transition-[background-color,color,transform] duration-300 ease-press hover:bg-ai-hover active:scale-[0.98] motion-reduce:transition-none motion-reduce:active:scale-100';

const primary =
  'inline-flex h-7 shrink-0 items-center gap-1 rounded-md bg-[color-mix(in_oklch,var(--ai-accent)_18%,transparent)] px-2.5 text-[12.5px] font-normal text-ai-accent-ink transition-[opacity,transform] duration-300 ease-press hover:opacity-90 active:scale-[0.98] motion-reduce:transition-none motion-reduce:active:scale-100';

/** 内嵌私有流式显现（源码 atoms/StreamText 缺失；错峰 delay 模拟流式，无定时器） */
function SelectionStreamText({ text, onProgress }: { text: string; onProgress?: () => void }) {
  const tokens = segmentStreamText(text);
  useEffect(() => {
    onProgress?.();
  }, [tokens.length, onProgress]);
  return (
    <span data-testid="ai-selection-stream">
      {tokens.map((token, i) => (
        <span
          key={i}
          className="animate-stream-in inline [will-change:filter,opacity]"
          style={{ animationDelay: `${i * 45}ms` }}
        >
          {token}
        </span>
      ))}
    </span>
  );
}

/** atoms/Shimmer 缺失：复刻 AiThinking.tsx:66-73 的 shimmer 渐变文字 */
function ShimmerLabel({ children }: { children: React.ReactNode }) {
  return (
    <span
      className="animate-shimmer-text bg-clip-text text-transparent"
      data-testid="ai-selection-busy"
      style={{
        backgroundImage:
          'linear-gradient(90deg, var(--ai-ink-3) 35%, var(--ai-ink) 50%, var(--ai-ink-3) 65%)',
        backgroundSize: '200% 100%',
      }}
    >
      {children}
    </span>
  );
}

export function AiSelectionActions({
  containerRef,
  selectedText,
  phase,
  resultText,
  onRun,
  onAccept,
  onDiscard,
}: AiSelectionActionsProps) {
  const [expanded, setExpanded] = useState(false);
  const [prompt, setPrompt] = useState('');
  const [lastAction, setLastAction] = useState<AiSelectionActionKey>('polish');
  // 重试时携带上次自定义指令（M2：否则 custom 重试静默无效）
  const [lastCustomInstruction, setLastCustomInstruction] = useState('');
  const [anchor, setAnchor] = useState<{ x: number; y: number } | null>(null);

  const barRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef<ReturnType<typeof requestAnimationFrame> | null>(null);
  const prevPhaseRef = useRef<AiSelectionPhase>('idle');
  const lastWidthRef = useRef(0);
  const widthAnimationRef = useRef<Animation | null>(null);

  const visible = selectedText.trim().length > 0;
  const hasPrompt = prompt.trim().length > 0;

  /* 贴在最末一个选区行下方，横向对准整个选区中心；rAF 批合测量 */
  const place = useCallback(() => {
    if (frameRef.current !== null) cancelAnimationFrame(frameRef.current);
    frameRef.current = requestAnimationFrame(() => {
      const host = containerRef.current;
      const selection = window.getSelection();
      if (!host || !selection || selection.rangeCount === 0) return;
      const range = selection.getRangeAt(0);
      const bounds = range.getBoundingClientRect();
      const lines = Array.from(range.getClientRects());
      // tsconfig lib 未达 es2022，不用 Array.prototype.at
      const lastLine = lines[lines.length - 1];
      if (!lastLine || (bounds.width === 0 && bounds.height === 0)) return;
      const hostBounds = host.getBoundingClientRect();
      const next = {
        x: Math.round(bounds.left - hostBounds.left + bounds.width / 2),
        y: Math.round(lastLine.bottom - hostBounds.top + 8),
      };
      setAnchor(current =>
        current && current.x === next.x && current.y === next.y ? current : next
      );
    });
  }, [containerRef]);

  useLayoutEffect(() => {
    if (visible) place();
  }, [visible, phase, place]);

  useEffect(() => {
    if (!visible) return;
    const host = containerRef.current;
    if (!host) return;
    const observer = new ResizeObserver(place);
    observer.observe(host);
    // 编辑器滚动容器在 host 内部；scroll 不冒泡，捕获阶段监听
    host.addEventListener('scroll', place, true);
    window.addEventListener('resize', place);
    return () => {
      observer.disconnect();
      host.removeEventListener('scroll', place, true);
      window.removeEventListener('resize', place);
      if (frameRef.current !== null) cancelAnimationFrame(frameRef.current);
    };
  }, [visible, place, containerRef]);

  /* phase 切换时从上一个渲染宽度动画到新的固有宽度（Web Animations API） */
  useLayoutEffect(() => {
    const bar = barRef.current;
    const content = contentRef.current;
    if (!bar || !content || !visible) return;
    const nextWidth = Math.ceil(content.getBoundingClientRect().width) + 8;
    const previousWidth = lastWidthRef.current || Math.ceil(bar.getBoundingClientRect().width);
    if (prevPhaseRef.current !== phase && Math.abs(nextWidth - previousWidth) > 1) {
      widthAnimationRef.current?.cancel();
      const animation = bar.animate(
        [{ width: `${previousWidth}px` }, { width: `${nextWidth}px` }],
        { duration: 320, easing: 'cubic-bezier(0.23,1,0.32,1)' }
      );
      widthAnimationRef.current = animation;
      animation.onfinish = () => {
        lastWidthRef.current = nextWidth;
        widthAnimationRef.current = null;
      };
    } else {
      lastWidthRef.current = nextWidth;
    }
    prevPhaseRef.current = phase;
  }, [phase, visible]);

  useEffect(() => {
    const content = contentRef.current;
    if (!content) return;
    const observer = new ResizeObserver(() => {
      if (widthAnimationRef.current?.playState === 'running') return;
      lastWidthRef.current = Math.ceil(content.getBoundingClientRect().width) + 8;
    });
    observer.observe(content);
    return () => {
      observer.disconnect();
      widthAnimationRef.current?.cancel();
    };
  }, []);

  if (!visible) return null;

  const run = (action: AiSelectionActionKey, customInstruction?: string) => {
    setLastAction(action);
    if (action === 'custom') setLastCustomInstruction(customInstruction ?? '');
    setExpanded(false);
    onRun(action, customInstruction);
  };

  const submitCustom = () => {
    const text = prompt.trim();
    if (!text) return;
    run('custom', text);
  };

  const shown = anchor !== null;

  return (
    <div
      className="absolute top-0 left-0 z-10"
      style={{
        transform: `translate3d(${anchor?.x ?? 0}px, ${anchor?.y ?? 0}px, 0) translateX(-50%)`,
        transition: 'transform 320ms cubic-bezier(0.77,0,0.175,1), opacity 180ms ease-out',
        opacity: shown ? 1 : 0,
        pointerEvents: shown ? 'auto' : 'none',
        willChange: 'transform',
      }}
    >
      <div
        ref={barRef}
        data-testid="ai-selection-actions"
        onMouseDown={e => {
          e.preventDefault();
          e.stopPropagation();
          // preventDefault 会取消 input 的默认焦点转移（真实浏览器中永远无法聚焦），
          // 手动恢复；选区仍因 preventDefault 不塌陷
          if (e.target instanceof HTMLInputElement) e.target.focus();
        }}
        className={cn(
          'flex h-9 w-fit max-w-[calc(100vw-48px)] items-center justify-center gap-0.5 overflow-hidden rounded-full border border-ai-line bg-ai-surface p-1 text-ai-ink shadow-float antialiased',
          shown && 'animate-pop-in'
        )}
      >
        <div ref={contentRef} className="flex w-fit shrink-0 items-center justify-center gap-0.5">
          {phase === 'thinking' && (
            <span className="inline-flex h-7 items-center gap-1.5 px-2.5 text-[12.5px] whitespace-nowrap text-ai-ink-2">
              <span className="animate-ai-spin size-3 shrink-0 rounded-full border-[1.5px] border-ai-line-strong border-t-ai-ink-2" />
              <ShimmerLabel>{BUSY_LABEL[lastAction]}…</ShimmerLabel>
            </span>
          )}

          {phase === 'result' && (
            <>
              <button type="button" onClick={onAccept} className={primary}>
                <Check size={14} strokeWidth={1.8} aria-hidden />
                保留
              </button>
              <button type="button" onClick={onDiscard} className={control}>
                <X size={14} strokeWidth={1.8} aria-hidden />
                放弃
              </button>
              <span className="mx-0.5 h-4 w-px shrink-0 bg-ai-line" />
              <button
                type="button"
                aria-label="重试"
                onClick={() =>
                  run(lastAction, lastAction === 'custom' ? lastCustomInstruction : undefined)
                }
                className="flex size-7 shrink-0 items-center justify-center rounded-full text-ai-ink-3 transition-[background-color,color,transform] duration-150 hover:bg-ai-hover-2 hover:text-ai-ink-2 active:scale-[0.96]"
              >
                <RefreshCw size={14} strokeWidth={1.8} aria-hidden />
              </button>
            </>
          )}

          {phase === 'idle' && (
            <>
              {/* 自定义指令输入（有内容时吃掉动作区宽度） */}
              <div
                className="flex min-w-0 items-center overflow-hidden transition-[max-width,opacity,transform] duration-[400ms]"
                style={{
                  maxWidth: expanded ? 0 : 145,
                  opacity: expanded ? 0 : 1,
                  transform: expanded ? 'translateX(-8px)' : 'translateX(0)',
                  transitionTimingFunction: 'cubic-bezier(0.23,1,0.32,1)',
                }}
              >
                <input
                  value={prompt}
                  onChange={e => setPrompt(e.target.value)}
                  onKeyDown={e => {
                    // IME 组词中 Enter 是上屏键，不得提交（同 AiPromptBar isComposing 守卫先例）
                    if (e.key === 'Enter' && !e.nativeEvent.isComposing) {
                      e.preventDefault();
                      submitCustom();
                    }
                  }}
                  aria-label="描述修改要求"
                  placeholder="描述修改要求…"
                  className="h-7 w-[145px] bg-transparent pr-2.5 pl-3 text-[12.5px] text-ai-ink placeholder:text-ai-ink-3"
                />
              </div>

              <div
                className="flex min-w-0 items-center gap-0.5 overflow-hidden transition-[max-width,opacity,transform] duration-[400ms]"
                style={{
                  maxWidth: hasPrompt ? 0 : expanded ? 300 : 150,
                  opacity: hasPrompt ? 0 : 1,
                  transform: hasPrompt ? 'translateX(-8px)' : 'translateX(0)',
                  transitionTimingFunction: 'cubic-bezier(0.23,1,0.32,1)',
                }}
              >
                {!expanded && <span className="mx-1 h-4 w-px shrink-0 bg-ai-line-strong" />}
                {ACTIONS.slice(0, expanded ? 3 : 2).map(({ key, label, icon: Icon }) => (
                  <button key={key} type="button" onClick={() => run(key)} className={control}>
                    <Icon size={14} strokeWidth={1.8} aria-hidden />
                    {label}
                  </button>
                ))}
                <span className="mx-0.5 h-4 w-px shrink-0 bg-ai-line" />
                <button
                  type="button"
                  aria-label={expanded ? '收起操作' : '展开更多操作'}
                  aria-expanded={expanded}
                  onClick={() => setExpanded(v => !v)}
                  className="flex size-7 shrink-0 items-center justify-center rounded-full text-ai-ink transition-[background-color,transform] duration-200 hover:bg-ai-hover active:scale-[0.96]"
                >
                  <ChevronRight
                    size={14}
                    strokeWidth={1.8}
                    aria-hidden
                    className="transition-transform duration-[400ms]"
                    style={{
                      transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
                      transitionTimingFunction: 'cubic-bezier(0.23,1,0.32,1)',
                    }}
                  />
                </button>
              </div>

              {/* 有自定义文本时的发送钮 */}
              <div
                className="flex min-w-0 items-center overflow-hidden transition-[max-width,opacity,transform] duration-[400ms]"
                style={{
                  maxWidth: hasPrompt ? 30 : 0,
                  opacity: hasPrompt ? 1 : 0,
                  transform: hasPrompt ? 'scale(1)' : 'scale(0.88)',
                  transitionTimingFunction: 'cubic-bezier(0.23,1,0.32,1)',
                }}
              >
                <button
                  type="button"
                  aria-label="发送修改指令"
                  onClick={submitCustom}
                  className="flex size-7 shrink-0 items-center justify-center rounded-md bg-[color-mix(in_oklch,var(--ai-accent)_18%,transparent)] text-ai-accent-ink transition-[opacity,transform] duration-300 ease-press hover:opacity-90 enabled:active:scale-[0.98] motion-reduce:transition-none motion-reduce:active:scale-100"
                >
                  <ArrowUp size={16} strokeWidth={2.4} aria-hidden />
                </button>
              </div>
            </>
          )}
        </div>
      </div>

      {/* 结果面板：浮条下方流式显现改写结果（保留/放弃在浮条内） */}
      {phase === 'result' && resultText && (
        <div className="absolute top-full left-1/2 mt-2 w-[min(420px,calc(100vw-48px))] -translate-x-1/2 rounded-[12px] border border-ai-line bg-ai-surface p-3 shadow-float">
          <p className="text-[13px] leading-relaxed text-ai-ink">
            <SelectionStreamText text={resultText} onProgress={place} />
          </p>
        </div>
      )}
    </div>
  );
}

export default AiSelectionActions;
