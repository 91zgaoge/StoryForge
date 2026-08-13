import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AiDiffTable, type AiDiffRow } from '../AiDiffTable';

const rows: AiDiffRow[] = [
  { key: 'words', label: '字数', base: '42000', compare: '43500', delta: 1500 },
  {
    key: 'tokens',
    label: 'tokens',
    base: '8000',
    compare: '9100',
    delta: 1100,
    betterWhen: 'lower',
  },
  {
    key: 'weighted',
    label: '加权分',
    base: '0.80',
    compare: '0.80',
    delta: 0,
    formatDelta: d => d.toFixed(2),
  },
];

describe('AiDiffTable', () => {
  it('渲染标题与表头（指标/基准/对比/Δ）', () => {
    render(<AiDiffTable title="指标对比" rows={rows} />);
    expect(screen.getByText('指标对比')).toBeInTheDocument();
    for (const h of ['指标', '基准', '对比', 'Δ']) {
      expect(screen.getByText(h)).toBeInTheDocument();
    }
  });

  it('渲染每行 label/base/compare', () => {
    render(<AiDiffTable rows={rows} />);
    expect(screen.getByText('字数')).toBeInTheDocument();
    expect(screen.getByText('42000')).toBeInTheDocument();
    expect(screen.getByText('43500')).toBeInTheDocument();
  });

  it('delta 默认带 + 号；formatDelta 自定义优先', () => {
    render(<AiDiffTable rows={rows} />);
    expect(screen.getByText('+1500')).toBeInTheDocument();
    expect(screen.getByText('0.00')).toBeInTheDocument();
  });

  it('betterWhen=higher 且 delta>0 → 绿；delta<0 → 红', () => {
    render(<AiDiffTable rows={rows} />);
    const pill = screen.getByText('+1500').closest('span')!;
    expect(pill.style.color).toContain('var(--ai-green)');
  });

  it('betterWhen=lower 且 delta>0 → 红（恶化）', () => {
    render(<AiDiffTable rows={rows} />);
    const pill = screen.getByText('+1100').closest('span')!;
    expect(pill.style.color).toContain('var(--ai-red)');
  });

  it('delta=0 → 中性 ink-3', () => {
    render(<AiDiffTable rows={rows} />);
    const pill = screen.getByText('0.00').closest('span')!;
    expect(pill.style.color).toContain('var(--ai-ink-3)');
  });

  it('非零 delta 徽章底色为 color-mix tint（零扩令牌）', () => {
    render(<AiDiffTable rows={rows} />);
    const pill = screen.getByText('+1500').closest('span')!;
    expect(pill.style.background).toContain('color-mix(in srgb, var(--ai-green) 12%, transparent)');
  });
});
