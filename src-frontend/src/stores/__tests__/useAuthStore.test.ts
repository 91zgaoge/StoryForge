/**
 * useAuthStore Tests — 登录轮询流程 / 取消 / checkAuth
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useAuthStore } from '@/stores/useAuthStore';
import { openOAuthBrowser, oauthPollLogin, getCurrentUser } from '@/services/auth';

vi.mock('@/services/auth', () => ({
  openOAuthBrowser: vi.fn(),
  oauthPollLogin: vi.fn(),
  getCurrentUser: vi.fn(),
  getAuthConfig: vi.fn(),
  logout: vi.fn(),
}));

const mockedOpenBrowser = vi.mocked(openOAuthBrowser);
const mockedPoll = vi.mocked(oauthPollLogin);
const mockedGetCurrentUser = vi.mocked(getCurrentUser);

const session = { user: { id: 'u1' }, token: 'tok' };

describe('useAuthStore login 轮询流程', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    localStorage.clear();
    useAuthStore.setState({
      user: null,
      isLoggedIn: false,
      isLoading: false,
      isWaitingForOAuth: false,
      authToken: null,
    });
    mockedOpenBrowser.mockResolvedValue({ auth_url: 'https://auth.example.com', dstate: 'd1' });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it('轮询成功后落 user/token 并写入 localStorage', async () => {
    mockedPoll
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(session);

    const p = useAuthStore.getState().login('google', 'BETA-1');
    await vi.advanceTimersByTimeAsync(4000);
    await vi.advanceTimersByTimeAsync(0);
    await p;

    expect(mockedOpenBrowser).toHaveBeenCalledWith('google', 'BETA-1');
    expect(mockedPoll).toHaveBeenCalledTimes(3);
    expect(mockedPoll).toHaveBeenCalledWith('d1');

    const state = useAuthStore.getState();
    expect(state.isLoggedIn).toBe(true);
    expect(state.authToken).toBe('tok');
    expect(state.user).toEqual({ id: 'u1' });
    expect(state.isWaitingForOAuth).toBe(false);
    expect(state.isLoading).toBe(false);
    expect(localStorage.getItem('sf_auth_token')).toBe('tok');
  });

  it('cancelLogin 后不再轮询且 isWaitingForOAuth=false', async () => {
    mockedPoll.mockResolvedValue(null);

    const p = useAuthStore.getState().login('google');
    // 让 openOAuthBrowser 与第一次 poll 完成
    await vi.advanceTimersByTimeAsync(0);
    expect(useAuthStore.getState().isWaitingForOAuth).toBe(true);
    expect(mockedPoll).toHaveBeenCalledTimes(1);

    useAuthStore.getState().cancelLogin();
    await vi.advanceTimersByTimeAsync(10_000);
    await p;

    expect(mockedPoll).toHaveBeenCalledTimes(1);
    const state = useAuthStore.getState();
    expect(state.isWaitingForOAuth).toBe(false);
    expect(state.isLoggedIn).toBe(false);
  });

  it('poll 抛 AUTH_FAILED:invalid_or_used_invite → 立即 reject 邀请码文案且不再轮询', async () => {
    mockedPoll.mockRejectedValue({
      code: 'INTERNAL_ERROR',
      message: 'AUTH_FAILED:invalid_or_used_invite',
      severity: 'Fatal',
    });

    const p = useAuthStore.getState().login('google', 'BAD-CODE');
    const assertion = expect(p).rejects.toThrow('邀请码无效或已被使用');
    await vi.advanceTimersByTimeAsync(0);
    await assertion;

    // 立即终止：只轮询一次，不再等 120s 超时
    expect(mockedPoll).toHaveBeenCalledTimes(1);
    const state = useAuthStore.getState();
    expect(state.isWaitingForOAuth).toBe(false);
    expect(state.isLoading).toBe(false);
    expect(state.isLoggedIn).toBe(false);
  });

  it('checkAuth 适配 CurrentSession（含 token）', async () => {
    mockedGetCurrentUser.mockResolvedValue(session);

    await useAuthStore.getState().checkAuth();

    const state = useAuthStore.getState();
    expect(state.isLoggedIn).toBe(true);
    expect(state.authToken).toBe('tok');
    expect(state.user).toEqual({ id: 'u1' });
    expect(localStorage.getItem('sf_auth_token')).toBe('tok');
  });
});
