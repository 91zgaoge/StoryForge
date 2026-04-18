/**
 * 订阅管理 Hook — Freemium 付费系统
 *
 * 管理用户订阅状态、AI 配额检查、付费功能权限。
 */

import { useState, useEffect, useCallback } from 'react';
import {
  getSubscriptionStatus,
  checkAiQuota,
  type SubscriptionStatus,
  type QuotaCheckResult,
} from '@/services/tauri';

export interface SubscriptionState {
  tier: 'free' | 'pro' | 'enterprise';
  status: string;
  dailyUsed: number;
  dailyLimit: number;
  quotaResetsAt: string;
  expiresAt?: string;
  isLoading: boolean;
  error: string | null;
}

const STORAGE_KEY = 'storyforge_subscription_cache';

function loadCachedState(): Partial<SubscriptionState> | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* ignore */ }
  return null;
}

function saveCachedState(state: SubscriptionState) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      tier: state.tier,
      status: state.status,
      dailyUsed: state.dailyUsed,
      dailyLimit: state.dailyLimit,
      quotaResetsAt: state.quotaResetsAt,
      expiresAt: state.expiresAt,
    }));
  } catch { /* ignore */ }
}

const cached = loadCachedState();
const DEFAULT_STATE: SubscriptionState = {
  tier: cached?.tier || 'free',
  status: cached?.status || 'active',
  dailyUsed: cached?.dailyUsed ?? 0,
  dailyLimit: cached?.dailyLimit ?? 10,
  quotaResetsAt: cached?.quotaResetsAt || '',
  expiresAt: cached?.expiresAt,
  isLoading: true,
  error: null,
};

export function useSubscription() {
  const [state, setState] = useState<SubscriptionState>(DEFAULT_STATE);

  const fetchStatus = useCallback(async () => {
    try {
      const status = await getSubscriptionStatus();
      const newState: SubscriptionState = {
        tier: (status.tier as 'free' | 'pro' | 'enterprise') || 'free',
        status: status.status,
        dailyUsed: status.daily_used,
        dailyLimit: status.daily_limit,
        quotaResetsAt: status.quota_resets_at,
        expiresAt: status.expires_at,
        isLoading: false,
        error: null,
      };
      saveCachedState(newState);
      setState(newState);
    } catch (err) {
      console.error('Failed to fetch subscription status:', err);
      setState(prev => ({ ...prev, isLoading: false, error: '获取订阅状态失败' }));
    }
  }, []);

  // 检查 AI 配额
  const checkQuota = useCallback(async (): Promise<QuotaCheckResult> => {
    try {
      return await checkAiQuota();
    } catch (err) {
      console.error('Failed to check AI quota:', err);
      // 乐观策略：前端检查失败时允许继续，后端会做最终校验
      // 避免网络/DB 抖动时免费用户被误杀
      return {
        allowed: true,
        remaining: Math.max(0, state.dailyLimit - state.dailyUsed),
        daily_limit: state.dailyLimit,
        daily_used: state.dailyUsed,
        resets_at: state.quotaResetsAt,
        message: '配额检查异常，已允许本次使用',
      };
    }
  }, [state.dailyLimit, state.dailyUsed, state.quotaResetsAt]);

  // 检查是否可以使用某项功能
  const canUseFeature = useCallback(
    (feature: string): boolean => {
      const proFeatures = [
        'inline-suggestion',
        'smart-ghost-text',
        'input-history',
        'style-dna',
        'methodology',
        'feedback-loop',
      ];

      if (state.tier === 'pro' || state.tier === 'enterprise') {
        return true;
      }

      // 免费版：限制专业功能
      if (proFeatures.includes(feature)) {
        return false;
      }

      return true;
    },
    [state.tier]
  );

  // 检查是否有剩余 AI 配额
  const hasQuota = useCallback(async (): Promise<boolean> => {
    if (state.tier === 'pro' || state.tier === 'enterprise') {
      return true;
    }
    const result = await checkQuota();
    return result.allowed;
  }, [state.tier, checkQuota]);

  // 初始加载
  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  return {
    ...state,
    isPro: state.tier === 'pro' || state.tier === 'enterprise',
    isFree: state.tier === 'free',
    fetchStatus,
    checkQuota,
    canUseFeature,
    hasQuota,
  };
}

export default useSubscription;
