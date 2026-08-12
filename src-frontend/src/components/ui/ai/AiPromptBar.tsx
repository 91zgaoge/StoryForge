/**
 * AiPromptBar — AI 指令输入条（适配自 beautifului PromptBar）
 *
 * 受控组件：value/onChange/onSend 由调用方持有。
 * 保留：自动增高 textarea；@ 数据源与 / 命令菜单（滑动高亮 + ↑↓/Enter/Tab/Esc
 * 键盘导航 + IME isComposing 守卫）；模型选择菜单（传入 models 才渲染，
 * 切换模型触发 ai-sweep 一次性渐变扫光——glimm canvas 的纯 CSS 替代）。
 * 移除：AUTO_STEPS 自动演示、听写、品牌 SVG、附件（无 attachments prop）、
 * inline/expanded 布局重排。
 */
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { ArrowUp, Check, ChevronDown, Plus } from 'lucide-react';

export interface AiPromptSource {
  key: string;
  name: string;
  desc: string;
}

export interface AiPromptCommand {
  key: string;
  name: string; // 含 / 前缀，如 /自动续写
  desc: string;
}

export interface AiPromptModel {
  key: string;
  name: string;
  tag?: string;
}

export interface AiPromptBarProps {
  value: string;
  onChange: (v: string) => void;
  onSend: () => void;
  placeholder?: string;
  disabled?: boolean;
  sources?: AiPromptSource[];
  commands?: AiPromptCommand[];
  models?: AiPromptModel[];
  model?: string;
  onModelChange?: (key: string) => void;
  onKeyDown?: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  onFocus?: () => void;
  trailingAction?: React.ReactNode;
}

/* 正在输入的最后一个 @词 或 /词（\w 之外补 CJK 区间，中文数据源名可续打筛选） */
function parseToken(draft: string): { kind: 'at' | 'slash'; query: string; start: number } | null {
  const match = /(^|\s)([@/])([\w一-鿿-]*)$/.exec(draft);
  if (!match) return null;
  return {
    kind: match[2] === '@' ? 'at' : 'slash',
    query: match[3].toLowerCase(),
    start: match.index + match[1].length,
  };
}

export function AiPromptBar({
  value,
  onChange,
  onSend,
  placeholder = '输入任意指令…',
  disabled = false,
  sources,
  commands,
  models,
  model,
  onModelChange,
  onKeyDown,
  onFocus,
  trailingAction,
}: AiPromptBarProps) {
  const [dismissed, setDismissed] = useState(false);
  const [plusOpen, setPlusOpen] = useState(false);
  const [modelOpen, setModelOpen] = useState(false);
  const [active, setActive] = useState(0);
  const [engaged, setEngaged] = useState(false);
  const [rowBox, setRowBox] = useState<{ top: number; height: number } | null>(null);
  const [modelHovered, setModelHovered] = useState<number | null>(null);
  const [modelBox, setModelBox] = useState<{ top: number; height: number } | null>(null);
  const [sweeping, setSweeping] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const rowRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const modelRowRefs = useRef<(HTMLButtonElement | null)[]>([]);

  const hasSources = (sources?.length ?? 0) > 0;
  const hasCommands = (commands?.length ?? 0) > 0;
  const hasModels = (models?.length ?? 0) > 0;

  const token = dismissed ? null : parseToken(value);
  const menu: 'at' | 'slash' | null = plusOpen
    ? 'at'
    : token?.kind === 'at' && hasSources
      ? 'at'
      : token?.kind === 'slash' && hasCommands
        ? 'slash'
        : null;
  const query = plusOpen ? '' : (token?.query ?? '');

  const rows: { key: string; name: string; desc: string }[] =
    menu === 'at'
      ? (sources ?? []).filter(s => s.name.toLowerCase().includes(query))
      : menu === 'slash'
        ? (commands ?? []).filter(c => c.name.slice(1).toLowerCase().includes(query))
        : [];

  useEffect(() => {
    setActive(0);
    setEngaged(false);
  }, [menu, query]);

  /* 单一滑动高亮块（gliding highlight）跟随 active 行，而非每行各自切换背景 */
  useLayoutEffect(() => {
    const target = rowRefs.current[active];
    if (target) setRowBox({ top: target.offsetTop, height: target.offsetHeight });
  }, [menu, query, active, rows.length]);

  const modelIndex = (models ?? []).findIndex(m => m.key === model);
  useLayoutEffect(() => {
    if (!modelOpen) return;
    const target = modelRowRefs.current[modelHovered ?? modelIndex];
    if (target) setModelBox({ top: target.offsetTop, height: target.offsetHeight });
  }, [modelOpen, modelHovered, modelIndex]);

  useEffect(() => {
    if (!modelOpen) setModelHovered(null);
  }, [modelOpen]);

  /* 自动增高：28px 起，160px 封顶 */
  useLayoutEffect(() => {
    const input = inputRef.current;
    if (!input) return;
    input.style.height = '0px';
    const contentHeight = input.scrollHeight;
    input.style.height = `${Math.min(Math.max(contentHeight, 28), 160)}px`;
    input.style.overflowY = contentHeight > 160 ? 'auto' : 'hidden';
  }, [value]);

  /* ai-sweep 一次性扫光（模型切换时）。jsdom 无 matchMedia，可选链兜底 */
  const fireSweep = () => {
    if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return;
    setSweeping(true);
  };

  const selectModel = (key: string) => {
    onModelChange?.(key);
    setModelOpen(false);
    if (key !== model) fireSweep();
    inputRef.current?.focus();
  };

  /* 选中菜单项：@源 插入 @名字；/命令 插入命令名（去 / 前缀，与直接在输入框
   * 打「自动续写」等指令的既有路由一致——统一由后端意图识别处理） */
  const pick = (row: { key: string; name: string }) => {
    const base = token ? value.slice(0, token.start) : value;
    onChange(menu === 'at' ? `${base}@${row.name} ` : `${base}${row.name.replace(/^\//, '')} `);
    setPlusOpen(false);
    setDismissed(false);
    inputRef.current?.focus();
  };

  const canSend = value.trim().length > 0 && !disabled;
  const send = () => {
    if (!canSend) return;
    onSend();
    setPlusOpen(false);
    setModelOpen(false);
    setDismissed(false);
  };

  const currentModel = (models ?? []).find(m => m.key === model);

  return (
    <div className="relative" data-testid="ai-prompt-bar">
      {/* ── @ / 命令菜单（从输入条上沿向上生长） ── */}
      {menu && (
        <div
          data-testid="ai-prompt-menu"
          onMouseLeave={() => setEngaged(false)}
          className="animate-pop-in absolute inset-x-0 bottom-full z-10 mb-2 rounded-[10px] border border-ai-line bg-ai-surface p-1 shadow-float"
          style={{ transformOrigin: 'bottom center' }}
        >
          <span
            aria-hidden
            className="pointer-events-none absolute inset-x-1 rounded-[6px] bg-ai-hover"
            style={{
              top: rowBox?.top ?? 0,
              height: rowBox?.height ?? 0,
              opacity: rowBox && engaged && rows.length > 0 ? 1 : 0,
              transition:
                'top 220ms cubic-bezier(0.23,1,0.32,1), height 220ms cubic-bezier(0.23,1,0.32,1), opacity 150ms ease',
            }}
          />
          {rows.map((row, i) => (
            <button
              key={row.key}
              type="button"
              ref={el => {
                rowRefs.current[i] = el;
              }}
              onMouseDown={e => e.preventDefault()}
              onMouseEnter={() => {
                setActive(i);
                setEngaged(true);
              }}
              onClick={() => pick(row)}
              className="relative z-10 flex h-9 w-full items-center gap-2.5 rounded-[6px] px-2 text-left"
              data-testid={`ai-prompt-menu-row-${row.key}`}
            >
              <span className="shrink-0 text-[12.5px] font-medium text-ai-ink">{row.name}</span>
              <span className="min-w-0 flex-1 truncate text-[12px] text-ai-ink-3">{row.desc}</span>
            </button>
          ))}
          {rows.length === 0 && (
            <div className="flex h-9 items-center px-2 text-[12px] text-ai-ink-3">
              无匹配「{query}」
            </div>
          )}
          <div className="mt-1 border-t border-ai-line px-2 pt-1.5 pb-1 text-[11px] text-ai-ink-3">
            {menu === 'at' ? '输入以筛选数据源' : '输入以筛选命令'}
          </div>
        </div>
      )}

      {/* ── 模型菜单 ── */}
      {modelOpen && hasModels && (
        <div
          data-testid="ai-model-menu"
          onMouseLeave={() => setModelHovered(null)}
          className="animate-pop-in absolute right-0 bottom-full z-10 mb-2 w-44 rounded-[10px] border border-ai-line bg-ai-surface p-1 shadow-float"
          style={{ transformOrigin: 'bottom right' }}
        >
          <span
            aria-hidden
            className="pointer-events-none absolute inset-x-1 rounded-[6px] bg-ai-hover"
            style={{
              top: modelBox?.top ?? 0,
              height: modelBox?.height ?? 0,
              opacity: modelBox && modelHovered !== null ? 1 : 0,
              transition:
                'top 220ms cubic-bezier(0.23,1,0.32,1), height 220ms cubic-bezier(0.23,1,0.32,1), opacity 150ms ease',
            }}
          />
          {(models ?? []).map((m, i) => (
            <button
              key={m.key}
              type="button"
              ref={el => {
                modelRowRefs.current[i] = el;
              }}
              onMouseDown={e => e.preventDefault()}
              onMouseEnter={() => setModelHovered(i)}
              onClick={() => selectModel(m.key)}
              className="relative z-10 flex h-8 w-full items-center gap-2 rounded-[6px] px-2 text-left"
            >
              <span className="min-w-0 flex-1 truncate text-[12.5px] font-medium text-ai-ink">
                {m.name}
              </span>
              {m.tag && <span className="shrink-0 text-[11px] text-ai-ink-3">{m.tag}</span>}
              <span className={`shrink-0 text-ai-ink ${m.key === model ? '' : 'invisible'}`}>
                <Check className="size-3.5" />
              </span>
            </button>
          ))}
        </div>
      )}

      {/* ── 输入条本体 ── */}
      <div className="relative isolate flex items-end gap-1 overflow-hidden rounded-[10px] border border-ai-line bg-ai-surface p-1.5 transition-colors duration-150 focus-within:border-ai-line-strong">
        {/* ai-sweep 扫光覆盖层（模型切换时播放一次；950ms 与 keyframes 同步） */}
        {sweeping && (
          <span
            aria-hidden
            data-testid="ai-sweep-overlay"
            onAnimationEnd={() => setSweeping(false)}
            className="animate-ai-sweep pointer-events-none absolute inset-y-0 left-0 -z-10 w-1/2"
            style={{
              background:
                'linear-gradient(105deg, transparent 0%, var(--ai-accent-tint) 30%, var(--ai-accent) 50%, var(--ai-accent-tint) 70%, transparent 100%)',
            }}
          />
        )}

        {hasSources && (
          <button
            type="button"
            aria-label="添加数据源"
            aria-expanded={plusOpen}
            onClick={() => {
              setModelOpen(false);
              setPlusOpen(c => !c);
              inputRef.current?.focus();
            }}
            className={`flex size-7 shrink-0 items-center justify-center rounded-[8px] transition-colors duration-150 hover:bg-ai-hover hover:text-ai-ink ${
              plusOpen ? 'bg-ai-hover text-ai-ink' : 'text-ai-ink-3'
            }`}
          >
            <Plus className="size-4" />
          </button>
        )}

        <textarea
          ref={inputRef}
          rows={1}
          value={value}
          onChange={e => {
            onChange(e.target.value);
            setDismissed(false);
            setPlusOpen(false);
          }}
          onKeyDown={e => {
            // 1) 菜单打开时的键盘导航（↑↓ 移动 / Enter·Tab 选中）
            if (menu && rows.length > 0) {
              if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                e.preventDefault();
                setEngaged(true);
                setActive(c => (c + (e.key === 'ArrowDown' ? 1 : rows.length - 1)) % rows.length);
                return;
              }
              if ((e.key === 'Enter' && !e.shiftKey) || e.key === 'Tab') {
                // IME 组合输入中 Enter 是上屏键，不得劫持为菜单选中（中文输入法）
                if (e.nativeEvent.isComposing) return;
                e.preventDefault();
                pick(rows[active]);
                return;
              }
            }
            // 2) Esc：菜单/模型菜单打开时仅关闭，不透传父级
            if (e.key === 'Escape' && (menu || modelOpen)) {
              setDismissed(true);
              setPlusOpen(false);
              setModelOpen(false);
              return;
            }
            // 3) 透传父级（幽灵提示 ↑↓/→/Esc 等）；父级 preventDefault 的键不再走内部发送
            onKeyDown?.(e);
            if (e.defaultPrevented) return;
            // 4) Enter 发送（IME 组合输入中除外——中文输入法上屏 Enter 不触发）
            if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
              e.preventDefault();
              send();
            }
          }}
          onFocus={onFocus}
          placeholder={placeholder}
          aria-label="AI 指令输入"
          disabled={disabled}
          className="min-h-7 min-w-0 flex-1 resize-none bg-transparent px-1 py-[5px] text-[13px] leading-[18px] text-ai-ink outline-none [overflow-wrap:anywhere] placeholder:text-ai-ink-3 disabled:opacity-50"
        />

        {hasModels && (
          <button
            type="button"
            aria-expanded={modelOpen}
            aria-label="选择模型"
            onClick={() => {
              setPlusOpen(false);
              setModelOpen(c => !c);
            }}
            className="flex h-7 shrink-0 items-center gap-1 rounded-[8px] px-1.5 text-[12px] font-medium text-ai-ink-2 transition-colors duration-150 hover:bg-ai-hover hover:text-ai-ink"
          >
            {currentModel?.name ?? '选择模型'}
            <ChevronDown className="size-3 text-ai-ink-3" />
          </button>
        )}

        {trailingAction ?? (
          <button
            type="button"
            title="发送"
            aria-label="发送"
            disabled={!canSend}
            onClick={send}
            className="flex size-7 shrink-0 items-center justify-center rounded-[8px] transition-[background-color,color,transform] duration-200 enabled:active:scale-[0.94] disabled:cursor-not-allowed"
            style={{
              background: canSend ? 'var(--ai-ink)' : 'var(--ai-line-strong)',
              color: canSend ? 'var(--ai-surface)' : 'var(--ai-ink-2)',
            }}
          >
            <ArrowUp className="size-4" />
          </button>
        )}
      </div>
    </div>
  );
}

export default AiPromptBar;
