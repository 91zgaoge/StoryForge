import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiContextCards, type AiContextCardItem } from '../AiContextCards';

const items: AiContextCardItem[] = [
  {
    key: 'a',
    title: '合同红线',
    meta: '290 字',
    body: '冷链接入前必须完成资质核验。',
    source: { label: 'worldbuilding.md', badge: 'MD', tone: 'green' },
  },
  { key: 'b', title: '角色', source: { label: '未注入', badge: '✗', tone: 'neutral' } },
];

describe('AiContextCards', () => {
  it('渲染标题与计数徽章', () => {
    render(<AiContextCards title="上下文槽位" count={7} items={items} />);
    expect(screen.getByText('上下文槽位')).toBeInTheDocument();
    expect(screen.getByText('7')).toBeInTheDocument();
  });

  it('渲染每张卡的标题 / meta / body', () => {
    render(<AiContextCards title="t" items={items} />);
    expect(screen.getByText('合同红线')).toBeInTheDocument();
    expect(screen.getByText('290 字')).toBeInTheDocument();
    expect(screen.getByText('冷链接入前必须完成资质核验。')).toBeInTheDocument();
    expect(screen.getByText('角色')).toBeInTheDocument();
  });

  it('无 count 时不渲染计数徽章；无 body 的卡不渲染正文段', () => {
    render(<AiContextCards title="t" items={items} />);
    // items[1] 无 body：只有一段正文
    expect(screen.getAllByText(/冷链接入/)).toHaveLength(1);
  });

  it('source chip 渲染 badge/label 且错峰 animationDelay 递增', () => {
    render(<AiContextCards title="t" items={items} />);
    const chipA = screen.getByText('worldbuilding.md').closest('span')!;
    const chipB = screen.getByText('未注入').closest('span')!;
    expect(screen.getByText('MD')).toBeInTheDocument();
    expect(chipA.style.animationDelay).toBe('400ms');
    expect(chipB.style.animationDelay).toBe('480ms');
  });

  it('tone 映射到 --ai-* 变量（neutral → --ai-ink-3）', () => {
    render(<AiContextCards title="t" items={items} />);
    const badge = screen.getByText('✗');
    expect(badge.style.background).toContain('var(--ai-ink-3)');
  });

  it('onItemActivate 时卡片可点', () => {
    const onItemActivate = vi.fn();
    render(<AiContextCards title="t" items={items} onItemActivate={onItemActivate} />);
    fireEvent.click(screen.getByText('合同红线'));
    expect(onItemActivate).toHaveBeenCalledTimes(1);
    expect(onItemActivate.mock.calls[0][0].key).toBe('a');
  });
});
