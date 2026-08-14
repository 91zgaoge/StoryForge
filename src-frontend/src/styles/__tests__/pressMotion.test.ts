import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const frontendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');

describe('press 运动令牌', () => {
  it('tokens.css 与 frontstage.css 都定义 --transition-press 并在 reduced-motion 冻结', () => {
    for (const rel of ['src/styles/tokens.css', 'src/frontstage/styles/frontstage.css']) {
      const css = readFileSync(resolve(frontendRoot, rel), 'utf-8');
      expect(css, rel).toMatch(
        /--transition-press:\s*0\.3s cubic-bezier\(0\.32,\s*0\.72,\s*0,\s*1\)/
      );
      expect(css, rel).toMatch(/prefers-reduced-motion[\s\S]*--transition-press:\s*0\.01s linear/);
    }
  });

  it('tailwind 注册 ease-press', () => {
    const tw = readFileSync(resolve(frontendRoot, 'tailwind.config.js'), 'utf-8');
    expect(tw).toMatch(/press:\s*'cubic-bezier\(0\.32,\s*0\.72,\s*0,\s*1\)'/);
  });

  it('Button 在 reduced-motion 下冻结 press scale', () => {
    const src = readFileSync(resolve(frontendRoot, 'src/components/ui/Button.tsx'), 'utf-8');
    expect(src).toContain('motion-reduce:active:scale-100');
  });
});
