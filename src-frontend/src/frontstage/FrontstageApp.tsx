import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { GitBranch, Eye, X } from 'lucide-react';
import { writerAgentExecute, recordFeedback } from '@/services/tauri';
import { cn } from '@/utils/cn';
import RichTextEditor, { RichTextEditorRef } from './components/RichTextEditor';
import { SmartHintSystem } from './ai-perception';
import { useCharacters } from '@/hooks/useCharacters';
import { useSubscription } from '@/hooks/useSubscription';
import { loadEditorConfig } from '@/components/EditorSettings';
import ColorThemeDot from './components/ColorThemeDot';
import { UpgradePanel } from './components/UpgradePanel';
import { WenSiPanel } from './components/WenSiPanel';
import toast from 'react-hot-toast';

interface Story {
  id: string;
  title: string;
  description?: string;
}

interface Chapter {
  id: string;
  story_id: string;
  title?: string;
  chapter_number: number;
  content?: string;
}

interface FrontstageEvent {
  type: string;
  payload?: {
    text?: string;
    chapter_id?: string;
    story_id?: string;
    title?: string;
    hint?: string;
    position?: { line: number; column: number };
    duration_ms?: number;
    saved?: boolean;
    timestamp?: string;
    entity?: string;
  };
}

type WensiMode = 'off' | 'passive' | 'active';

const FrontstageApp: React.FC = () => {
  const [stories, setStories] = useState<Story[]>([]);
  const [currentStory, setCurrentStory] = useState<Story | null>(null);
  const [chapters, setChapters] = useState<Chapter[]>([]);
  const [currentChapter, setCurrentChapter] = useState<Chapter | null>(null);
  const [content, setContent] = useState('');
  const [isSaved, setIsSaved] = useState(true);
  const [generatedText, setGeneratedText] = useState('');
  const [wordCount, setWordCount] = useState(0);
  const [fontSize, setFontSize] = useState(() => loadEditorConfig().fontSize);
  const [isZenMode, setIsZenMode] = useState(false);
  const [isRevisionMode, setIsRevisionMode] = useState(false);

  // 文思三态：关闭 / 被动提示 / 主动辅助
  const [wensiMode, setWensiMode] = useState<WensiMode>('passive');

  const [smartGhostText, setSmartGhostText] = useState('');
  const [inlineSuggestion, setInlineSuggestion] = useState<{ instruction: string; targetText: string; category: string; targetParagraphIndex: number } | null>(null);
  const [showUpgradePanel, setShowUpgradePanel] = useState(false);
  const [upgradeTrigger, setUpgradeTrigger] = useState('');
  const [quotaExhausted, setQuotaExhausted] = useState(false);
  const subscription = useSubscription();
  const [isGenerating, setIsGenerating] = useState(false);
  const [orchestratorStatus, setOrchestratorStatus] = useState<{
    stepType: string;
    loopIdx?: number;
    score?: number;
    message: string;
  } | null>(null);

  // WenSi 浮动面板
  const [showWenSiPanel, setShowWenSiPanel] = useState(false);
  const [wenSiTab, setWenSiTab] = useState<'write' | 'revise'>('write');

  const editorRef = useRef<RichTextEditorRef>(null);
  const autoSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const typewriterIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // 监听编辑器配置变化（同步幕后设置到幕前）
  useEffect(() => {
    const handleConfigChange = (e: CustomEvent) => {
      const config = e.detail;
      if (config?.fontSize) setFontSize(config.fontSize);
    };
    window.addEventListener('editor-config-changed', handleConfigChange as EventListener);
    return () => window.removeEventListener('editor-config-changed', handleConfigChange as EventListener);
  }, []);

  // 加载当前故事的角色
  const { data: characters = [] } = useCharacters(currentStory?.id || null);

  // Load stories on mount
  useEffect(() => {
    loadStories();
    setupEventListeners();
    return () => {
      if (typewriterIntervalRef.current) {
        clearInterval(typewriterIntervalRef.current);
        typewriterIntervalRef.current = null;
      }
    };
  }, []);

  // Setup Tauri event listeners
  const setupEventListeners = async () => {
    try {
      await listen<FrontstageEvent>('frontstage-update', (event) => {
        const { type, payload } = event.payload;

        switch (type) {
          case 'ContentUpdate':
            if (payload?.text !== undefined) {
              setContent(payload.text);
            }
            break;
          case 'AppendContent':
            if (payload?.text !== undefined) {
              setContent(prev => prev + '\n\n' + payload.text);
            }
            break;
          case 'DataRefresh':
            loadStories();
            if (payload?.entity === 'characters') {
              window.dispatchEvent(new CustomEvent('characters-refreshed'));
            }
            break;
          case 'SaveStatus':
            setIsSaved(payload?.saved ?? true);
            break;
          case 'ChapterSwitch':
            if (payload?.chapter_id) {
              if (payload?.story_id && payload.story_id !== currentStory?.id) {
                (async () => {
                  try {
                    const allStories = await invoke<Story[]>('list_stories');
                    const targetStory = allStories.find(s => s.id === payload.story_id);
                    if (targetStory) {
                      const storyChapters = await invoke<Chapter[]>('get_story_chapters', { story_id: targetStory.id });
                      setCurrentStory(targetStory);
                      setChapters(storyChapters);
                      const targetChapter = storyChapters.find(c => c.id === payload.chapter_id);
                      if (targetChapter) {
                        selectChapter(targetChapter);
                      }
                    }
                  } catch (e) {
                    console.error('Failed to switch to new story:', e);
                  }
                })();
              } else {
                const chapter = chapters.find(c => c.id === payload.chapter_id);
                if (chapter) {
                  selectChapter(chapter);
                }
              }
            }
            break;
        }
      });
    } catch (e) {
      console.error('Failed to setup event listeners:', e);
    }
  };

  const loadStories = async () => {
    try {
      const result = await invoke<Story[]>('list_stories');
      setStories(result);
      if (result.length > 0 && !currentStory) {
        selectStory(result[0]);
      }
    } catch (e) {
      console.error('Failed to load stories:', e);
    }
  };

  const selectStory = async (story: Story) => {
    setCurrentStory(story);
    try {
      const result = await invoke<Chapter[]>('get_story_chapters', { story_id: story.id });
      setChapters(result);
      if (result.length > 0) {
        selectChapter(result[0]);
      } else {
        setCurrentChapter(null);
        setContent('');
      }
    } catch (e) {
      console.error('Failed to load chapters:', e);
    }
  };

  const selectChapter = (chapter: Chapter) => {
    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current);
      autoSaveTimerRef.current = null;
    }
    setCurrentChapter(chapter);
    setContent(chapter.content || '');
    setIsSaved(true);
  };

  const handleContentChange = useCallback(async (newContent: string) => {
    setContent(newContent);
    setIsSaved(false);

    const text = newContent.replace(/<[^>]*>/g, '');
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
    setWordCount(chineseChars + englishWords);

    if (currentChapter) {
      if (autoSaveTimerRef.current) {
        clearTimeout(autoSaveTimerRef.current);
      }
      autoSaveTimerRef.current = setTimeout(async () => {
        try {
          await invoke('update_chapter', {
            id: currentChapter.id,
            title: currentChapter.title,
            content: newContent,
            word_count: wordCount
          });
          setIsSaved(true);
        } catch (e) {
          console.error('Auto-save failed:', e);
        }
      }, 2000);
    }

    if (currentChapter) {
      invoke('notify_backstage_content_changed', {
        text: newContent,
        chapter_id: currentChapter.id
      }).catch(e => console.error('Failed to notify content change:', e));
    }
  }, [currentChapter]);

  const openBackstage = async () => {
    try {
      await invoke('show_backstage');
    } catch (e) {
      console.error('Failed to open backstage:', e);
      const isTauri = !!(window as any).__TAURI__;
      if (!isTauri) {
        window.open('http://127.0.0.1:5173/index.html', '_blank');
      }
    }
  };

  // 文思三态循环切换
  const cycleWensiMode = useCallback(() => {
    setWensiMode(prev => {
      if (prev === 'off') return 'passive';
      if (prev === 'passive') return 'active';
      return 'off';
    });
  }, []);

  // Request AI generation via writer_agent_execute
  const handleRequestGeneration = useCallback(async (context: string) => {
    if (!currentChapter || wensiMode !== 'active' || isGenerating) return;

    if (typewriterIntervalRef.current) {
      clearInterval(typewriterIntervalRef.current);
      typewriterIntervalRef.current = null;
    }

    setGeneratedText('');
    setIsGenerating(true);
    setOrchestratorStatus(null);

    const instruction = context || '请根据上下文续写接下来的内容，保持文风一致，情节连贯。';
    const plainContent = content.replace(/<[^>]*>/g, '');

    let unlisten: (() => void) | null = null;
    try {
      unlisten = await listen<{
        task_id: string;
        step_type: string;
        loop_idx?: number;
        score?: number;
      }>('orchestrator-step', (event) => {
        const p = event.payload;
        const stepNames: Record<string, string> = {
          '生成': '生成中...',
          '质检': '质检中...',
          '改写': '改写中...',
        };
        let message = stepNames[p.step_type] || p.step_type;
        if (p.step_type === '改写' && typeof p.loop_idx === 'number') {
          message = `第 ${p.loop_idx + 1} 轮优化中...`;
        }
        if (p.step_type === '质检' && typeof p.score === 'number') {
          message = `质检中... 评分 ${p.score}%`;
        }
        setOrchestratorStatus({
          stepType: p.step_type,
          loopIdx: p.loop_idx,
          score: p.score,
          message,
        });
      });

      const result = await writerAgentExecute({
        story_id: currentStory?.id || '',
        chapter_number: currentChapter?.chapter_number,
        current_content: plainContent,
        instruction,
      });

      setOrchestratorStatus({ stepType: '完成', message: '质检通过，生成完成' });

      const text = result.content || '';
      let index = 0;
      typewriterIntervalRef.current = setInterval(() => {
        index += 3;
        if (index >= text.length) {
          if (typewriterIntervalRef.current) {
            clearInterval(typewriterIntervalRef.current);
            typewriterIntervalRef.current = null;
          }
          setGeneratedText(text);
          setIsGenerating(false);
          setOrchestratorStatus(null);
        } else {
          setGeneratedText(text.slice(0, index));
        }
      }, 16);
    } catch (error) {
      console.error('Generation request failed:', error);
      const msg = error instanceof Error ? error.message : String(error);
      const isQuotaError = /quota|exhausted|limit|配额|用完|不足|次数已达/i.test(msg);
      if (isQuotaError) {
        setQuotaExhausted(true);
        toast.error('AI 创作配额已用完，请升级专业版或明日再试');
      } else {
        toast.error(`生成失败: ${msg}`);
      }
      setIsGenerating(false);
      setOrchestratorStatus(null);
    } finally {
      if (unlisten) {
        unlisten();
      }
    }
  }, [currentChapter, wensiMode, isGenerating, content, currentStory]);

  // Accept AI generation
  const handleAcceptGeneration = useCallback(() => {
    if (generatedText && editorRef.current) {
      editorRef.current.insertText(generatedText);
      if (currentStory?.id) {
        recordFeedback({
          story_id: currentStory.id,
          chapter_id: currentChapter?.id,
          feedback_type: 'accept',
          agent_type: 'writer',
          original_ai_text: generatedText,
        }).catch(e => console.error('Feedback record failed:', e));
      }
      setGeneratedText('');
    }
  }, [generatedText, currentStory, currentChapter]);

  // Reject AI generation
  const handleRejectGeneration = useCallback(() => {
    if (generatedText && currentStory?.id) {
      recordFeedback({
        story_id: currentStory.id,
        chapter_id: currentChapter?.id,
        feedback_type: 'reject',
        agent_type: 'writer',
        original_ai_text: generatedText,
      }).catch(e => console.error('Feedback record failed:', e));
    }
    setGeneratedText('');
  }, [generatedText, currentStory, currentChapter]);

  // 处理内联修改建议
  const handleInlineSuggestion = useCallback((suggestion: any, targetText: string) => {
    setInlineSuggestion({
      instruction: suggestion.instruction || '润色这段文字',
      targetText,
      category: suggestion.category,
      targetParagraphIndex: suggestion.targetParagraphIndex ?? -1,
    });
  }, []);

  // 适配器：将编辑器的 onRequestGeneration 调用转为 handleRequestGeneration
  const handleRequestGenerationForEditor = useCallback((instruction?: string) => {
    handleRequestGeneration(instruction || '');
  }, [handleRequestGeneration]);

  // 处理编辑器 Slash 命令
  const handleSlashCommand = useCallback((commandId: string) => {
    if (commandId === 'auto_write') {
      setWenSiTab('write');
      setShowWenSiPanel(true);
    } else if (commandId === 'auto_revise') {
      setWenSiTab('revise');
      setShowWenSiPanel(true);
    } else if (commandId === 'commentary') {
      editorRef.current?.generateCommentary();
    }
  }, []);

  // 全局快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // F11 禅模式
      if (e.key === 'F11') {
        e.preventDefault();
        setIsZenMode(prev => !prev);
        return;
      }
      // Ctrl+Enter / Cmd+Enter 续写（仅 active 模式）
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey) && wensiMode === 'active' && !isZenMode) {
        e.preventDefault();
        handleRequestGeneration('');
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [wensiMode, isZenMode, handleRequestGeneration]);

  // Calculate total story word count
  const totalWordCount = chapters.reduce((sum, c) => {
    const text = c.content || '';
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
    return sum + chineseChars + englishWords;
  }, 0);

  // 文思图标 tooltip
  const wensiTooltip = wensiMode === 'active'
    ? '文思活跃 — Ctrl+Enter 续写'
    : wensiMode === 'passive'
      ? '文思被动 — 仅萤火提示'
      : '文思关闭';

  return (
    <div className={`frontstage-container ${isZenMode ? 'zen-mode' : ''}`}>
      {/* Header */}
      <header className="frontstage-header">
        <div className="frontstage-header-left">
          <span
            className="frontstage-story-name"
            onClick={openBackstage}
            title="点击回幕后工作室"
          >
            {currentStory?.title || '草苔'}
          </span>
          <div className="frontstage-status-bar">
            <span className="status-item">
              {currentChapter?.title || (currentChapter ? `第${currentChapter.chapter_number}章` : '')}
            </span>
            <span className="status-separator">·</span>
            <span className="status-item" title="当前章节字数 / 全文字数">
              {wordCount} 字 / {totalWordCount} 字
            </span>
            <span className="status-separator">·</span>
            <span className="status-item" title="字体大小">
              {fontSize}px
            </span>
            {!isSaved && (
              <>
                <span className="status-separator">·</span>
                <span className="status-item saving">保存中...</span>
              </>
            )}
            {orchestratorStatus && (
              <>
                <span className="status-separator">·</span>
                <span className="status-item saving" title="AI 编排器状态">
                  {orchestratorStatus.message}
                </span>
              </>
            )}
          </div>
        </div>

        {!isZenMode && (
          <div className="frontstage-header-right">
            <ColorThemeDot isZenMode={isZenMode} />
            <button
              className={`wensi-mode-toggle wensi-${wensiMode}`}
              onClick={cycleWensiMode}
              title={wensiTooltip}
            >
              <span className="wensi-icon">
                {wensiMode === 'active' ? '🔥' : wensiMode === 'passive' ? '✨' : '·'}
              </span>
            </button>
            <button
              className="zen-mode-btn"
              onClick={() => setIsZenMode(!isZenMode)}
              title="F11 禅模式"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="3" y="3" width="18" height="18" rx="2" />
                <path d="M9 3v18" />
              </svg>
            </button>
          </div>
        )}
      </header>

      {/* Main Content */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Sidebar - Dock 工具栏 */}
        {!isZenMode && (
          <aside className="frontstage-sidebar" style={{ width: '48px' }}>
            <div className="frontstage-sidebar-content h-full flex flex-col items-center py-3 gap-1">
              <button
                className={cn('sidebar-dock-btn', isRevisionMode && 'active')}
                onClick={() => setIsRevisionMode(!isRevisionMode)}
                title="修订模式"
              >
                <GitBranch className="w-4 h-4" />
              </button>
              <button
                className="sidebar-dock-btn"
                onClick={() => editorRef.current?.generateCommentary()}
                disabled={!currentStory}
                title="生成古典评点"
              >
                <span className="text-xs font-serif">批</span>
              </button>

              <div className="flex-1 min-h-0" />

              <button
                className="sidebar-dock-btn backstage-dock-btn"
                onClick={openBackstage}
                title="打开幕后工作室"
              >
                <Eye className="w-4 h-4" />
              </button>
            </div>
          </aside>
        )}

        {/* Editor */}
        <main className="frontstage-main">
          {currentChapter && (
            <div className="chapter-header">
              <h1 className="chapter-title">
                {currentChapter.title || `第${currentChapter.chapter_number}章`}
              </h1>
            </div>
          )}

          <RichTextEditor
            ref={editorRef}
            content={content}
            onChange={handleContentChange}
            wensiMode={wensiMode}
            generatedText={generatedText}
            isGenerating={isGenerating}
            onAcceptGeneration={handleAcceptGeneration}
            onRejectGeneration={handleRejectGeneration}
            onRequestGeneration={handleRequestGenerationForEditor}
            onSlashCommand={handleSlashCommand}
            placeholder={currentChapter ? '开始写作...' : '请选择一个章节开始创作'}
            characters={characters}
            fontSize={fontSize}
            onFontSizeChange={setFontSize}
            isZenMode={isZenMode}
            onZenModeChange={setIsZenMode}
            storyId={currentStory?.id}
            chapterId={currentChapter?.id}
            chapterNumber={currentChapter?.chapter_number}
            isRevisionMode={isRevisionMode}
            onRevisionModeChange={setIsRevisionMode}

            smartGhostText={smartGhostText}
            inlineSuggestion={subscription.isPro ? inlineSuggestion : null}
            onClearInlineSuggestion={() => setInlineSuggestion(null)}
            subscription={subscription}
            onQuotaExhausted={() => {
              setQuotaExhausted(true);
              setUpgradeTrigger('文思泉涌专业版');
              setShowUpgradePanel(true);
            }}
          />
        </main>
      </div>

      {/* Floating WenSi Panel */}
      {showWenSiPanel && (
        <div className="fixed bottom-6 right-6 w-[420px] max-w-[calc(100vw-3rem)] z-40">
          <div className="bg-[var(--parchment-dark)] border border-[var(--warm-sand)] rounded-xl shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--warm-sand)]">
              <span className="text-sm font-medium text-[var(--charcoal)]">文思泉涌</span>
              <button
                onClick={() => setShowWenSiPanel(false)}
                className="text-[var(--stone-gray)] hover:text-[var(--charcoal)] transition-colors"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
            <div className="p-3">
              <WenSiPanel
                storyId={currentStory?.id}
                chapterId={currentChapter?.id}
                isPro={subscription?.isPro ?? false}
                quotaText={subscription?.getQuotaText ? subscription.getQuotaText() : (subscription?.tier ? (subscription.isPro ? 'Pro · 无限' : `免费版`) : '加载中...')}
                onShowUpgrade={(trigger) => {
                  setUpgradeTrigger(trigger);
                  setShowUpgradePanel(true);
                }}
                hasAutoWriteQuota={subscription?.hasAutoWriteQuota || (async () => true)}
                hasAutoReviseQuota={subscription?.hasAutoReviseQuota || (async () => true)}
                editorContent={editorRef.current?.getText()}
                selectedText={editorRef.current?.getSelectedText()}
                onReviseResult={(text) => {
                  if (editorRef.current) {
                    const html = '<p>' + text.replace(/\n+/g, '</p><p>') + '</p>';
                    editorRef.current.insertText(html);
                    toast.success('修改内容已应用到编辑器');
                  }
                }}
              />
            </div>
          </div>
        </div>
      )}

      {/* 智能文思 — 统一提示系统 */}
      <SmartHintSystem
        htmlContent={content}
        isEnabled={!isZenMode && wensiMode !== 'off'}
        isZenMode={isZenMode}
        onGhostSuggestion={setSmartGhostText}
        onInlineSuggestion={subscription.isPro ? handleInlineSuggestion : undefined}
        subscription={subscription}
      />

      {/* 配额用尽提示 */}
      {quotaExhausted && subscription.isFree && (
        <div className="quota-exhausted-toast">
          <p className="quota-exhausted-title">⚡ 今日配额已用完</p>
          <p className="quota-exhausted-message">
            免费用户每日可使用 10 次 AI 创作。升级专业版，享受无限次文思泉涌。
          </p>
          <div className="quota-exhausted-actions">
            <button
              className="quota-exhausted-upgrade"
              onClick={() => {
                setQuotaExhausted(false);
                setUpgradeTrigger('AI 创作配额');
                setShowUpgradePanel(true);
              }}
            >
              升级专业版
            </button>
            <button
              className="quota-exhausted-dismiss"
              onClick={() => setQuotaExhausted(false)}
            >
              我知道了
            </button>
          </div>
        </div>
      )}

      {/* 付费引导面板 */}
      <UpgradePanel
        isOpen={showUpgradePanel}
        onClose={() => setShowUpgradePanel(false)}
        trigger={upgradeTrigger}
        onUpgraded={() => subscription.fetchStatus()}
      />

      {/* 禅模式退出提示 */}
      {isZenMode && (
        <button
          onClick={() => setIsZenMode(false)}
          className="zen-mode-exit"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"/>
          </svg>
          退出禅模式 (F11)
        </button>
      )}
    </div>
  );
};

export default FrontstageApp;
