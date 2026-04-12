/**
 * RichTextEditor - 富文本编辑器组件 (v3.0)
 * 
 * 整合了原 ReaderWriter 的功能
 * 工具栏位于底部，默认隐藏，悬停显示
 * 仿 Claude 纸质平面设计风格
 */

import React, { useEffect, useCallback, forwardRef, useImperativeHandle, useRef, useState } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import Underline from '@tiptap/extension-underline';
import Highlight from '@tiptap/extension-highlight';
import { 
  Bold, Italic, Underline as UnderlineIcon, Strikethrough, 
  Heading1, Heading2, List, ListOrdered, Quote, Code,
  Undo, Redo, Highlighter, Sparkles, Minimize2
} from 'lucide-react';
import { cn } from '@/utils/cn';
import type { Character } from '@/types/index';
import { CharacterCardPopup } from './CharacterCardPopup';
import { 
  loadEditorConfig, 
  type EditorConfig 
} from '@/components/EditorSettings';
import { defaultStyle } from '@/frontstage/config/writingStyles';

interface RichTextEditorProps {
  content: string;
  onChange: (content: string) => void;
  placeholder?: string;
  className?: string;
  characters?: Character[];
  onRequestGeneration?: (context: string) => Promise<string>;
  aiEnabled?: boolean;
  generatedText?: string;
  onAcceptGeneration?: () => void;
  onRejectGeneration?: () => void;
}

export interface RichTextEditorRef {
  insertText: (text: string) => void;
  getText: () => string;
  focus: () => void;
}

const RichTextEditor = forwardRef<RichTextEditorRef, RichTextEditorProps>(
  ({ 
    content, 
    onChange, 
    placeholder = '开始写作...', 
    className, 
    characters = [],
    onRequestGeneration,
    aiEnabled = false,
    generatedText = '',
    onAcceptGeneration,
    onRejectGeneration,
  }, ref) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const [editorConfig, setEditorConfig] = useState<EditorConfig>(loadEditorConfig());
    const [isZenMode, setIsZenMode] = useState(false);
    const [isGenerating, setIsGenerating] = useState(false);
    const [wordCount, setWordCount] = useState(0);
    const [showToolbar, setShowToolbar] = useState(false);
    
    // 角色卡片弹窗状态
    const [selectedCharacter, setSelectedCharacter] = useState<Character | null>(null);
    const [popupPosition, setPopupPosition] = useState({ x: 0, y: 0 });
    const [showPopup, setShowPopup] = useState(false);
    const [popupAnchor, setPopupAnchor] = useState<HTMLElement | null>(null);

    const editor = useEditor({
      extensions: [
        StarterKit.configure({
          heading: { levels: [1, 2, 3] },
          bulletList: { keepMarks: true, keepAttributes: false },
          orderedList: { keepMarks: true, keepAttributes: false },
        }),
        Placeholder.configure({ placeholder }),
        Underline,
        Highlight.configure({ multicolor: true }),
      ],
      content,
      onUpdate: ({ editor }) => {
        onChange(editor.getHTML());
        const text = editor.getText();
        const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
        const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
        setWordCount(chineseChars + englishWords);
      },
      editorProps: {
        attributes: {
          class: 'prose prose-lg focus:outline-none',
        },
      },
    });

    // 监听配置变化
    useEffect(() => {
      const handleStorageChange = () => {
        setEditorConfig(loadEditorConfig());
      };
      window.addEventListener('storage', handleStorageChange);
      return () => window.removeEventListener('storage', handleStorageChange);
    }, []);

    // 同步外部内容变化
    useEffect(() => {
      if (editor && content !== editor.getHTML()) {
        editor.commands.setContent(content);
      }
    }, [content, editor]);

    // 处理角色名点击
    useEffect(() => {
      if (!editor || !containerRef.current || characters.length === 0) return;

      const editorElement = containerRef.current?.querySelector('.ProseMirror');
      if (!editorElement) return;

      const handleClick = (e: Event) => {
        const target = e.target as HTMLElement;
        if (target.tagName === 'P' || target.closest('p')) {
          const text = target.textContent || '';
          const selection = window.getSelection();
          if (selection && selection.toString()) {
            const selectedText = selection.toString().trim();
            const character = characters.find(c => c.name === selectedText);
            if (character) {
              const rect = target.getBoundingClientRect();
              setPopupPosition({ x: rect.left, y: rect.bottom + 8 });
              setPopupAnchor(target);
              setSelectedCharacter(character);
              setShowPopup(true);
            }
          }
        }
      };

      editorElement.addEventListener('click', handleClick);
      return () => editorElement.removeEventListener('click', handleClick);
    }, [editor, characters]);

    // AI 生成处理函数
    const handleRequestGeneration = useCallback(async () => {
      if (!onRequestGeneration || !editor) return;
      
      const text = editor.getText();
      const context = text.slice(-500);
      
      try {
        setIsGenerating(true);
        await onRequestGeneration(context);
      } catch (error) {
        console.error('Generation failed:', error);
      } finally {
        setIsGenerating(false);
      }
    }, [onRequestGeneration, editor]);

    // 键盘快捷键
    useEffect(() => {
      const handleKeyDown = (e: KeyboardEvent) => {
        // AI Generation
        if (e.code === 'Space' && e.ctrlKey && !e.shiftKey) {
          e.preventDefault();
          handleRequestGeneration();
          return;
        }

        // Accept AI suggestion
        if (e.key === 'Tab' && generatedText && onAcceptGeneration) {
          e.preventDefault();
          onAcceptGeneration();
          return;
        }

        // Reject AI suggestion
        if (e.key === 'Escape' && generatedText && onRejectGeneration) {
          e.preventDefault();
          onRejectGeneration();
          return;
        }

        // Zen mode
        if (e.key === 'F11') {
          e.preventDefault();
          setIsZenMode(prev => !prev);
        }
      };

      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
    }, [generatedText, onAcceptGeneration, onRejectGeneration, handleRequestGeneration]);

    // 暴露方法给父组件
    useImperativeHandle(ref, () => ({
      insertText: (text: string) => {
        if (editor) {
          editor.chain().focus().insertContent(text).run();
        }
      },
      getText: () => editor?.getText() || '',
      focus: () => editor?.commands.focus(),
    }), [editor]);

    const toggleBold = useCallback(() => editor?.chain().focus().toggleBold().run(), [editor]);
    const toggleItalic = useCallback(() => editor?.chain().focus().toggleItalic().run(), [editor]);
    const toggleUnderline = useCallback(() => editor?.chain().focus().toggleUnderline().run(), [editor]);
    const toggleStrike = useCallback(() => editor?.chain().focus().toggleStrike().run(), [editor]);
    const toggleHeading1 = useCallback(() => editor?.chain().focus().toggleHeading({ level: 1 }).run(), [editor]);
    const toggleHeading2 = useCallback(() => editor?.chain().focus().toggleHeading({ level: 2 }).run(), [editor]);
    const toggleBulletList = useCallback(() => editor?.chain().focus().toggleBulletList().run(), [editor]);
    const toggleOrderedList = useCallback(() => editor?.chain().focus().toggleOrderedList().run(), [editor]);
    const toggleBlockquote = useCallback(() => editor?.chain().focus().toggleBlockquote().run(), [editor]);
    const toggleCode = useCallback(() => editor?.chain().focus().toggleCode().run(), [editor]);
    const toggleHighlight = useCallback(() => editor?.chain().focus().toggleHighlight().run(), [editor]);
    const undo = useCallback(() => editor?.chain().focus().undo().run(), [editor]);
    const redo = useCallback(() => editor?.chain().focus().redo().run(), [editor]);

    if (!editor) return null;

    // 获取当前风格
    const currentStyle = defaultStyle;

    // 生成CSS变量
    const styleVars = {
      '--fs-font-family': editorConfig.fontFamily,
      '--fs-font-size': `${editorConfig.fontSize}px`,
      '--fs-line-height': editorConfig.lineHeight,
      '--fs-letter-spacing': 'normal',
      '--fs-paragraph-spacing': '1.5em',
      '--fs-paper-color': currentStyle.paperColor,
      '--fs-ink-color': currentStyle.inkColor,
      '--fs-accent-color': currentStyle.accentColor,
    } as React.CSSProperties;

    const ToolbarButton = ({
      onClick, isActive, disabled, children, title,
    }: {
      onClick: () => void;
      isActive?: boolean;
      disabled?: boolean;
      children: React.ReactNode;
      title: string;
    }) => (
      <button
        onClick={onClick}
        disabled={disabled}
        title={title}
        className={cn(
          'relative px-2.5 py-1.5 text-xs font-serif',
          'bg-[var(--parchment)]',
          'border border-[var(--warm-sand)]',
          'rounded',
          'text-[var(--charcoal)]',
          'transition-all duration-150 ease-out',
          'hover:border-[var(--terracotta)]/50',
          'hover:bg-[var(--ivory)]',
          'hover:shadow-sm',
          isActive && [
            'bg-[var(--terracotta)]/10',
            'border-[var(--terracotta)]',
            'text-[var(--terracotta-dark)]',
            'shadow-inner',
          ],
          disabled && [
            'opacity-40',
            'cursor-not-allowed',
            'hover:border-[var(--warm-sand)]',
            'hover:bg-[var(--parchment)]',
            'hover:shadow-none',
          ],
        )}
      >
        {children}
      </button>
    );

    return (
      <div 
        ref={containerRef}
        className={cn(
          'rich-text-editor flex flex-col h-full relative',
          isZenMode && 'zen-mode',
          className
        )}
        style={styleVars}
        onMouseEnter={() => setShowToolbar(true)}
        onMouseLeave={() => setShowToolbar(false)}
      >
        {/* 编辑器内容区 */}
        <div className="flex-1 overflow-auto">
          <EditorContent editor={editor} />
        </div>

        {/* AI 生成预览 */}
        {generatedText && (
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

        {/* 底部工具栏 - 默认隐藏，悬停显示 */}
        <div 
          className={cn(
            'editor-toolbar absolute bottom-0 left-0 right-0',
            'bg-[var(--parchment)]/98 border-t border-[var(--warm-sand)]',
            'px-4 py-3',
            'transition-all duration-300 ease-out',
            'flex flex-col items-center gap-3',
            showToolbar ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-full pointer-events-none'
          )}
        >
          {/* 第一行：AI 控制 */}
          <div className="flex items-center justify-center gap-4 w-full">
            <button
              onClick={handleRequestGeneration}
              disabled={isGenerating || !aiEnabled}
              className={cn(
                'flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-serif',
                'bg-[var(--parchment)] border border-[var(--warm-sand)]',
                'text-[var(--charcoal)]',
                'hover:border-[var(--terracotta)]/50 hover:bg-[var(--ivory)]',
                'transition-all duration-200',
                (isGenerating || !aiEnabled) && 'opacity-50 cursor-not-allowed'
              )}
            >
              <Sparkles className="w-4 h-4" />
              {isGenerating ? '生成中...' : 'AI 续写'}
            </button>
            
            <div className="text-xs text-[var(--stone-gray)] font-serif">
              {wordCount} 字 · {editorConfig.fontSize}px · Ctrl+Space AI续写 · F11 禅模式
            </div>
          </div>

          {/* 第二行：格式工具 */}
          <div className="flex items-center justify-center gap-2 flex-wrap">
            {/* 历史 */}
            <div className="flex items-center gap-1 px-2 py-1 bg-[var(--parchment-dark)]/50 rounded-lg border border-[var(--warm-sand)]">
              <span className="text-[10px] text-[var(--stone-gray)] mr-1 font-serif italic uppercase">历史</span>
              <ToolbarButton onClick={undo} disabled={!editor.can().undo()} title="撤销 (Ctrl+Z)">
                <Undo className="w-3 h-3" />
              </ToolbarButton>
              <ToolbarButton onClick={redo} disabled={!editor.can().redo()} title="重做 (Ctrl+Y)">
                <Redo className="w-3 h-3" />
              </ToolbarButton>
            </div>

            {/* 格式 */}
            <div className="flex items-center gap-1 px-2 py-1 bg-[var(--parchment-dark)]/50 rounded-lg border border-[var(--warm-sand)]">
              <span className="text-[10px] text-[var(--stone-gray)] mr-1 font-serif italic uppercase">格式</span>
              <ToolbarButton onClick={toggleBold} isActive={editor.isActive('bold')} title="粗体 (Ctrl+B)">
                <span className="font-bold">B</span>
              </ToolbarButton>
              <ToolbarButton onClick={toggleItalic} isActive={editor.isActive('italic')} title="斜体 (Ctrl+I)">
                <span className="italic">I</span>
              </ToolbarButton>
              <ToolbarButton onClick={toggleUnderline} isActive={editor.isActive('underline')} title="下划线 (Ctrl+U)">
                <span className="underline">U</span>
              </ToolbarButton>
              <ToolbarButton onClick={toggleStrike} isActive={editor.isActive('strike')} title="删除线">
                <span className="line-through">S</span>
              </ToolbarButton>
              <ToolbarButton onClick={toggleHighlight} isActive={editor.isActive('highlight')} title="高亮">
                <Highlighter className="w-3 h-3" />
              </ToolbarButton>
            </div>

            {/* 标题 */}
            <div className="flex items-center gap-1 px-2 py-1 bg-[var(--parchment-dark)]/50 rounded-lg border border-[var(--warm-sand)]">
              <span className="text-[10px] text-[var(--stone-gray)] mr-1 font-serif italic uppercase">标题</span>
              <ToolbarButton onClick={toggleHeading1} isActive={editor.isActive('heading', { level: 1 })} title="标题 1">
                <Heading1 className="w-3 h-3" />
              </ToolbarButton>
              <ToolbarButton onClick={toggleHeading2} isActive={editor.isActive('heading', { level: 2 })} title="标题 2">
                <Heading2 className="w-3 h-3" />
              </ToolbarButton>
            </div>

            {/* 列表 */}
            <div className="flex items-center gap-1 px-2 py-1 bg-[var(--parchment-dark)]/50 rounded-lg border border-[var(--warm-sand)]">
              <span className="text-[10px] text-[var(--stone-gray)] mr-1 font-serif italic uppercase">列表</span>
              <ToolbarButton onClick={toggleBulletList} isActive={editor.isActive('bulletList')} title="无序列表">
                <List className="w-3 h-3" />
              </ToolbarButton>
              <ToolbarButton onClick={toggleOrderedList} isActive={editor.isActive('orderedList')} title="有序列表">
                <ListOrdered className="w-3 h-3" />
              </ToolbarButton>
            </div>

            {/* 其他 */}
            <div className="flex items-center gap-1 px-2 py-1 bg-[var(--parchment-dark)]/50 rounded-lg border border-[var(--warm-sand)]">
              <span className="text-[10px] text-[var(--stone-gray)] mr-1 font-serif italic uppercase">其他</span>
              <ToolbarButton onClick={toggleBlockquote} isActive={editor.isActive('blockquote')} title="引用">
                <Quote className="w-3 h-3" />
              </ToolbarButton>
              <ToolbarButton onClick={toggleCode} isActive={editor.isActive('code')} title="行内代码">
                <Code className="w-3 h-3" />
              </ToolbarButton>
            </div>
          </div>
        </div>

        {/* 禅模式退出提示 */}
        {isZenMode && (
          <button
            onClick={() => setIsZenMode(false)}
            className="fixed bottom-8 left-1/2 -translate-x-1/2 px-4 py-2 bg-white/90 backdrop-blur-sm rounded-full shadow-lg text-sm text-[var(--stone-gray)] hover:text-[var(--charcoal)] transition-colors flex items-center gap-2 z-50"
          >
            <Minimize2 className="w-4 h-4" />
            退出禅模式 (F11)
          </button>
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
);

RichTextEditor.displayName = 'RichTextEditor';

export default RichTextEditor;
