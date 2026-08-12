import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AiStreamingText, segmentStreamText } from '../AiStreamingText';

describe('segmentStreamText', () => {
  it('中文按词切分（非逐字）', () => {
    const tokens = segmentStreamText('他走进雨里');
    expect(tokens.join('')).toBe('他走进雨里');
    expect(tokens.length).toBeGreaterThan(1);
    expect(tokens.length).toBeLessThan(5); // 词级切分，不会拆成 5 个单字
  });

  it('空串返回空数组', () => {
    expect(segmentStreamText('')).toEqual([]);
  });
});

describe('AiStreamingText', () => {
  it('渲染全部已到达文本，未完成时显示闪烁光标', () => {
    render(<AiStreamingText text="他走进雨里" done={false} />);
    expect(screen.getByTestId('ai-streaming-text').textContent).toBe('他走进雨里');
    expect(screen.getByTestId('ai-streaming-cursor')).toBeInTheDocument();
  });

  it('done 后光标消失，文本完整', () => {
    const { rerender } = render(<AiStreamingText text="你好" done={false} />);
    rerender(<AiStreamingText text="你好世界" done={true} />);
    expect(screen.getByTestId('ai-streaming-text').textContent).toBe('你好世界');
    expect(screen.queryByTestId('ai-streaming-cursor')).not.toBeInTheDocument();
  });

  it('增量到达时旧 token 节点复用（动画不重播），新 token 追加', () => {
    const { rerender } = render(<AiStreamingText text="你好" done={false} />);
    const first = screen.getByTestId('ai-streaming-text').querySelector('span');
    expect(first).not.toBeNull();
    rerender(<AiStreamingText text="你好世界" done={false} />);
    const spans = screen.getByTestId('ai-streaming-text').querySelectorAll('span');
    expect(spans[0]).toBe(first); // 同一 DOM 节点 → CSS 动画不重播
    expect(spans.length).toBeGreaterThan(1);
  });
});
