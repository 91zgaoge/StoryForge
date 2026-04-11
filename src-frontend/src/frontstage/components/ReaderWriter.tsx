/**
 * ReaderWriter - 极简阅读写作界面
 * 
 * 设计理念：
 * - 接近最终阅读界面的排版
 * - 沉浸式的写作体验
 * - 支持 AI 辅助快捷键
 */

import { useState, useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ReaderWriterProps {
  content: string;
  onChange: (content: string) => void;
  onRequestGeneration: (context: string) => void;
}

export function ReaderWriter({ content, onChange, onRequestGeneration }: ReaderWriterProps) {
  const editorRef = useRef<HTMLTextAreaElement>(null);
  const [cursorPosition, setCursorPosition] = useState(0);
  const [isComposing, setIsComposing] = useState(false);
  const [wordCount, setWordCount] = useState(0);
  const [showZenMode, setShowZenMode] = useState(false);

  // Calculate word count (Chinese characters + English words)
  useEffect(() => {
    const chineseChars = (content.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = (content.match(/[a-zA-Z]+/g) || []).length;
    setWordCount(chineseChars + englishWords);
  }, [content]);

  // Handle content change
  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = e.target.value;
    onChange(newContent);
    setCursorPosition(e.target.selectionStart || 0);
  };

  // Handle selection change
  const handleSelect = (e: React.SyntheticEvent<HTMLTextAreaElement>) => {
    const target = e.target as HTMLTextAreaElement;
    setCursorPosition(target.selectionStart || 0);
  };

  // Handle keyboard shortcuts
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // AI Generation shortcuts
    if (e.key === ' ' && e.ctrlKey) {
      e.preventDefault();
      // Get context around cursor (last 200 chars)
      const beforeCursor = content.slice(Math.max(0, cursorPosition - 200), cursorPosition);
      onRequestGeneration(beforeCursor);
      return;
    }

    // Tab to accept AI suggestion (when implemented)
    if (e.key === 'Tab') {
      // Could be used to accept inline AI suggestions
      return;
    }

    // Zen mode toggle
    if (e.key === 'F11' || (e.key === 'z' && e.ctrlKey && e.altKey)) {
      e.preventDefault();
      setShowZenMode(prev => !prev);
    }

    // Save shortcut
    if (e.key === 's' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      // Auto-save is handled automatically
    }
  };

  // Auto-resize textarea
  useEffect(() => {
    if (editorRef.current) {
      editorRef.current.style.height = 'auto';
      editorRef.current.style.height = `${editorRef.current.scrollHeight}px`;
    }
  }, [content]);

  // Focus on mount
  useEffect(() => {
    if (editorRef.current && content === '') {
      editorRef.current.focus();
    }
  }, []);

  // Get current line and column for AI hints positioning
  const getCursorLineInfo = useCallback(() => {
    const textBeforeCursor = content.slice(0, cursorPosition);
    const lines = textBeforeCursor.split('\n');
    const currentLine = lines.length;
    const currentColumn = lines[lines.length - 1].length + 1;
    return { line: currentLine, column: currentColumn };
  }, [content, cursorPosition]);

  return (
    <div className={`reader-writer ${showZenMode ? 'zen-mode' : ''}`}>
      <textarea
        ref={editorRef}
        className="reader-writer-editor"
        value={content}
        onChange={handleChange}
        onSelect={handleSelect}
        onKeyDown={handleKeyDown}
        onCompositionStart={() => setIsComposing(true)}
        onCompositionEnd={() => setIsComposing(false)}
        placeholder="开始你的创作..."
        spellCheck={false}
        autoComplete="off"
        autoCorrect="off"
        autoCapitalize="off"
      />
      
      <div className="reader-writer-stats">
        <span>{wordCount} 字</span>
        <span className="divider">|</span>
        <span>Ctrl+Space AI续写</span>
        <span className="divider">|</span>
        <span>Ctrl+Alt+Z 禅模式</span>
      </div>
      
      {showZenMode && (
        <div 
          className="zen-exit-hint"
          onClick={() => setShowZenMode(false)}
        >
          点击退出禅模式
        </div>
      )}
    </div>
  );
}