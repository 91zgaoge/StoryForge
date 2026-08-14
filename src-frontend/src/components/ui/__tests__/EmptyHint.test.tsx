import { it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EmptyHint } from '../EmptyHint';

it('渲染说明文字且无 pulse', () => {
  const { container } = render(<EmptyHint>暂无活动</EmptyHint>);
  expect(screen.getByText('暂无活动')).toBeInTheDocument();
  expect(container.firstChild).toHaveClass('text-ai-ink-3');
  expect((container.firstChild as HTMLElement).className).not.toMatch(/animate-pulse/);
});
