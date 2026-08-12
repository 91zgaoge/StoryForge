/**
 * 幕后深色调主题系统（P0）
 *
 * Tailwind 的 cinema 色已映射 var(--cinema-*)（tailwind.config.js），
 * 主题切换 = 运行时重写 documentElement 上的同名变量。
 * 选项 id 与幕前色调主题（colorThemes.ts）一致：warm/cool/amber/indigo，
 * localStorage key 复用 storymoss-color-theme，天然双向同步。
 */
import type { ColorThemeId } from '@/frontstage/config/colorThemes';

export interface BackstageTheme {
  id: ColorThemeId;
  name: string;
  description: string;
  vars: Record<string, string>;
}

/** 每套主题必须给齐的变量（完整性测试遍历此表） */
export const BACKSTAGE_THEME_VARS = [
  '--cinema-950',
  '--cinema-900',
  '--cinema-850',
  '--cinema-800',
  '--cinema-700',
  '--cinema-600',
  '--cinema-500',
  '--cinema-gold',
  '--cinema-gold-light',
  '--cinema-gold-dark',
  '--cinema-velvet',
  '--status-success',
  '--status-success-dim',
  '--status-warning',
  '--status-danger',
  '--status-danger-dim',
] as const;

const STATUS = {
  '--status-success': '#22c55e',
  '--status-success-dim': 'rgba(34, 197, 94, 0.4)',
  '--status-warning': '#facc15',
  '--status-danger': '#ef4444',
  '--status-danger-dim': 'rgba(239, 68, 68, 0.4)',
};

export const backstageThemes: Record<ColorThemeId, BackstageTheme> = {
  warm: {
    id: 'warm',
    name: '暖金',
    description: '深色底 + 金色强调（默认，与现状一致）',
    vars: {
      '--cinema-950': '#050508',
      '--cinema-900': '#0a0a0f',
      '--cinema-850': '#0f0f16',
      '--cinema-800': '#151520',
      '--cinema-700': '#1e1e2e',
      '--cinema-600': '#2a2a3c',
      '--cinema-500': '#3a3a50',
      '--cinema-gold': '#d4af37',
      '--cinema-gold-light': '#e8c547',
      '--cinema-gold-dark': '#b8941f',
      '--cinema-velvet': '#7c3aed',
      ...STATUS,
    },
  },
  cool: {
    id: 'cool',
    name: '冷青',
    description: '深夜蓝底 + 青色强调，清新理性',
    vars: {
      '--cinema-950': '#04080c',
      '--cinema-900': '#081018',
      '--cinema-850': '#0b1620',
      '--cinema-800': '#101d29',
      '--cinema-700': '#162636',
      '--cinema-600': '#1f3347',
      '--cinema-500': '#2c455e',
      '--cinema-gold': '#22d3ee',
      '--cinema-gold-light': '#67e8f9',
      '--cinema-gold-dark': '#0891b2',
      '--cinema-velvet': '#38bdf8',
      ...STATUS,
    },
  },
  amber: {
    id: 'amber',
    name: '琥珀',
    description: '暖褐底 + 琥珀橙强调，温润古典',
    vars: {
      '--cinema-950': '#0a0705',
      '--cinema-900': '#120c07',
      '--cinema-850': '#181008',
      '--cinema-800': '#201609',
      '--cinema-700': '#2c1e0d',
      '--cinema-600': '#3d2a12',
      '--cinema-500': '#523a1b',
      '--cinema-gold': '#f59e0b',
      '--cinema-gold-light': '#fbbf24',
      '--cinema-gold-dark': '#d97706',
      '--cinema-velvet': '#fb923c',
      ...STATUS,
    },
  },
  indigo: {
    id: 'indigo',
    name: '靛紫',
    description: '紫夜底 + 靛蓝强调，沉静深邃',
    vars: {
      '--cinema-950': '#06060c',
      '--cinema-900': '#0b0b16',
      '--cinema-850': '#100f20',
      '--cinema-800': '#16152c',
      '--cinema-700': '#1d1b3a',
      '--cinema-600': '#282450',
      '--cinema-500': '#373266',
      '--cinema-gold': '#818cf8',
      '--cinema-gold-light': '#a5b4fc',
      '--cinema-gold-dark': '#6366f1',
      '--cinema-velvet': '#a78bfa',
      ...STATUS,
    },
  },
};

/** 将幕后主题应用到 documentElement；未知 id 回退 warm */
export function applyBackstageTheme(themeId: ColorThemeId) {
  const theme = backstageThemes[themeId] ?? backstageThemes.warm;
  const root = document.documentElement;
  for (const key of BACKSTAGE_THEME_VARS) {
    root.style.setProperty(key, theme.vars[key]);
  }
}
