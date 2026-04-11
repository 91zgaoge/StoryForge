import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ReaderWriter } from './components/ReaderWriter';
import { ChapterOutline } from './components/ChapterOutline';
import { AiSuggestionBubble, FloatingAmbientHint } from './components/AiSuggestionBubble';

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
  const [wordCount, setWordCount] = useState(0);
  const [isZenMode, setIsZenMode] = useState(false);
  const [isSaved, setIsSaved] = useState(true);

  // Load stories on mount
  useEffect(() => { 
    loadStories();
    setupEventListeners();
  }, []);

  // Calculate word count
  useEffect(() => {
    const chineseChars = (content.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = (content.match(/[a-zA-Z]+/g) || []).length;
    setWordCount(chineseChars + englishWords);
  }, [content]);

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
      // In a real implementation, this would call a Tauri command
      // For now, we simulate the response
      const sampleTexts = [
        '夜风轻轻拂过窗棂，带来远处桂花的香气。她放下手中的笔，望向窗外那轮明月，心中涌起无限思绪。',
        '他的声音低沉而温柔，像是大提琴的最后一个音符，在空气中缓缓消散。',
        '雨点开始敲打屋顶，节奏清晰而有力，仿佛大自然在谱写一首独特的乐章。',
        '那一刻，时间仿佛静止。所有的喧嚣都远去，只剩下心跳的声音在耳畔回响。',
        '烛光摇曳，在墙上投下舞动的影子。她轻抚那本泛黄的书页，指尖传来岁月的温度。',
      ];
      
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 500));
      
      return sampleTexts[Math.floor(Math.random() * sampleTexts.length)];
    } catch (error) {
      console.error('Generation request failed:', error);
      return '';
    }
  }, [currentChapter]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Zen mode toggle
      if (e.key === 'F11') {
        e.preventDefault();
        setIsZenMode(prev => !prev);
      }
      
      // AI toggle
      if (e.key === ' ' && e.ctrlKey && !e.shiftKey) {
        e.preventDefault();
        setShowAI(prev => !prev);
      }

      // Save
      if (e.key === 's' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        if (currentChapter) {
          invoke('update_chapter', {
            id: currentChapter.id,
            title: currentChapter.title,
            content
          }).then(() => setIsSaved(true));
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [content, currentChapter]);

  return (
    <div className={`frontstage-container ${isZenMode ? 'zen-mode' : ''}`}>
      {/* Header */}
      {!isZenMode && (
        <header className="frontstage-header">
          <div className="frontstage-header-left">
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
            <span className="frontstage-logo">草苔</span>
          </div>
          <div className="frontstage-header-left" style={{ marginLeft: 'auto' }}>
            <button
              className={`frontstage-ai-toggle ${showAI ? 'active' : ''}`}
              onClick={() => setShowAI(!showAI)}
              title="Ctrl+Space 开启/关闭文思"
            >
              {showAI ? '文思泉涌中...' : '开启文思'}
            </button>
          </div>
        </header>
      )}

      {/* Main Content */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Sidebar with Chapter Outline */}
        <aside 
          className={`frontstage-sidebar ${sidebarOpen ? '' : 'collapsed'}`}
          style={{ width: sidebarOpen ? '320px' : '0px' }}
        >
          <div className="frontstage-sidebar-content h-full flex flex-col">
            {/* 故事选择 */}
            <div className="px-4 py-3 border-b border-[var(--warm-sand)]">
              <label className="text-xs text-[var(--stone-gray)] uppercase tracking-wider">当前故事</label>
              <select
                value={currentStory?.id || ''}
                onChange={(e) => {
                  const story = stories.find(s => s.id === e.target.value);
                  if (story) selectStory(story);
                }}
                className="w-full mt-1 px-3 py-2 bg-[var(--parchment)] border border-[var(--warm-sand)] rounded-lg text-[var(--charcoal)] text-sm focus:outline-none focus:border-[var(--terracotta)]"
              >
                {stories.map(story => (
                  <option key={story.id} value={story.id}>{story.title}</option>
                ))}
              </select>
            </div>

            {/* 章节大纲 */}
            <div className="flex-1 overflow-hidden">
              <ChapterOutline
                items={chapters.map((c, index) => ({
                  id: c.id,
                  level: 0,
                  title: c.title || `第${c.chapter_number}章`,
                  wordCount: c.content ? Math.floor(c.content.length / 2) : 0,
                }))}
                selectedId={currentChapter?.id}
                onReorder={(newItems) => {
                  // TODO: 实现章节重新排序
                  console.log('Reordered:', newItems);
                }}
                onSelect={(id) => {
                  const chapter = chapters.find(c => c.id === id);
                  if (chapter) selectChapter(chapter);
                }}
                onEdit={(id, title) => {
                  const chapter = chapters.find(c => c.id === id);
                  if (chapter) {
                    invoke('update_chapter', {
                      id: chapter.id,
                      title,
                      content: chapter.content,
                    }).then(() => {
                      loadStories();
                    });
                  }
                }}
                onDelete={(id) => {
                  if (confirm('确定要删除这个章节吗？')) {
                    invoke('delete_chapter', { id }).then(() => {
                      loadStories();
                    });
                  }
                }}
                onAdd={() => {
                  if (currentStory) {
                    const nextNumber = chapters.length + 1;
                    invoke('create_chapter', {
                      storyId: currentStory.id,
                      chapterNumber: nextNumber,
                      title: `第${nextNumber}章`,
                    }).then(() => {
                      loadStories();
                    });
                  }
                }}
              />
            </div>

            {/* 底部按钮 */}
            <div className="p-4 border-t border-[var(--warm-sand)]">
              <button 
                className="backstage-btn" 
                onClick={openBackstage}
              >
                打开幕后工作室 →
              </button>
            </div>
          </div>
        </aside>

        {/* Editor */}
        <main className={`frontstage-main ${isZenMode ? 'zen-mode' : ''}`}>
          {currentChapter && (
            <div className="chapter-header">
              <h1 className={`chapter-title ${isZenMode ? 'zen' : ''}`}>
                {currentChapter.title || `第${currentChapter.chapter_number}章`}
              </h1>
              <div className="story-title">{currentStory?.title}</div>
            </div>
          )}
          
          {/* Rich Text Editor */}
          <ReaderWriter
            content={content}
            onChange={handleContentChange}
            onRequestGeneration={handleRequestGeneration}
            aiEnabled={showAI}
            placeholder={currentChapter ? '开始写作...' : '请选择一个章节开始创作'}
          />
        </main>
      </div>

      {/* Stats Bar */}
      <div className="reader-writer-stats">
        <span>{wordCount} 字</span>
        <span className="divider">|</span>
        <span>{isSaved ? '已保存' : '保存中...'}</span>
        <span className="divider">|</span>
        <span>Ctrl+Space AI续写</span>
        <span className="divider">|</span>
        <span>F11 禅模式</span>
      </div>

      {/* AI Suggestion Bubbles */}
      <AiSuggestionBubble 
        enabled={showAI}
        interval={12000}
        duration={8000}
      />

      {/* Floating Ambient Hints */}
      <FloatingAmbientHint enabled={showAI} />

      {/* Zen Mode Exit Hint */}
      {isZenMode && (
        <div className="zen-exit-hint" onClick={() => setIsZenMode(false)}>
          点击退出禅模式 (F11)
        </div>
      )}
    </div>
  );
};

export default FrontstageApp;
