import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiThinking } from '../AiThinking';

const rows = [
  { primary: '主创 生成 概念包', secondary: 'concept' },
  { primary: '管理 生成 世界观', mono: true },
];

describe('AiThinking', () => {
  it('working 时末行显示旋转指示，标题为传入 title', () => {
    render(<AiThinking title="当前执行轨迹" working={true} rows={rows} defaultExpanded />);
    expect(screen.getByText('当前执行轨迹')).toBeInTheDocument();
    expect(screen.getByTestId('ai-thinking-spinner')).toBeInTheDocument();
  });

  it('非 working 显示 doneTitle 且无 spinner', () => {
    render(
      <AiThinking
        title="当前执行轨迹"
        doneTitle="执行轨迹（已结束）"
        working={false}
        rows={rows}
        defaultExpanded
      />
    );
    expect(screen.getByText('执行轨迹（已结束）')).toBeInTheDocument();
    expect(screen.queryByTestId('ai-thinking-spinner')).not.toBeInTheDocument();
  });

  it('默认收起（0fr），点击标题展开（1fr）', () => {
    render(<AiThinking title="轨迹" working={false} rows={rows} />);
    const btn = screen.getByRole('button', { name: /轨迹/ });
    const trace = screen.getByTestId('ai-thinking-trace');
    expect(btn).toHaveAttribute('aria-expanded', 'false');
    expect(trace.style.gridTemplateRows).toBe('0fr');
    fireEvent.click(btn);
    expect(btn).toHaveAttribute('aria-expanded', 'true');
    expect(trace.style.gridTemplateRows).toBe('1fr');
  });

  it('href 行渲染为新窗口链接', () => {
    render(
      <AiThinking
        title="t"
        working={false}
        defaultExpanded
        rows={[{ primary: '设计文档', href: 'https://example.com/spec' }]}
      />
    );
    const link = screen.getByRole('link', { name: /设计文档/ });
    expect(link).toHaveAttribute('href', 'https://example.com/spec');
    expect(link).toHaveAttribute('target', '_blank');
  });

  it('add/del 行显示增删计数', () => {
    render(
      <AiThinking
        title="t"
        working={false}
        defaultExpanded
        rows={[{ primary: 'Edit', secondary: 'a.ts', mono: true, add: 74, del: 41 }]}
      />
    );
    expect(screen.getByText('+74')).toBeInTheDocument();
    expect(screen.getByText('−41')).toBeInTheDocument();
  });
});
