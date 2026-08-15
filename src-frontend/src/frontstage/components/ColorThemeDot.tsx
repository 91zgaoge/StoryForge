/**
 * ColorThemeDot - 幕前色调主题切换器
 *
 * 嵌入在顶部 header 中，"开启文思"按钮左侧
 * 平时：12px 半透明小圆点
 * 悬停：展开 12 色选择面板（只改幕前）
 * Zen 模式：隐藏
 */

import React, { useState, useCallback, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { cn } from '@/utils/cn';
import {
  colorThemes,
  colorThemeList,
  type ColorThemeId,
  loadColorTheme,
  saveColorTheme,
  applyColorTheme,
  parseThemeEventPayload,
  COLOR_THEME_STORAGE_KEY_FRONT,
  COLOR_THEME_STORAGE_KEY_LEGACY,
} from '@/frontstage/config/colorThemes';

interface ColorThemeDotProps {
  isZenMode?: boolean;
}

const ColorThemeDot: React.FC<ColorThemeDotProps> = ({ isZenMode = false }) => {
  const [currentThemeId, setCurrentThemeId] = useState<ColorThemeId>(() => loadColorTheme('front'));
  const [isHovered, setIsHovered] = useState(false);
  const [panelOpen, setPanelOpen] = useState(false);
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    applyColorTheme(currentThemeId);
  }, []);

  useEffect(() => {
    const handleThemeChange = (themeId: ColorThemeId) => {
      setCurrentThemeId(themeId);
      applyColorTheme(themeId);
    };

    const handleStorageChange = (e: StorageEvent) => {
      if (
        e.key === COLOR_THEME_STORAGE_KEY_FRONT ||
        e.key === COLOR_THEME_STORAGE_KEY_LEGACY ||
        e.key === null
      ) {
        handleThemeChange(loadColorTheme('front'));
      }
    };
    window.addEventListener('storage', handleStorageChange);

    let unlisten: (() => void) | undefined;
    void listen('color-theme-changed', event => {
      const parsed = parseThemeEventPayload(event.payload);
      if (parsed?.surface === 'front') {
        handleThemeChange(parsed.id);
      }
    })
      .then(fn => {
        unlisten = fn;
      })
      .catch(() => {
        /* non-Tauri / test env */
      });

    return () => {
      window.removeEventListener('storage', handleStorageChange);
      unlisten?.();
    };
  }, []);

  const handleSelect = useCallback((themeId: ColorThemeId) => {
    setCurrentThemeId(themeId);
    saveColorTheme('front', themeId);
    applyColorTheme(themeId);
    setPanelOpen(false);
    setIsHovered(false);
  }, []);

  const handleMouseEnter = useCallback(() => {
    if (hideTimer.current) {
      clearTimeout(hideTimer.current);
      hideTimer.current = null;
    }
    setIsHovered(true);
    setPanelOpen(true);
  }, []);

  const handleMouseLeave = useCallback(() => {
    hideTimer.current = setTimeout(() => {
      setPanelOpen(false);
      setIsHovered(false);
    }, 200);
  }, []);

  if (isZenMode) return null;

  const currentTheme = colorThemes[currentThemeId] ?? colorThemes.zhuhong;

  return (
    <div
      className="color-theme-dot-wrapper"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <div className={cn('color-theme-panel', panelOpen && 'open')}>
        <div className="color-theme-panel-title">幕前色调</div>
        <div className="color-theme-options">
          {colorThemeList.map(theme => (
            <button
              key={theme.id}
              className={cn('color-theme-option', currentThemeId === theme.id && 'active')}
              onClick={() => handleSelect(theme.id)}
              title={theme.description}
            >
              <span className="color-theme-swatch" style={{ backgroundColor: theme.terracotta }} />
              <span className="color-theme-label">{theme.name}</span>
            </button>
          ))}
        </div>
      </div>

      <div
        className={cn('color-theme-dot', isHovered && 'hovered')}
        style={{ backgroundColor: currentTheme.terracotta }}
        title="切换幕前色调"
      />
    </div>
  );
};

export default ColorThemeDot;
