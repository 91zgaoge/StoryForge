/**
 * 智能文思 — 表达层：统一智能提示系统
 *
 * 替代原有的 AiSuggestionBubble + FloatingAmbientHint，
 * 基于感知层和决策层的真实分析结果展示建议。
 *
 * 展示形式：
 *   - bubble:  右侧浮动气泡（与段落相关）
 *   - ambient: 屏幕边缘轻量提示
 *   - ghost:   通过 onGhostSuggestion 回调传给输入栏
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { X, ThumbsUp, ThumbsDown, Lightbulb, Zap, BookOpen, Type, MessageCircle, Wind, Clock, Layout } from 'lucide-react';
import type {
  WritingSuggestion,
  PresentationConfig,
} from './types';
import { DEFAULT_PRESENTATION_CONFIG } from './types';
import { analyzeText, hasEnoughContent } from './textAnalyzer';
import { generateSuggestions, filterSuggestions, selectNextSuggestion, recordFeedback } from './suggestionEngine';

// ==================== 类型图标映射 ====================

const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  pacing: <Clock className="w-3.5 h-3.5" />,
  dialogue: <MessageCircle className="w-3.5 h-3.5" />,
  description: <BookOpen className="w-3.5 h-3.5" />,
  vocabulary: <Type className="w-3.5 h-3.5" />,
  sentence: <Wind className="w-3.5 h-3.5" />,
  emotion: <Lightbulb className="w-3.5 h-3.5" />,
  plot: <Zap className="w-3.5 h-3.5" />,
  structure: <Layout className="w-3.5 h-3.5" />,
};

const CATEGORY_LABELS: Record<string, string> = {
  pacing: '节奏',
  dialogue: '对话',
  description: '描写',
  vocabulary: '词汇',
  sentence: '句式',
  emotion: '情感',
  plot: '情节',
  structure: '结构',
};

const CATEGORY_COLORS: Record<string, { bg: string; border: string; text: string; icon: string }> = {
  pacing:      { bg: 'bg-amber-50', border: 'border-amber-200', text: 'text-amber-700', icon: 'text-amber-500' },
  dialogue:    { bg: 'bg-sky-50', border: 'border-sky-200', text: 'text-sky-700', icon: 'text-sky-500' },
  description: { bg: 'bg-emerald-50', border: 'border-emerald-200', text: 'text-emerald-700', icon: 'text-emerald-500' },
  vocabulary:  { bg: 'bg-violet-50', border: 'border-violet-200', text: 'text-violet-700', icon: 'text-violet-500' },
  sentence:    { bg: 'bg-rose-50', border: 'border-rose-200', text: 'text-rose-700', icon: 'text-rose-500' },
  emotion:     { bg: 'bg-pink-50', border: 'border-pink-200', text: 'text-pink-700', icon: 'text-pink-500' },
  plot:        { bg: 'bg-orange-50', border: 'border-orange-200', text: 'text-orange-700', icon: 'text-orange-500' },
  structure:   { bg: 'bg-slate-50', border: 'border-slate-200', text: 'text-slate-700', icon: 'text-slate-500' },
};

// ==================== 组件 ====================

interface SmartHintSystemProps {
  /** 编辑器 HTML 内容 */
  htmlContent: string;
  /** 是否启用 */
  isEnabled: boolean;
  /** 禅模式（完全禁用） */
  isZenMode: boolean;
  /** 传递 Ghost Text 建议给输入栏 */
  onGhostSuggestion?: (text: string) => void;
  /** 展示配置 */
  config?: PresentationConfig;
}

interface DisplayedHint {
  suggestion: WritingSuggestion;
  visible: boolean;
  position: { top: number; right: number };
}

export const SmartHintSystem: React.FC<SmartHintSystemProps> = ({
  htmlContent,
  isEnabled,
  isZenMode,
  onGhostSuggestion,
  config = DEFAULT_PRESENTATION_CONFIG,
}) => {
  const [displayedHints, setDisplayedHints] = useState<DisplayedHint[]>([]);
  const [ambientHint, setAmbientHint] = useState<WritingSuggestion | null>(null);
  const [ambientVisible, setAmbientVisible] = useState(false);
  const [feedbackHistory, setFeedbackHistory] = useState<Map<string, boolean>>(() => {
    try {
      const saved = localStorage.getItem('storyforge-suggestion-feedback');
      if (saved) {
        const parsed = JSON.parse(saved);
        return new Map(Object.entries(parsed));
      }
    } catch { /* ignore */ }
    return new Map();
  });

  const analysisTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const displayTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastAnalyzedRef = useRef<string>('');
  const currentlyDisplayedIds = useRef<Set<string>>(new Set());

  // 分析文本并生成建议
  const performAnalysis = useCallback(() => {
    if (!isEnabled || isZenMode) return;
    if (!hasEnoughContent(htmlContent)) return;
    if (htmlContent === lastAnalyzedRef.current) return;

    lastAnalyzedRef.current = htmlContent;

    const perception = analyzeText(htmlContent);
    const decision = generateSuggestions(perception);
    const filtered = filterSuggestions(decision, feedbackHistory, {
      enableBubbles: config.enableBubbles,
      enableGhost: config.enableGhost,
      enableAmbient: config.enableAmbient,
    });

    // 处理 ghost 建议（通过回调）
    const ghostSuggestions = filtered.filter(s => s.presentation === 'ghost');
    if (ghostSuggestions.length > 0 && onGhostSuggestion) {
      onGhostSuggestion(ghostSuggestions[0].message);
    }

    // 处理 bubble 建议
    const bubbleSuggestions = filtered.filter(s => s.presentation === 'bubble');
    if (bubbleSuggestions.length > 0) {
      const next = selectNextSuggestion(
        bubbleSuggestions,
        currentlyDisplayedIds.current
      );
      if (next) {
        showBubbleHint(next);
      }
    }

    // 处理 ambient 建议（低优先级，不频繁展示）
    const ambientSuggestions = filtered.filter(s => s.presentation === 'ambient');
    if (ambientSuggestions.length > 0 && Math.random() > 0.5) {
      showAmbientHint(ambientSuggestions[0]);
    }
  }, [htmlContent, isEnabled, isZenMode, feedbackHistory, config, onGhostSuggestion]);

  // 显示气泡提示
  const showBubbleHint = useCallback((suggestion: WritingSuggestion) => {
    const position = {
      top: 15 + Math.random() * 55, // 15% - 70%
      right: 5 + Math.random() * 20, // 5% - 25%
    };

    const hint: DisplayedHint = {
      suggestion,
      visible: false,
      position,
    };

    currentlyDisplayedIds.current.add(suggestion.id);
    setDisplayedHints(prev => [...prev, hint]);

    // 渐显
    requestAnimationFrame(() => {
      setDisplayedHints(prev =>
        prev.map(h =>
          h.suggestion.id === suggestion.id ? { ...h, visible: true } : h
        )
      );
    });

    // 8秒后自动消失
    setTimeout(() => {
      setDisplayedHints(prev =>
        prev.map(h =>
          h.suggestion.id === suggestion.id ? { ...h, visible: false } : h
        )
      );
      setTimeout(() => {
        setDisplayedHints(prev => prev.filter(h => h.suggestion.id !== suggestion.id));
        currentlyDisplayedIds.current.delete(suggestion.id);
      }, 600);
    }, 8000);
  }, []);

  // 显示环境提示
  const showAmbientHint = useCallback((suggestion: WritingSuggestion) => {
    setAmbientHint(suggestion);
    setAmbientVisible(false);

    requestAnimationFrame(() => {
      setAmbientVisible(true);
    });

    setTimeout(() => {
      setAmbientVisible(false);
      setTimeout(() => setAmbientHint(null), 500);
    }, 6000);
  }, []);

  // 持久化反馈历史
  useEffect(() => {
    try {
      const obj = Object.fromEntries(feedbackHistory);
      localStorage.setItem('storyforge-suggestion-feedback', JSON.stringify(obj));
    } catch { /* ignore */ }
  }, [feedbackHistory]);

  // 用户反馈：有用
  const handleUseful = useCallback((suggestion: WritingSuggestion) => {
    setFeedbackHistory(prev => recordFeedback(suggestion, true, prev));
    setDisplayedHints(prev => prev.filter(h => h.suggestion.id !== suggestion.id));
    currentlyDisplayedIds.current.delete(suggestion.id);
  }, []);

  // 用户反馈：无用
  const handleUseless = useCallback((suggestion: WritingSuggestion) => {
    setFeedbackHistory(prev => recordFeedback(suggestion, false, prev));
    setDisplayedHints(prev => prev.filter(h => h.suggestion.id !== suggestion.id));
    currentlyDisplayedIds.current.delete(suggestion.id);
  }, []);

  // 关闭提示
  const handleDismiss = useCallback((suggestion: WritingSuggestion) => {
    setDisplayedHints(prev => prev.filter(h => h.suggestion.id !== suggestion.id));
    currentlyDisplayedIds.current.delete(suggestion.id);
  }, []);

  // 防抖分析：用户停止输入 3 秒后触发
  useEffect(() => {
    if (!isEnabled || isZenMode) {
      setDisplayedHints([]);
      setAmbientHint(null);
      return;
    }

    if (analysisTimerRef.current) {
      clearTimeout(analysisTimerRef.current);
    }

    analysisTimerRef.current = setTimeout(() => {
      performAnalysis();
    }, 3000);

    return () => {
      if (analysisTimerRef.current) {
        clearTimeout(analysisTimerRef.current);
      }
    };
  }, [htmlContent, isEnabled, isZenMode, performAnalysis]);

  // 定期清理过期提示
  useEffect(() => {
    if (!isEnabled || isZenMode) return;

    const cleanup = setInterval(() => {
      setDisplayedHints(prev => {
        const now = Date.now();
        const expired = prev.filter(h => now - h.suggestion.createdAt > 12000);
        if (expired.length > 0) {
          expired.forEach(h => currentlyDisplayedIds.current.delete(h.suggestion.id));
          return prev.filter(h => now - h.suggestion.createdAt <= 12000);
        }
        return prev;
      });
    }, 5000);

    return () => clearInterval(cleanup);
  }, [isEnabled, isZenMode]);

  if (!isEnabled || isZenMode) return null;

  return (
    <>
      {/* ===== 气泡提示 ===== */}
      <div className="smart-hint-container" aria-live="polite" aria-atomic="true">
        {displayedHints.map((hint) => {
          const cat = hint.suggestion.category;
          const colors = CATEGORY_COLORS[cat] || CATEGORY_COLORS.structure;
          const icon = CATEGORY_ICONS[cat] || <Lightbulb className="w-3.5 h-3.5" />;
          const label = CATEGORY_LABELS[cat] || '提示';

          return (
            <div
              key={hint.suggestion.id}
              className={`smart-hint-bubble ${hint.visible ? 'visible' : ''} ${colors.bg} ${colors.border}`}
              style={{
                top: `${hint.position.top}%`,
                right: `${hint.position.right}%`,
              }}
            >
              {/* 头部 */}
              <div className="flex items-center gap-1.5 mb-1.5">
                <span className={`${colors.icon}`}>{icon}</span>
                <span className={`text-[11px] font-medium ${colors.text}`}>{label}</span>
                <span className="text-[10px] text-gray-400 ml-auto opacity-60">{hint.suggestion.priority === 'high' ? '重要' : ''}</span>
              </div>

              {/* 内容 */}
              <p className={`text-xs leading-relaxed ${colors.text} opacity-90`}>
                {hint.suggestion.message}
              </p>

              {/* 反馈按钮 */}
              <div className="flex items-center justify-end gap-1 mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  onClick={() => handleUseful(hint.suggestion)}
                  className="p-1 rounded hover:bg-black/5 text-gray-400 hover:text-green-500 transition-colors"
                  title="有用"
                >
                  <ThumbsUp className="w-3 h-3" />
                </button>
                <button
                  onClick={() => handleUseless(hint.suggestion)}
                  className="p-1 rounded hover:bg-black/5 text-gray-400 hover:text-red-400 transition-colors"
                  title="无用"
                >
                  <ThumbsDown className="w-3 h-3" />
                </button>
                <button
                  onClick={() => handleDismiss(hint.suggestion)}
                  className="p-1 rounded hover:bg-black/5 text-gray-400 hover:text-gray-600 transition-colors"
                  title="关闭"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            </div>
          );
        })}
      </div>

      {/* ===== 环境提示 ===== */}
      {ambientHint && (
        <div className={`smart-ambient-hint ${ambientVisible ? 'visible' : ''}`}>
          <span className="smart-ambient-dot" />
          <span className="smart-ambient-text">{ambientHint.message}</span>
        </div>
      )}
    </>
  );
};

export default SmartHintSystem;
