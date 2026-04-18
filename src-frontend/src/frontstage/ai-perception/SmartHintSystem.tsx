/**
 * 智能文思 — 感知层+决策层集成组件
 *
 * 职责：分析编辑器文本，发现问题，通过回调通知父组件生成内联修改建议。
 * 不再直接展示任何 UI（气泡/环境提示已移除），所有建议以 TipTap aiSuggestion 节点呈现。
 */

import React, { useEffect, useRef, useCallback } from 'react';
import type { WritingSuggestion } from './types';
import { analyzeText, hasEnoughContent } from './textAnalyzer';
import { generateSuggestions } from './suggestionEngine';

interface SmartHintSystemProps {
  /** 编辑器 HTML 内容 */
  htmlContent: string;
  /** 是否启用分析 */
  isEnabled: boolean;
  /** 禅模式（完全禁用） */
  isZenMode: boolean;
  /** 当发现需要内联修改的高优先级建议时回调 */
  onInlineSuggestion?: (suggestion: WritingSuggestion, targetParagraphText: string) => void;
  /** 传递 Ghost Text 建议给输入栏（低优先级建议） */
  onGhostSuggestion?: (text: string) => void;
}

export const SmartHintSystem: React.FC<SmartHintSystemProps> = ({
  htmlContent,
  isEnabled,
  isZenMode,
  onInlineSuggestion,
  onGhostSuggestion,
}) => {
  const analysisTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastAnalyzedRef = useRef<string>('');
  const pendingSuggestionRef = useRef<Set<string>>(new Set());

  const performAnalysis = useCallback(() => {
    if (!isEnabled || isZenMode) return;
    if (!hasEnoughContent(htmlContent)) return;
    if (htmlContent === lastAnalyzedRef.current) return;

    lastAnalyzedRef.current = htmlContent;

    const perception = analyzeText(htmlContent);
    const decision = generateSuggestions(perception);

    // 优先处理高优先级的内联修改建议
    const highPriority = decision.suggestions.filter(
      s => s.priority === 'high' && !pendingSuggestionRef.current.has(s.id)
    );

    if (highPriority.length > 0 && onInlineSuggestion) {
      // 每次只处理一个最重要的建议
      const topSuggestion = highPriority[0];
      pendingSuggestionRef.current.add(topSuggestion.id);

      // 提取目标段落文本
      const tmp = document.createElement('div');
      tmp.innerHTML = htmlContent;
      const paragraphs = Array.from(tmp.querySelectorAll('p'))
        .map(p => p.textContent || '')
        .filter(t => t.trim().length > 0);

      const targetIndex = topSuggestion.targetParagraphIndex >= 0
        ? topSuggestion.targetParagraphIndex
        : paragraphs.length - 1;

      const targetText = paragraphs[targetIndex] || '';

      if (targetText.length > 10) {
        onInlineSuggestion(topSuggestion, targetText);
      }
    }

    // 低优先级建议作为 Ghost Text 传给输入栏
    const ghostSuggestions = decision.suggestions.filter(
      s => s.priority !== 'high' && s.presentation === 'ghost'
    );
    if (ghostSuggestions.length > 0 && onGhostSuggestion) {
      onGhostSuggestion(ghostSuggestions[0].message);
    }
  }, [htmlContent, isEnabled, isZenMode, onInlineSuggestion, onGhostSuggestion]);

  // 防抖分析：用户停止输入 3 秒后触发
  useEffect(() => {
    if (!isEnabled || isZenMode) return;

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

  // 此组件不渲染任何 DOM
  return null;
};

export default SmartHintSystem;
