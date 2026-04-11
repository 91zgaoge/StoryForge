/**
 * WritingStyleSwitcher - 写作风格切换组件
 * 
 * 在编辑器工具栏中提供风格切换下拉菜单
 */

import { useState, useRef, useEffect } from 'react';
import { Palette, Check, ChevronDown, Sparkles } from 'lucide-react';
import { cn } from '@/utils/cn';
import { WritingStyle, WritingStyleId } from '@/frontstage/config/writingStyles';

interface WritingStyleSwitcherProps {
  currentStyle: WritingStyle;
  availableStyles: WritingStyle[];
  onStyleChange: (styleId: WritingStyleId) => void;
}

export function WritingStyleSwitcher({
  currentStyle,
  availableStyles,
  onStyleChange,
}: WritingStyleSwitcherProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [showPreview, setShowPreview] = useState<WritingStyleId | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // 点击外部关闭
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleSelect = (styleId: WritingStyleId) => {
    onStyleChange(styleId);
    setIsOpen(false);
  };

  return (
    <div ref={containerRef} className="relative">
      {/* 触发按钮 */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          'flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm transition-colors',
          'hover:bg-[var(--warm-sand)] text-[var(--charcoal)]',
          isOpen && 'bg-[var(--warm-sand)]'
        )}
        title="切换写作风格"
      >
        <Palette className="w-4 h-4 text-[var(--terracotta)]" />
        <span className="hidden sm:inline">{currentStyle.name}</span>
        <ChevronDown className={cn('w-3 h-3 transition-transform', isOpen && 'rotate-180')} />
      </button>

      {/* 下拉菜单 */}
      {isOpen && (
        <div className="absolute right-0 top-full mt-2 w-72 bg-white rounded-xl shadow-xl border border-[var(--warm-sand)] z-50 overflow-hidden">
          {/* 头部 */}
          <div className="px-4 py-3 border-b border-[var(--warm-sand)] bg-[var(--parchment-dark)]/30">
            <h4 className="font-display font-semibold text-[var(--charcoal)] flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-[var(--terracotta)]" />
              选择写作风格
            </h4>
            <p className="text-xs text-[var(--stone-gray)] mt-1">
              不同的风格带来不同的写作体验
            </p>
          </div>

          {/* 风格列表 */}
          <div className="max-h-80 overflow-y-auto py-2">
            {availableStyles.map((style) => (
              <button
                key={style.id}
                onClick={() => handleSelect(style.id)}
                onMouseEnter={() => setShowPreview(style.id)}
                onMouseLeave={() => setShowPreview(null)}
                className={cn(
                  'w-full px-4 py-3 text-left transition-colors relative',
                  'hover:bg-[var(--parchment-dark)]/50',
                  currentStyle.id === style.id && 'bg-[var(--terracotta)]/5'
                )}
              >
                <div className="flex items-start gap-3">
                  {/* 选中标记 */}
                  <div className={cn(
                    'w-5 h-5 rounded-full border-2 flex items-center justify-center flex-shrink-0 mt-0.5',
                    currentStyle.id === style.id
                      ? 'border-[var(--terracotta)] bg-[var(--terracotta)]'
                      : 'border-[var(--warm-sand)]'
                  )}>
                    {currentStyle.id === style.id && (
                      <Check className="w-3 h-3 text-white" />
                    )}
                  </div>

                  {/* 风格信息 */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className={cn(
                        'font-medium text-[var(--charcoal)]',
                        currentStyle.id === style.id && 'text-[var(--terracotta)]'
                      )}>
                        {style.name}
                      </span>
                      {style.author && (
                        <span className="text-xs text-[var(--stone-gray)]">
                          · {style.author}
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-[var(--stone-gray)] mt-0.5">
                      {style.description}
                    </p>

                    {/* 预览文字 */}
                    {showPreview === style.id && (
                      <div
                        className="mt-2 p-2 rounded-lg text-xs leading-relaxed border-l-2 border-[var(--terracotta)]"
                        style={{
                          fontFamily: style.fontFamily,
                          backgroundColor: style.paperColor,
                          color: style.inkColor,
                        }}
                      >
                        {style.preview}
                      </div>
                    )}
                  </div>
                </div>
              </button>
            ))}
          </div>

          {/* 底部提示 */}
          <div className="px-4 py-2 border-t border-[var(--warm-sand)] bg-[var(--parchment-dark)]/20">
            <p className="text-xs text-[var(--stone-gray)] text-center">
              风格设置会自动保存
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
