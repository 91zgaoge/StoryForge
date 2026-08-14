import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Button } from '../Button';

describe('Button', () => {
  it('renders paper variant', () => {
    render(<Button variant="paper">写</Button>);
    expect(screen.getByRole('button', { name: '写' })).toHaveClass('bg-terracotta/15');
  });

  it('renders cinema variant', () => {
    render(<Button variant="cinema">保存</Button>);
    expect(screen.getByRole('button', { name: '保存' })).toHaveClass('bg-cinema-gold/15');
  });

  it('press 用 scale-98 而非 scale-95', () => {
    render(<Button>确定</Button>);
    const cls = screen.getByRole('button').className;
    expect(cls).toMatch(/active:scale-\[0\.98\]/);
    expect(cls).not.toMatch(/active:scale-95/);
    expect(cls).toMatch(/ease-press/);
    expect(cls).toMatch(/motion-reduce:active:scale-100/);
  });

  it('paper 与 cinema 主按钮是淡彩不是实心高饱和填充', () => {
    const { rerender } = render(<Button variant="paper">写</Button>);
    // [^-] 会误伤 /15（`/` 也是 [^-]）。合同是：禁止实心 bg-terracotta / bg-cinema-gold，允许淡彩 /15。
    expect(screen.getByRole('button').className).not.toMatch(/bg-terracotta(?!\/)/);
    rerender(<Button variant="cinema">做</Button>);
    expect(screen.getByRole('button').className).not.toMatch(/bg-cinema-gold(?!\/)/);
  });
});
