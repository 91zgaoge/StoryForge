import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Hash } from 'lucide-react';
import { AiInsightCards, type AiInsightCardItem } from '../AiInsightCards';

const items: AiInsightCardItem[] = [
  {
    key: 'calls',
    label: '总调用次数',
    value: '128',
    sub: '本故事: 40',
    tone: 'accent',
    icon: <Hash size={20} />,
  },
  {
    key: 'tokens',
    label: '总 Token 数',
    value: '12.4K',
    tone: 'neutral',
    series: [3, 5, 4, 8, 6],
    seriesLabel: 'token 趋势',
  },
  { key: 'cost', label: '预估费用', value: '$0.42', tone: 'green' },
];

describe('AiInsightCards', () => {
  it('渲染每卡的 label/value/sub', () => {
    render(<AiInsightCards items={items} />);
    expect(screen.getByText('总调用次数')).toBeInTheDocument();
    expect(screen.getByText('128')).toBeInTheDocument();
    expect(screen.getByText('本故事: 40')).toBeInTheDocument();
    expect(screen.getByText('$0.42')).toBeInTheDocument();
  });

  it('有 series 的卡渲染 SVG 折线（polyline 路径 + 末点），无 series 不渲染图表', () => {
    render(<AiInsightCards items={items} />);
    const charts = screen.getAllByTestId('ai-insight-chart');
    expect(charts).toHaveLength(1);
    expect(charts[0].querySelector('path[stroke]')).toBeTruthy();
    expect(charts[0].querySelector('circle')).toBeTruthy();
    expect(charts[0]).toHaveAttribute('aria-label', 'token 趋势');
  });

  it('series 折线颜色映射 tone（neutral → --ai-ink-3）', () => {
    render(<AiInsightCards items={items} />);
    const path = screen.getByTestId('ai-insight-chart').querySelector('path[stroke]')!;
    expect(path.getAttribute('stroke')).toBe('var(--ai-ink-3)');
  });

  it('series=[1,1] 零跨度数据不 NaN（y 坐标有效）', () => {
    render(<AiInsightCards items={[{ key: 'f', label: 'l', value: 'v', series: [1, 1] }]} />);
    const path = screen.getByTestId('ai-insight-chart').querySelector('path[stroke]')!;
    expect(path.getAttribute('d')).not.toContain('NaN');
  });

  it('columns=3 时使用三列响应式类；卡入场错峰 animationDelay 递增', () => {
    render(<AiInsightCards items={items} columns={3} />);
    const grid = screen.getByTestId('ai-insight-cards');
    expect(grid.className).toContain('md:grid-cols-3');
    const cards = grid.children;
    expect((cards[0] as HTMLElement).style.animationDelay).toBe('0ms');
    expect((cards[1] as HTMLElement).style.animationDelay).toBe('80ms');
  });

  it('sub 色调映射 tone（green → --ai-green）', () => {
    render(
      <AiInsightCards items={[{ key: 'c', label: 'l', value: 'v', sub: '改善', tone: 'green' }]} />
    );
    expect(screen.getByText('改善').style.color).toContain('var(--ai-green)');
  });
});
