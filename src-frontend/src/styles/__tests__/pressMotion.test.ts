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

  it('tokens.css 的 --transition-spring 为 0.5s 并在 reduced-motion 冻结', () => {
    const css = readFileSync(resolve(frontendRoot, 'src/styles/tokens.css'), 'utf-8');
    expect(css).toMatch(
      /--transition-spring:\s*0\.5s cubic-bezier\(0\.34,\s*1\.56,\s*0\.64,\s*1\)/
    );
    expect(css).toMatch(/prefers-reduced-motion[\s\S]*--transition-spring:\s*0\.01s linear/);
  });

  it('SceneEditor 与 Stories 页签走 duration-500 ease-spring', () => {
    const scene = readFileSync(resolve(frontendRoot, 'src/components/SceneEditor.tsx'), 'utf-8');
    const stories = readFileSync(resolve(frontendRoot, 'src/pages/Stories.tsx'), 'utf-8');
    expect(scene).toMatch(/duration-500 ease-spring/);
    expect(stories).toMatch(/duration-500 ease-spring/);
  });

  it('弹簧接线在 reduced-motion 下冻结为 0.01s linear', () => {
    const files = [
      'src/components/ui/Panel.tsx',
      'src/components/SceneEditor.tsx',
      'src/pages/Stories.tsx',
      'src/pages/settings/ModelCard.tsx',
    ];
    for (const rel of files) {
      const src = readFileSync(resolve(frontendRoot, rel), 'utf-8');
      expect(src, rel).toMatch(/duration-500 ease-spring/);
      expect(src, rel).toMatch(/motion-reduce:duration-\[0\.01s\]/);
      expect(src, rel).toMatch(/motion-reduce:ease-linear/);
    }
  });
});
