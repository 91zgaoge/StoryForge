import { describe, it, expect } from 'vitest';
import { existsSync, readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const frontendRoot = resolve(__dirname, '..', '..', '..', '..');

describe('霞鹜文楷本地加载', () => {
  it('public/fonts 含 regular woff2 与 OFL', () => {
    const woff2Path = resolve(frontendRoot, 'public/fonts/lxgwwenkai-regular.woff2');
    expect(existsSync(woff2Path)).toBe(true);
    expect(existsSync(resolve(frontendRoot, 'public/fonts/OFL.txt'))).toBe(true);
    const woff2 = readFileSync(woff2Path);
    expect(woff2.subarray(0, 4).toString('ascii')).toBe('wOF2');
    expect(woff2.byteLength).toBeGreaterThan(1_000_000);
  });

  it('frontstage.css 以 @font-face 声明 LXGW WenKai 且 font-display:swap', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).toMatch(/@font-face\s*\{[^}]*font-family:\s*['"]LXGW WenKai['"]/s);
    expect(css).toMatch(/font-display:\s*swap/);
    expect(css).toMatch(/\/fonts\/lxgwwenkai-regular\.woff2/);
    expect(css).toMatch(/font-weight:\s*400 500/);
  });

  it('frontstage.html 不再请求 jsdelivr 或 fonts.googleapis', () => {
    const html = readFileSync(resolve(frontendRoot, 'frontstage.html'), 'utf-8');
    expect(html).not.toMatch(/jsdelivr/);
    expect(html).not.toMatch(/fonts\.googleapis/);
    expect(html).not.toMatch(/fonts\.gstatic/);
  });
});
