import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useBackstageTheme } from '../useBackstageTheme';
import { BACKSTAGE_THEME_VARS, backstageThemes } from '@/styles/backstageThemes';
import { COLOR_THEME_STORAGE_KEY } from '@/frontstage/config/colorThemes';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const readVars = () =>
  BACKSTAGE_THEME_VARS.map(k => document.documentElement.style.getPropertyValue(k));

describe('useBackstageTheme', () => {
  beforeEach(() => {
    localStorage.clear();
    for (const k of BACKSTAGE_THEME_VARS) document.documentElement.style.removeProperty(k);
  });

  it('挂载时按 localStorage 应用主题（无保存值 → warm）', () => {
    renderHook(() => useBackstageTheme());
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.warm.vars[k]));
  });

  it('localStorage 存了 cool → 挂载应用 cool', () => {
    localStorage.setItem(COLOR_THEME_STORAGE_KEY, 'cool');
    renderHook(() => useBackstageTheme());
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.cool.vars[k]));
  });

  it('storage 事件触发主题切换', () => {
    renderHook(() => useBackstageTheme());
    localStorage.setItem(COLOR_THEME_STORAGE_KEY, 'indigo');
    window.dispatchEvent(
      new StorageEvent('storage', { key: COLOR_THEME_STORAGE_KEY, newValue: 'indigo' })
    );
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.indigo.vars[k]));
  });
});
