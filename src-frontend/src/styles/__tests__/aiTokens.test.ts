// src-frontend/src/styles/__tests__/aiTokens.test.ts
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
// 从 src-frontend/src/styles/__tests__/ 回到 src-frontend/：../../../
const frontendRoot = resolve(__dirname, '..', '..', '..');

const tokensCss = readFileSync(resolve(frontendRoot, 'src/styles/tokens.css'), 'utf-8');
const frontstageCss = readFileSync(
  resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
  'utf-8'
);
const tailwindConfig = readFileSync(resolve(frontendRoot, 'tailwind.config.js'), 'utf-8');

/** P1 语义令牌全集（两窗口必须各自定义） */
const AI_VARS = [
  '--ai-surface',
  '--ai-inset',
  '--ai-field',
  '--ai-hover',
  '--ai-hover-2',
  '--ai-ink',
  '--ai-ink-2',
  '--ai-ink-3',
  '--ai-line',
  '--ai-line-strong',
  '--ai-accent',
  '--ai-accent-ink',
  '--ai-accent-tint',
  '--ai-green',
  '--ai-red',
  '--ai-orange',
  '--ai-on-accent',
];

/** tailwind.config.js 中 ai 色组的 key（映射目标即同名变量） */
const AI_COLOR_KEYS = [
  'surface',
  'inset',
  'field',
  'hover',
  'hover-2',
  'ink',
  'ink-2',
  'ink-3',
  'line',
  'line-strong',
  'accent',
  'accent-ink',
  'accent-tint',
  'green',
  'red',
  'orange',
  'on-accent',
];

/** tailwind.config.js keyframes 注册表（ai-fade-up 工具的 keyframe 名为 fade-up） */
const AI_KEYFRAME_ENTRIES = [
  "'pixel-on': {",
  "'shimmer-text': {",
  "'fade-up': {",
  "'pop-in': {",
  "'stream-in': {",
  "'ai-spin': {",
  "'eq-bounce': {",
  "'ai-sweep': {",
  "'ai-blink': {",
];

const AI_ANIMATION_UTILS = [
  'pixel-on',
  'shimmer-text',
  'ai-fade-up',
  'pop-in',
  'stream-in',
  'ai-spin',
  'eq-bounce',
  'ai-sweep',
  'ai-blink',
];

describe('AI 语义令牌桥（P1 Task1）', () => {
  it('tokens.css（幕后）定义全部 17 个 --ai-* 变量', () => {
    for (const v of AI_VARS) {
      expect(tokensCss, `tokens.css 缺 ${v}`).toContain(`${v}:`);
    }
  });

  it('frontstage.css（幕前）定义全部 17 个 --ai-* 变量', () => {
    for (const v of AI_VARS) {
      expect(frontstageCss, `frontstage.css 缺 ${v}`).toContain(`${v}:`);
    }
  });

  it('tailwind.config.js 将 ai 色组映射到 var(--ai-*)', () => {
    for (const key of AI_COLOR_KEYS) {
      expect(tailwindConfig, `tailwind 缺 ai.${key} 映射`).toContain(`var(--ai-${key})`);
    }
  });

  it('tailwind.config.js 注册全部 AI keyframes 与动画工具', () => {
    for (const entry of AI_KEYFRAME_ENTRIES) {
      expect(tailwindConfig, `tailwind 缺 keyframe ${entry}`).toContain(entry);
    }
    for (const util of AI_ANIMATION_UTILS) {
      expect(tailwindConfig, `tailwind 缺 animation 工具 ${util}`).toContain(`'${util}':`);
    }
  });

  it('两个窗口 CSS 均有 prefers-reduced-motion 动画冻结选择器', () => {
    for (const [name, css] of [
      ['tokens.css', tokensCss],
      ['frontstage.css', frontstageCss],
    ] as const) {
      expect(css, `${name} 缺 .animate-pixel-on 冻结`).toContain('.animate-pixel-on');
      expect(css, `${name} 缺 .animate-ai-sweep 冻结`).toContain('.animate-ai-sweep');
    }
  });
});
