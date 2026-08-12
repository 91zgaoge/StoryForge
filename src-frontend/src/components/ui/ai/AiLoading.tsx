/**
 * AiLoading — 像素格点加载器（适配自 beautifului LoadingState）
 *
 * 受控组件：label 由调用方提供；elapsed 计时从 startedAt（默认挂载时刻）起算。
 * variant：drive（方块 chevron 波前）/ dots（圆点波前）/ orbit（彗星绕边，中心格不点亮）。
 * 动画 = Task 1 注册的 CSS keyframes（pixel-on / shimmer-text）；
 * prefers-reduced-motion 下动画冻结（tokens.css / frontstage.css 冻结块），计时仍走。
 */
import { useEffect, useState } from 'react';

export interface AiLoadingProps {
  label: string;
  variant?: 'drive' | 'dots' | 'orbit';
  startedAt?: number;
}

/* chevron 波前：格点按列+行距依次点亮，650ms 周期短于扫描，两个波前同时在场 */
const chevron = Array.from({ length: 9 }, (_, i) => {
  const r = Math.floor(i / 3);
  const c = i % 3;
  return (c + Math.abs(r - 1)) * 90;
});

/* orbit：彗星绕格外圈一周，中心格（index 4，不在外圈序列中）保持暗态 */
const ORBIT_ORDER = [0, 1, 2, 5, 8, 7, 6, 3];
const orbit = Array.from({ length: 9 }, (_, i) => {
  const k = ORBIT_ORDER.indexOf(i);
  return k === -1 ? null : k * 110;
});

const PATTERNS = {
  drive: { delays: chevron, dur: 650, round: false },
  dots: { delays: chevron, dur: 650, round: true },
  orbit: { delays: orbit, dur: 950, round: false },
} as const;

function formatElapsed(totalSeconds: number): string {
  if (totalSeconds < 60) return `${totalSeconds.toFixed(1)}s`;
  return `${Math.floor(totalSeconds / 60)}m ${(totalSeconds % 60).toFixed(1)}s`;
}

export function AiLoading({ label, variant = 'drive', startedAt }: AiLoadingProps) {
  const [start, setStart] = useState(() => startedAt ?? Date.now());
  const [now, setNow] = useState(() => Date.now());

  // 新一轮任务传入新 startedAt 时归零重计
  useEffect(() => {
    if (startedAt !== undefined) setStart(startedAt);
  }, [startedAt]);

  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 100);
    return () => clearInterval(t);
  }, []);

  const elapsed = formatElapsed(Math.max(0, (now - start) / 1000));
  const { delays, dur, round } = PATTERNS[variant];

  return (
    <div className="flex w-fit items-center gap-2.5" data-testid="ai-loading">
      <span aria-hidden className="grid grid-cols-[repeat(3,4px)] gap-[1.5px]">
        {delays.map((d, i) => (
          <span
            key={i}
            className={`size-[4px] bg-ai-ink ${round ? 'rounded-full' : 'rounded-[1px]'} ${
              d === null ? '' : 'animate-pixel-on'
            }`}
            style={{
              opacity: d === null ? 0.07 : 0.15,
              animationDelay: d === null ? undefined : `${d}ms`,
              animationDuration: `${dur}ms`,
            }}
          />
        ))}
      </span>
      <span
        className="animate-shimmer-text bg-clip-text text-[13px] font-medium text-transparent"
        style={{
          backgroundImage:
            'linear-gradient(90deg, var(--ai-ink-3) 35%, var(--ai-ink) 50%, var(--ai-ink-3) 65%)',
          backgroundSize: '200% 100%',
        }}
      >
        {label}
      </span>
      <span
        className="font-mono text-[12px] text-ai-ink-3 tabular-nums"
        data-testid="ai-loading-elapsed"
      >
        {elapsed}
      </span>
    </div>
  );
}

export default AiLoading;
