/**
 * 幕后主题全局接线：挂载即应用当前色调对应的幕后深色调，
 * 并监听 storage / Tauri color-theme-changed 双通道实时切换。
 * 在幕后根组件（App.tsx）调用一次。
 */
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  COLOR_THEME_STORAGE_KEY,
  loadColorTheme,
  type ColorThemeId,
} from '@/frontstage/config/colorThemes';
import { applyBackstageTheme } from '@/styles/backstageThemes';

export function useBackstageTheme(): void {
  useEffect(() => {
    applyBackstageTheme(loadColorTheme());

    const handleStorage = (e: StorageEvent) => {
      if (e.key === COLOR_THEME_STORAGE_KEY || e.key === null) {
        applyBackstageTheme(loadColorTheme());
      }
    };
    window.addEventListener('storage', handleStorage);

    // cancelled 标志防止 cleanup 先于 listen() promise resolve 时监听器泄漏
    // （StrictMode 双挂载场景：首次挂载的 cleanup 跑完后 promise 才 resolve）
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void listen<ColorThemeId>('color-theme-changed', event => {
      applyBackstageTheme(event.payload);
    })
      .then(fn => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch(() => {
        /* non-Tauri / test env */
      });

    return () => {
      cancelled = true;
      window.removeEventListener('storage', handleStorage);
      unlisten?.();
    };
  }, []);
}
