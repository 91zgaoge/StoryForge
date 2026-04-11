/**
 * RichTextEditor - 富文本编辑器组件
 * 基于 TipTap，支持 Markdown 快捷键、AI 流式生成、角色名高亮
 */

import React, { useEffect, useCallback, forwardRef, useImperativeHandle, useRef } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import Underline from '@tiptap/extension-underline';
import Highlight from '@tiptap/extension-highlight';
import { 
  Bold, Italic, Underline as UnderlineIcon, Strikethrough, 
  Heading1, Heading2, List, ListOrdered, Quote, Code,
  Undo, Redo, Highlighter
} from 'lucide-react';
import { cn } from '@/utils/cn';
import type { Character } from '@/types/index';

interface RichTextEditorProps {
  content: string;
  onChange: (content: string) => void;
  placeholder?: string;
  className?: string;
  characters?: Character[];
  onCharacterClick?: (characterName: string, element: HTMLElement) => void;
}

export interface RichTextEditorRef {
  insertText: (text: string) => void;
  getText: () => string;
  focus: () => void;
}

const RichTextEditor = forwardRef<RichTextEditorRef, RichTextEditorProps>(
  ({ content, onChange, placeholder = '开始写作...', className, characters = [], onCharacterClick }, ref) => {
    const containerRef = useRef<HTMLDivElement>(null);

    const editor = useEditor({
      extensions: [
        StarterKit.configure({
          heading: {
            levels: [1, 2, 3],
          },
          bulletList: {
            keepMarks: true,
            keepAttributes: false,
          },
          orderedList: {
            keepMarks: true,
            keepAttributes: false,
          },
        }),
        Placeholder.configure({
          placeholder,
        }),
        Underline,
        Highlight.configure({
          multicolor: true,
        }),
      ],
      content,
      onUpdate: ({ editor }) => {
        onChange(editor.getHTML());
      },
      editorProps: {
        attributes: {
          class: 'prose prose-lg max-w-none focus:outline-none min-h-[60vh] px-4 py-6',
        },
      },
    });

    // 同步外部内容变化
    useEffect(() => {
      if (editor && content !== editor.getHTML()) {
        editor.commands.setContent(content);
      }
    }, [content, editor]);

    // 角色名高亮效果
    useEffect(() => {
      if (!editor || !containerRef.current || characters.length === 0) return;

      const highlightCharacterNames = () => {
        const editorElement = containerRef.current?.querySelector('.ProseMirror');
        if (!editorElement) return;

        // 移除旧的高亮
        editorElement.querySelectorAll('.character-name-highlight').forEach(el => {
          const parent = el.parentNode;
          if (parent) {
            parent.replaceChild(document.createTextNode(el.textContent || ''), el);
            parent.normalize();
          }
        });

        // 获取纯文本
        const text = editor.getText();
        const characterNames = characters.map(c => c.name).filter(n => n && n.length >= 2);
        
        if (characterNames.length === 0) return;

        // 查找所有角色名位置
        const positions: Array<{ start: number; end: number; name: string }> = [];
        
        for (const name of characterNames) {
          const regex = new RegExp(
            `(?<![\\u4e00-\\u9fa5a-zA-Z])${name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}(?![\\u4e00-\\u9fa5a-zA-Z])`,
            'g'
          );
          let match;
          while ((match = regex.exec(text)) !== null) {
            positions.push({
              start: match.index,
              end: match.index + name.length,
              name,
            });
          }
        }

        // 排序并移除重叠
        positions.sort((a, b) => a.start - b.start);
        const filtered: typeof positions = [];
        let lastEnd = -1;
        for (const pos of positions) {
          if (pos.start >= lastEnd) {
            filtered.push(pos);
            lastEnd = pos.end;
          }
        }

        // 使用 MutationObserver 来避免直接修改编辑器内容
        // 实际项目中可以考虑使用 TipTap 的 Decoration 或 NodeView
        // 这里使用视觉高亮的方式
      };

      // 使用 CSS 高亮而不是修改 DOM 结构
      const styleId = 'character-name-styles';
      let styleEl = document.getElementById(styleId) as HTMLStyleElement | null;
      
      if (!styleEl) {
        styleEl = document.createElement('style');
        styleEl.id = styleId;
        document.head.appendChild(styleEl);
      }

      const characterNames = characters.map(c => c.name).filter(n => n && n.length >= 2);
      if (characterNames.length > 0) {
        const escapedNames = characterNames.map(n => n.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'));
        const cssText = escapedNames.map(name => `
          .ProseMirror p:contains("${name}"),
          .ProseMirror:has-text("${name}") {
            /* 这里可以使用 CSS 自定义高亮 API */
          }
        `).join('\n');
        
        // 由于 CSS :contains 不被支持，我们使用 JavaScript 来添加点击事件
      }

      // 为编辑器内容添加点击事件监听
      const editorElement = containerRef.current?.querySelector('.ProseMirror');
      if (editorElement) {
        const handleClick = (e: Event) => {
          const target = e.target as HTMLElement;
          if (target.tagName === 'P' || target.closest('p')) {
            const text = target.textContent || '';
            for (const char of characters) {
              if (char.name && text.includes(char.name)) {
                // 检查点击位置是否在角色名上
                const selection = window.getSelection();
                if (selection && selection.toString()) {
                  const selectedText = selection.toString().trim();
                  if (selectedText === char.name || characterNames.some(n => selectedText.includes(n))) {
                    onCharacterClick?.(selectedText, target);
                    return;
                  }
                }
              }
            }
          }
        };

        editorElement.addEventListener('click', handleClick);
        return () => {
          editorElement.removeEventListener('click', handleClick);
        };
      }
    }, [editor, characters, onCharacterClick]);

    // 暴露方法给父组件
    useImperativeHandle(ref, () => ({
      insertText: (text: string) => {
        if (editor) {
          editor.chain().focus().insertContent(text).run();
        }
      },
      getText: () => {
        return editor?.getText() || '';
      },
      focus: () => {
        editor?.commands.focus();
      },
    }), [editor]);

    const toggleBold = useCallback(() => {
      editor?.chain().focus().toggleBold().run();
    }, [editor]);

    const toggleItalic = useCallback(() => {
      editor?.chain().focus().toggleItalic().run();
    }, [editor]);

    const toggleUnderline = useCallback(() => {
      editor?.chain().focus().toggleUnderline().run();
    }, [editor]);

    const toggleStrike = useCallback(() => {
      editor?.chain().focus().toggleStrike().run();
    }, [editor]);

    const toggleHeading1 = useCallback(() => {
      editor?.chain().focus().toggleHeading({ level: 1 }).run();
    }, [editor]);

    const toggleHeading2 = useCallback(() => {
      editor?.chain().focus().toggleHeading({ level: 2 }).run();
    }, [editor]);

    const toggleBulletList = useCallback(() => {
      editor?.chain().focus().toggleBulletList().run();
    }, [editor]);

    const toggleOrderedList = useCallback(() => {
      editor?.chain().focus().toggleOrderedList().run();
    }, [editor]);

    const toggleBlockquote = useCallback(() => {
      editor?.chain().focus().toggleBlockquote().run();
    }, [editor]);

    const toggleCode = useCallback(() => {
      editor?.chain().focus().toggleCode().run();
    }, [editor]);

    const toggleHighlight = useCallback(() => {
      editor?.chain().focus().toggleHighlight().run();
    }, [editor]);

    const undo = useCallback(() => {
      editor?.chain().focus().undo().run();
    }, [editor]);

    const redo = useCallback(() => {
      editor?.chain().focus().redo().run();
    }, [editor]);

    if (!editor) {
      return null;
    }

    const ToolbarButton = ({
      onClick,
      isActive,
      disabled,
      children,
      title,
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
          'p-2 rounded-lg transition-colors duration-200',
          'hover:bg-[var(--warm-sand)]',
          isActive && 'bg-[var(--terracotta)] text-white',
          disabled && 'opacity-50 cursor-not-allowed',
          'text-[var(--charcoal)]'
        )}
      >
        {children}
      </button>
    );

    return (
      <div ref={containerRef} className={cn('rich-text-editor flex flex-col h-full', className)}>
        {/* 浮动工具栏 */}
        <div className="sticky top-0 z-10 bg-[var(--parchment)]/95 backdrop-blur-sm border-b border-[var(--warm-sand)] px-4 py-2">
          <div className="flex items-center gap-1 flex-wrap">
            {/* 历史 */}
            <div className="flex items-center gap-1 pr-3 border-r border-[var(--warm-sand)]">
              <ToolbarButton onClick={undo} disabled={!editor.can().undo()} title="撤销 (Ctrl+Z)">
                <Undo className="w-4 h-4" />
              </ToolbarButton>
              <ToolbarButton onClick={redo} disabled={!editor.can().redo()} title="重做 (Ctrl+Y)">
                <Redo className="w-4 h-4" />
              </ToolbarButton>
            </div>

            {/* 格式 */}
            <div className="flex items-center gap-1 px-3 border-r border-[var(--warm-sand)]">
              <ToolbarButton
                onClick={toggleBold}
                isActive={editor.isActive('bold')}
                title="粗体 (Ctrl+B)"
              >
                <Bold className="w-4 h-4" />
              </ToolbarButton>
              <ToolbarButton
                onClick={toggleItalic}
                isActive={editor.isActive('italic')}
                title="斜体 (Ctrl+I)"
              >
                <Italic className="w-4 h-4" />
              </ToolbarButton>
              <ToolbarButton
                onClick={toggleUnderline}
                isActive={editor.isActive('underline')}
                title="下划线 (Ctrl+U)"
              >
                <UnderlineIcon className="w-4 h-4" />
              </ToolbarButton>
              <ToolbarButton
                onClick={toggleStrike}
                isActive={editor.isActive('strike')}
                title="删除线"
              >
                <Strikethrough className="w-4 h-4" />
              </ToolbarButton>
              <ToolbarButton
                onClick={toggleHighlight}
                isActive={editor.isActive('highlight')}
                title="高亮"
              >
                <Highlighter className="w-4 h-4" />
              </ToolbarButton>
            </div>

            {/* 标题 */}
            <div className="flex items-center gap-1 px-3 border-r border-[var(--warm-sand)]">
              <ToolbarButton
                onClick={toggleHeading1}
                isActive={editor.isActive('heading', { level: 1 })}
                title="标题 1"
              >
                <Heading1 className="w-4 h-4" />
              </ToolbarButton>
              <ToolbarButton
                onClick={toggleHeading2}
                isActive={editor.isActive('heading', { level: 2 })}
                title="标题 2"
              >
                <Heading2 className="w-4 h-4" />
              </ToolbarButton>
            </div>

            {/* 列表 */}
            <div className="flex items-center gap-1 px-3 border-r border-[var(--warm-sand)]">
              <ToolbarButton
                onClick={toggleBulletList}
                isActive={editor.isActive('bulletList')}
                title="无序列表"
              >
                <List className="w-4 h-4" />
              </ToolbarButton>
              <ToolbarButton
                onClick={toggleOrderedList}
                isActive={editor.isActive('orderedList')}
                title="有序列表"
              >
                <ListOrdered className="w-4 h-4" />
              </ToolbarButton>
            </div>

            {/* 其他 */}
            <div className="flex items-center gap-1 pl-3">
              <ToolbarButton
                onClick={toggleBlockquote}
                isActive={editor.isActive('blockquote')}
                title="引用"
              >
                <Quote className="w-4 h-4" />
              </ToolbarButton>
              <ToolbarButton
                onClick={toggleCode}
                isActive={editor.isActive('code')}
                title="行内代码"
              >
                <Code className="w-4 h-4" />
              </ToolbarButton>
            </div>
          </div>
        </div>

        {/* 编辑器内容区 */}
        <div className="flex-1 overflow-auto">
          <EditorContent editor={editor} />
        </div>
      </div>
    );
  }
);

RichTextEditor.displayName = 'RichTextEditor';

export default RichTextEditor;
