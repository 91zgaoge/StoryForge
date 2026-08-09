/**
 * Auth Store — 认证状态管理
 * v4.5.0
 */

import { create } from 'zustand';
import { createLogger } from '@/utils/logger';
import { extractMessage } from '@/utils/errorHandler';
import type { UserInfo, AuthConfig } from '@/services/auth';
import {
  getAuthConfig,
  getCurrentUser,
  logout as logoutApi,
  openOAuthBrowser,
  oauthPollLogin,
} from '@/services/auth';

const authLogger = createLogger('auth:store');

/** server 类型化失败通道：desktop_poll 403 时 Rust 侧以 `AUTH_FAILED:<code>` 前缀上抛 */
const AUTH_FAILED_RE = /AUTH_FAILED:([a-z_]+)/;

/** 提取轮询错误中的失败码并映射为中文文案；非失败通道错误返回 null */
const mapAuthFailedMessage = (error: unknown): string | null => {
  const code = AUTH_FAILED_RE.exec(extractMessage(error))?.[1];
  if (!code) return null;
  return code === 'invalid_or_used_invite' ? '邀请码无效或已被使用' : '登录失败，请重试';
};

interface AuthState {
  // State
  user: UserInfo | null;
  isLoggedIn: boolean;
  isLoading: boolean;
  isWaitingForOAuth: boolean;
  authConfig: AuthConfig | null;
  authToken: string | null;

  // Actions
  setUser: (user: UserInfo | null) => void;
  setAuthToken: (token: string | null) => void;
  login: (provider: string, invite?: string) => Promise<void>;
  cancelLogin: () => void;
  logout: () => Promise<void>;
  checkAuth: () => Promise<void>;
  loadAuthConfig: () => Promise<void>;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  user: null,
  isLoggedIn: false,
  isLoading: false,
  isWaitingForOAuth: false,
  authConfig: null,
  authToken: localStorage.getItem('sf_auth_token'),

  setUser: user => set({ user, isLoggedIn: !!user }),

  setAuthToken: token => {
    if (token) {
      localStorage.setItem('sf_auth_token', token);
    } else {
      localStorage.removeItem('sf_auth_token');
    }
    set({ authToken: token });
  },

  login: async (provider: string, invite?: string) => {
    set({ isLoading: true, isWaitingForOAuth: true });
    try {
      const resp = await openOAuthBrowser(provider, invite);
      const deadline = Date.now() + 120_000;
      while (Date.now() < deadline) {
        if (!get().isWaitingForOAuth) return; // 用户取消
        let session;
        try {
          session = await oauthPollLogin(resp.dstate);
        } catch (e) {
          // server 类型化失败通道（如错邀请码）：立即终止轮询，按 code 映射文案
          const mapped = mapAuthFailedMessage(e);
          if (mapped) throw new Error(mapped);
          throw e;
        }
        if (session) {
          get().setAuthToken(session.token);
          set({ user: session.user, isLoggedIn: true });
          return;
        }
        await new Promise(r => setTimeout(r, 2000));
      }
      throw new Error('登录超时，请重试');
    } finally {
      set({ isLoading: false, isWaitingForOAuth: false });
    }
  },

  cancelLogin: () => set({ isWaitingForOAuth: false }),

  logout: async () => {
    const { authToken, setAuthToken } = get();
    if (authToken) {
      try {
        await logoutApi(authToken);
      } catch (e) {
        authLogger.error('Logout API error', { error: e });
      }
    }
    setAuthToken(null);
    set({ user: null, isLoggedIn: false });
  },

  checkAuth: async () => {
    try {
      const session = await getCurrentUser();
      if (session) {
        get().setAuthToken(session.token);
        set({ user: session.user, isLoggedIn: true });
      }
    } catch (e) {
      authLogger.error('Auth check failed', { error: e });
    }
  },

  loadAuthConfig: async () => {
    try {
      const config = await getAuthConfig();
      set({ authConfig: config });
    } catch (e) {
      authLogger.error('Failed to load auth config', { error: e });
    }
  },
}));
