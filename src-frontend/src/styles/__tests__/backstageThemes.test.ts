import { describe, it, expect, afterEach } from 'vitest';
import { BACKSTAGE_THEME_VARS, backstageThemes, applyBackstageTheme } from '../backstageThemes';
import { colorThemeList } from '@/frontstage/config/colorThemes';

/** tokens.css 中 cinema/status 变量的现状值（warm 必须与 tokens.css 完全一致） */
const TOKENS_CSS_CURRENT: Record<string, string> = {
  '--cinema-950': '#0c0b09',
  '--cinema-900': '#12110e',
  '--cinema-850': '#161310',
  '--cinema-800': '#1c1916',
  '--cinema-700': '#26211c',
  '--cinema-600': '#322c25',
  '--cinema-500': '#423a31',
  '--cinema-gold': '#d4af37',
  '--cinema-gold-light': '#e8c547',
  '--cinema-gold-dark': '#b8941f',
  '--cinema-velvet': '#5c5470',
  '--status-success': '#4a9a6a',
  '--status-success-dim': 'rgba(74, 154, 106, 0.4)',
  '--status-warning': '#c4a035',
  '--status-danger': '#c45c4a',
  '--status-danger-dim': 'rgba(196, 92, 74, 0.4)',
};

afterEach(() => {
  applyBackstageTheme('warm'); // 复位，避免污染其他测试
});

describe('backstageThemes', () => {
  it('每套主题覆盖全部 16 个必需变量，且选项与幕前色调同 id', () => {
    const ids = colorThemeList.map(t => t.id).sort();
    expect(Object.keys(backstageThemes).sort()).toEqual(ids);
    for (const theme of Object.values(backstageThemes)) {
      for (const key of BACKSTAGE_THEME_VARS) {
        expect(theme.vars[key], `${theme.id} 缺 ${key}`).toBeTruthy();
      }
      // 反向：不得有多余变量（防止主题间互相污染、漏清理）
      for (const key of Object.keys(theme.vars)) {
        expect(
          (BACKSTAGE_THEME_VARS as readonly string[]).includes(key),
          `${theme.id} 多出未登记变量 ${key}`
        ).toBe(true);
      }
    }
  });

  it('warm 主题全量 16 值与 tokens.css 完全一致', () => {
    expect(backstageThemes.warm.vars).toEqual(TOKENS_CSS_CURRENT);
  });

  it('cinema-950 不是 OLED 纯黑', () => {
    for (const theme of Object.values(backstageThemes)) {
      expect(theme.vars['--cinema-950'].toLowerCase()).not.toBe('#050508');
      expect(theme.vars['--cinema-950'].toLowerCase()).not.toBe('#000000');
    }
  });

  it('warm velvet 饱和度低于旧 AI 紫', () => {
    expect(backstageThemes.warm.vars['--cinema-velvet']).toBe('#5c5470');
  });

  it('applyBackstageTheme 注入全部变量到 documentElement', () => {
    applyBackstageTheme('cool');
    const style = document.documentElement.style;
    for (const key of BACKSTAGE_THEME_VARS) {
      expect(style.getPropertyValue(key)).toBe(backstageThemes.cool.vars[key]);
    }
  });

  it('未知 id 回退 warm', () => {
    applyBackstageTheme('nope' as never);
    expect(document.documentElement.style.getPropertyValue('--cinema-gold')).toBe(
      backstageThemes.warm.vars['--cinema-gold']
    );
  });
});
