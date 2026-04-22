import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import { Sparkles, X, Check } from 'lucide-react';

import { Eye, GitBranch, StickyNote, MessageSquarePlus, Quote, Play } from 'lucide-react';
import { writerAgentExecute, recordFeedback } from '@/services/tauri';
import { cn } from '@/utils/cn';
import RichTextEditor, { RichTextEditorRef } from './components/RichTextEditor';
import { SmartHintSystem } from './ai-perception';
import { useCharacters } from '@/hooks/useCharacters';
import { useExecutionState, resolvePrimaryAction } from '@/hooks/useExecutionState';
import { useSubscription } from '@/hooks/useSubscription';
import { loadColorTheme, applyColorTheme } from './config/colorThemes';
import ColorThemeDot from './components/ColorThemeDot';
import { loadEditorConfig } from '@/components/EditorSettings';
import { UpgradePanel } from './components/UpgradePanel';
import { StreamOutput } from '@/components/StreamOutput';
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

const FrontstageApp: React.FC = () => {
  const [stories, setStories] = useState<Story[]>([]);
  const [currentStory, setCurrentStory] = useState<Story | null>(null);
  const [chapters, setChapters] = useState<Chapter[]>([]);
  const [currentChapter, setCurrentChapter] = useState<Chapter | null>(null);
  const [content, setContent] = useState('');
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [showAI, setShowAI] = useState(false);
  const [isSaved, setIsSaved] = useState(true);
  const [generatedText, setGeneratedText] = useState('');
  const [wordCount, setWordCount] = useState(0);
  const [fontSize, setFontSize] = useState(() => loadEditorConfig().fontSize);
  const [isZenMode, setIsZenMode] = useState(false);
  const [isRevisionMode, setIsRevisionMode] = useState(false);
  const [showAnnotationPanel, setShowAnnotationPanel] = useState(false);
  const [showCommentPanel, setShowCommentPanel] = useState(false);
  const [smartGhostText, setSmartGhostText] = useState('');
  const [inlineSuggestion, setInlineSuggestion] = useState<{ instruction: string; targetText: string; category: string; targetParagraphIndex: number } | null>(null);
  const [freeHint, setFreeHint] = useState<{ title: string; message: string; visible: boolean } | null>(null);
  const [showUpgradePanel, setShowUpgradePanel] = useState(false);
  const [upgradeTrigger, setUpgradeTrigger] = useState('');
  const [quotaExhausted, setQuotaExhausted] = useState(false);
  const subscription = useSubscription();
  const { state: executionState } = useExecutionState(currentStory?.id || null);
  const primaryAction = resolvePrimaryAction(executionState);
  const [isGenerating, setIsGenerating] = useState(false);
  const [aiOutputText, setAiOutputText] = useState('');
  const [showAiOutputPanel, setShowAiOutputPanel] = useState(false);
  const [orchestratorStatus, setOrchestratorStatus] = useState<{
    stepType: string;
    loopIdx?: number;
    score?: number;
    message: string;
  } | null>(null);

  // 稳定回调引用，避免 SmartHintSystem 的 useEffect 被频繁重置
  const handleFreeHint = useCallback((title: string, message: string) => {
    setFreeHint({ title, message, visible: true });
  }, []);
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
            // 幕后数据变更，刷新故事/章节列表
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
    // 清理待执行的 auto-save，避免保存到错误章节
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
    
    // Update word count
    const text = newContent.replace(/<[^>]*>/g, '');
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
    setWordCount(chineseChars + englishWords);
    
    // Auto-save after 2 seconds of inactivity
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
    
    // Notify backstage of content change
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
      // 浏览器开发环境 fallback：直接在新标签页打开幕后界面
      const isTauri = !!(window as any).__TAURI__;
      if (!isTauri) {
        window.open('http://127.0.0.1:5173/index.html', '_blank');
      }
    }
  };

  // Request AI generation via writer_agent_execute (full smart engine pipeline)
  const handleRequestGeneration = useCallback(async (context: string) => {
    if (!currentChapter || !showAI || isGenerating) return;

    // Clear any existing typewriter interval
    if (typewriterIntervalRef.current) {
      clearInterval(typewriterIntervalRef.current);
      typewriterIntervalRef.current = null;
    }

    setGeneratedText('');
    setAiOutputText('');
    setShowAiOutputPanel(true);
    setIsGenerating(true);
    setOrchestratorStatus(null);

    const instruction = context || '请根据上下文续写接下来的内容，保持文风一致，情节连贯。';
    const plainContent = content.replace(/<[^>]*>/g, '');

    let unlisten: (() => void) | null = null;
    try {
      // 提前监听 orchestrator 步骤事件
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

      // Typewriter effect: reveal content character by character
      const text = result.content || '';
      setAiOutputText(text);
      let index = 0;
      typewriterIntervalRef.current = setInterval(() => {
        index += 3; // reveal 3 chars at a time for smooth effect
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
      }, 16); // ~60fps
    } catch (error) {
      console.error('Generation request failed:', error);
      const msg = error instanceof Error ? error.message : String(error);
      // 检测配额相关错误（防御性处理）
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
  }, [currentChapter, showAI, isGenerating, content, currentStory]);

  // Accept AI generation
  const handleAcceptGeneration = useCallback(() => {
    if (generatedText && editorRef.current) {
      editorRef.current.insertText(generatedText);
      // Record feedback for adaptive learning
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
      setAiOutputText('');
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
    setAiOutputText('');
  }, [generatedText, currentStory, currentChapter]);

  const handleWriterResult = useCallback((text: string) => {
    // Clear any existing typewriter interval
    if (typewriterIntervalRef.current) {
      clearInterval(typewriterIntervalRef.current);
      typewriterIntervalRef.current = null;
    }
    setIsGenerating(true);
    setAiOutputText(text);
    setShowAiOutputPanel(true);

    // Typewriter effect for chat-generated content
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
      } else {
        setGeneratedText(text.slice(0, index));
      }
    }, 16);
  }, []);

  // 处理内联修改建议：将分析结果传给 RichTextEditor，由编辑器内部调用 Writer Agent
  const handleInlineSuggestion = useCallback((suggestion: any, targetText: string) => {
    setInlineSuggestion({
      instruction: suggestion.instruction || '润色这段文字',
      targetText,
      category: suggestion.category,
      targetParagraphIndex: suggestion.targetParagraphIndex ?? -1,
    });
  }, []);

  // AI toggle shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === ' ' && e.ctrlKey && !e.shiftKey && !isZenMode) {
        e.preventDefault();
        setShowAI(prev => !prev);
      }
      if (e.key === 'F11') {
        e.preventDefault();
        setIsZenMode(prev => !prev);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Calculate total story word count
  const totalWordCount = chapters.reduce((sum, c) => {
    const text = c.content || '';
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
    return sum + chineseChars + englishWords;
  }, 0);

  return (
    <div className={`frontstage-container ${isZenMode ? 'zen-mode' : ''}`}>
      {/* Header */}
      <header className="frontstage-header">
        <div className="frontstage-header-left">
          {!isZenMode && (
            <button
              className="frontstage-menu-btn"
              onClick={() => setSidebarOpen(!sidebarOpen)}
              title="切换侧边栏"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="3" y1="6" x2="21" y2="6" />
                <line x1="3" y1="12" x2="21" y2="12" />
                <line x1="3" y1="18" x2="21" y2="18" />
              </svg>
            </button>
          )}
          <span className="frontstage-logo">草苔</span>
          
          {/* 动态状态信息 */}
          <div className="frontstage-status-bar">
            <span className="status-item" title="当前章节字数">
              {wordCount} 字
            </span>
            <span className="status-separator">·</span>
            <span className="status-item" title="全文字数">
              共 {totalWordCount} 字
            </span>
            <span className="status-separator">·</span>
            <span className="status-item" title="字体大小">
              {fontSize}px
            </span>
            <span className="status-separator">·</span>
            <span className="status-item" title="快捷键提示">
              Ctrl+Space 文思 · F11 禅模式
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
            {/* 订阅状态指示 */}
            {!subscription.isLoading && (
              <span className={`subscription-status ${subscription.isPro ? 'pro' : 'free'}`}>
                {subscription.isPro ? '✨ 专业版' : `🌱 免费版 · ${subscription.dailyLimit - subscription.dailyUsed}次`}
              </span>
            )}
            <ColorThemeDot isZenMode={isZenMode} />
            <button
              className={`frontstage-ai-toggle ${showAI ? 'active' : ''}`}
              onClick={() => setShowAI(!showAI)}
              title="Ctrl+Space 开启/关闭文思"
            >
              {showAI ? '文思泉涌中...' : '开启文思'}
            </button>
            {showAI && (
              <button
                className={`frontstage-ai-toggle ${isGenerating ? 'streaming' : ''}`}
                onClick={() => handleRequestGeneration('')}
                disabled={isGenerating || !currentChapter}
                title="AI 续写"
              >
                <Sparkles className="w-4 h-4 mr-1" />
                {isGenerating ? '生成中...' : 'AI 续写'}
              </button>
            )}
            {/* 下一步快捷按钮 */}
            {currentStory && (
              <button
                className="frontstage-ai-toggle"
                onClick={() => {
                  if (primaryAction.action === 'open_payoff_ledger') {
                    openBackstage();
                    emit('backstage-update', { type: 'NavigateTo', payload: { view: 'foreshadowing' } }).catch(console.error);
                  } else if (primaryAction.action === 'create_first_scene') {
                    handleRequestGeneration('');
                  } else {
                    handleRequestGeneration('');
                  }
                }}
                title={primaryAction.label}
                style={{
                  borderColor: primaryAction.variant === 'danger' ? '#ef4444' : undefined,
                  color: primaryAction.variant === 'danger' ? '#ef4444' : undefined,
                }}
              >
                <Play className="w-4 h-4 mr-1" />
                {primaryAction.label}
              </button>
            )}
          </div>
        )}
      </header>

      {/* Main Content */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Sidebar - Dock 工具栏 */}
        {!isZenMode && (
          <aside 
            className={`frontstage-sidebar ${sidebarOpen ? '' : 'collapsed'}`}
            style={{ width: sidebarOpen ? '64px' : '0px' }}
          >
            <div className="frontstage-sidebar-content h-full flex flex-col items-center py-4 gap-2">
              <button
                className={cn('sidebar-dock-btn', isRevisionMode && 'active')}
                onClick={() => setIsRevisionMode(!isRevisionMode)}
                title="修订模式"
              >
                <GitBranch className="w-5 h-5" />
              </button>
              <button
                className={cn('sidebar-dock-btn', showAnnotationPanel && 'active')}
                onClick={() => setShowAnnotationPanel(!showAnnotationPanel)}
                title="文本批注"
              >
                <StickyNote className="w-5 h-5" />
              </button>
              <button
                className={cn('sidebar-dock-btn', showCommentPanel && 'active')}
                onClick={() => setShowCommentPanel(!showCommentPanel)}
                title="评论线程"
              >
                <MessageSquarePlus className="w-5 h-5" />
              </button>
              <button
                className="sidebar-dock-btn"
                onClick={() => editorRef.current?.generateCommentary()}
                disabled={!currentStory}
                title="生成古典评点"
              >
                <Quote className="w-5 h-5" />
              </button>

              <div className="flex-1 min-h-0" />

              <button 
                className="sidebar-dock-btn backstage-dock-btn" 
                onClick={openBackstage}
                title="打开幕后工作室"
              >
                <Eye className="w-5 h-5" />
              </button>
            </div>
          </aside>
        )}

        {/* AI Output Stream Panel */}
        {showAiOutputPanel && (
          <div className="fixed bottom-24 right-6 w-[480px] max-w-[calc(100vw-3rem)] z-40 shadow-2xl">
            <StreamOutput
              text={aiOutputText}
              isStreaming={isGenerating}
              streamType="simulated"
              onStop={() => {
                if (typewriterIntervalRef.current) {
                  clearInterval(typewriterIntervalRef.current);
                  typewriterIntervalRef.current = null;
                }
                setIsGenerating(false);
                setOrchestratorStatus(null);
              }}
              title="AI 续写预览"
              extraActions={
                <>
                  <button
                    className="stream-btn text-xs"
                    onClick={handleAcceptGeneration}
                    disabled={!generatedText}
                    title="采纳并插入到编辑器"
                  >
                    <Check className="w-3 h-3" />
                    采纳
                  </button>
                  <button
                    className="stream-btn text-xs"
                    onClick={handleRejectGeneration}
                    title="弃用"
                  >
                    <X className="w-3 h-3" />
                    弃用
                  </button>
                  <button
                    className="stream-btn text-xs p-1"
                    onClick={() => setShowAiOutputPanel(false)}
                    title="关闭面板"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </>
              }
            />
          </div>
        )}

        {/* Editor */}
        <main className="frontstage-main">
          {currentChapter && (
            <div className="chapter-header">
              <h1 className="chapter-title">
                {currentChapter.title || `第${currentChapter.chapter_number}章`}
              </h1>
              <div className="story-title">{currentStory?.title}</div>
            </div>
          )}
          
          {/* Rich Text Editor */}
          <RichTextEditor
            ref={editorRef}
            content={content}
            onChange={handleContentChange}
            aiEnabled={showAI}
            generatedText={generatedText}
            onAcceptGeneration={handleAcceptGeneration}
            onRejectGeneration={handleRejectGeneration}
            onWriterResult={handleWriterResult}
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
            showAnnotationPanel={showAnnotationPanel}
            onShowAnnotationPanelChange={setShowAnnotationPanel}
            showCommentPanel={showCommentPanel}
            onShowCommentPanelChange={setShowCommentPanel}
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

      {/* 免费用户分析提示浮层 */}
      {freeHint?.visible && subscription.isFree && (
        <div className="free-hint-toast">
          <div className="free-hint-content">
            <span className="free-hint-icon">💡</span>
            <div>
              <p className="free-hint-title">{freeHint.title}</p>
              <p className="free-hint-message">{freeHint.message}</p>
            </div>
          </div>
          <div className="free-hint-actions">
            <button
              className="free-hint-upgrade"
              onClick={() => {
                setFreeHint(null);
                setUpgradeTrigger('AI 智能改写');
                setShowUpgradePanel(true);
              }}
            >
              🔒 查看 AI 改写
            </button>
            <button
              className="free-hint-dismiss"
              onClick={() => setFreeHint(null)}
            >
              稍后
            </button>
          </div>
        </div>
      )}

      {/* 智能文思 — 统一提示系统 */}
      <SmartHintSystem
        htmlContent={content}
        isEnabled={!isZenMode && showAI}
        isZenMode={isZenMode}
        onGhostSuggestion={setSmartGhostText}
        onInlineSuggestion={subscription.isPro ? handleInlineSuggestion : undefined}
        onFreeHint={subscription.isFree ? handleFreeHint : undefined}
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
