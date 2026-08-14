import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { colorThemes } from '../colorThemes';
import { writingStyles } from '@/frontstage/config/writingStyles';

const frontendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../..');

describe('幕前墨色抬升', () => {
  it('frontstage.css 默认 --ink 为 oklch 32%', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).toMatch(/--ink:\s*oklch\(32%/);
  });

  it('deriveTheme ink 亮于旧 25% 印刷黑', () => {
    // fmt 用 toFixed(1)，暖主题 38-6=32 → oklch(32.0% …)
    expect(colorThemes.warm.ink).toMatch(/oklch\(\s*32/);
  });

  it('幕前 --shadow-float 不用纯黑 rgba(0,0,0', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    const block = css.match(/--shadow-float:\s*([^;]+);/)?.[1] ?? '';
    expect(block).not.toMatch(/rgba\(0,\s*0,\s*0/);
  });

  it('暖赭默认纸 chroma 为 0.012', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    const tokens = readFileSync(resolve(frontendRoot, 'src/styles/tokens.css'), 'utf-8');
    expect(css).toMatch(/--parchment:\s*oklch\(96\.5%\s+0\.012\s+95\)/);
    expect(css).toMatch(/--parchment-dark:\s*oklch\(93\.5%\s+0\.014\s+95\)/);
    expect(css).toMatch(/--warm-sand:\s*oklch\(91(?:\.0)?%\s+0\.018\s+95\)/);
    expect(css).toMatch(/--border-cream:\s*oklch\(94(?:\.0)?%\s+0\.013\s+95\)/);
    expect(colorThemes.warm.parchment).toMatch(/oklch\(96\.5%\s+0\.012\s+95\)/);
    expect(colorThemes.cool.parchment).toMatch(/220/);
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
