/**
 * ReaderWriter - 极简阅读写作界面 (v2.0)
 * 
 * 设计理念：
 * - 接近最终阅读界面的排版
 * - 沉浸式的写作体验
 * - 支持 AI 辅助快捷键
 * - 富文本编辑支持
 * - 写作风格切换
 * - 角色卡片弹窗
 */

import { useState, useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import RichTextEditor, { RichTextEditorRef } from './RichTextEditor';
import { WritingStyleSwitcher } from './WritingStyleSwitcher';
import { CharacterCardPopup } from './CharacterCardPopup';
import { useWritingStyle } from '@/frontstage/hooks/useWritingStyle';
import { Sparkles, Type, Focus, Minimize2 } from 'lucide-react';
import { cn } from '@/utils/cn';
import type { Character } from '@/types/index';

interface ReaderWriterProps {
  content: string;
  onChange: (content: string) => void;
  onRequestGeneration?: (context: string) => Promise<string>;
  aiEnabled?: boolean;
  isGenerating?: boolean;
  generatedText?: string;
  onAcceptGeneration?: () => void;
  onRejectGeneration?: () => void;
  placeholder?: string;
  characters?: Character[];
}

export function ReaderWriter({
  content,
  onChange,
  onRequestGeneration,
  aiEnabled = false,
  isGenerating = false,
  generatedText = '',
  onAcceptGeneration,
  onRejectGeneration,
  placeholder,
  characters = [],
}: ReaderWriterProps) {
  const editorRef = useRef<RichTextEditorRef>(null);
  const [wordCount, setWordCount] = useState(0);
  const [showZenMode, setShowZenMode] = useState(false);
  const [fontSize, setFontSize] = useState(18);
  const [lineHeight, setLineHeight] = useState(1.8);
  const [showSettings, setShowSettings] = useState(false);
  const [showAiPreview, setShowAiPreview] = useState(false);

  // 写作风格管理
  const { currentStyle, setStyle, getStyleVariables, availableStyles } = useWritingStyle();

  // 角色卡片弹窗状态
  const [selectedCharacter, setSelectedCharacter] = useState<Character | null>(null);
  const [popupPosition, setPopupPosition] = useState({ x: 0, y: 0 });
  const [showPopup, setShowPopup] = useState(false);
  const [popupAnchor, setPopupAnchor] = useState<HTMLElement | null>(null);

  // Calculate word count (Chinese characters + English words)
  useEffect(() => {
    const text = editorRef.current?.getText() || '';
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
    setWordCount(chineseChars + englishWords);
  }, [content]);

  // 处理角色名点击
  const handleCharacterClick = useCallback((characterName: string, element: HTMLElement) => {
    const character = characters.find(c => c.name === characterName);
    if (character) {
      const rect = element.getBoundingClientRect();
      setPopupPosition({ x: rect.left, y: rect.bottom + 8 });
      setPopupAnchor(element);
      setSelectedCharacter(character);
      setShowPopup(true);
    }
  }, [characters]);

  // Handle AI generation request
  const handleRequestGeneration = useCallback(async () => {
    if (!onRequestGeneration) return;
    
    const text = editorRef.current?.getText() || '';
    const context = text.slice(-500); // Get last 500 chars as context
    
    try {
      setShowAiPreview(true);
      await onRequestGeneration(context);
    } catch (error) {
      console.error('Generation failed:', error);
    }
  }, [onRequestGeneration]);

  // Handle keyboard shortcuts
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    // AI Generation shortcuts
    if (e.code === 'Space' && e.ctrlKey && !e.shiftKey) {
      e.preventDefault();
      handleRequestGeneration();
      return;
    }

    // Tab to accept AI suggestion
    if (e.key === 'Tab' && generatedText && onAcceptGeneration) {
      e.preventDefault();
      onAcceptGeneration();
      return;
    }

    // Esc to reject AI suggestion
    if (e.key === 'Escape' && generatedText && onRejectGeneration) {
      e.preventDefault();
      onRejectGeneration();
      return;
    }

    // Zen mode toggle
    if (e.key === 'F11') {
      e.preventDefault();
      setShowZenMode(prev => !prev);
    }
  }, [generatedText, onAcceptGeneration, onRejectGeneration, handleRequestGeneration]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return (
    <div
      className={cn(
        'reader-writer flex flex-col h-full transition-all duration-300',
        showZenMode && 'zen-mode fixed inset-0 z-50'
      )}
      style={{
        fontSize: `${fontSize}px`,
        lineHeight: lineHeight,
        ...getStyleVariables(),
        backgroundColor: 'var(--fs-paper-color, var(--parchment))',
      }}
    >
      {/* 顶部工具栏 */}
      {!showZenMode && (
        <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--warm-sand)] bg-[var(--parchment-dark)]">
          {/* 左侧：AI 控制 */}
          <div className="flex items-center gap-2">
            <button
              onClick={handleRequestGeneration}
              disabled={isGenerating || !aiEnabled}
              className={cn(
                'flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm transition-colors',
                'hover:bg-[var(--warm-sand)] disabled:opacity-50',
                isGenerating && 'animate-pulse text-[var(--terracotta)]'
              )}
            >
              <Sparkles className="w-4 h-4" />
              {isGenerating ? '生成中...' : 'AI 续写'}
            </button>
            
            {generatedText && (
              <div className="flex items-center gap-2 text-sm">
                <span className="text-[var(--stone-gray)]">AI 建议:</span>
                <button
                  onClick={onAcceptGeneration}
                  className="px-2 py-1 bg-[var(--terracotta)] text-white rounded hover:bg-[var(--terracotta-dark)]"
                >
                  Tab 接受
                </button>
                <button
                  onClick={onRejectGeneration}
                  className="px-2 py-1 text-[var(--stone-gray)] hover:text-[var(--charcoal)]"
                >
                  Esc 拒绝
                </button>
              </div>
            )}
          </div>

          {/* 右侧：设置 */}
          <div className="flex items-center gap-2">
            {/* 写作风格切换 */}
            <WritingStyleSwitcher
              currentStyle={currentStyle}
              availableStyles={availableStyles}
              onStyleChange={setStyle}
            />

            {/* 字体大小 */}
            <div className="relative">
              <button
                onClick={() => setShowSettings(!showSettings)}
                className="p-2 rounded-lg hover:bg-[var(--warm-sand)] text-[var(--charcoal)]"
                title="排版设置"
              >
                <Type className="w-4 h-4" />
              </button>
              
              {showSettings && (
                <div className="absolute right-0 top-full mt-2 w-64 bg-white rounded-xl shadow-lg border border-[var(--warm-sand)] p-4 z-50">
                  <h4 className="text-sm font-medium text-[var(--charcoal)] mb-3">排版设置</h4>
                  
                  {/* 字号 */}
                  <div className="mb-4">
                    <label className="flex items-center justify-between text-sm text-[var(--stone-gray)] mb-2">
                      <span>字号</span>
                      <span>{fontSize}px</span>
                    </label>
                    <input
                      type="range"
                      min={14}
                      max={24}
                      step={1}
                      value={fontSize}
                      onChange={(e) => setFontSize(Number(e.target.value))}
                      className="w-full accent-[var(--terracotta)]"
                    />
                  </div>
                  
                  {/* 行距 */}
                  <div>
                    <label className="flex items-center justify-between text-sm text-[var(--stone-gray)] mb-2">
                      <span>行距</span>
                      <span>{lineHeight.toFixed(1)}</span>
                    </label>
                    <input
                      type="range"
                      min={1.5}
                      max={2.5}
                      step={0.1}
                      value={lineHeight}
                      onChange={(e) => setLineHeight(Number(e.target.value))}
                      className="w-full accent-[var(--terracotta)]"
                    />
                  </div>
                </div>
              )}
            </div>

            {/* 禅模式 */}
            <button
              onClick={() => setShowZenMode(true)}
              className="p-2 rounded-lg hover:bg-[var(--warm-sand)] text-[var(--charcoal)]"
              title="禅模式 (F11)"
            >
              <Focus className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}

      {/* AI 生成预览 */}
      {showAiPreview && generatedText && (
        <div className="mx-8 my-4 p-4 bg-[var(--terracotta)]/5 border-l-4 border-[var(--terracotta)] rounded-r-lg">
          <p className="text-sm text-[var(--stone-gray)] italic mb-2">AI 建议续写：</p>
          <p className="text-[var(--charcoal)] leading-relaxed">{generatedText}</p>
          <div className="flex items-center gap-2 mt-3 text-sm">
            <button
              onClick={onAcceptGeneration}
              className="px-3 py-1 bg-[var(--terracotta)] text-white rounded hover:bg-[var(--terracotta-dark)]"
            >
              Tab 接受
            </button>
            <button
              onClick={onRejectGeneration}
              className="px-3 py-1 text-[var(--stone-gray)] hover:text-[var(--charcoal)]"
            >
              Esc 拒绝
            </button>
          </div>
        </div>
      )}

      {/* 编辑器 */}
      <div className="flex-1 overflow-hidden">
        <RichTextEditor
          ref={editorRef}
          content={content}
          onChange={onChange}
          placeholder="开始你的创作之旅..."
          className="h-full"
        />
      </div>

      {/* 底部状态栏 */}
      {!showZenMode && (
        <div className="flex items-center justify-between px-4 py-2 border-t border-[var(--warm-sand)] bg-[var(--parchment-dark)] text-sm text-[var(--stone-gray)]">
          <div className="flex items-center gap-4">
            <span>{wordCount} 字</span>
            <span className="text-[var(--warm-sand)]">|</span>
            <span>Ctrl+Space AI续写</span>
            <span className="text-[var(--warm-sand)]">|</span>
            <span>F11 禅模式</span>
          </div>
          
          {isGenerating && (
            <div className="flex items-center gap-2 text-[var(--terracotta)]">
              <span className="w-2 h-2 bg-[var(--terracotta)] rounded-full animate-pulse" />
              <span>AI 正在思考...</span>
            </div>
          )}
        </div>
      )}

      {/* 禅模式退出提示 */}
      {showZenMode && (
        <button
          onClick={() => setShowZenMode(false)}
          className="fixed bottom-8 left-1/2 -translate-x-1/2 px-4 py-2 bg-white/90 backdrop-blur-sm rounded-full shadow-lg text-sm text-[var(--stone-gray)] hover:text-[var(--charcoal)] transition-colors flex items-center gap-2"
        >
          <Minimize2 className="w-4 h-4" />
          退出禅模式 (F11)
        </button>
      )}

      {/* 点击外部关闭设置面板 */}
      {showSettings && (
        <div
          className="fixed inset-0 z-40"
          onClick={() => setShowSettings(false)}
        />
      )}

      {/* 角色卡片弹窗 */}
      <CharacterCardPopup
        character={selectedCharacter || { id: '', story_id: '', name: '', created_at: '', updated_at: '' }}
        position={popupPosition}
        visible={showPopup}
        onClose={() => setShowPopup(false)}
        anchorEl={popupAnchor}
      />
    </div>
  );
}
