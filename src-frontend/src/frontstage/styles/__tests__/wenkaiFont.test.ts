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

  it('frontstage.css 分别声明 Regular 400 与 Medium 500', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).not.toMatch(/font-weight:\s*400 500/);
    const faces = [...css.matchAll(/@font-face\s*\{[^}]*\}/g)].map(m => m[0]);
    const wenkai = faces.filter(block => /font-family:\s*['"]LXGW WenKai['"]/.test(block));
    expect(wenkai.length).toBeGreaterThanOrEqual(2);
    const regularFace = wenkai.find(block => /lxgwwenkai-regular\.woff2/.test(block));
    expect(regularFace).toBeDefined();
    expect(regularFace).toMatch(/font-weight:\s*400;/);
    expect(regularFace).toMatch(/font-display:\s*swap/);
    expect(regularFace).not.toMatch(/400 500/);
    const mediumFace = wenkai.find(block => /lxgwwenkai-medium\.woff2/.test(block));
    expect(mediumFace).toBeDefined();
    expect(mediumFace).toMatch(/font-weight:\s*500;/);
    expect(mediumFace).toMatch(/font-display:\s*swap/);
  });

  it('public/fonts 含 medium woff2 且体积不超过 regular 的 1.15 倍', () => {
    const regular = readFileSync(resolve(frontendRoot, 'public/fonts/lxgwwenkai-regular.woff2'));
    const mediumPath = resolve(frontendRoot, 'public/fonts/lxgwwenkai-medium.woff2');
    expect(existsSync(mediumPath)).toBe(true);
    const medium = readFileSync(mediumPath);
    expect(medium.subarray(0, 4).toString('ascii')).toBe('wOF2');
    expect(medium.byteLength).toBeGreaterThan(1_000_000);
    expect(medium.byteLength).toBeLessThanOrEqual(Math.ceil(regular.byteLength * 1.15));
    expect(medium.equals(regular)).toBe(false);
  });

  it('frontstage.html 不再请求 jsdelivr 或 fonts.googleapis', () => {
    const html = readFileSync(resolve(frontendRoot, 'frontstage.html'), 'utf-8');
    expect(html).not.toMatch(/jsdelivr/);
    expect(html).not.toMatch(/fonts\.googleapis/);
    expect(html).not.toMatch(/fonts\.gstatic/);
  });
});
