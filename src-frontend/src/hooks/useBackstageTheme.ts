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

export function useBackstageTheme() {
  useEffect(() => {
    applyBackstageTheme(loadColorTheme());

    const handleStorage = (e: StorageEvent) => {
      if (e.key === COLOR_THEME_STORAGE_KEY || e.key === null) {
        applyBackstageTheme(loadColorTheme());
      }
    };
    window.addEventListener('storage', handleStorage);

    let unlisten: (() => void) | undefined;
    void listen<ColorThemeId>('color-theme-changed', event => {
      applyBackstageTheme(event.payload);
    })
      .then(fn => {
        unlisten = fn;
      })
      .catch(() => {
        /* non-Tauri / test env */
      });

    return () => {
      window.removeEventListener('storage', handleStorage);
      unlisten?.();
    };
  }, []);
}
