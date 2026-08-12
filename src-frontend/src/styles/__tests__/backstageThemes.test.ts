import { describe, it, expect } from 'vitest';
import { BACKSTAGE_THEME_VARS, backstageThemes, applyBackstageTheme } from '../backstageThemes';
import { colorThemeList } from '@/frontstage/config/colorThemes';

describe('backstageThemes', () => {
  it('每套主题覆盖全部 16 个必需变量，且选项与幕前色调同 id', () => {
    const ids = colorThemeList.map(t => t.id).sort();
    expect(Object.keys(backstageThemes).sort()).toEqual(ids);
    for (const theme of Object.values(backstageThemes)) {
      for (const key of BACKSTAGE_THEME_VARS) {
        expect(theme.vars[key], `${theme.id} 缺 ${key}`).toBeTruthy();
      }
    }
  });

  it('warm 主题与现状色值一致（零视觉回归）', () => {
    const warm = backstageThemes.warm.vars;
    expect(warm['--cinema-950']).toBe('#050508');
    expect(warm['--cinema-800']).toBe('#151520');
    expect(warm['--cinema-500']).toBe('#3a3a50');
    expect(warm['--cinema-gold']).toBe('#d4af37');
    expect(warm['--cinema-velvet']).toBe('#7c3aed');
    expect(warm['--status-success']).toBe('#22c55e');
  });

  it('applyBackstageTheme 注入全部变量到 documentElement', () => {
    applyBackstageTheme('cool');
    const style = document.documentElement.style;
    for (const key of BACKSTAGE_THEME_VARS) {
      expect(style.getPropertyValue(key)).toBe(backstageThemes.cool.vars[key]);
    }
    applyBackstageTheme('warm'); // 复位，避免污染其他测试
  });

  it('未知 id 回退 warm', () => {
    applyBackstageTheme('nope' as never);
    expect(document.documentElement.style.getPropertyValue('--cinema-gold')).toBe(
      backstageThemes.warm.vars['--cinema-gold']
    );
    applyBackstageTheme('warm');
  });
});
