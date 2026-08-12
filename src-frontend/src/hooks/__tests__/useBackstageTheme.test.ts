import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { listen } from '@tauri-apps/api/event';
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

  it('Tauri color-theme-changed 事件触发主题切换', async () => {
    renderHook(() => useBackstageTheme());
    // 等 listen() promise 注册完成，取出注册的事件回调
    await act(async () => {});
    const callback = vi.mocked(listen).mock.calls[0]?.[1] as
      | ((e: { payload: string }) => void)
      | undefined;
    expect(callback).toBeTypeOf('function');
    act(() => callback!({ payload: 'amber' }));
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.amber.vars[k]));
  });

  it('卸载后再触发 storage 不再改主题（cleanup 生效）', () => {
    const { unmount } = renderHook(() => useBackstageTheme());
    unmount();
    for (const k of BACKSTAGE_THEME_VARS) document.documentElement.style.removeProperty(k);
    localStorage.setItem(COLOR_THEME_STORAGE_KEY, 'cool');
    window.dispatchEvent(
      new StorageEvent('storage', { key: COLOR_THEME_STORAGE_KEY, newValue: 'cool' })
    );
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(() => ''));
  });
});
