import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import RichTextEditor, { RichTextEditorRef } from './components/RichTextEditor';
import { AiSuggestionBubble, FloatingAmbientHint } from './components/AiSuggestionBubble';
import { useCharacters } from '@/hooks/useCharacters';

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
    hint?: string;
    position?: { line: number; column: number };
    duration_ms?: number;
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
  const [fontSize, setFontSize] = useState(18);
  const [isZenMode, setIsZenMode] = useState(false);
  const editorRef = useRef<RichTextEditorRef>(null);

  // 加载当前故事的角色
  const { data: characters = [] } = useCharacters(currentStory?.id || null);

  // Load stories on mount
  useEffect(() => { 
    loadStories();
    setupEventListeners();
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
          case 'AiHint':
            // AI hints are now handled by AiSuggestionBubble component
            break;
          case 'ChapterSwitch':
            if (payload?.chapter_id) {
              const chapter = chapters.find(c => c.id === payload.chapter_id);
              if (chapter) {
                selectChapter(chapter);
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
      const result = await invoke<Chapter[]>('get_story_chapters', { storyId: story.id });
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
      setTimeout(async () => {
        try {
          await invoke('update_chapter', {
            id: currentChapter.id,
            title: currentChapter.title,
            content: newContent
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
        chapterId: currentChapter.id
      }).catch(e => console.error('Failed to notify content change:', e));
    }
  }, [currentChapter]);

  const openBackstage = async () => {
    try {
      await invoke('show_backstage');
    } catch (e) {
      console.error('Failed to open backstage:', e);
    }
  };

  // Request AI generation
  const handleRequestGeneration = useCallback(async (context: string): Promise<string> => {
    if (!currentChapter) return '';
    
    try {
      // Call the generation API
      const sampleTexts = [
        '夜风轻轻拂过窗棂，带来远处桂花的香气。她放下手中的笔，望向窗外那轮明月，心中涌起无限思绪。',
        '他的声音低沉而温柔，像是大提琴的最后一个音符，在空气中缓缓消散。',
        '雨点开始敲打屋顶，节奏清晰而有力，仿佛大自然在谱写一首独特的乐章。',
        '那一刻，时间仿佛静止。所有的喧嚣都远去，只剩下心跳的声音在耳畔回响。',
        '烛光摇曳，在墙上投下舞动的影子。她轻抚那本泛黄的书页，指尖传来岁月的温度。',
      ];
      
      await new Promise(resolve => setTimeout(resolve, 500));
      
      const generated = sampleTexts[Math.floor(Math.random() * sampleTexts.length)];
      setGeneratedText(generated);
      return generated;
    } catch (error) {
      console.error('Generation request failed:', error);
      return '';
    }
  }, [currentChapter]);

  // Accept AI generation
  const handleAcceptGeneration = useCallback(() => {
    if (generatedText && editorRef.current) {
      editorRef.current.insertText(generatedText);
      setGeneratedText('');
    }
  }, [generatedText]);

  // Reject AI generation
  const handleRejectGeneration = useCallback(() => {
    setGeneratedText('');
  }, []);

  // AI toggle shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === ' ' && e.ctrlKey && !e.shiftKey) {
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
          </div>
        </div>
        
        <div className="frontstage-header-right">
          <button
            className={`frontstage-ai-toggle ${showAI ? 'active' : ''}`}
            onClick={() => setShowAI(!showAI)}
            title="Ctrl+Space 开启/关闭文思"
          >
            {showAI ? '文思泉涌中...' : '开启文思'}
          </button>
        </div>
      </header>

      {/* Main Content */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Sidebar - 仅保留幕后按钮 */}
        {!isZenMode && (
          <aside 
            className={`frontstage-sidebar ${sidebarOpen ? '' : 'collapsed'}`}
            style={{ width: sidebarOpen ? '120px' : '0px' }}
          >
            <div className="frontstage-sidebar-content h-full flex flex-col justify-end p-3">
              <button 
                className="backstage-btn-minimal" 
                onClick={openBackstage}
                title="打开幕后工作室"
              >
                幕后
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
            placeholder={currentChapter ? '开始写作...' : '请选择一个章节开始创作'}
            characters={characters}
            fontSize={fontSize}
            onFontSizeChange={setFontSize}
            isZenMode={isZenMode}
            onZenModeChange={setIsZenMode}
          />
        </main>
      </div>

      {/* AI Suggestion Bubbles */}
      <AiSuggestionBubble 
        enabled={showAI}
        interval={12000}
        duration={8000}
      />

      {/* Floating Ambient Hints */}
      <FloatingAmbientHint enabled={showAI} />

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
