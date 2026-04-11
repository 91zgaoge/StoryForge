import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface UpdateInfo {
  version: string;
  notes: string;
  pub_date: string;
  signature: string;
}

export interface CheckUpdateResult {
  has_update: boolean;
  current_version: string;
  latest_version: string | null;
  update_info: UpdateInfo | null;
}

export interface UseUpdaterReturn {
  currentVersion: string;
  hasUpdate: boolean;
  latestVersion: string | null;
  updateInfo: UpdateInfo | null;
  isChecking: boolean;
  isInstalling: boolean;
  error: string | null;
  checkUpdate: () => Promise<void>;
  installUpdate: () => Promise<void>;
  dismissUpdate: () => void;
}

export function useUpdater(autoCheck: boolean = true): UseUpdaterReturn {
  const [currentVersion, setCurrentVersion] = useState<string>('');
  const [hasUpdate, setHasUpdate] = useState<boolean>(false);
  const [latestVersion, setLatestVersion] = useState<string | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [isChecking, setIsChecking] = useState<boolean>(false);
  const [isInstalling, setIsInstalling] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // 获取当前版本
  useEffect(() => {
    invoke<string>('get_current_version')
      .then(setCurrentVersion)
      .catch(console.error);
  }, []);

  // 检查更新
  const checkUpdate = useCallback(async () => {
    setIsChecking(true);
    setError(null);

    try {
      const result = await invoke<CheckUpdateResult>('check_update');
      
      setHasUpdate(result.has_update);
      setLatestVersion(result.latest_version);
      setUpdateInfo(result.update_info);

      if (result.has_update) {
        console.log(`[Updater] New version available: ${result.latest_version}`);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('[Updater] Check update failed:', err);
    } finally {
      setIsChecking(false);
    }
  }, []);

  // 安装更新
  const installUpdate = useCallback(async () => {
    if (!hasUpdate) return;

    setIsInstalling(true);
    setError(null);

    try {
      await invoke('install_update');
      // 如果安装成功，应用会重启，这里不会执行到
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('[Updater] Install update failed:', err);
    } finally {
      setIsInstalling(false);
    }
  }, [hasUpdate]);

  // 忽略更新
  const dismissUpdate = useCallback(() => {
    setHasUpdate(false);
    // 可以在这里记录忽略的版本，避免频繁提示
    if (latestVersion) {
      localStorage.setItem('dismissed_update_version', latestVersion);
    }
  }, [latestVersion]);

  // 自动检查更新
  useEffect(() => {
    if (autoCheck) {
      // 启动时检查一次
      checkUpdate();

      // 每24小时检查一次
      const interval = setInterval(checkUpdate, 24 * 60 * 60 * 1000);
      return () => clearInterval(interval);
    }
  }, [autoCheck, checkUpdate]);

  return {
    currentVersion,
    hasUpdate,
    latestVersion,
    updateInfo,
    isChecking,
    isInstalling,
    error,
    checkUpdate,
    installUpdate,
    dismissUpdate,
  };
}
