/**
 * 幕后深色调主题（纸 · 帘 · 印）
 *
 * 与幕前同 id 的 12 套传统色；暗面机械。分选后不再与幕前共用一个 storage key。
 */
import {
  DEFAULT_COLOR_THEME,
  THEME_CATALOG,
  colorThemeList,
  type ColorThemeId,
  type DarkSeed,
} from '@/frontstage/config/colorThemes';

export interface BackstageTheme {
  id: ColorThemeId;
  name: string;
  description: string;
  vars: Record<string, string>;
}

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
  '--status-success': '#4a9a6a',
  '--status-success-dim': 'rgba(74, 154, 106, 0.4)',
  '--status-warning': '#c4a035',
  '--status-danger': '#c45c4a',
  '--status-danger-dim': 'rgba(196, 92, 74, 0.4)',
};

function mix(a: string, b: string, pctA: number): string {
  return `color-mix(in oklab, ${a} ${pctA}%, ${b})`;
}

function varsFromDark(dark: DarkSeed): Record<string, string> {
  return {
    '--cinema-950': dark.paper,
    '--cinema-900': dark.l1,
    '--cinema-850': dark.l2,
    '--cinema-800': dark.l3,
    '--cinema-700': mix(dark.l3, dark.overlay, 55),
    '--cinema-600': dark.overlay,
    '--cinema-500': mix(dark.overlay, dark.ink, 70),
    '--cinema-gold': dark.brand,
    '--cinema-gold-light': mix(dark.brand, dark.ink, 70),
    '--cinema-gold-dark': mix(dark.brand, dark.paper, 82),
    '--cinema-velvet': dark.seal,
    ...STATUS,
  };
}

export const backstageThemes: Record<ColorThemeId, BackstageTheme> = Object.fromEntries(
  colorThemeList.map(front => {
    const entry = THEME_CATALOG[front.id];
    return [
      front.id,
      {
        id: front.id,
        name: front.name,
        description: `${entry.family}暗面 + ${front.name}强调`,
        vars: varsFromDark(entry.dark),
      } satisfies BackstageTheme,
    ];
  })
) as Record<ColorThemeId, BackstageTheme>;

export function applyBackstageTheme(themeId: ColorThemeId): void {
  const theme = backstageThemes[themeId] ?? backstageThemes[DEFAULT_COLOR_THEME];
  const root = document.documentElement;
  for (const key of BACKSTAGE_THEME_VARS) {
    root.style.setProperty(key, theme.vars[key]);
  }
  const gold = theme.vars['--cinema-gold'];
  const catalog = THEME_CATALOG[theme.id] ?? THEME_CATALOG[DEFAULT_COLOR_THEME];
  root.style.setProperty('--ai-accent-tint', `color-mix(in srgb, ${gold} 12%, transparent)`);
  root.style.setProperty('--ai-on-accent', catalog.dark.onAccent);
  root.style.setProperty('--ai-ink', catalog.dark.ink);
}
