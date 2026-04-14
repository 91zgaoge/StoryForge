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
  Loader2
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
    aiEnabled = false,
    generatedText = '',
    onAcceptGeneration,
    onRejectGeneration,
    fontSize: externalFontSize,
    onFontSizeChange,
    isZenMode = false,
    onZenModeChange,
    storyId,
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
    
    // 使用模型管理Hook
    const { currentModel, status, chat } = useModel();
    // 使用意图解析Hook
    const { parseIntent, executeIntent, buildMessages, isParsing: isParsingIntent, isExecuting: isExecutingIntent } = useIntent();
    
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
    }), [editor]);

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
      >
        {/* 编辑器内容区 */}
        <div className="flex-1 overflow-auto">
          <EditorContent editor={editor} />
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
                <div className="chat-input-right">
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
