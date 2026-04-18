/**
 * 付费引导面板 — 转化漏斗核心 UI
 *
 * 当免费用户触发 Pro 功能时弹出，展示升级价值并引导转化。
 */

import React, { useState } from 'react';
import { Sparkles, Zap, Palette, BookOpen, Infinity, X, Loader2 } from 'lucide-react';
import { devUpgradeSubscription } from '@/services/tauri';
import toast from 'react-hot-toast';

interface UpgradePanelProps {
  isOpen: boolean;
  onClose: () => void;
  trigger?: string;
  onUpgraded?: () => void;
}

const features = [
  { icon: Zap, title: '无限 AI 创作', desc: '突破每日 10 次限制，文思泉涌不间断' },
  { icon: Palette, title: '风格 DNA 植入', desc: '让 AI 学习并模仿你的专属文风' },
  { icon: BookOpen, title: '创作方法论', desc: '雪花法、英雄之旅等结构化创作辅助' },
  { icon: Sparkles, title: '智能内联改写', desc: '选中文字一键 AI 润色，Tab 接受修改' },
];

export const UpgradePanel: React.FC<UpgradePanelProps> = ({
  isOpen,
  onClose,
  trigger,
  onUpgraded,
}) => {
  const [isUpgrading, setIsUpgrading] = useState(false);

  if (!isOpen) return null;

  const handleUpgrade = async () => {
    if (isUpgrading) return;
    setIsUpgrading(true);
    try {
      await devUpgradeSubscription('pro');
      toast.success('🎉 升级成功！欢迎体验专业版功能');
      onUpgraded?.();
      onClose();
    } catch (err) {
      console.error('Upgrade failed:', err);
      toast.error('升级失败，请稍后重试');
    } finally {
      setIsUpgrading(false);
    }
  };

  return (
    <div className="upgrade-panel-overlay" onClick={onClose}>
      <div className="upgrade-panel" onClick={e => e.stopPropagation()}>
        <button className="upgrade-panel-close" onClick={onClose}>
          <X size={18} />
        </button>

        <div className="upgrade-panel-header">
          <div className="upgrade-panel-icon">
            <Infinity size={32} />
          </div>
          <h2 className="upgrade-panel-title">解锁文思泉涌</h2>
          <p className="upgrade-panel-subtitle">
            {trigger || '升级专业版，释放 AI 创作全部潜能'}
          </p>
        </div>

        <div className="upgrade-panel-features">
          {features.map((f, i) => (
            <div key={i} className="upgrade-feature">
              <div className="upgrade-feature-icon">
                <f.icon size={18} />
              </div>
              <div className="upgrade-feature-text">
                <span className="upgrade-feature-title">{f.title}</span>
                <span className="upgrade-feature-desc">{f.desc}</span>
              </div>
            </div>
          ))}
        </div>

        <div className="upgrade-panel-pricing">
          <div className="upgrade-price">
            <span className="upgrade-price-amount">¥19</span>
            <span className="upgrade-price-unit">/月</span>
          </div>
          <p className="upgrade-price-note">限时早鸟价 · 随时可退订</p>
        </div>

        <div className="upgrade-panel-actions">
          <button className="upgrade-btn-primary" onClick={handleUpgrade} disabled={isUpgrading}>
            {isUpgrading ? <Loader2 size={16} className="spin" /> : <Sparkles size={16} />}
            {isUpgrading ? '升级中...' : '立即升级'}
          </button>
          <button className="upgrade-btn-secondary" onClick={onClose}>
            继续使用免费版
          </button>
        </div>

        <p className="upgrade-panel-footer">
          当前为开发测试模式，点击升级即可解锁全部功能
        </p>
      </div>
    </div>
  );
};

export default UpgradePanel;
