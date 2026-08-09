/**
 * Auth Store — 认证状态管理
 * v4.5.0
 */

import { create } from 'zustand';
import { createLogger } from '@/utils/logger';
import type { UserInfo, AuthConfig } from '@/services/auth';
import {
  getAuthConfig,
  getCurrentUser,
  logout as logoutApi,
  openOAuthBrowser,
  oauthPollLogin,
} from '@/services/auth';

const authLogger = createLogger('auth:store');

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
        const session = await oauthPollLogin(resp.dstate);
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
