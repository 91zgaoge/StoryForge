import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ColorThemeSelector } from '../GeneralSettings';
import { BACKSTAGE_THEME_VARS, backstageThemes } from '@/styles/backstageThemes';
import { COLOR_THEME_STORAGE_KEY } from '@/frontstage/config/colorThemes';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}));

describe('ColorThemeSelector', () => {
  beforeEach(() => {
    localStorage.clear();
    for (const k of BACKSTAGE_THEME_VARS) document.documentElement.style.removeProperty(k);
  });

  it('选择 cool 后幕后 cinema 变量切换为 cool 深色调', () => {
    render(<ColorThemeSelector />);
    fireEvent.click(screen.getByText('冷青'));
    for (const key of BACKSTAGE_THEME_VARS) {
      expect(document.documentElement.style.getPropertyValue(key), `变量 ${key}`).toBe(
        backstageThemes.cool.vars[key]
      );
    }
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY)).toBe('cool');
  });

  it('每个选项渲染幕前/幕后双预览色点', () => {
    render(<ColorThemeSelector />);
    expect(screen.getAllByTestId(/theme-swatch-frontstage-/)).toHaveLength(4);
    expect(screen.getAllByTestId(/theme-swatch-backstage-/)).toHaveLength(4);
  });
});
