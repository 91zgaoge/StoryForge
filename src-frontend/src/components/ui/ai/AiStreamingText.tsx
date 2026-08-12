/**
 * AiStreamingText — 流式文字渲染（适配自 beautifului StreamingText）
 *
 * 受控组件：text 为截至目前已到达的完整文本（调用方逐步增长），
 * 每个新到达的单位以 stream-in 模糊入场动画出现（稳定 key → 旧节点不重播）；
 * done=false 时末尾显示闪烁光标。
 *
 * 中文感知分词：Intl.Segmenter('zh', { granularity: 'word' })，不支持时逐字回退。
 *
 * 未来工作（本应用暂无数据源，P1 未适配）：行内引用 citations、sources 面板、
 * follow-ups 操作区——见 .superpowers/sdd/reference/beautifului/StreamingText.tsx。
 */
import { useRef } from 'react';

export interface AiStreamingTextProps {
  text: string;
  done: boolean;
  className?: string;
}

interface SegmenterLike {
  segment(input: string): Iterable<{ segment: string }>;
}

/** 中文按词切分；无 Intl.Segmenter 的环境回退逐字符（保证多字节字符不被字节级切开） */
export function segmentStreamText(text: string): string[] {
  const Seg = (
    Intl as unknown as {
      Segmenter?: new (locale: string, opts: { granularity: 'word' }) => SegmenterLike;
    }
  ).Segmenter;
  if (Seg) {
    return Array.from(new Seg('zh', { granularity: 'word' }).segment(text), s => s.segment);
  }
  return Array.from(text);
}

export function AiStreamingText({ text, done, className }: AiStreamingTextProps) {
  // 流重置检测：text 不再以既有文本为前缀（新一轮生成）时 epoch +1，
  // key 变化强制全部 token 重新入场
  const epochRef = useRef(0);
  const prevTextRef = useRef('');
  if (prevTextRef.current && !text.startsWith(prevTextRef.current)) {
    epochRef.current += 1;
  }
  prevTextRef.current = text;

  const tokens = segmentStreamText(text);

  return (
    <span className={className} data-testid="ai-streaming-text">
      {tokens.map((token, i) => (
        <span
          key={`${epochRef.current}:${i}`}
          className="animate-stream-in inline [will-change:filter,opacity]"
        >
          {token}
        </span>
      ))}
      {!done && (
        <span
          aria-hidden
          data-testid="ai-streaming-cursor"
          className="animate-ai-blink ml-0.5 inline-block h-3 w-0.5 translate-y-0.5 rounded-full bg-ai-ink"
        />
      )}
    </span>
  );
}

export default AiStreamingText;
