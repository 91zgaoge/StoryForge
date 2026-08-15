import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { colorThemes } from '../colorThemes';
import { writingStyles } from '@/frontstage/config/writingStyles';

const frontendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../..');

describe('幕前墨色抬升', () => {
  it('frontstage.css 默认 --ink 为朱红墨', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).toMatch(/--ink:\s*rgb\(25,\s*14,\s*9\)/);
  });

  it('幕前金标与强调同色（印 = 锚色）', () => {
    expect(colorThemes.zhuhong.gold).toBe(colorThemes.zhuhong.terracotta);
    expect(colorThemes.zhuhong.terracotta).toBe('rgb(223,67,19)');
  });

  it('幕前 --shadow-float 不用纯黑 rgba(0,0,0', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    const block = css.match(/--shadow-float:\s*([^;]+);/)?.[1] ?? '';
    expect(block).not.toMatch(/rgba\(0,\s*0,\s*0/);
  });

  it('朱红默认纸来自熟宣交付色', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    const tokens = readFileSync(resolve(frontendRoot, 'src/styles/tokens.css'), 'utf-8');
    expect(css).toMatch(/--parchment:\s*rgb\(255,\s*239,\s*232\)/);
    expect(css).toMatch(/--parchment-dark:\s*rgb\(254,\s*225,\s*217\)/);
    expect(colorThemes.zhuhong.parchment).toBe('rgb(255,239,232)');
    expect(colorThemes.qunqing.parchment).toBe('rgb(236,246,255)');
    expect(writingStyles.default.paperColor).toMatch(/oklch\(96\.5%\s+0\.012\s+95\)/);
    expect(tokens).toMatch(/--paper-50:\s*#f6f4eb/);
    expect(tokens).toMatch(/--paper-100:\s*#eceadf/);
    expect(tokens).toMatch(/--paper-200:\s*#e5e1d4/);
    expect(tokens).toMatch(/--paper-300:\s*#d4d1c6/);
  });

  it('选区陶土写进 background 的 22% mix，不用 opacity', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    const pm = css.match(/\.ProseMirror ::selection\s*\{[^}]+\}/s)?.[0] ?? '';
    const cssWithoutPm = css.replace(/\.ProseMirror ::selection\s*\{[^}]+\}/gs, '');
    const sel = cssWithoutPm.match(/::selection\s*\{[^}]+\}/s)?.[0] ?? '';
    expect(sel).toMatch(/color-mix\(in oklch,\s*var\(--terracotta\)\s+22%/);
    expect(pm).toMatch(/color-mix\(in oklch,\s*var\(--terracotta\)\s+22%/);
    expect(sel).toMatch(/color:\s*var\(--ink\)/);
    expect(pm).toMatch(/color:\s*var\(--ink\)/);
    expect(sel).not.toMatch(/opacity:/);
    expect(pm).not.toMatch(/opacity:/);
  });
});
