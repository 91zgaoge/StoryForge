import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { colorThemes } from '../colorThemes';

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
});
