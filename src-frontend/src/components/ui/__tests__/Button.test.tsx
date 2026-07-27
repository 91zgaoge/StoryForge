import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Button } from '../Button';

describe('Button', () => {
  it('renders paper variant', () => {
    render(<Button variant="paper">写</Button>);
    expect(screen.getByRole('button', { name: '写' })).toHaveClass('bg-terracotta');
  });

  it('renders cinema variant', () => {
    render(<Button variant="cinema">保存</Button>);
    expect(screen.getByRole('button', { name: '保存' })).toHaveClass('bg-cinema-gold');
  });
});
