/**
 * 幕前传统色主题（纸 · 帘 · 印）
 *
 * 12 套写作向锚色。亮纸只给幕前；暗机械在 backstageThemes.ts。
 * 幕前 / 幕后分选：两个 localStorage key + 带 surface 的事件。
 */

import { emit } from '@tauri-apps/api/event';
import { createLogger } from '@/utils/logger';

const colorThemeLogger = createLogger('hooks:colorThemes');

export type ColorThemeId =
  | 'zhuqing'
  | 'zhuhong'
  | 'qunqing'
  | 'tenghuang'
  | 'jiangzi'
  | 'lingmenghong'
  | 'heyelv'
  | 'fenlv'
  | 'daizi'
  | 'yanlan'
  | 'pibian'
  | 'hanxiulv';

export type ColorThemeSurface = 'front' | 'back';

export type ColorThemeChangedPayload = {
  surface: ColorThemeSurface;
  id: ColorThemeId;
};

export type PaperFamily = '素绢' | '熟宣' | '雪青' | '赭纸';

export interface ColorTheme {
  id: ColorThemeId;
  name: string;
  description: string;
  family: PaperFamily;
  parchment: string;
  parchmentDark: string;
  warmSand: string;
  borderCream: string;
  terracotta: string;
  terracottaLight: string;
  terracottaDark: string;
  charcoal: string;
  charcoalLight: string;
  oliveGray: string;
  stoneGray: string;
  ink: string;
  ivory: string;
  gold: string;
  textOnAccent: string;
}

/** 旧四套 → 最近邻。新 key 皆空时才读。 */
export const COLOR_THEME_STORAGE_KEY_LEGACY = 'storymoss-color-theme';
export const COLOR_THEME_STORAGE_KEY_FRONT = 'storymoss-color-theme-front';
export const COLOR_THEME_STORAGE_KEY_BACK = 'storymoss-color-theme-back';
/** @deprecated 用 FRONT / BACK；保留给未迁移的监听方识别旧 storage 事件 */
export const COLOR_THEME_STORAGE_KEY = COLOR_THEME_STORAGE_KEY_LEGACY;

export const DEFAULT_COLOR_THEME: ColorThemeId = 'zhuhong';

export const LEGACY_THEME_ID_MAP: Record<string, ColorThemeId> = {
  warm: 'zhuhong',
  cool: 'qunqing',
  amber: 'tenghuang',
  indigo: 'daizi',
};

const THEME_IDS: ColorThemeId[] = [
  'zhuqing',
  'zhuhong',
  'qunqing',
  'tenghuang',
  'jiangzi',
  'lingmenghong',
  'heyelv',
  'fenlv',
  'daizi',
  'yanlan',
  'pibian',
  'hanxiulv',
];

function mix(a: string, b: string, pctA: number): string {
  return `color-mix(in oklab, ${a} ${pctA}%, ${b})`;
}

type LightSeed = {
  paper: string;
  sidebar: string;
  brand: string;
  ink: string;
  onAccent: string;
};

export type DarkSeed = {
  paper: string;
  l1: string;
  l2: string;
  l3: string;
  overlay: string;
  sidebar: string;
  brand: string;
  ink: string;
  onAccent: string;
  seal: string;
};

export type ThemeCatalogEntry = {
  name: string;
  family: PaperFamily;
  description: string;
  light: LightSeed;
  dark: DarkSeed;
};

/** dsh-theme-plugin 已交付令牌（MIT），只抄 paper/帘/brand/ink/layers/seal。 */
export const THEME_CATALOG: Record<ColorThemeId, ThemeCatalogEntry> = {
  zhuqing: {
    name: '竹青',
    family: '素绢',
    description: '素绢纸 + 竹青强调',
    light: {
      paper: 'rgb(239,248,241)',
      sidebar: 'rgb(219,240,227)',
      brand: 'rgb(0,155,95)',
      ink: 'rgb(13,18,14)',
      onAccent: 'rgb(239,248,241)',
    },
    dark: {
      paper: 'rgb(14,21,15)',
      l1: 'rgb(30,42,33)',
      l2: 'rgb(38,51,42)',
      l3: 'rgb(47,60,51)',
      overlay: 'rgb(91,105,95)',
      sidebar: 'rgb(8,33,19)',
      brand: 'rgb(62,180,137)',
      ink: 'rgb(239,245,240)',
      onAccent: 'rgb(48,57,50)',
      seal: '#C70039',
    },
  },
  zhuhong: {
    name: '朱红',
    family: '熟宣',
    description: '熟宣纸 + 朱红强调',
    light: {
      paper: 'rgb(255,239,232)',
      sidebar: 'rgb(254,225,217)',
      brand: 'rgb(223,67,19)',
      ink: 'rgb(25,14,9)',
      onAccent: 'rgb(255,250,242)',
    },
    dark: {
      paper: 'rgb(30,14,8)',
      l1: 'rgb(50,26,16)',
      l2: 'rgb(60,35,25)',
      l3: 'rgb(71,45,35)',
      overlay: 'rgb(123,94,83)',
      sidebar: 'rgb(43,20,13)',
      brand: 'rgb(246,173,143)',
      ink: 'rgb(249,241,236)',
      onAccent: 'rgb(68,49,41)',
      seal: '#862617',
    },
  },
  qunqing: {
    name: '群青',
    family: '雪青',
    description: '雪青绢 + 群青强调',
    light: {
      paper: 'rgb(236,246,255)',
      sidebar: 'rgb(211,231,249)',
      brand: 'rgb(23,114,180)',
      ink: 'rgb(11,18,25)',
      onAccent: 'rgb(245,255,255)',
    },
    dark: {
      paper: 'rgb(10,19,29)',
      l1: 'rgb(12,24,36)',
      l2: 'rgb(20,34,49)',
      l3: 'rgb(32,46,61)',
      overlay: 'rgb(87,104,121)',
      sidebar: 'rgb(10,29,45)',
      brand: 'rgb(126,192,238)',
      ink: 'rgb(236,243,247)',
      onAccent: 'rgb(45,55,67)',
      seal: '#C21F30',
    },
  },
  tenghuang: {
    name: '藤黄',
    family: '赭纸',
    description: '赭纸 + 藤黄强调',
    light: {
      paper: 'rgb(251,243,227)',
      sidebar: 'rgb(241,232,208)',
      brand: 'rgb(164,120,0)',
      ink: 'rgb(22,16,6)',
      onAccent: 'rgb(255,254,238)',
    },
    dark: {
      paper: 'rgb(25,17,4)',
      l1: 'rgb(41,30,7)',
      l2: 'rgb(51,40,17)',
      l3: 'rgb(62,50,27)',
      overlay: 'rgb(112,100,75)',
      sidebar: 'rgb(35,26,3)',
      brand: 'rgb(248,223,114)',
      ink: 'rgb(248,242,232)',
      onAccent: 'rgb(62,52,36)',
      seal: '#4B9CD3',
    },
  },
  jiangzi: {
    name: '绛紫',
    family: '熟宣',
    description: '熟宣纸 + 绛紫强调',
    light: {
      paper: 'rgb(255,239,238)',
      sidebar: 'rgb(248,218,221)',
      brand: 'rgb(142,53,74)',
      ink: 'rgb(24,14,13)',
      onAccent: 'rgb(255,250,249)',
    },
    dark: {
      paper: 'rgb(29,14,14)',
      l1: 'rgb(52,27,27)',
      l2: 'rgb(62,36,36)',
      l3: 'rgb(72,46,45)',
      overlay: 'rgb(122,93,92)',
      sidebar: 'rgb(43,19,22)',
      brand: 'rgb(197,112,139)',
      ink: 'rgb(248,240,238)',
      onAccent: 'rgb(67,49,47)',
      seal: '#A8456B',
    },
  },
  lingmenghong: {
    name: '菱锰红',
    family: '熟宣',
    description: '熟宣纸 + 菱锰红强调',
    light: {
      paper: 'rgb(255,239,242)',
      sidebar: 'rgb(252,224,235)',
      brand: 'rgb(184,94,139)',
      ink: 'rgb(24,14,15)',
      onAccent: 'rgb(255,250,253)',
    },
    dark: {
      paper: 'rgb(28,14,17)',
      l1: 'rgb(50,27,33)',
      l2: 'rgb(60,36,42)',
      l3: 'rgb(70,46,52)',
      overlay: 'rgb(120,93,99)',
      sidebar: 'rgb(42,19,28)',
      brand: 'rgb(233,215,223)',
      ink: 'rgb(248,240,241)',
      onAccent: 'rgb(65,49,51)',
      seal: '#9B1E64',
    },
  },
  heyelv: {
    name: '荷叶绿',
    family: '素绢',
    description: '素绢纸 + 荷叶绿强调',
    light: {
      paper: 'rgb(240,248,241)',
      sidebar: 'rgb(213,234,220)',
      brand: 'rgb(26,104,64)',
      ink: 'rgb(14,18,14)',
      onAccent: 'rgb(240,248,241)',
    },
    dark: {
      paper: 'rgb(14,21,15)',
      l1: 'rgb(30,42,33)',
      l2: 'rgb(39,51,41)',
      l3: 'rgb(48,60,50)',
      overlay: 'rgb(92,105,94)',
      sidebar: 'rgb(9,33,18)',
      brand: 'rgb(60,179,113)',
      ink: 'rgb(239,245,240)',
      onAccent: 'rgb(49,56,49)',
      seal: '#5A191B',
    },
  },
  fenlv: {
    name: '粉绿',
    family: '素绢',
    description: '素绢纸 + 粉绿强调',
    light: {
      paper: 'rgb(240,247,242)',
      sidebar: 'rgb(217,240,229)',
      brand: 'rgb(69,140,111)',
      ink: 'rgb(14,18,15)',
      onAccent: 'rgb(233,241,236)',
    },
    dark: {
      paper: 'rgb(14,20,16)',
      l1: 'rgb(33,44,37)',
      l2: 'rgb(42,53,45)',
      l3: 'rgb(50,62,54)',
      overlay: 'rgb(93,105,96)',
      sidebar: 'rgb(5,33,22)',
      brand: 'rgb(188,229,214)',
      ink: 'rgb(240,244,241)',
      onAccent: 'rgb(49,56,51)',
      seal: '#B34B43',
    },
  },
  daizi: {
    name: '黛紫',
    family: '雪青',
    description: '雪青绢 + 黛紫强调',
    light: {
      paper: 'rgb(247,243,255)',
      sidebar: 'rgb(235,222,244)',
      brand: 'rgb(93,58,111)',
      ink: 'rgb(18,15,23)',
      onAccent: 'rgb(255,252,255)',
    },
    dark: {
      paper: 'rgb(20,16,27)',
      l1: 'rgb(25,20,33)',
      l2: 'rgb(36,29,47)',
      l3: 'rgb(48,41,59)',
      overlay: 'rgb(106,97,118)',
      sidebar: 'rgb(33,21,41)',
      brand: 'rgb(129,92,148)',
      ink: 'rgb(244,241,248)',
      onAccent: 'rgb(55,52,65)',
      seal: '#5f3c71',
    },
  },
  yanlan: {
    name: '鷃蓝',
    family: '雪青',
    description: '雪青绢 + 鷃蓝强调',
    light: {
      paper: 'rgb(236,246,255)',
      sidebar: 'rgb(211,231,249)',
      brand: 'rgb(20,74,116)',
      ink: 'rgb(11,18,25)',
      onAccent: 'rgb(245,255,255)',
    },
    dark: {
      paper: 'rgb(10,19,29)',
      l1: 'rgb(12,24,36)',
      l2: 'rgb(20,34,49)',
      l3: 'rgb(32,46,61)',
      overlay: 'rgb(87,104,121)',
      sidebar: 'rgb(10,29,45)',
      brand: 'rgb(33,119,184)',
      ink: 'rgb(236,243,247)',
      onAccent: 'rgb(45,55,67)',
      seal: '#7C1823',
    },
  },
  pibian: {
    name: '皮弁',
    family: '赭纸',
    description: '赭纸 + 皮弁强调',
    light: {
      paper: 'rgb(255,240,227)',
      sidebar: 'rgb(244,222,205)',
      brand: 'rgb(139,93,51)',
      ink: 'rgb(25,14,6)',
      onAccent: 'rgb(255,251,237)',
    },
    dark: {
      paper: 'rgb(30,14,3)',
      l1: 'rgb(44,25,9)',
      l2: 'rgb(54,35,19)',
      l3: 'rgb(65,45,29)',
      overlay: 'rgb(119,96,79)',
      sidebar: 'rgb(41,22,6)',
      brand: 'rgb(199,154,106)',
      ink: 'rgb(249,241,233)',
      onAccent: 'rgb(68,50,36)',
      seal: '#2E5D8C',
    },
  },
  hanxiulv: {
    name: '汉绣绿',
    family: '素绢',
    description: '素绢纸 + 汉绣绿强调',
    light: {
      paper: 'rgb(241,247,240)',
      sidebar: 'rgb(217,233,216)',
      brand: 'rgb(46,125,50)',
      ink: 'rgb(14,18,13)',
      onAccent: 'rgb(241,247,240)',
    },
    dark: {
      paper: 'rgb(15,21,14)',
      l1: 'rgb(32,42,31)',
      l2: 'rgb(41,51,39)',
      l3: 'rgb(50,60,48)',
      overlay: 'rgb(94,105,92)',
      sidebar: 'rgb(15,32,14)',
      brand: 'rgb(112,200,112)',
      ink: 'rgb(240,244,239)',
      onAccent: 'rgb(50,56,48)',
      seal: '#8E354A',
    },
  },
};

function themeFromLight(id: ColorThemeId, entry: ThemeCatalogEntry): ColorTheme {
  const { paper, sidebar, brand, ink, onAccent } = entry.light;
  return {
    id,
    name: entry.name,
    description: entry.description,
    family: entry.family,
    parchment: paper,
    parchmentDark: sidebar,
    warmSand: mix(paper, ink, 88),
    borderCream: mix(paper, ink, 94),
    terracotta: brand,
    terracottaLight: mix(brand, paper, 70),
    terracottaDark: mix(brand, ink, 82),
    charcoal: mix(ink, paper, 88),
    charcoalLight: mix(ink, paper, 72),
    oliveGray: mix(ink, paper, 62),
    stoneGray: mix(ink, paper, 52),
    ink,
    ivory: mix(paper, 'rgb(255,255,255)', 70),
    gold: brand,
    textOnAccent: onAccent,
  };
}

export const colorThemes: Record<ColorThemeId, ColorTheme> = Object.fromEntries(
  THEME_IDS.map(id => [id, themeFromLight(id, THEME_CATALOG[id])])
) as Record<ColorThemeId, ColorTheme>;

export const defaultColorTheme: ColorTheme = colorThemes[DEFAULT_COLOR_THEME];
export const colorThemeList = THEME_IDS.map(id => colorThemes[id]);

export function isColorThemeId(value: string): value is ColorThemeId {
  return value in THEME_CATALOG;
}

export function resolveColorThemeId(raw: string | null | undefined): ColorThemeId {
  if (!raw) return DEFAULT_COLOR_THEME;
  if (isColorThemeId(raw)) return raw;
  return LEGACY_THEME_ID_MAP[raw] ?? DEFAULT_COLOR_THEME;
}

function persistMigrated(id: ColorThemeId) {
  try {
    if (!localStorage.getItem(COLOR_THEME_STORAGE_KEY_FRONT)) {
      localStorage.setItem(COLOR_THEME_STORAGE_KEY_FRONT, id);
    }
    if (!localStorage.getItem(COLOR_THEME_STORAGE_KEY_BACK)) {
      localStorage.setItem(COLOR_THEME_STORAGE_KEY_BACK, id);
    }
  } catch {
    colorThemeLogger.error('Failed to persist migrated color theme');
  }
}

export function loadColorTheme(surface: ColorThemeSurface = 'front'): ColorThemeId {
  const key = surface === 'front' ? COLOR_THEME_STORAGE_KEY_FRONT : COLOR_THEME_STORAGE_KEY_BACK;
  try {
    const saved = localStorage.getItem(key);
    if (saved) return resolveColorThemeId(saved);
    const legacy = localStorage.getItem(COLOR_THEME_STORAGE_KEY_LEGACY);
    if (legacy) {
      const id = resolveColorThemeId(legacy);
      persistMigrated(id);
      return id;
    }
  } catch {
    colorThemeLogger.error('Failed to load color theme');
  }
  return DEFAULT_COLOR_THEME;
}

export function saveColorTheme(surface: ColorThemeSurface, themeId: ColorThemeId) {
  const key = surface === 'front' ? COLOR_THEME_STORAGE_KEY_FRONT : COLOR_THEME_STORAGE_KEY_BACK;
  try {
    localStorage.setItem(key, themeId);
    const payload: ColorThemeChangedPayload = { surface, id: themeId };
    void emit('color-theme-changed', payload);
  } catch {
    colorThemeLogger.error('Failed to save color theme');
  }
}

export function parseThemeEventPayload(payload: unknown): ColorThemeChangedPayload | null {
  if (!payload || typeof payload !== 'object') return null;
  const surface = (payload as { surface?: unknown }).surface;
  const id = (payload as { id?: unknown }).id;
  if ((surface === 'front' || surface === 'back') && typeof id === 'string' && isColorThemeId(id)) {
    return { surface, id };
  }
  return null;
}

export function applyColorTheme(themeId: ColorThemeId) {
  const theme = colorThemes[themeId] || defaultColorTheme;
  const root = document.documentElement;
  const vars: Record<string, string> = {
    '--parchment': theme.parchment,
    '--parchment-dark': theme.parchmentDark,
    '--warm-sand': theme.warmSand,
    '--border-cream': theme.borderCream,
    '--terracotta': theme.terracotta,
    '--terracotta-light': theme.terracottaLight,
    '--terracotta-dark': theme.terracottaDark,
    '--charcoal': theme.charcoal,
    '--charcoal-light': theme.charcoalLight,
    '--olive-gray': theme.oliveGray,
    '--stone-gray': theme.stoneGray,
    '--ink': theme.ink,
    '--ivory': theme.ivory,
    '--gold': theme.gold,
    '--text-on-accent': theme.textOnAccent,
    '--ai-accent-tint': `color-mix(in srgb, ${theme.terracotta} 12%, transparent)`,
    '--ai-on-accent': theme.textOnAccent,
  };
  Object.entries(vars).forEach(([key, value]) => {
    root.style.setProperty(key, value);
  });
}

export function getCurrentEditorColors(themeId?: ColorThemeId) {
  const id = themeId || loadColorTheme('front');
  const theme = colorThemes[id] || defaultColorTheme;
  return {
    paperColor: theme.parchment,
    inkColor: theme.ink,
    accentColor: theme.terracotta,
  };
}
