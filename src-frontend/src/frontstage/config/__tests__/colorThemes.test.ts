import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  colorThemes,
  colorThemeList,
  resolveColorThemeId,
  loadColorTheme,
  saveColorTheme,
  applyColorTheme,
  parseThemeEventPayload,
  COLOR_THEME_STORAGE_KEY_FRONT,
  COLOR_THEME_STORAGE_KEY_BACK,
  COLOR_THEME_STORAGE_KEY_LEGACY,
  DEFAULT_COLOR_THEME,
} from '../colorThemes';

vi.mock('@tauri-apps/api/event', () => ({
  emit: vi.fn().mockResolvedValue(undefined),
}));

describe('传统色主题身份', () => {
  it('12 套写作向名单，不含玫红/紫云/浅紫藤萝', () => {
    expect(colorThemeList.map(t => t.id)).toEqual([
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
    ]);
    expect(colorThemes).not.toHaveProperty('warm');
    expect(colorThemes).not.toHaveProperty('meihongse');
    expect(colorThemes).not.toHaveProperty('ziyun');
    expect(colorThemes).not.toHaveProperty('qianzitengluo');
  });

  it('每套幕前金标等于锚色强调，不再偏相', () => {
    for (const theme of colorThemeList) {
      expect(theme.gold, theme.id).toBe(theme.terracotta);
    }
  });
});

describe('resolveColorThemeId', () => {
  it('旧四套落到最近邻传统色', () => {
    expect(resolveColorThemeId('warm')).toBe('zhuhong');
    expect(resolveColorThemeId('cool')).toBe('qunqing');
    expect(resolveColorThemeId('amber')).toBe('tenghuang');
    expect(resolveColorThemeId('indigo')).toBe('daizi');
  });

  it('未知值与空值回落朱红', () => {
    expect(resolveColorThemeId(null)).toBe(DEFAULT_COLOR_THEME);
    expect(resolveColorThemeId('nope')).toBe('zhuhong');
    expect(resolveColorThemeId('zhuhong')).toBe('zhuhong');
  });
});

describe('分选存储', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('无保存时幕前幕后都是朱红', () => {
    expect(loadColorTheme('front')).toBe('zhuhong');
    expect(loadColorTheme('back')).toBe('zhuhong');
  });

  it('旧 key 只在新 key 皆空时迁移到两边', () => {
    localStorage.setItem(COLOR_THEME_STORAGE_KEY_LEGACY, 'cool');
    expect(loadColorTheme('front')).toBe('qunqing');
    expect(loadColorTheme('back')).toBe('qunqing');
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY_FRONT)).toBe('qunqing');
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY_BACK)).toBe('qunqing');
  });

  it('已有分选时旧 key 不再覆盖', () => {
    localStorage.setItem(COLOR_THEME_STORAGE_KEY_FRONT, 'zhuqing');
    localStorage.setItem(COLOR_THEME_STORAGE_KEY_BACK, 'daizi');
    localStorage.setItem(COLOR_THEME_STORAGE_KEY_LEGACY, 'cool');
    expect(loadColorTheme('front')).toBe('zhuqing');
    expect(loadColorTheme('back')).toBe('daizi');
  });

  it('saveColorTheme 只写对应表面', async () => {
    const { emit } = await import('@tauri-apps/api/event');
    saveColorTheme('front', 'yanlan');
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY_FRONT)).toBe('yanlan');
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY_BACK)).toBeNull();
    expect(emit).toHaveBeenCalledWith('color-theme-changed', {
      surface: 'front',
      id: 'yanlan',
    });
  });
});

describe('applyColorTheme', () => {
  it('注入纸/强调/金同色，以及跟随强调的 ai tint', () => {
    applyColorTheme('qunqing');
    const style = document.documentElement.style;
    const theme = colorThemes.qunqing;
    expect(style.getPropertyValue('--parchment')).toBe(theme.parchment);
    expect(style.getPropertyValue('--terracotta')).toBe(theme.terracotta);
    expect(style.getPropertyValue('--gold')).toBe(theme.terracotta);
    expect(style.getPropertyValue('--text-on-accent')).toBe(theme.textOnAccent);
    expect(style.getPropertyValue('--ai-accent-tint')).toContain(theme.terracotta);
  });
});

describe('parseThemeEventPayload', () => {
  it('只接受带 surface 的新载荷，丢弃旧字符串', () => {
    expect(parseThemeEventPayload('cool')).toBeNull();
    expect(parseThemeEventPayload({ surface: 'back', id: 'pibian' })).toEqual({
      surface: 'back',
      id: 'pibian',
    });
    expect(parseThemeEventPayload({ surface: 'front', id: 'warm' })).toBeNull();
  });
});
