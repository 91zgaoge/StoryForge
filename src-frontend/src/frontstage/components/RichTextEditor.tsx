/**
 * RichTextEditor - 富文本编辑器组件 (v3.3)
 * 
 * 集成模型服务的对话栏
 * 支持模型切换和状态显示
 */

import React, { useEffect, useCallback, forwardRef, useImperativeHandle, useRef, useState } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import Underline from '@tiptap/extension-underline';
import Highlight from '@tiptap/extension-highlight';
import { 
  Send,
  Sparkles,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Quote,
  MessageSquarePlus,
  X,
  StickyNote,
  Check,
  Trash2,
  GitBranch,
  CheckCheck,
  Undo2,
  FileCheck
} from 'lucide-react';
import { cn } from '@/utils/cn';
import type { Character } from '@/types/index';
import { CharacterCardPopup } from './CharacterCardPopup';
import { 
  loadEditorConfig, 
  type EditorConfig 
} from '@/components/EditorSettings';
import { defaultStyle } from '@/frontstage/config/writingStyles';
import { getCurrentEditorColors } from '@/frontstage/config/colorThemes';
import { useModel } from '@/hooks/useModel';
import type { ParagraphCommentary } from '@/types/v3';
import toast from 'react-hot-toast';
import { generateParagraphCommentaries, writerAgentExecute } from '@/services/tauri';
import { TextAnnotationMark } from '@/frontstage/extensions/TextAnnotationMark';
import { TrackInsertMark, TrackDeleteMark } from '@/frontstage/extensions/TrackChanges';
import { CommentAnchorMark } from '@/frontstage/extensions/CommentAnchor';
import { useTextAnnotationsByChapter, useCreateTextAnnotation, useDeleteTextAnnotation, TEXT_ANNOTATION_TYPE_COLORS, TEXT_ANNOTATION_TYPE_LABELS } from '@/hooks/useTextAnnotations';
import { EditorContextMenu } from './EditorContextMenu';
import { usePendingChanges, useTrackChange, useAcceptChange, useRejectChange, useAcceptAllChanges, useRejectAllChanges } from '@/hooks/useChangeTracking';
import { useCommentThreads, useCreateCommentThread, useAddCommentMessage, useResolveCommentThread, useDeleteCommentThread } from '@/hooks/useCommentThreads';
import type { TextAnnotation, ChangeTrack, CommentThreadWithMessages } from '@/types/v3';

interface RichTextEditorProps {
  content: string;
  onChange: (content: string) => void;
  placeholder?: string;
  className?: string;
  characters?: Character[];
  aiEnabled?: boolean;
  generatedText?: string;
  onAcceptGeneration?: () => void;
  onRejectGeneration?: () => void;
  fontSize?: number;
  onFontSizeChange?: (size: number) => void;
  isZenMode?: boolean;
  onZenModeChange?: (zen: boolean) => void;
  storyId?: string;
  chapterId?: string;
  chapterNumber?: number;
  onWriterResult?: (text: string) => void;
  isRevisionMode?: boolean;
  onRevisionModeChange?: (v: boolean) => void;
  showAnnotationPanel?: boolean;
  onShowAnnotationPanelChange?: (v: boolean) => void;
  showCommentPanel?: boolean;
  onShowCommentPanelChange?: (v: boolean) => void;
}

export interface RichTextEditorRef {
  insertText: (text: string) => void;
  getText: () => string;
  getSelectedText: () => string;
  focus: () => void;
  generateCommentary: () => void;
}

const RichTextEditor = forwardRef<RichTextEditorRef, RichTextEditorProps>(
  ({ 
    content, 
    onChange, 
    placeholder = '开始写作...', 
    className, 
    characters = [],
    aiEnabled = false,
    generatedText = '',
    onAcceptGeneration,
    onRejectGeneration,
    fontSize: externalFontSize,
    onFontSizeChange,
    isZenMode = false,
    onZenModeChange,
    storyId,
    chapterId,
    chapterNumber,
    onWriterResult,
    isRevisionMode: externalIsRevisionMode = false,
    onRevisionModeChange,
    showAnnotationPanel: externalShowAnnotationPanel = false,
    onShowAnnotationPanelChange,
    showCommentPanel: externalShowCommentPanel = false,
    onShowCommentPanelChange,
  }, ref) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const [editorConfig, setEditorConfig] = useState<EditorConfig>(loadEditorConfig());
    const [chatInput, setChatInput] = useState('');
    const [isAiThinking, setIsAiThinking] = useState(false);
    const [lastInstruction, setLastInstruction] = useState<string | null>(null);
    const [showModelTooltip, setShowModelTooltip] = useState(false);
    const [isGeneratingCommentary, setIsGeneratingCommentary] = useState(false);
    
    // ===== Kimi Code style input features =====
    const [ghostText, setGhostText] = useState('');
    const [showSlashMenu, setShowSlashMenu] = useState(false);
    const [slashMenuIndex, setSlashMenuIndex] = useState(0);
    const [inputHistory, setInputHistory] = useState<string[]>(() => loadInputHistory());
    const [historyIndex, setHistoryIndex] = useState(-1);
    
    // 文本批注状态
    const [selectedRange, setSelectedRange] = useState<{ from: number; to: number; text: string } | null>(null);
    const [annotationContent, setAnnotationContent] = useState('');
    const [annotationType, setAnnotationType] = useState<TextAnnotation['annotation_type']>('note');
    const [annotationPopupPos, setAnnotationPopupPos] = useState({ x: 0, y: 0 });
    const [hoveredAnnotationId, setHoveredAnnotationId] = useState<string | null>(null);
    const [popupMode, setPopupMode] = useState<'annotation' | 'comment'>('annotation');
    
    // 评论线程状态
    const [activeCommentThreadId, setActiveCommentThreadId] = useState<string | null>(null);
    const [newCommentContent, setNewCommentContent] = useState('');
    const [hoveredThreadId, setHoveredThreadId] = useState<string | null>(null);
    
    // 使用模型管理Hook
    const { currentModel, status } = useModel();
    
    // 文本批注数据
    const { data: textAnnotations = [] } = useTextAnnotationsByChapter(chapterId || null);
    const createAnnotationMutation = useCreateTextAnnotation();
    const deleteAnnotationMutation = useDeleteTextAnnotation();
    
    // 评论线程数据
    const { data: commentThreads = [] } = useCommentThreads(undefined, chapterId || undefined);
    const createCommentThreadMutation = useCreateCommentThread();
    const addCommentMessageMutation = useAddCommentMessage();
    const resolveCommentThreadMutation = useResolveCommentThread();
    const deleteCommentThreadMutation = useDeleteCommentThread();
    
    // 修订模式状态（受控）
    const isRevisionMode = externalIsRevisionMode;
    const setIsRevisionMode = (v: boolean) => onRevisionModeChange?.(v);
    const prevTextRef = useRef('');
    const isRevisionModeRef = useRef(isRevisionMode);
    isRevisionModeRef.current = isRevisionMode;
    const chapterIdRef = useRef(chapterId);
    chapterIdRef.current = chapterId;
    
    // 文本批注面板（受控）
    const showAnnotationPanel = externalShowAnnotationPanel;
    const setShowAnnotationPanel = (v: boolean) => onShowAnnotationPanelChange?.(v);
    
    // 评论线程面板（受控）
    const showCommentPanel = externalShowCommentPanel;
    const setShowCommentPanel = (v: boolean) => onShowCommentPanelChange?.(v);
    
    // 右键菜单状态
    const [contextMenu, setContextMenu] = useState<{ visible: boolean; x: number; y: number }>({ visible: false, x: 0, y: 0 });
    const { data: pendingChanges = [] } = usePendingChanges(undefined, chapterId || undefined);
    const trackChangeMutation = useTrackChange();
    const acceptChangeMutation = useAcceptChange();
    const rejectChangeMutation = useRejectChange();
    const acceptAllMutation = useAcceptAllChanges();
    const rejectAllMutation = useRejectAllChanges();
    
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
        TextAnnotationMark,
        TrackInsertMark,
        TrackDeleteMark,
        CommentAnchorMark,
      ],
      content,
      onUpdate: ({ editor }) => {
        onChange(editor.getHTML());
        
        if (isRevisionModeRef.current && chapterIdRef.current) {
          const currentText = editor.getText();
          const prevText = prevTextRef.current;
          
          if (prevText && currentText !== prevText) {
            // Simple LCS-like detection for single insertion or deletion
            if (currentText.length > prevText.length) {
              // Likely insertion
              const insertPos = findFirstDiff(prevText, currentText);
              const insertedText = currentText.slice(insertPos, insertPos + (currentText.length - prevText.length));
              if (insertedText.trim()) {
                trackChangeMutation.mutate({
                  chapterId: chapterIdRef.current,
                  changeType: 'Insert',
                  fromPos: insertPos,
                  toPos: insertPos + insertedText.length,
                  content: insertedText,
                });
                // Apply visual mark
                const pmPos = textOffsetToPmPosition(editor, insertPos, insertedText.length);
                if (pmPos) {
                  editor.chain().focus().setTextSelection({ from: pmPos.from, to: pmPos.to }).setMark('trackInsert', { changeId: `temp-${Date.now()}` }).setTextSelection(pmPos.to).run();
                }
              }
            } else if (currentText.length < prevText.length) {
              // Likely deletion
              const deletePos = findFirstDiff(prevText, currentText);
              const deletedText = prevText.slice(deletePos, deletePos + (prevText.length - currentText.length));
              if (deletedText.trim()) {
                trackChangeMutation.mutate({
                  chapterId: chapterIdRef.current,
                  changeType: 'Delete',
                  fromPos: deletePos,
                  toPos: deletePos + deletedText.length,
                  content: deletedText,
                });
              }
            }
          }
          prevTextRef.current = currentText;
        }
      },
      editorProps: {
        attributes: {
          class: 'prose prose-lg focus:outline-none',
        },
        handleDOMEvents: {
          mousedown: (view, event) => {
            if ((event as MouseEvent).button === 0) {
              setSelectedRange(null);
            }
            return false;
          },
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

    // 编辑器区域右键菜单（直接绑定到编辑器容器，避免捕获阶段 stopPropagation 副作用）
    useEffect(() => {
      const editorEl = containerRef.current;
      if (!editorEl || !editor) return;

      const handleContextMenu = (e: MouseEvent) => {
        e.preventDefault();
        setContextMenu({ visible: true, x: e.clientX, y: e.clientY });
      };

      const handleMouseDown = (e: MouseEvent) => {
        if (e.button === 2) {
          // 右键：显示菜单（部分 WebView 中 contextmenu 不触发，用 mousedown 兜底）
          e.preventDefault();
          setContextMenu({ visible: true, x: e.clientX, y: e.clientY });
        }
      };

      // 点击页面其他区域关闭菜单（右键点击用于打开菜单，不关闭）
      const handleDocumentMouseDown = (e: MouseEvent) => {
        if (e.button === 2) return;
        setContextMenu(prev => (prev.visible ? { ...prev, visible: false } : prev));
      };

      editorEl.addEventListener('contextmenu', handleContextMenu, true);
      editorEl.addEventListener('mousedown', handleMouseDown, true);
      document.addEventListener('mousedown', handleDocumentMouseDown);

      return () => {
        editorEl.removeEventListener('contextmenu', handleContextMenu, true);
        editorEl.removeEventListener('mousedown', handleMouseDown, true);
        document.removeEventListener('mousedown', handleDocumentMouseDown);
      };
    }, [editor]);

    // 同步外部内容变化
    useEffect(() => {
      if (editor && content !== editor.getHTML()) {
        editor.commands.setContent(content);
      }
    }, [content, editor]);

    // 修订模式：初始化/同步 prevTextRef
    useEffect(() => {
      if (editor) {
        prevTextRef.current = editor.getText();
      }
    }, [editor, isRevisionMode]);

    // 文本批注：选区变化时显示添加按钮
    useEffect(() => {
      if (!editor) return;

      const handleSelectionUpdate = () => {
        if (contextMenu.visible) return;
        const { selection } = editor.state;
        if (selection.empty) {
          setSelectedRange(null);
          return;
        }
        const text = editor.state.doc.textBetween(selection.from, selection.to, '\n');
        if (!text.trim()) {
          setSelectedRange(null);
          return;
        }
        
        // 获取选区在视口中的位置
        const domSel = window.getSelection();
        if (domSel && domSel.rangeCount > 0) {
          const rect = domSel.getRangeAt(0).getBoundingClientRect();
          const containerRect = containerRef.current?.getBoundingClientRect();
          if (containerRect) {
            setAnnotationPopupPos({
              x: rect.left - containerRect.left + rect.width / 2,
              y: rect.top - containerRect.top - 8,
            });
          }
        }
        
        setSelectedRange({ from: selection.from, to: selection.to, text: text.trim() });
      };

      editor.on('selectionUpdate', handleSelectionUpdate);
      return () => {
        editor.off('selectionUpdate', handleSelectionUpdate);
      };
    }, [editor]);

    // 文本批注：加载后应用高亮标记
    useEffect(() => {
      if (!editor || !textAnnotations.length || !chapterId) return;
      
      // 清除旧标记并重新应用
      editor.commands.unsetTextAnnotation();
      
      for (const annotation of textAnnotations) {
        const { from_pos, to_pos, annotation_type, id } = annotation;
        // 将纯文本位置映射回 ProseMirror 位置
        // 由于精确映射复杂，这里使用文本搜索策略在文档中查找
        const docText = editor.state.doc.textContent;
        // 优先使用原始位置附近的文本
        const searchText = docText.slice(from_pos, to_pos);
        if (!searchText) continue;
        
        let foundIndex = docText.indexOf(searchText);
        if (foundIndex === -1) continue;
        
        // 从文本偏移映射到 ProseMirror 位置
        // 使用 editor.state.doc.resolve 不太适合从字符偏移转换
        // 简单做法：通过 nodesBetween 找到对应文本节点
        let startPos = -1;
        let endPos = -1;
        let currentOffset = 0;
        
        editor.state.doc.descendants((node, pos) => {
          if (!node.isText) return;
          const text = node.text || '';
          const nodeStart = currentOffset;
          const nodeEnd = currentOffset + text.length;
          
          if (startPos === -1 && nodeEnd > foundIndex) {
            startPos = pos + (foundIndex - nodeStart);
          }
          if (endPos === -1 && nodeEnd >= foundIndex + searchText.length) {
            endPos = pos + (foundIndex + searchText.length - nodeStart);
          }
          currentOffset += text.length;
        });
        
        if (startPos !== -1 && endPos !== -1) {
          editor.commands.setTextAnnotation({ type: annotation_type, annotationId: id });
          editor.chain().focus().setTextSelection({ from: startPos, to: endPos }).setMark('textAnnotation', { type: annotation_type, annotationId: id }).setTextSelection(endPos).run();
        }
      }
    }, [editor, textAnnotations, chapterId]);

    // 处理角色名点击 - 自动扩展选区到完整词
    useEffect(() => {
      if (!editor || !containerRef.current || characters.length === 0) return;

      const editorElement = containerRef.current?.querySelector('.ProseMirror');
      if (!editorElement) return;

      const extractWordAtPoint = (node: Node, offset: number): string | null => {
        if (node.nodeType !== Node.TEXT_NODE) return null;
        const text = node.textContent || '';
        
        // 扩展选区到完整的中文词或英文单词
        let start = offset;
        let end = offset;
        
        // 向前扩展
        while (start > 0) {
          const char = text[start - 1];
          if (/[\s\n\r.,;:!?，。；：！？""''（）【】]/.test(char)) break;
          start--;
        }
        
        // 向后扩展
        while (end < text.length) {
          const char = text[end];
          if (/[\s\n\r.,;:!?，。；：！？""''（）【】]/.test(char)) break;
          end++;
        }
        
        return text.slice(start, end).trim();
      };

      const handleClick = (e: Event) => {
        const mouseEvent = e as MouseEvent;
        const target = mouseEvent.target as HTMLElement;
        const paragraph = target.tagName === 'P' ? target : target.closest('p');
        if (!paragraph) return;

        const selection = window.getSelection();
        if (!selection || selection.rangeCount === 0) return;

        const range = selection.getRangeAt(0);
        let word: string | null = null;

        if (selection.toString().trim()) {
          // 用户已有选区，直接使用
          word = selection.toString().trim();
        } else {
          // 自动提取点击位置附近的词
          const node = range.startContainer;
          const offset = range.startOffset;
          word = extractWordAtPoint(node, offset);
        }

        if (word) {
          const character = characters.find(c => c.name === word);
          if (character) {
            // 高亮选中该角色名
            if (!selection.toString().trim()) {
              try {
                const textNode = range.startContainer;
                const text = textNode.textContent || '';
                const index = text.indexOf(word);
                if (index >= 0 && textNode.nodeType === Node.TEXT_NODE) {
                  const newRange = document.createRange();
                  newRange.setStart(textNode, index);
                  newRange.setEnd(textNode, index + (word?.length || 0));
                  selection.removeAllRanges();
                  selection.addRange(newRange);
                }
              } catch {
                // 忽略选区设置失败
              }
            }

            const rect = paragraph.getBoundingClientRect();
            setPopupPosition({ x: rect.left, y: rect.bottom + 8 });
            setPopupAnchor(paragraph as HTMLElement);
            setSelectedCharacter(character);
            setShowPopup(true);
          }
        }
      };

      (editorElement as HTMLElement).addEventListener('click', handleClick);
      return () => (editorElement as HTMLElement).removeEventListener('click', handleClick);
    }, [editor, characters]);

    // ===== Input History Helpers =====
    const HISTORY_KEY = 'storyforge-chat-history';
    const MAX_HISTORY = 20;
    
    function loadInputHistory(): string[] {
      try {
        const saved = localStorage.getItem(HISTORY_KEY);
        if (saved) return JSON.parse(saved);
      } catch { /* ignore */ }
      return [];
    }
    
    function saveInputHistory(history: string[]) {
      try {
        localStorage.setItem(HISTORY_KEY, JSON.stringify(history.slice(0, MAX_HISTORY)));
      } catch { /* ignore */ }
    }
    
    function addToHistory(text: string) {
      if (!text.trim()) return;
      setInputHistory(prev => {
        const filtered = prev.filter(h => h !== text.trim());
        const next = [text.trim(), ...filtered].slice(0, MAX_HISTORY);
        saveInputHistory(next);
        return next;
      });
    }
    
    // ===== Slash Commands =====
    const slashCommands = [
      { id: 'continue', name: '/续写', description: '自动续写当前段落', instruction: '续写' },
      { id: 'revise', name: '/修订', description: '进入修订模式审阅文本', instruction: '修订当前段落' },
      { id: 'chapter', name: '/生成章节', description: '基于故事生成新章节', instruction: '生成新章节' },
      { id: 'scene', name: '/补充场景', description: '补充环境/动作描写', instruction: '补充场景描写' },
      { id: 'ancient', name: '/改写古风', description: '改写成古风文风', instruction: '改写成古风' },
      { id: 'polish', name: '/润色', description: '润色当前段落', instruction: '润色' },
    ];
    
    // ===== Ghost Text Generation =====
    useEffect(() => {
      if (showSlashMenu) {
        setGhostText('');
        return;
      }
      const input = chatInput.trim();
      if (!input) {
        setGhostText('输入指令，按 / 查看快捷命令');
        return;
      }
      const suggestions: Record<string, string> = {
        '续': '写当前段落',
        '改': '写当前段落',
        '润': '色当前段落',
        '扩': '展这段描写',
        '生': '成新章节',
        '补': '充场景描写',
      };
      for (const [prefix, suffix] of Object.entries(suggestions)) {
        if (input === prefix) {
          setGhostText(suffix);
          return;
        }
      }
      setGhostText('');
    }, [chatInput, showSlashMenu]);
    
    // 发送消息（正文助手指令栏）
    const executeWriterAgent = useCallback(async (instruction: string) => {
      if (isAiThinking) return;
      setLastInstruction(instruction);
      setIsAiThinking(true);

      try {
        const currentContent = editor?.getHTML() || '';
        const selectedText = (editor && !editor.state.selection.empty)
          ? editor.state.doc.textBetween(editor.state.selection.from, editor.state.selection.to, '\n')
          : undefined;

        const result = await writerAgentExecute({
          story_id: storyId || '',
          chapter_number: chapterNumber,
          current_content: currentContent,
          selected_text: selectedText,
          instruction,
        });

        onWriterResult?.(result.content);
        toast.success('正文助手已完成');
      } catch (error) {
        console.error('Writer agent error:', error);
        const msg = error instanceof Error ? error.message : String(error);
        toast.error(`正文助手调用失败：${msg}`);
      } finally {
        setIsAiThinking(false);
      }
    }, [isAiThinking, storyId, chapterNumber, editor, onWriterResult]);

    const handleSendMessage = useCallback(async () => {
      if (!chatInput.trim() || isAiThinking) return;
      const instruction = chatInput.trim();
      addToHistory(instruction);
      setHistoryIndex(-1);
      setChatInput('');
      setGhostText('');
      setShowSlashMenu(false);
      await executeWriterAgent(instruction);
    }, [chatInput, isAiThinking, executeWriterAgent]);

    const handleAcceptAndContinue = useCallback(() => {
      onAcceptGeneration?.();
      if (aiEnabled && !isZenMode) {
        setTimeout(() => {
          executeWriterAgent('续写');
        }, 300);
      }
    }, [onAcceptGeneration, aiEnabled, isZenMode, executeWriterAgent]);

    // 生成古典评点
    const handleGenerateCommentary = useCallback(async () => {
      if (!editor || !storyId) return;
      
      const text = editor.getText();
      if (!text.trim()) return;

      setIsGeneratingCommentary(true);
      try {
        const result = await generateParagraphCommentaries({
          story_id: storyId,
          story_title: '',
          genre: '',
          text,
        });
        
        const commentaries: ParagraphCommentary[] = JSON.parse(result);
        if (!commentaries.length) return;

        // 从后往前插入，避免位置偏移
        const paragraphs: { pos: number; nodeSize: number }[] = [];
        editor.state.doc.descendants((node, pos) => {
          if (node.type.name === 'paragraph') {
            paragraphs.push({ pos, nodeSize: node.nodeSize });
          }
        });

        const chain = editor.chain().focus();
        // 按 paragraph_index 从大到小排序，从后往前插入
        const sorted = [...commentaries]
          .filter(c => c.paragraph_index < paragraphs.length)
          .sort((a, b) => b.paragraph_index - a.paragraph_index);

        for (const c of sorted) {
          const para = paragraphs[c.paragraph_index];
          const insertPos = para.pos + para.nodeSize;
          chain.insertContentAt(insertPos, {
            type: 'paragraph',
            attrs: { class: 'commentary-paragraph' },
            content: [{ type: 'text', text: `【批】${c.commentary}` }],
          });
        }
        chain.run();
      } catch (error) {
        console.error('Commentary error:', error);
      } finally {
        setIsGeneratingCommentary(false);
      }
    }, [editor, storyId]);

    // 创建文本批注
    const handleCreateAnnotation = async () => {
      if (!selectedRange || !chapterId || !storyId) return;
      try {
        const fromPos = editor?.state.doc.textBetween(0, selectedRange.from).length ?? 0;
        const toPos = editor?.state.doc.textBetween(0, selectedRange.to).length ?? fromPos;
        
        await createAnnotationMutation.mutateAsync({
          story_id: storyId,
          chapter_id: chapterId,
          content: annotationContent,
          annotation_type: annotationType,
          from_pos: fromPos,
          to_pos: toPos,
        });
        
        // 应用临时高亮
        editor?.chain().focus().setTextSelection({ from: selectedRange.from, to: selectedRange.to }).setMark('textAnnotation', { type: annotationType, annotationId: 'temp' }).setTextSelection(selectedRange.to).run();
        
        setAnnotationContent('');
        setSelectedRange(null);
        toast.success('批注已添加');
      } catch (error) {
        console.error('Failed to create annotation:', error);
        toast.error('添加批注失败');
      }
    };

    const handleDeleteAnnotation = async (annotation: TextAnnotation) => {
      try {
        await deleteAnnotationMutation.mutateAsync(annotation.id);
        toast.success('批注已删除');
      } catch (error) {
        console.error('Failed to delete annotation:', error);
        toast.error('删除批注失败');
      }
    };

    const handleCreateCommentThread = async () => {
      if (!selectedRange || !chapterId) return;
      try {
        const fromPos = editor?.state.doc.textBetween(0, selectedRange.from).length ?? 0;
        const toPos = editor?.state.doc.textBetween(0, selectedRange.to).length ?? fromPos;
        
        const thread = await createCommentThreadMutation.mutateAsync({
          anchorType: 'TextRange',
          chapterId,
          fromPos,
          toPos,
          selectedText: selectedRange.text,
        });
        
        await addCommentMessageMutation.mutateAsync({
          threadId: thread.id,
          content: annotationContent,
          chapterId,
        });
        
        // 应用评论锚点高亮
        editor?.chain().focus().setTextSelection({ from: selectedRange.from, to: selectedRange.to }).setMark('commentAnchor', { threadId: thread.id }).setTextSelection(selectedRange.to).run();
        
        setAnnotationContent('');
        setSelectedRange(null);
        setPopupMode('annotation');
        toast.success('评论已添加');
      } catch (error) {
        console.error('Failed to create comment thread:', error);
        toast.error('添加评论失败');
      }
    };

    const handleDeleteCommentThread = async (threadId: string) => {
      try {
        await deleteCommentThreadMutation.mutateAsync({ threadId, chapterId });
        toast.success('评论已删除');
      } catch (error) {
        console.error('Failed to delete comment thread:', error);
        toast.error('删除评论失败');
      }
    };

    const handleResolveCommentThread = async (threadId: string) => {
      try {
        await resolveCommentThreadMutation.mutateAsync({ threadId, chapterId });
        toast.success('评论已解决');
      } catch (error) {
        console.error('Failed to resolve comment thread:', error);
        toast.error('操作失败');
      }
    };

    // 辅助函数：查找第一个不同字符的位置
    const findFirstDiff = (a: string, b: string): number => {
      let i = 0;
      while (i < a.length && i < b.length && a[i] === b[i]) i++;
      return i;
    };

    // 辅助函数：将纯文本偏移转换为 ProseMirror 位置
    const textOffsetToPmPosition = (editorInstance: any, offset: number, length: number) => {
      let currentOffset = 0;
      let startPos = -1;
      let endPos = -1;
      editorInstance.state.doc.descendants((node: any, pos: number) => {
        if (!node.isText) return;
        const text = node.text || '';
        const nodeStart = currentOffset;
        const nodeEnd = currentOffset + text.length;
        if (startPos === -1 && nodeEnd > offset) {
          startPos = pos + (offset - nodeStart);
        }
        if (endPos === -1 && nodeEnd >= offset + length) {
          endPos = pos + (offset + length - nodeStart);
        }
        currentOffset += text.length;
      });
      if (startPos !== -1 && endPos !== -1) {
        return { from: startPos, to: endPos };
      }
      return null;
    };

    // 获取状态图标
    const getStatusIcon = () => {
      switch (status) {
        case 'connected':
          return <CheckCircle2 className="w-3 h-3 text-green-500" />;
        case 'disconnected':
          return <AlertCircle className="w-3 h-3 text-red-500" />;
        case 'connecting':
          return <Loader2 className="w-3 h-3 text-yellow-500 animate-spin" />;
      }
    };

    // 键盘快捷键
    useEffect(() => {
      const handleKeyDown = (e: KeyboardEvent) => {
        if (isZenMode) return;

        // Accept AI suggestion
        if (e.key === 'Tab' && generatedText && handleAcceptAndContinue) {
          e.preventDefault();
          handleAcceptAndContinue();
          return;
        }

        // Reject AI suggestion
        if (e.key === 'Escape' && generatedText && onRejectGeneration) {
          e.preventDefault();
          onRejectGeneration();
          return;
        }
      };

      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
    }, [generatedText, handleAcceptAndContinue, onRejectGeneration, isZenMode]);

    // 当 generatedText 被清空（接受/拒绝）时，也清空 lastInstruction
    useEffect(() => {
      if (!generatedText) {
        setLastInstruction(null);
      }
    }, [generatedText]);

    // 暴露方法给父组件
    useImperativeHandle(ref, () => ({
      insertText: (text: string) => {
        if (editor) {
          if (selectedRange) {
            editor.chain().focus().setTextSelection({ from: selectedRange.from, to: selectedRange.to }).insertContent(text).run();
          } else {
            editor.chain().focus().insertContent(text).run();
          }
        }
      },
      getText: () => editor?.getText() || '',
      getSelectedText: () => {
        if (!editor) return '';
        const { from, to } = editor.state.selection;
        if (from === to) return '';
        return editor.state.doc.textBetween(from, to, '\n');
      },
      focus: () => editor?.commands.focus(),
      generateCommentary: () => {
        handleGenerateCommentary();
      },
    }), [editor, handleGenerateCommentary, selectedRange]);

    if (!editor) return null;

    // 获取当前风格与色调
    const currentStyle = defaultStyle;
    const themeColors = getCurrentEditorColors();

    // 生成CSS变量
    const styleVars = {
      '--fs-font-family': editorConfig.fontFamily,
      '--fs-font-size': externalFontSize ? `${externalFontSize}px` : `${editorConfig.fontSize}px`,
      '--fs-line-height': editorConfig.lineHeight,
      '--fs-letter-spacing': 'normal',
      '--fs-paragraph-spacing': '1.5em',
      '--fs-paper-color': themeColors.paperColor,
      '--fs-ink-color': themeColors.inkColor,
      '--fs-accent-color': themeColors.accentColor,
    } as React.CSSProperties;

    return (
      <div 
        ref={containerRef}
        className={cn(
          'rich-text-editor flex flex-col h-full relative',
          isZenMode && 'zen-mode',
          className
        )}
        style={styleVars}
        onContextMenu={(e) => {
          e.preventDefault();
          e.stopPropagation();
          setContextMenu({ visible: true, x: e.clientX, y: e.clientY });
        }}
      >
        {/* 编辑器内容区 */}
        <div className="flex-1 overflow-auto relative">
          {/* 修订模式横幅 */}
          {isRevisionMode && (
            <div className="absolute top-0 left-0 right-0 z-40 bg-blue-500/10 border-b border-blue-500/30 px-4 py-2 flex items-center justify-between backdrop-blur-sm">
              <div className="flex items-center gap-2 text-sm text-blue-400">
                <GitBranch className="w-4 h-4" />
                <span>修订模式已开启</span>
                <span className="text-xs text-blue-500/70">({pendingChanges.length} 处待审变更)</span>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => chapterId && acceptAllMutation.mutate({ chapterId })}
                  disabled={acceptAllMutation.isPending || pendingChanges.length === 0}
                  className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-blue-500/10 text-blue-400 text-xs hover:bg-blue-500/20 disabled:opacity-50 transition-colors"
                >
                  <CheckCheck className="w-3.5 h-3.5" />
                  全部接受
                </button>
                <button
                  onClick={() => chapterId && rejectAllMutation.mutate({ chapterId })}
                  disabled={rejectAllMutation.isPending || pendingChanges.length === 0}
                  className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-red-500/10 text-red-400 text-xs hover:bg-red-500/20 disabled:opacity-50 transition-colors"
                >
                  <Undo2 className="w-3.5 h-3.5" />
                  全部拒绝
                </button>
                <button
                  onClick={() => setIsRevisionMode(false)}
                  className="px-2.5 py-1 rounded-md bg-cinema-800 text-gray-300 text-xs hover:bg-cinema-700 transition-colors"
                >
                  退出
                </button>
              </div>
            </div>
          )}
          
          <EditorContent editor={editor} />
          
          {/* 文本批注/评论选区浮动按钮 */}
          {selectedRange && (
            <div
              className="absolute z-50 -translate-x-1/2 -translate-y-full"
              style={{ left: annotationPopupPos.x, top: annotationPopupPos.y }}
            >
              <div className="bg-cinema-900 border border-cinema-700 rounded-lg shadow-xl p-3 w-64 mb-2">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex gap-1">
                    <button
                      onClick={() => setPopupMode('annotation')}
                      className={cn(
                        'px-2 py-0.5 rounded text-[10px] font-medium transition-colors',
                        popupMode === 'annotation' ? 'bg-cinema-700 text-white' : 'text-gray-400 hover:text-white'
                      )}
                    >
                      批注
                    </button>
                    <button
                      onClick={() => setPopupMode('comment')}
                      className={cn(
                        'px-2 py-0.5 rounded text-[10px] font-medium transition-colors',
                        popupMode === 'comment' ? 'bg-cinema-700 text-white' : 'text-gray-400 hover:text-white'
                      )}
                    >
                      评论
                    </button>
                  </div>
                  <button onClick={() => setSelectedRange(null)} className="text-gray-500 hover:text-white">
                    <X className="w-3 h-3" />
                  </button>
                </div>
                {popupMode === 'annotation' ? (
                  <>
                    <div className="flex gap-1 mb-2">
                      {(['note', 'todo', 'warning', 'idea'] as const).map((t) => (
                        <button
                          key={t}
                          onClick={() => setAnnotationType(t)}
                          className={cn(
                            'px-2 py-0.5 rounded text-[10px] font-medium transition-colors',
                            annotationType === t ? 'bg-cinema-700 text-white' : 'bg-cinema-800 text-gray-400 hover:text-white'
                          )}
                        >
                          {TEXT_ANNOTATION_TYPE_LABELS[t]}
                        </button>
                      ))}
                    </div>
                    <textarea
                      value={annotationContent}
                      onChange={(e) => setAnnotationContent(e.target.value)}
                      placeholder="输入批注内容..."
                      className="w-full px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-xs text-white placeholder-gray-500 focus:border-cinema-gold focus:outline-none resize-none"
                      rows={2}
                    />
                    <button
                      onClick={handleCreateAnnotation}
                      disabled={!annotationContent.trim() || createAnnotationMutation.isPending}
                      className="w-full mt-2 py-1.5 rounded bg-cinema-gold/10 text-cinema-gold text-xs font-medium hover:bg-cinema-gold/20 disabled:opacity-50 transition-colors"
                    >
                      {createAnnotationMutation.isPending ? '保存中...' : '保存批注'}
                    </button>
                  </>
                ) : (
                  <>
                    <textarea
                      value={annotationContent}
                      onChange={(e) => setAnnotationContent(e.target.value)}
                      placeholder="输入评论内容..."
                      className="w-full px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-xs text-white placeholder-gray-500 focus:border-cinema-gold focus:outline-none resize-none"
                      rows={2}
                    />
                    <button
                      onClick={handleCreateCommentThread}
                      disabled={!annotationContent.trim() || createCommentThreadMutation.isPending}
                      className="w-full mt-2 py-1.5 rounded bg-yellow-500/10 text-yellow-400 text-xs font-medium hover:bg-yellow-500/20 disabled:opacity-50 transition-colors"
                    >
                      {createCommentThreadMutation.isPending ? '保存中...' : '发起评论'}
                    </button>
                  </>
                )}
              </div>
            </div>
          )}
        </div>

        {/* 底部对话栏 */}
        {!isZenMode && (
          <div 
            className={cn(
              'chat-toolbar absolute bottom-0 left-0 right-0',
              'bg-[var(--parchment)]/95 backdrop-blur-sm',
              'px-6 pb-5 pt-3',
              'border-t border-[var(--warm-sand)]',
              'transition-opacity duration-300 ease-out transition-transform duration-300 ease-out',
              'opacity-100 translate-y-0'
            )}
          >
            {/* AI 生成状态 / 预览 */}
            {isAiThinking && (
              <div className="mx-2 mb-3 p-4 bg-[var(--warm-sand)] rounded-xl relative overflow-hidden border border-[var(--terracotta)]/10">
                <div className="flex items-center gap-3 text-[var(--stone-gray)]">
                  <Loader2 className="w-5 h-5 animate-spin text-[var(--terracotta)]" />
                  <div>
                    <p className="text-sm font-medium text-[var(--charcoal)]">正文助手思考中...</p>
                    {lastInstruction && (
                      <p className="text-xs text-[var(--stone-gray)]/80 mt-0.5">「{lastInstruction}」</p>
                    )}
                  </div>
                </div>
              </div>
            )}

            {generatedText && (
              <div className="mx-2 mb-3 p-4 bg-[var(--parchment-dark)] rounded-xl relative overflow-hidden">
                <div className="absolute top-0 left-0 right-0 h-0.5 bg-[var(--terracotta)]/30" />
                <p className="text-sm text-[var(--stone-gray)] italic mb-2 flex items-center gap-1.5">
                  <Sparkles className="w-3.5 h-3.5 text-[var(--terracotta)]" />
                  AI 建议续写
                </p>
                <p className="text-[var(--charcoal)] leading-relaxed">{generatedText}</p>
                <div className="flex items-center gap-2 mt-3 text-sm">
                  <button
                    onClick={handleAcceptAndContinue}
                    className="px-3 py-1.5 bg-[var(--terracotta)] text-white rounded-lg hover:bg-[var(--terracotta-dark)] active:scale-95 transition-colors duration-200"
                  >
                    Tab 接受
                  </button>
                  <button
                    onClick={onRejectGeneration}
                    className="px-3 py-1.5 text-[var(--stone-gray)] hover:text-[var(--charcoal)] hover:bg-[var(--warm-sand)] rounded-lg active:scale-95 transition-colors duration-200"
                  >
                    Esc 拒绝
                  </button>
                </div>
              </div>
            )}

            {/* 批注面板 */}
            {showAnnotationPanel && (
              <div className="annotation-panel mb-3 max-h-40 overflow-y-auto bg-cinema-900/50 border border-cinema-800 rounded-xl p-3">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium text-white flex items-center gap-1.5">
                    <StickyNote className="w-4 h-4 text-cinema-gold" />
                    文本批注
                    <span className="text-xs text-gray-500 font-normal">({textAnnotations.length})</span>
                  </span>
                  <button onClick={() => setShowAnnotationPanel(false)} className="text-gray-500 hover:text-white">
                    <X className="w-4 h-4" />
                  </button>
                </div>
                {textAnnotations.length === 0 ? (
                  <p className="text-xs text-gray-500">暂无批注。在编辑器中选中文本即可添加。</p>
                ) : (
                  <div className="space-y-2">
                    {textAnnotations.map((annotation) => (
                      <div
                        key={annotation.id}
                        className={cn(
                          'text-xs p-2 rounded-lg border transition-colors',
                          hoveredAnnotationId === annotation.id ? 'bg-cinema-800 border-cinema-700' : 'bg-cinema-900 border-cinema-800'
                        )}
                        onMouseEnter={() => setHoveredAnnotationId(annotation.id)}
                        onMouseLeave={() => setHoveredAnnotationId(null)}
                      >
                        <div className="flex items-center justify-between gap-2 mb-1">
                          <span className={cn('px-1.5 py-0.5 rounded text-[10px] font-medium text-white', TEXT_ANNOTATION_TYPE_COLORS[annotation.annotation_type])}>
                            {TEXT_ANNOTATION_TYPE_LABELS[annotation.annotation_type]}
                          </span>
                          <button
                            onClick={() => handleDeleteAnnotation(annotation)}
                            className="text-gray-500 hover:text-red-400"
                          >
                            <Trash2 className="w-3 h-3" />
                          </button>
                        </div>
                        <p className="text-gray-300 line-clamp-2">{annotation.content}</p>
                        {annotation.resolved_at && (
                          <span className="flex items-center gap-1 text-[10px] text-green-500 mt-1">
                            <Check className="w-3 h-3" />
                            已解决
                          </span>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* 评论线程面板 */}
            {showCommentPanel && (
              <div className="comment-panel mb-3 max-h-48 overflow-y-auto bg-cinema-900/50 border border-cinema-800 rounded-xl p-3">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium text-white flex items-center gap-1.5">
                    <MessageSquarePlus className="w-4 h-4 text-yellow-400" />
                    评论线程
                    <span className="text-xs text-gray-500 font-normal">({commentThreads.length})</span>
                  </span>
                  <button onClick={() => setShowCommentPanel(false)} className="text-gray-500 hover:text-white">
                    <X className="w-4 h-4" />
                  </button>
                </div>
                {commentThreads.length === 0 ? (
                  <p className="text-xs text-gray-500">暂无评论。在编辑器中选中文本，切换到「评论」标签即可添加。</p>
                ) : (
                  <div className="space-y-3">
                    {commentThreads.map((item: CommentThreadWithMessages) => (
                      <div
                        key={item.thread.id}
                        className={cn(
                          'text-xs p-2 rounded-lg border transition-colors',
                          hoveredThreadId === item.thread.id ? 'bg-cinema-800 border-cinema-700' : 'bg-cinema-900 border-cinema-800'
                        )}
                        onMouseEnter={() => setHoveredThreadId(item.thread.id)}
                        onMouseLeave={() => setHoveredThreadId(null)}
                      >
                        <div className="flex items-center justify-between gap-2 mb-1">
                          <span className="text-[10px] text-gray-400">
                            {item.thread.selected_text ? `「${item.thread.selected_text.slice(0, 20)}${item.thread.selected_text.length > 20 ? '...' : ''}」` : '场景评论'}
                          </span>
                          <div className="flex items-center gap-1">
                            {item.thread.status === 'Open' ? (
                              <button
                                onClick={() => handleResolveCommentThread(item.thread.id)}
                                className="text-gray-500 hover:text-green-400"
                                title="解决"
                              >
                                <Check className="w-3 h-3" />
                              </button>
                            ) : (
                              <span className="text-[10px] text-green-500">已解决</span>
                            )}
                            <button
                              onClick={() => handleDeleteCommentThread(item.thread.id)}
                              className="text-gray-500 hover:text-red-400"
                              title="删除"
                            >
                              <Trash2 className="w-3 h-3" />
                            </button>
                          </div>
                        </div>
                        <div className="space-y-1.5">
                          {item.messages.map((msg) => (
                            <div key={msg.id} className="text-gray-300">
                              <span className="text-[10px] text-gray-500">{msg.author_id}</span>
                              <p className="leading-relaxed">{msg.content}</p>
                            </div>
                          ))}
                        </div>
                        {item.thread.status === 'Open' && (
                          <div className="flex gap-1.5 mt-2">
                            <input
                              type="text"
                              value={activeCommentThreadId === item.thread.id ? newCommentContent : ''}
                              onChange={(e) => {
                                setActiveCommentThreadId(item.thread.id);
                                setNewCommentContent(e.target.value);
                              }}
                              placeholder="回复..."
                              className="flex-1 px-2 py-1 bg-cinema-800 border border-cinema-700 rounded text-xs text-white placeholder-gray-500 focus:border-cinema-gold focus:outline-none"
                              onKeyDown={(e) => {
                                if (e.key === 'Enter' && newCommentContent.trim()) {
                                  addCommentMessageMutation.mutate({
                                    threadId: item.thread.id,
                                    content: newCommentContent,
                                    chapterId,
                                  });
                                  setNewCommentContent('');
                                }
                              }}
                            />
                            <button
                              onClick={() => {
                                if (newCommentContent.trim()) {
                                  addCommentMessageMutation.mutate({
                                    threadId: item.thread.id,
                                    content: newCommentContent,
                                    chapterId,
                                  });
                                  setNewCommentContent('');
                                }
                              }}
                              disabled={!newCommentContent.trim() || addCommentMessageMutation.isPending}
                              className="px-2 py-1 rounded bg-yellow-500/10 text-yellow-400 text-xs hover:bg-yellow-500/20 disabled:opacity-50 transition-colors"
                            >
                              回复
                            </button>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* 模型状态与输入框一体化设计 */}
            <div className="chat-input-wrapper">
              <div className="chat-input-container">
                {/* 左侧：模型状态 */}
                <div className="chat-input-left">
                  <div
                    className="model-status-wrapper relative"
                    onMouseEnter={() => setShowModelTooltip(true)}
                    onMouseLeave={() => setShowModelTooltip(false)}
                  >
                    <div className={cn(
                      'model-status-dot',
                      status === 'connected' && 'status-connected',
                      status === 'disconnected' && 'status-disconnected',
                      status === 'connecting' && 'status-connecting'
                    )} />

                    {/* 悬停提示 */}
                    {showModelTooltip && (
                      <div className="model-tooltip">
                        <div className="model-tooltip-header">
                          <span className="model-name">{currentModel.name}</span>
                          <span className={cn(
                            'model-status-text',
                            status === 'connected' && 'text-green-600',
                            status === 'disconnected' && 'text-red-500',
                            status === 'connecting' && 'text-yellow-500'
                          )}>
                            {status === 'connected' && '已连接'}
                            {status === 'disconnected' && '未连接'}
                            {status === 'connecting' && '连接中...'}
                          </span>
                        </div>
                        <div className="model-id">{currentModel.id}</div>
                        <div className="model-url">{currentModel.baseUrl}</div>
                      </div>
                    )}
                  </div>
                </div>

                {/* 中间：输入框 + Ghost Text + Slash Menu */}
                <div className="chat-input-middle relative">
                  {/* Ghost text overlay */}
                  {ghostText && !showSlashMenu && (
                    <div className="chat-ghost-text" aria-hidden="true">
                      <span style={{ opacity: 0 }}>{chatInput}</span>
                      <span className="ghost-suffix">{ghostText}</span>
                    </div>
                  )}
                  
                  <textarea
                    value={chatInput}
                    onChange={(e) => {
                      const val = e.target.value;
                      setChatInput(val);
                      setHistoryIndex(-1);
                      // Show slash menu when typing /
                      if (val === '/') {
                        setShowSlashMenu(true);
                        setSlashMenuIndex(0);
                      } else if (showSlashMenu && !val.startsWith('/')) {
                        setShowSlashMenu(false);
                      }
                    }}
                    onKeyDown={(e) => {
                      // Slash menu navigation
                      if (showSlashMenu) {
                        if (e.key === 'ArrowDown') {
                          e.preventDefault();
                          setSlashMenuIndex(i => (i + 1) % slashCommands.length);
                          return;
                        }
                        if (e.key === 'ArrowUp') {
                          e.preventDefault();
                          setSlashMenuIndex(i => (i - 1 + slashCommands.length) % slashCommands.length);
                          return;
                        }
                        if (e.key === 'Enter' || e.key === 'Tab') {
                          e.preventDefault();
                          const cmd = slashCommands[slashMenuIndex];
                          if (cmd) {
                            setChatInput(cmd.instruction);
                            setShowSlashMenu(false);
                            setGhostText('');
                          }
                          return;
                        }
                        if (e.key === 'Escape') {
                          e.preventDefault();
                          setShowSlashMenu(false);
                          return;
                        }
                      }
                      
                      // Ghost text accept with →
                      if (e.key === 'ArrowRight' && ghostText && !e.shiftKey && !showSlashMenu) {
                        e.preventDefault();
                        setChatInput(prev => prev + ghostText);
                        setGhostText('');
                        return;
                      }
                      
                      // History browsing with ↑↓
                      if (e.key === 'ArrowUp' && !e.shiftKey && !showSlashMenu) {
                        e.preventDefault();
                        if (historyIndex < inputHistory.length - 1) {
                          const newIndex = historyIndex + 1;
                          setHistoryIndex(newIndex);
                          setChatInput(inputHistory[newIndex] || '');
                        }
                        return;
                      }
                      if (e.key === 'ArrowDown' && !e.shiftKey && !showSlashMenu && historyIndex >= 0) {
                        e.preventDefault();
                        const newIndex = historyIndex - 1;
                        setHistoryIndex(newIndex);
                        setChatInput(newIndex >= 0 ? inputHistory[newIndex] : '');
                        return;
                      }
                      
                      // Send with Enter
                      if (e.key === 'Enter' && !e.shiftKey) {
                        e.preventDefault();
                        handleSendMessage();
                      }
                    }}
                    placeholder="输入指令，如：续写、改写成古风、扩展这段描写"
                    className="chat-textarea"
                    rows={1}
                    disabled={status === 'disconnected' || isAiThinking}
                  />
                  
                  {/* Slash command menu */}
                  {showSlashMenu && (
                    <div className="slash-command-menu">
                      {slashCommands.map((cmd, i) => (
                        <div
                          key={cmd.id}
                          className={cn(
                            'slash-command-item',
                            i === slashMenuIndex && 'active'
                          )}
                          onClick={() => {
                            setChatInput(cmd.instruction);
                            setShowSlashMenu(false);
                            setGhostText('');
                          }}
                        >
                          <span className="slash-cmd-name">{cmd.name}</span>
                          <span className="slash-cmd-desc">{cmd.description}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* 右侧：发送按钮 */}
                <div className="chat-input-right flex items-center gap-1.5">
                  <button
                    onClick={handleSendMessage}
                    disabled={!chatInput.trim() || isAiThinking || status === 'disconnected'}
                    className={cn(
                      'chat-send-btn',
                      chatInput.trim() && !isAiThinking && status === 'connected' && 'active'
                    )}
                  >
                    {isAiThinking ? (
                      <Loader2 className="w-4 h-4 animate-spin" />
                    ) : (
                      <Send className="w-4 h-4" />
                    )}
                  </button>
                </div>
              </div>

              {/* 提示文字 */}
              <div className="chat-hint">
                <span>Enter 发送 · Shift+Enter 换行 · → 接受建议 · ↑ 历史 · / 命令</span>
                {aiEnabled && !isZenMode && (
                  <span className="hint-wensi">
                    <Sparkles className="w-3 h-3" />
                    文思已开启
                  </span>
                )}
              </div>
            </div>
          </div>
        )}

        {/* 编辑器右键菜单 */}
        <EditorContextMenu
          visible={contextMenu.visible}
          x={contextMenu.x}
          y={contextMenu.y}
          onClose={() => setContextMenu({ visible: false, x: 0, y: 0 })}
          editor={editor}
          isRevisionMode={isRevisionMode}
          onToggleRevision={() => setIsRevisionMode(!isRevisionMode)}
          onOpenAnnotation={() => {
            setPopupMode('annotation');
            setShowAnnotationPanel(true);
            setContextMenu({ visible: false, x: 0, y: 0 });
          }}
          onOpenComment={() => {
            setPopupMode('comment');
            setShowCommentPanel(true);
            setContextMenu({ visible: false, x: 0, y: 0 });
          }}
          onGenerateCommentary={() => {
            handleGenerateCommentary();
            setContextMenu({ visible: false, x: 0, y: 0 });
          }}
          isGeneratingCommentary={isGeneratingCommentary}
          hasSelection={!!selectedRange}
        />

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
