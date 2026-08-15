import { describe, it, expect, afterEach } from 'vitest';
import { BACKSTAGE_THEME_VARS, backstageThemes, applyBackstageTheme } from '../backstageThemes';
import { colorThemeList, DEFAULT_COLOR_THEME } from '@/frontstage/config/colorThemes';

/** tokens.css 默认值必须与默认主题（朱红·暗）完全一致 */
const TOKENS_CSS_CURRENT: Record<string, string> = {
  '--cinema-950': 'rgb(30,14,8)',
  '--cinema-900': 'rgb(50,26,16)',
  '--cinema-850': 'rgb(60,35,25)',
  '--cinema-800': 'rgb(71,45,35)',
  '--cinema-700': 'color-mix(in oklab, rgb(71,45,35) 55%, rgb(123,94,83))',
  '--cinema-600': 'rgb(123,94,83)',
  '--cinema-500': 'color-mix(in oklab, rgb(123,94,83) 70%, rgb(249,241,236))',
  '--cinema-gold': 'rgb(246,173,143)',
  '--cinema-gold-light': 'color-mix(in oklab, rgb(246,173,143) 70%, rgb(249,241,236))',
  '--cinema-gold-dark': 'color-mix(in oklab, rgb(246,173,143) 82%, rgb(30,14,8))',
  '--cinema-velvet': '#862617',
  '--status-success': '#4a9a6a',
  '--status-success-dim': 'rgba(74, 154, 106, 0.4)',
  '--status-warning': '#c4a035',
  '--status-danger': '#c45c4a',
  '--status-danger-dim': 'rgba(196, 92, 74, 0.4)',
};

afterEach(() => {
  applyBackstageTheme(DEFAULT_COLOR_THEME);
});

describe('backstageThemes', () => {
  it('每套主题覆盖全部 16 个必需变量，且选项与幕前色调同 id', () => {
    const ids = colorThemeList.map(t => t.id).sort();
    expect(Object.keys(backstageThemes).sort()).toEqual(ids);
    for (const theme of Object.values(backstageThemes)) {
      for (const key of BACKSTAGE_THEME_VARS) {
        expect(theme.vars[key], `${theme.id} 缺 ${key}`).toBeTruthy();
      }
      for (const key of Object.keys(theme.vars)) {
        expect(
          (BACKSTAGE_THEME_VARS as readonly string[]).includes(key),
          `${theme.id} 多出未登记变量 ${key}`
        ).toBe(true);
      }
    }
  });

  it('朱红暗面全量 16 值与 tokens.css 完全一致', () => {
    expect(backstageThemes.zhuhong.vars).toEqual(TOKENS_CSS_CURRENT);
  });

  it('cinema-950 不是 OLED 纯黑', () => {
    for (const theme of Object.values(backstageThemes)) {
      expect(theme.vars['--cinema-950'].toLowerCase()).not.toBe('#050508');
      expect(theme.vars['--cinema-950'].toLowerCase()).not.toBe('#000000');
    }
  });

  it('印色进 velvet，金标是锚色暗面 brand', () => {
    expect(backstageThemes.zhuhong.vars['--cinema-velvet']).toBe('#862617');
    expect(backstageThemes.zhuhong.vars['--cinema-gold']).toBe('rgb(246,173,143)');
  });

  it('applyBackstageTheme 注入全部变量到 documentElement', () => {
    applyBackstageTheme('qunqing');
    const style = document.documentElement.style;
    for (const key of BACKSTAGE_THEME_VARS) {
      expect(style.getPropertyValue(key)).toBe(backstageThemes.qunqing.vars[key]);
    }
  });

  it('未知 id 回退朱红', () => {
    applyBackstageTheme('nope' as never);
    expect(document.documentElement.style.getPropertyValue('--cinema-gold')).toBe(
      backstageThemes.zhuhong.vars['--cinema-gold']
    );
  });
});
