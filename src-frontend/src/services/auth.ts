/**
 * Auth Service — 认证相关 IPC 调用
 * v4.5.0
 */

import { loggedInvoke } from '@/services/tauri';
import { open } from '@tauri-apps/plugin-shell';

export interface AuthConfig {
  google_enabled: boolean;
  github_enabled: boolean;
  wechat_enabled: boolean;
  qq_enabled: boolean;
}

export interface UserInfo {
  id: string;
  email?: string;
  display_name?: string;
  avatar_url?: string;
}

export interface OAuthStartResponse {
  auth_url: string;
  dstate: string;
}

export interface CurrentSession {
  user: UserInfo;
  token: string;
}

/**
 * 获取认证配置
 */
export const getAuthConfig = () => loggedInvoke<AuthConfig>('get_auth_config');

/**
 * 开始 OAuth 登录流程（可携带邀请码，新用户注册必填）
 */
export const oauthStart = (provider: string, invite?: string) =>
  loggedInvoke<OAuthStartResponse>('oauth_start', { provider, invite: invite ?? null });

/**
 * 轮询 OAuth 登录结果，完成授权后返回会话
 */
export const oauthPollLogin = (dstate: string) =>
  loggedInvoke<CurrentSession | null>('oauth_poll_login', { dstate });

/**
 * 获取当前登录会话
 */
export const getCurrentUser = () => loggedInvoke<CurrentSession | null>('get_current_user');

/**
 * 注销登录
 */
export const logout = (token: string) => loggedInvoke<void>('logout', { token });

/**
 * 打开系统浏览器进行 OAuth 授权
 */
export const openOAuthBrowser = async (
  provider: string,
  invite?: string
): Promise<OAuthStartResponse> => {
  const resp = await oauthStart(provider, invite);
  await open(resp.auth_url);
  return resp;
};
