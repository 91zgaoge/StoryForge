import { describe, it, expect } from 'vitest';

describe('测试框架冒烟', () => {
  it('vitest 应该正常工作', () => {
    expect(1 + 1).toBe(2);
  });

  it('jest-dom 匹配器应该可用', () => {
    const element = document.createElement('div');
    element.classList.add('active');
    expect(element).toHaveClass('active');
  });
});
