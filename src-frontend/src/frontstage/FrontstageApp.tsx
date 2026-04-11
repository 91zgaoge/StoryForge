/**
 * FrontStage (幕前) - 极简阅读写作界面
 * 
 * 这是 StoryForge 的核心创新功能：
 * - 接近最终阅读界面的极简设计
 * - AI 提示以灰色小字浮现，如文思泉涌
 * - 接收来自幕后(BackStage)的智能输出
 */

import { useEffect, useState, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { ReaderWriter } from './components/ReaderWriter';
import { AiHintOverlay } from './components/AiHintOverlay';
import { FrontstageToolbar } from './components/FrontstageToolbar';
import { useFrontstageStore } from './store/frontstageStore';
import type { FrontstageEvent } from './types';
import './styles/frontstage.css';

function FrontstageApp() {
  const [isLoading, setIsLoading] = useState(true);
  const [isReady, setIsReady] = useState(false);
  
  const {
    content,
    chapterId,
    chapterTitle,
    aiHints,
    setContent,
    setChapterInfo,
    addAiHint,
    removeAiHint,
    clearAiHints,
    setSaveStatus,
  } = useFrontstageStore();

  // Initialize and listen for events from backstage
  useEffect(() => {
    const setupEventListeners = async () => {
      // Listen for updates from backstage
      const unlisten = await listen<FrontstageEvent>('frontstage-update', (event) => {
        const { payload } = event;
        
        switch (payload.type) {
          case 'ContentUpdate':
            setContent(payload.payload.text);
            setChapterInfo(payload.payload.chapter_id, chapterTitle || '');
            break;
            
          case 'AiHint':
            addAiHint({
              id: `hint-${Date.now()}`,
              text: payload.payload.hint,
              position: payload.payload.position,
              duration: payload.payload.duration_ms,
            });
            setTimeout(() => {
              removeAiHint(`hint-${Date.now()}`);
            }, payload.payload.duration_ms);
            break;
            
          case 'AiPreview':
            // Show AI generation preview
            addAiHint({
              id: 'preview',
              text: payload.payload.text,
              position: { line: 0, column: 0, offset: payload.payload.insert_position },
              duration: 10000,
              isPreview: true,
            });
            break;
            
          case 'ChapterSwitch':
            setChapterInfo(payload.payload.chapter_id, payload.payload.title);
            clearAiHints();
            break;
            
          case 'SaveStatus':
            setSaveStatus(payload.payload.saved, payload.payload.timestamp);
            break;
        }
      });

      setIsReady(true);
      setIsLoading(false);

      return unlisten;
    };

    let unlistenFn: (() => void) | undefined;
    setupEventListeners().then((fn) => {
      unlistenFn = fn;
    });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  // Send content changes to backstage
  const handleContentChange = useCallback((newContent: string) => {
    setContent(newContent);
    
    // Notify backstage of changes
    if (chapterId) {
      invoke('notify_backstage_content_changed', {
        text: newContent,
        chapterId,
      }).catch(console.error);
    }
  }, [chapterId, setContent]);

  // Request AI generation
  const handleRequestGeneration = useCallback((context: string) => {
    if (chapterId) {
      invoke('notify_backstage_generation_requested', {
        chapterId,
        context,
      }).catch(console.error);
    }
  }, [chapterId]);

  if (isLoading) {
    return (
      <div className="frontstage-loading">
        <div className="ink-drop"></div>
        <p>正在准备创作空间...</p>
      </div>
    );
  }

  return (
    <div className="frontstage-container">
      <FrontstageToolbar
        chapterTitle={chapterTitle}
        onRequestGeneration={handleRequestGeneration}
      />
      
      <main className="frontstage-main">
        <ReaderWriter
          content={content}
          onChange={handleContentChange}
          onRequestGeneration={handleRequestGeneration}
        />
        
        <AiHintOverlay hints={aiHints} />
      </main>
      
      <div className="frontstage-watermark">
        <span>草苔</span>
        <small>StoryForge</small>
      </div>
    </div>
  );
}

export default FrontstageApp;