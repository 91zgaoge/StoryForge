import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { listen } from '@tauri-apps/api/event';
import { useBackstageTheme } from '../useBackstageTheme';
import { BACKSTAGE_THEME_VARS, backstageThemes } from '@/styles/backstageThemes';
import { COLOR_THEME_STORAGE_KEY_BACK } from '@/frontstage/config/colorThemes';

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

  it('挂载时按 localStorage 应用主题（无保存值 → 朱红）', () => {
    renderHook(() => useBackstageTheme());
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.zhuhong.vars[k]));
  });

  it('localStorage 存了群青 → 挂载应用群青', () => {
    localStorage.setItem(COLOR_THEME_STORAGE_KEY_BACK, 'qunqing');
    renderHook(() => useBackstageTheme());
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.qunqing.vars[k]));
  });

  it('storage 事件触发主题切换', () => {
    renderHook(() => useBackstageTheme());
    localStorage.setItem(COLOR_THEME_STORAGE_KEY_BACK, 'daizi');
    window.dispatchEvent(
      new StorageEvent('storage', { key: COLOR_THEME_STORAGE_KEY_BACK, newValue: 'daizi' })
    );
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.daizi.vars[k]));
  });

  it('只响应 surface=back 的 color-theme-changed', async () => {
    renderHook(() => useBackstageTheme());
    await act(async () => {});
    const callback = vi.mocked(listen).mock.calls[0]?.[1] as
      | ((e: { payload: unknown }) => void)
      | undefined;
    expect(callback).toBeTypeOf('function');
    act(() => callback!({ payload: { surface: 'front', id: 'zhuqing' } }));
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.zhuhong.vars[k]));
    act(() => callback!({ payload: { surface: 'back', id: 'tenghuang' } }));
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.tenghuang.vars[k]));
  });

  it('卸载后再触发 storage 不再改主题（cleanup 生效）', () => {
    const { unmount } = renderHook(() => useBackstageTheme());
    unmount();
    for (const k of BACKSTAGE_THEME_VARS) document.documentElement.style.removeProperty(k);
    localStorage.setItem(COLOR_THEME_STORAGE_KEY_BACK, 'qunqing');
    window.dispatchEvent(
      new StorageEvent('storage', { key: COLOR_THEME_STORAGE_KEY_BACK, newValue: 'qunqing' })
    );
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(() => ''));
  });
});
