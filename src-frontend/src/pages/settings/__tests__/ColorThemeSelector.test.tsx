import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ColorThemeSelector } from '../GeneralSettings';
import { BACKSTAGE_THEME_VARS, backstageThemes } from '@/styles/backstageThemes';
import {
  COLOR_THEME_STORAGE_KEY_FRONT,
  COLOR_THEME_STORAGE_KEY_BACK,
} from '@/frontstage/config/colorThemes';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}));

describe('ColorThemeSelector', () => {
  beforeEach(() => {
    localStorage.clear();
    for (const k of BACKSTAGE_THEME_VARS) document.documentElement.style.removeProperty(k);
  });

  it('点幕后群青只改机械色，不写幕前 key', () => {
    render(<ColorThemeSelector />);
    fireEvent.click(screen.getAllByText('群青')[1]);
    for (const key of BACKSTAGE_THEME_VARS) {
      expect(document.documentElement.style.getPropertyValue(key), `变量 ${key}`).toBe(
        backstageThemes.qunqing.vars[key]
      );
    }
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY_BACK)).toBe('qunqing');
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY_FRONT)).toBeNull();
  });

  it('点幕前竹青只写 front key，不改 cinema-gold', () => {
    render(<ColorThemeSelector />);
    fireEvent.click(screen.getAllByText('竹青')[0]);
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY_FRONT)).toBe('zhuqing');
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY_BACK)).toBeNull();
    expect(document.documentElement.style.getPropertyValue('--cinema-gold')).toBe('');
  });

  it('每个表面渲染 12 个预览色点', () => {
    render(<ColorThemeSelector />);
    expect(screen.getAllByTestId(/theme-swatch-frontstage-/)).toHaveLength(12);
    expect(screen.getAllByTestId(/theme-swatch-backstage-/)).toHaveLength(12);
  });
});
