import { describe, it, expect } from 'vitest';
import { cn } from '../cn';

describe('cn', () => {
  it('should merge tailwind classes correctly', () => {
    const result = cn('px-2 py-1', 'px-4');
    expect(result).toBe('py-1 px-4'); // tailwind-merge resolves px conflict
  });

  it('should handle conditional classes', () => {
    const isActive = true;
    const result = cn('base-class', isActive && 'active-class');
    expect(result).toContain('base-class');
    expect(result).toContain('active-class');
  });

  it('should filter out falsy values', () => {
    const result = cn('a', false, null, undefined, '', 'b');
    expect(result).toBe('a b');
  });

  it('should handle empty input', () => {
    expect(cn()).toBe('');
  });

  it('should merge conflicting colors', () => {
    const result = cn('text-red-500', 'text-blue-500');
    expect(result).toBe('text-blue-500');
  });
});
