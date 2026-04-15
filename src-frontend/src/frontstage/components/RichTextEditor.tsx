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
  ChevronUp,
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
import { useModel } from '@/hooks/useModel';
import { useIntent } from '@/hooks/useIntent';
import { ChatMessage } from '@/services/modelService';
import { ModelConfig } from '@/config/models';
import type { IntentType, FeedbackType } from '@/types/index';
import type { ParagraphCommentary } from '@/types/v3';
import toast from 'react-hot-toast';
import { generateParagraphCommentaries } from '@/services/tauri';
import { TextAnnotationMark } from '@/frontstage/extensions/TextAnnotationMark';
import { TrackInsertMark, TrackDeleteMark } from '@/frontstage/extensions/TrackChanges';
import { CommentAnchorMark } from '@/frontstage/extensions/CommentAnchor';
import { useTextAnnotationsByChapter, useCreateTextAnnotation, useDeleteTextAnnotation, TEXT_ANNOTATION_TYPE_COLORS, TEXT_ANNOTATION_TYPE_LABELS } from '@/hooks/useTextAnnotations';
import { EditorContextMenu } from './EditorContextMenu';
import { usePendingChanges, useTrackChange, useAcceptChange, useRejectChange, useAcceptAllChanges, useRejectAllChanges } from '@/hooks/useChangeTracking';
import { useCommentThreads, useCreateCommentThread, useAddCommentMessage, useResolveCommentThread, useDeleteCommentThread } from '@/hooks/useCommentThreads';
import type { TextAnnotation, ChangeTrack, CommentThreadWithMessages } from '@/types/v3';

const INTENT_LABELS: Record<IntentType, string> = {
  text_generate: '续写生成',
  text_rewrite: '文本改写',
  plot_suggest: '情节建议',
  character_check: '角色检查',
  world_consistency: '世界观检查',
  style_shift: '文风切换',
  memory_ingest: '知识摄取',
  visual_generate: '视觉生成',
  scene_reorder: '场景调整',
  outline_expand: '大纲扩展',
  unknown: '自由对话',
};

const FEEDBACK_LABELS: Record<FeedbackType, string> = {
  direct_apply: '直接应用',
  suggestion_card: '建议卡片',
  diff_preview: '差异预览',
  system_notice: '系统通知',
  visual_highlight: '高亮提示',
};

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
    isRevisionMode: externalIsRevisionMode = false,
    onRevisionModeChange,
    showAnnotationPanel: externalShowAnnotationPanel = false,
    onShowAnnotationPanelChange,
    showCommentPanel: externalShowCommentPanel = false,
    onShowCommentPanelChange,
  }, ref) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const [editorConfig, setEditorConfig] = useState<EditorConfig>(loadEditorConfig());
    const [showToolbar, setShowToolbar] = useState(false);
    const [chatInput, setChatInput] = useState('');
    const [chatHistory, setChatHistory] = useState<Array<{type: 'user' | 'ai', content: string, intentLabel?: string}>>([]);
    const [isAiThinking, setIsAiThinking] = useState(false);
    const [isExpanded, setIsExpanded] = useState(false);
    const [showModelTooltip, setShowModelTooltip] = useState(false);
    const [streamingContent, setStreamingContent] = useState('');
    const [isGeneratingCommentary, setIsGeneratingCommentary] = useState(false);
    
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
    const { currentModel, status, chat } = useModel();
    // 使用意图解析Hook
    const { parseIntent, executeIntent, buildMessages, isParsing: isParsingIntent, isExecuting: isExecutingIntent } = useIntent();
    
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

    // 发送消息（意图感知）
    const handleSendMessage = useCallback(async () => {
      if (!chatInput.trim() || isAiThinking || isParsingIntent || isExecutingIntent) return;
      
      const userMessage = chatInput.trim();
      setChatHistory(prev => [...prev, { type: 'user', content: userMessage }]);
      setChatInput('');
      setIsAiThinking(true);
      setStreamingContent('');
      
      try {
        // Step 1: 解析意图
        const intent = await parseIntent(userMessage);
        const intentLabel = intent 
          ? `${INTENT_LABELS[intent.intent_type] || '自由对话'} · ${FEEDBACK_LABELS[intent.feedback_type] || '建议卡片'}`
          : undefined;
        
        // Step 2: 根据意图类型选择执行路径
        const shouldUseAgentExecution = intent && 
          !['unknown', 'text_generate', 'text_rewrite'].includes(intent.intent_type) &&
          storyId;
        
        if (shouldUseAgentExecution) {
          // Agent 调度执行路径
          const result = await executeIntent(intent, storyId);
          if (result && result.steps.length > 0) {
            const agentContent = result.steps
              .filter(s => s.success && s.result)
              .map(s => `【${s.agent_name}】\n${s.result!.content}`)
              .join('\n\n');
            const finalContent = agentContent || result.summary;
            setChatHistory(prev => [...prev, { type: 'ai', content: finalContent, intentLabel }]);
          } else {
            const fallbackMessages = buildMessages(intent, chatHistory, userMessage, editor?.getText());
            let aiResponse = '';
            await chat(fallbackMessages as ChatMessage[], {
              stream: true,
              onStream: (chunk) => {
                aiResponse += chunk;
                setStreamingContent(aiResponse);
              }
            });
            setChatHistory(prev => [...prev, { type: 'ai', content: aiResponse, intentLabel }]);
          }
        } else {
          // 直接对话流式输出路径
          const messages = intent
            ? buildMessages(intent, chatHistory, userMessage, editor?.getText())
            : [
                {
                  role: 'system',
                  content: '你是一位专业的写作助手，擅长帮助作者改进文章、提供创作灵感和续写建议。请用中文回答，语言要优美、富有文学性。'
                },
                ...chatHistory.map(h => ({
                  role: (h.type === 'user' ? 'user' : 'assistant') as 'user' | 'assistant',
                  content: h.content
                })),
                { role: 'user' as const, content: userMessage }
              ];

          let aiResponse = '';
          
          await chat(messages as ChatMessage[], {
            stream: true,
            onStream: (chunk) => {
              aiResponse += chunk;
              setStreamingContent(aiResponse);
            }
          });

          setChatHistory(prev => [...prev, { type: 'ai', content: aiResponse, intentLabel }]);
        }
      } catch (error) {
        console.error('Chat error:', error);
        setChatHistory(prev => [...prev, { 
          type: 'ai', 
          content: '抱歉，我暂时无法回应。请检查模型连接状态。' 
        }]);
      } finally {
        setIsAiThinking(false);
        setStreamingContent('');
      }
    }, [chatInput, chatHistory, chat, isAiThinking, isParsingIntent, isExecutingIntent, parseIntent, executeIntent, buildMessages, editor, storyId]);

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
      };

      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
    }, [generatedText, onAcceptGeneration, onRejectGeneration]);

    // 暴露方法给父组件
    useImperativeHandle(ref, () => ({
      insertText: (text: string) => {
        if (editor) {
          editor.chain().focus().insertContent(text).run();
        }
      },
      getText: () => editor?.getText() || '',
      focus: () => editor?.commands.focus(),
      generateCommentary: () => {
        handleGenerateCommentary();
      },
    }), [editor, handleGenerateCommentary]);

    if (!editor) return null;

    // 获取当前风格
    const currentStyle = defaultStyle;

    // 生成CSS变量
    const styleVars = {
      '--fs-font-family': editorConfig.fontFamily,
      '--fs-font-size': externalFontSize ? `${externalFontSize}px` : `${editorConfig.fontSize}px`,
      '--fs-line-height': editorConfig.lineHeight,
      '--fs-letter-spacing': 'normal',
      '--fs-paragraph-spacing': '1.5em',
      '--fs-paper-color': currentStyle.paperColor,
      '--fs-ink-color': currentStyle.inkColor,
      '--fs-accent-color': currentStyle.accentColor,
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
        onMouseEnter={() => setShowToolbar(true)}
        onMouseLeave={() => {
          if (!isExpanded && !chatInput) {
            setShowToolbar(false);
          }
        }}
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

        {/* AI 生成预览 */}
        {generatedText && (
          <div className="mx-8 my-4 p-4 bg-[var(--parchment-dark)] rounded-xl relative overflow-hidden">
            <div className="absolute top-0 left-0 right-0 h-0.5 bg-[var(--terracotta)]/30" />
            <p className="text-sm text-[var(--stone-gray)] italic mb-2 flex items-center gap-1.5">
              <Sparkles className="w-3.5 h-3.5 text-[var(--terracotta)]" />
              AI 建议续写
            </p>
            <p className="text-[var(--charcoal)] leading-relaxed">{generatedText}</p>
            <div className="flex items-center gap-2 mt-3 text-sm">
              <button
                onClick={onAcceptGeneration}
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

        {/* 底部对话栏 */}
        {!isZenMode && (
          <div 
            className={cn(
              'chat-toolbar absolute bottom-0 left-0 right-0',
              'bg-[var(--parchment)]/95 backdrop-blur-sm',
              'px-6 pb-5 pt-3',
              'border-t border-[var(--warm-sand)]',
              'transition-opacity duration-300 ease-out transition-transform duration-300 ease-out',
              showToolbar || isExpanded || chatInput ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-full pointer-events-none'
            )}
          >
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

            {/* 对话历史 - 仅在展开时显示 */}
            {isExpanded && (chatHistory.length > 0 || streamingContent) && (
              <div className="chat-history mb-3 max-h-40 overflow-y-auto space-y-2 px-1">
                {chatHistory.map((msg, idx) => (
                  <div 
                    key={idx} 
                    className={cn(
                      'chat-message text-sm p-2.5 rounded-2xl max-w-[85%]',
                      msg.type === 'user' 
                        ? 'bg-[var(--terracotta)]/10 ml-auto rounded-br-md' 
                        : 'bg-[var(--warm-sand)] mr-auto rounded-bl-md'
                    )}
                  >
                    {msg.type === 'ai' && msg.intentLabel && (
                      <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-[var(--terracotta)]/10 text-[var(--terracotta)] mb-1.5">
                        <Sparkles className="w-3 h-3" />
                        {msg.intentLabel}
                      </span>
                    )}
                    <p className="text-[var(--charcoal)] leading-relaxed">{msg.content}</p>
                  </div>
                ))}
                {streamingContent && (
                  <div className="chat-message text-sm p-2.5 bg-[var(--warm-sand)] rounded-2xl rounded-bl-md max-w-[85%] mr-auto">
                    <p className="text-[var(--charcoal)] leading-relaxed">{streamingContent}</p>
                  </div>
                )}
                {isParsingIntent && (
                  <div className="chat-message text-sm p-3 bg-[var(--warm-sand)] rounded-2xl rounded-bl-md max-w-[60%] mr-auto">
                    <div className="flex items-center gap-2 text-[var(--stone-gray)]">
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                      <span className="text-xs">解析意图中...</span>
                    </div>
                  </div>
                )}
                {isExecutingIntent && (
                  <div className="chat-message text-sm p-3 bg-[var(--warm-sand)] rounded-2xl rounded-bl-md max-w-[60%] mr-auto">
                    <div className="flex items-center gap-2 text-[var(--stone-gray)]">
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                      <span className="text-xs">Agent 执行中...</span>
                    </div>
                  </div>
                )}
                {isAiThinking && !streamingContent && !isParsingIntent && !isExecutingIntent && (
                  <div className="chat-message text-sm p-3 bg-[var(--warm-sand)] rounded-2xl rounded-bl-md max-w-[60%] mr-auto">
                    <div className="flex items-center gap-1.5">
                      <span className="w-1.5 h-1.5 bg-[var(--stone-gray)] rounded-full animate-bounce" />
                      <span className="w-1.5 h-1.5 bg-[var(--stone-gray)] rounded-full animate-bounce delay-100" />
                      <span className="w-1.5 h-1.5 bg-[var(--stone-gray)] rounded-full animate-bounce delay-200" />
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* 模型状态与输入框一体化设计 */}
            <div className="chat-input-wrapper">
              <div className={cn(
                'chat-input-container',
                isExpanded && 'expanded'
              )}>
                {/* 左侧：模型状态 + 展开按钮 */}
                <div className="chat-input-left">
                  <button
                    onClick={() => setIsExpanded(!isExpanded)}
                    className="chat-toggle-btn"
                    title={isExpanded ? '收起对话' : '展开对话'}
                  >
                    <ChevronUp className={cn('w-4 h-4 transition-transform duration-200', isExpanded && 'rotate-180')} />
                  </button>

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

                {/* 中间：输入框 */}
                <div className="chat-input-middle">
                  <textarea
                    value={chatInput}
                    onChange={(e) => setChatInput(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && !e.shiftKey) {
                        e.preventDefault();
                        handleSendMessage();
                      }
                    }}
                    onFocus={() => setIsExpanded(true)}
                    placeholder="在此驾驭智能文思"
                    className="chat-textarea"
                    rows={isExpanded ? 2 : 1}
                    disabled={status === 'disconnected' || isParsingIntent || isExecutingIntent}
                  />
                </div>

                {/* 右侧：发送按钮 */}
                <div className="chat-input-right flex items-center gap-1.5">
                  <button
                    onClick={handleSendMessage}
                    disabled={!chatInput.trim() || isAiThinking || isParsingIntent || isExecutingIntent || status === 'disconnected'}
                    className={cn(
                      'chat-send-btn',
                      chatInput.trim() && !isAiThinking && !isParsingIntent && !isExecutingIntent && status === 'connected' && 'active'
                    )}
                  >
                    {isAiThinking || isParsingIntent || isExecutingIntent ? (
                      <Loader2 className="w-4 h-4 animate-spin" />
                    ) : (
                      <Send className="w-4 h-4" />
                    )}
                  </button>
                </div>
              </div>

              {/* 提示文字 */}
              <div className="chat-hint">
                <span>Enter 发送 · Shift+Enter 换行</span>
                {aiEnabled && (
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
