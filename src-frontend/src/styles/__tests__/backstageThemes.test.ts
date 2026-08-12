import { describe, it, expect, afterEach } from 'vitest';
import { BACKSTAGE_THEME_VARS, backstageThemes, applyBackstageTheme } from '../backstageThemes';
import { colorThemeList } from '@/frontstage/config/colorThemes';

/** tokens.css 中 cinema/status 变量的现状值（warm 必须与其完全一致，零视觉回归） */
const TOKENS_CSS_CURRENT: Record<string, string> = {
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
  '--status-success': '#22c55e',
  '--status-success-dim': 'rgba(34, 197, 94, 0.4)',
  '--status-warning': '#facc15',
  '--status-danger': '#ef4444',
  '--status-danger-dim': 'rgba(239, 68, 68, 0.4)',
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

  it('warm 主题全量 16 值与 tokens.css 现状一致（零视觉回归）', () => {
    expect(backstageThemes.warm.vars).toEqual(TOKENS_CSS_CURRENT);
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
