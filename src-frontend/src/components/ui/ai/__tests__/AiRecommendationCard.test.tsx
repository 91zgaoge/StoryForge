import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiRecommendationCard, type AiRecommendationOption } from '../AiRecommendationCard';

const twoOptions: AiRecommendationOption[] = [
  { key: 'a', body: <p>方案 A 正文</p>, short: '方案 A', signal: 3, label: '高置信' },
  { key: 'b', body: <p>方案 B 正文</p>, short: '方案 B', signal: 1, label: '需复核' },
];

describe('AiRecommendationCard', () => {
  it('渲染标题与当前选项 body / 信号标签', () => {
    render(
      <AiRecommendationCard title="段落 3：时序矛盾" options={twoOptions} onAccept={() => {}} />
    );
    expect(screen.getByText('段落 3：时序矛盾')).toBeInTheDocument();
    expect(screen.getByText('方案 A 正文')).toBeInTheDocument();
    expect(screen.getAllByText('高置信').length).toBeGreaterThan(0);
  });

  it('signal 渲染 3 根信号条', () => {
    const { container } = render(
      <AiRecommendationCard title="t" options={twoOptions} onAccept={() => {}} />
    );
    expect(container.querySelectorAll('[data-testid="ai-rec-meter"] > span')).toHaveLength(3);
  });

  it('点击接受调用 onAccept(key)，点击拒绝调用 onReject(key)', () => {
    const onAccept = vi.fn();
    const onReject = vi.fn();
    render(
      <AiRecommendationCard
        title="t"
        options={twoOptions}
        onAccept={onAccept}
        onReject={onReject}
      />
    );
    fireEvent.click(screen.getByRole('button', { name: '接受' }));
    expect(onAccept).toHaveBeenCalledWith('a');
    fireEvent.click(screen.getByRole('button', { name: '拒绝' }));
    expect(onReject).toHaveBeenCalledWith('a');
  });

  it('Alternatives 抽屉切换到方案 B 后正文与接受键随之更新', () => {
    const onAccept = vi.fn();
    render(<AiRecommendationCard title="t" options={twoOptions} onAccept={onAccept} />);
    fireEvent.click(screen.getByRole('button', { name: '备选' }));
    fireEvent.click(screen.getByRole('button', { name: /方案 B/ }));
    expect(screen.getByText('方案 B 正文')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '接受' }));
    expect(onAccept).toHaveBeenCalledWith('b');
  });

  it('status=accepted 时接受按钮变为已接受并禁用；status=rejected 时显示已拒绝', () => {
    const { rerender } = render(
      <AiRecommendationCard title="t" options={twoOptions} status="accepted" onAccept={() => {}} />
    );
    expect(screen.getByRole('button', { name: '已接受' })).toBeDisabled();
    rerender(
      <AiRecommendationCard title="t" options={twoOptions} status="rejected" onAccept={() => {}} />
    );
    expect(screen.getByText('已拒绝')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '接受' })).not.toBeInTheDocument();
  });

  it('单选项时不渲染 Alternatives 按钮', () => {
    render(<AiRecommendationCard title="t" options={[twoOptions[0]]} onAccept={() => {}} />);
    expect(screen.queryByRole('button', { name: '备选' })).not.toBeInTheDocument();
  });
});
