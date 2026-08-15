/**
 * 幕后主题全局接线：挂载即应用当前幕后色调，
 * 并监听 storage / Tauri color-theme-changed（仅 surface=back）。
 */
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  COLOR_THEME_STORAGE_KEY_BACK,
  COLOR_THEME_STORAGE_KEY_LEGACY,
  loadColorTheme,
  parseThemeEventPayload,
} from '@/frontstage/config/colorThemes';
import { applyBackstageTheme } from '@/styles/backstageThemes';

export function useBackstageTheme(): void {
  useEffect(() => {
    applyBackstageTheme(loadColorTheme('back'));

    const handleStorage = (e: StorageEvent) => {
      if (
        e.key === COLOR_THEME_STORAGE_KEY_BACK ||
        e.key === COLOR_THEME_STORAGE_KEY_LEGACY ||
        e.key === null
      ) {
        applyBackstageTheme(loadColorTheme('back'));
      }
    };
    window.addEventListener('storage', handleStorage);

    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void listen('color-theme-changed', event => {
      const parsed = parseThemeEventPayload(event.payload);
      if (parsed?.surface === 'back') {
        applyBackstageTheme(parsed.id);
      }
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
