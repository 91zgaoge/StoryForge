/**
 * 订阅管理 Hook — Freemium 付费系统 V2
 *
 * 管理用户订阅状态、AI 使用配额追踪、付费功能权限。
 * V2: 仅限制 auto_write / auto_revise，其余功能全部免费。
 */

import { useState, useEffect, useCallback } from 'react';
import {
  getSubscriptionStatus,
  getQuotaDetail,
  checkAutoWriteQuota,
  checkAutoReviseQuota,
  type SubscriptionStatus,
  type QuotaCheckResult,
  type QuotaDetail,
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
  // V2 按功能区分配额
  quotaDetail?: QuotaDetail;
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
      quotaDetail: state.quotaDetail,
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
  quotaDetail: cached?.quotaDetail,
};

export function useSubscription() {
  const [state, setState] = useState<SubscriptionState>(DEFAULT_STATE);

  const fetchStatus = useCallback(async () => {
    try {
      const [status, detail] = await Promise.all([
        getSubscriptionStatus(),
        getQuotaDetail().catch(() => undefined),
      ]);
      const newState: SubscriptionState = {
        tier: (status.tier as 'free' | 'pro' | 'enterprise') || 'free',
        status: status.status,
        dailyUsed: status.daily_used,
        dailyLimit: status.daily_limit,
        quotaResetsAt: status.quota_resets_at,
        expiresAt: status.expires_at,
        isLoading: false,
        error: null,
        quotaDetail: detail,
      };
      saveCachedState(newState);
      setState(newState);
    } catch (err) {
      console.error('Failed to fetch subscription status:', err);
      setState(prev => ({ ...prev, isLoading: false, error: '获取订阅状态失败' }));
    }
  }, []);

  // 检查自动续写配额
  const checkAutoWrite = useCallback(async (requestedChars: number): Promise<QuotaCheckResult> => {
    try {
      return await checkAutoWriteQuota(requestedChars);
    } catch (err) {
      console.error('Failed to check auto-write quota:', err);
      const detail = state.quotaDetail;
      const remaining = detail ? detail.auto_write_limit - detail.auto_write_used : 10;
      return {
        allowed: remaining > 0,
        remaining: Math.max(0, remaining),
        daily_limit: detail?.auto_write_limit ?? 10,
        daily_used: detail?.auto_write_used ?? 0,
        resets_at: state.quotaResetsAt,
        message: undefined,
      };
    }
  }, [state.quotaDetail, state.quotaResetsAt]);

  // 检查自动修改配额
  const checkAutoRevise = useCallback(async (requestedChars: number): Promise<QuotaCheckResult> => {
    try {
      return await checkAutoReviseQuota(requestedChars);
    } catch (err) {
      console.error('Failed to check auto-revise quota:', err);
      const detail = state.quotaDetail;
      const remaining = detail ? detail.auto_revise_limit - detail.auto_revise_used : 10;
      return {
        allowed: remaining > 0,
        remaining: Math.max(0, remaining),
        daily_limit: detail?.auto_revise_limit ?? 10,
        daily_used: detail?.auto_revise_used ?? 0,
        resets_at: state.quotaResetsAt,
        message: undefined,
      };
    }
  }, [state.quotaDetail, state.quotaResetsAt]);

  // V2: 仅限制 auto_write / auto_revise，其余全部免费
  const canUseFeature = useCallback(
    (feature: string): boolean => {
      // Pro 用户无限制
      if (state.tier === 'pro' || state.tier === 'enterprise') {
        return true;
      }

      // 免费用户：仅 auto_write / auto_revise 受限（需单独检查配额）
      // 其余所有功能（writer_agent_execute、format_text、chat 等）全部免费
      if (feature === 'auto_write' || feature === 'auto_revise') {
        // 返回 true 允许 UI 显示，实际配额检查在调用时进行
        return true;
      }

      return true;
    },
    [state.tier]
  );

  // 向后兼容：通用配额检查（V2 中所有功能已免费，返回 true）
  const hasQuota = useCallback(async (): Promise<boolean> => {
    return true;
  }, []);

  // 检查自动续写是否还有配额
  const hasAutoWriteQuota = useCallback(async (requestedChars: number): Promise<boolean> => {
    if (state.tier === 'pro' || state.tier === 'enterprise') return true;
    const result = await checkAutoWrite(requestedChars);
    return result.allowed;
  }, [state.tier, checkAutoWrite]);

  // 检查自动修改是否还有配额
  const hasAutoReviseQuota = useCallback(async (requestedChars: number): Promise<boolean> => {
    if (state.tier === 'pro' || state.tier === 'enterprise') return true;
    const result = await checkAutoRevise(requestedChars);
    return result.allowed;
  }, [state.tier, checkAutoRevise]);

  // 获取配额状态文本
  const getQuotaText = useCallback((): string => {
    if (state.tier === 'pro' || state.tier === 'enterprise') {
      return 'Pro · 无限';
    }
    const d = state.quotaDetail;
    if (!d) return '免费版';
    const awRemaining = d.auto_write_limit - d.auto_write_used;
    const arRemaining = d.auto_revise_limit - d.auto_revise_used;
    return `续写 ${awRemaining}/${d.auto_write_limit} · 修改 ${arRemaining}/${d.auto_revise_limit}`;
  }, [state.tier, state.quotaDetail]);

  // 初始加载
  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  return {
    ...state,
    isPro: state.tier === 'pro' || state.tier === 'enterprise',
    isFree: state.tier === 'free',
    fetchStatus,
    hasQuota,
    checkAutoWrite,
    checkAutoRevise,
    canUseFeature,
    hasAutoWriteQuota,
    hasAutoReviseQuota,
    getQuotaText,
  };
}

export default useSubscription;
