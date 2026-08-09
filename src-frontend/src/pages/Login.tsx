/**
 * Login Page — 登录弹窗/页面
 * v4.5.0
 */

import { useState, useEffect } from 'react';
import { X, Chrome, Github, MessageCircle, Loader2 } from 'lucide-react';
import { useAuthStore } from '@/stores/useAuthStore';
import { extractMessage } from '@/utils/errorHandler';
import { Card } from '@/components/ui/Card';
import toast from 'react-hot-toast';

interface LoginModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function LoginModal({ isOpen, onClose }: LoginModalProps) {
  const { authConfig, login, isLoading, isWaitingForOAuth, cancelLogin } = useAuthStore();
  const [isVisible, setIsVisible] = useState(false);
  const [invite, setInvite] = useState('');

  useEffect(() => {
    if (isOpen) {
      setIsVisible(true);
    } else {
      const timer = setTimeout(() => setIsVisible(false), 200);
      return () => clearTimeout(timer);
    }
  }, [isOpen]);

  useEffect(() => {
    if (isOpen) {
      useAuthStore.getState().loadAuthConfig();
    }
  }, [isOpen]);

  if (!isVisible) return null;

  const handleLogin = async (provider: string) => {
    try {
      await login(provider, invite.trim() || undefined);
      if (useAuthStore.getState().isLoggedIn) {
        toast.success('登录成功');
        onClose();
      }
    } catch (error) {
      const message = extractMessage(error);
      // 以 server 失败码为准；正则兜底兼容旧文案
      const code = /AUTH_FAILED:([a-z_]+)/.exec(message)?.[1];
      if (code === 'invalid_or_used_invite' || /邀请|invite/i.test(message)) {
        toast.error('邀请码无效或已使用');
      } else {
        toast.error(`登录失败: ${message}`);
      }
    }
  };

  return (
    <div
      className={`fixed inset-0 z-50 flex items-center justify-center transition-opacity duration-200 ${
        isOpen ? 'opacity-100' : 'opacity-0'
      }`}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/40 backdrop-blur-sm" onClick={onClose} />

      {/* Modal */}
      <Card className="relative w-full max-w-md mx-4 p-6 transform transition-all duration-200">
        {/* Close button */}
        <button
          onClick={onClose}
          className="absolute right-4 top-4 p-1 text-stone-400 hover:text-stone-600 rounded-md hover:bg-stone-100 transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Header */}
        <div className="text-center mb-6">
          <h2 className="text-xl font-semibold text-stone-800">登录 StoryMoss</h2>
          <p className="text-sm text-stone-500 mt-1">登录后可解锁云同步等跨设备功能</p>
        </div>

        {/* OAuth Buttons / 等待授权 */}
        {isWaitingForOAuth ? (
          <div className="text-center py-6 space-y-4">
            <Loader2 className="w-8 h-8 mx-auto animate-spin text-stone-500" />
            <p className="text-sm text-stone-500">等待浏览器授权…</p>
            <button
              onClick={cancelLogin}
              className="px-4 py-2 text-sm text-stone-600 border border-stone-200 rounded-lg hover:bg-stone-50 hover:border-stone-300 transition-all"
            >
              取消
            </button>
          </div>
        ) : (
          <div className="space-y-3">
            {/* 邀请码（新用户注册必填） */}
            <input
              type="text"
              value={invite}
              onChange={e => setInvite(e.target.value)}
              placeholder="邀请码（新用户注册必填）"
              disabled={isLoading}
              className="w-full px-3 py-2.5 text-sm bg-white border border-stone-200 rounded-lg text-stone-700 placeholder:text-stone-400 focus:outline-none focus:border-stone-400 transition-all disabled:opacity-50"
            />

            {authConfig?.google_enabled && (
              <button
                onClick={() => handleLogin('google')}
                disabled={isLoading}
                className="w-full flex items-center justify-center gap-3 px-4 py-2.5 bg-white border border-stone-200 rounded-lg text-stone-700 hover:bg-stone-50 hover:border-stone-300 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Chrome className="w-5 h-5 text-blue-500" />
                <span className="text-sm font-medium">使用 Google 登录</span>
              </button>
            )}

            {authConfig?.github_enabled && (
              <button
                onClick={() => handleLogin('github')}
                disabled={isLoading}
                className="w-full flex items-center justify-center gap-3 px-4 py-2.5 bg-white border border-stone-200 rounded-lg text-stone-700 hover:bg-stone-50 hover:border-stone-300 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Github className="w-5 h-5 text-stone-800" />
                <span className="text-sm font-medium">使用 GitHub 登录</span>
              </button>
            )}

            {/* WeChat — 预留，未启用时显示提示 */}
            {authConfig?.wechat_enabled && (
              <button
                onClick={() => handleLogin('wechat')}
                disabled={isLoading}
                className="w-full flex items-center justify-center gap-3 px-4 py-2.5 bg-white border border-stone-200 rounded-lg text-stone-700 hover:bg-stone-50 hover:border-stone-300 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <MessageCircle className="w-5 h-5 text-green-500" />
                <span className="text-sm font-medium">使用微信登录</span>
              </button>
            )}

            {/* QQ — 预留 */}
            {authConfig?.qq_enabled && (
              <button
                onClick={() => handleLogin('qq')}
                disabled={isLoading}
                className="w-full flex items-center justify-center gap-3 px-4 py-2.5 bg-white border border-stone-200 rounded-lg text-stone-700 hover:bg-stone-50 hover:border-stone-300 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <MessageCircle className="w-5 h-5 text-blue-400" />
                <span className="text-sm font-medium">使用 QQ 登录</span>
              </button>
            )}

            {/* 无可用provider时的提示 */}
            {authConfig &&
              !authConfig.google_enabled &&
              !authConfig.github_enabled &&
              !authConfig.wechat_enabled &&
              !authConfig.qq_enabled && (
                <div className="text-center py-4">
                  <p className="text-sm text-stone-500">尚未配置 OAuth 登录选项</p>
                  <p className="text-xs text-stone-400 mt-1">请在设置中配置 OAuth 客户端信息</p>
                </div>
              )}
          </div>
        )}

        {/* Footer */}
        <div className="mt-6 pt-4 border-t border-stone-100">
          <p className="text-xs text-stone-400 text-center">
            登录即表示您同意我们的服务条款和隐私政策
          </p>
        </div>
      </Card>
    </div>
  );
}

export default LoginModal;
