/**
 * 文思泉涌面板 — 自动续写 & 自动修改控制
 *
 * 集成在 RichTextEditor 底部输入栏上方，提供：
 * - 自动续写：循环调用 WriterAgent，显示实时进度
 * - 自动修改：基于故事设定的全文/选中修改
 * - 配额状态显示
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Zap, Wand2, Play, Square, Loader2, Settings2, X } from 'lucide-react';
import { cn } from '@/utils/cn';
import { autoWrite, autoWriteCancel, autoRevise } from '@/services/tauri';
import { listen } from '@tauri-apps/api/event';
import toast from 'react-hot-toast';

export interface WenSiPanelProps {
  storyId?: string;
  chapterId?: string;
  isPro: boolean;
  quotaText: string;
  onShowUpgrade: (trigger: string) => void;
  hasAutoWriteQuota: (chars: number) => Promise<boolean>;
  hasAutoReviseQuota: (chars: number) => Promise<boolean>;
  editorContent?: string;
  selectedText?: string;
  onReviseResult?: (text: string) => void;
}

type PanelTab = 'none' | 'write' | 'revise';

export const WenSiPanel: React.FC<WenSiPanelProps> = ({
  storyId,
  chapterId,
  isPro,
  quotaText,
  onShowUpgrade,
  hasAutoWriteQuota,
  hasAutoReviseQuota,
  editorContent,
  selectedText,
  onReviseResult,
}) => {
  const [activeTab, setActiveTab] = useState<PanelTab>('none');

  // 自动续写状态
  const [isAutoWriting, setIsAutoWriting] = useState(false);
  const [autoWriteTaskId, setAutoWriteTaskId] = useState<string | null>(null);
  const [targetChars, setTargetChars] = useState(5000);
  const [charsPerLoop, setCharsPerLoop] = useState(1000);
  const [progress, setProgress] = useState({ current: 0, target: 0, percentage: 0, loop: 0 });

  // 自动修改状态
  const [isAutoRevising, setIsAutoRevising] = useState(false);
  const [reviseScope, setReviseScope] = useState<'full' | 'chapter' | 'selection'>('chapter');
  const [reviseType, setReviseType] = useState('comprehensive');

  const unlistenRef = useRef<(() => void) | null>(null);

  // 监听自动续写进度事件
  useEffect(() => {
    if (!autoWriteTaskId || !isAutoWriting) return;

    const setupListener = async () => {
      const unlisten = await listen<{
        task_id: string;
        current_chars: number;
        target_chars: number;
        percentage: number;
        current_loop: number;
        status: string;
      }>(`auto-write-progress-${autoWriteTaskId}`, (event) => {
        const p = event.payload;
        setProgress({
          current: p.current_chars,
          target: p.target_chars,
          percentage: p.percentage,
          loop: p.current_loop,
        });
      });
      unlistenRef.current = unlisten;
    };
    setupListener();

    return () => {
      unlistenRef.current?.();
    };
  }, [autoWriteTaskId, isAutoWriting]);

  // 监听完成事件
  useEffect(() => {
    if (!autoWriteTaskId) return;

    const setupComplete = async () => {
      const unlisten = await listen<{ status: string; current_chars: number }>(
        `auto-write-complete-${autoWriteTaskId}`,
        (event) => {
          setIsAutoWriting(false);
          setProgress(prev => ({ ...prev, percentage: 100 }));
          toast.success(`自动续写完成！共生成 ${event.payload.current_chars} 字`);
        }
      );
      return unlisten;
    };
    const unlistenPromise = setupComplete();

    const setupError = async () => {
      const unlisten = await listen<string>(
        `auto-write-error-${autoWriteTaskId}`,
        (event) => {
          setIsAutoWriting(false);
          toast.error(`自动续写出错：${event.payload}`);
        }
      );
      return unlisten;
    };
    const unlistenErrorPromise = setupError();

    return () => {
      unlistenPromise.then(u => u());
      unlistenErrorPromise.then(u => u());
    };
  }, [autoWriteTaskId]);

  const handleStartAutoWrite = useCallback(async () => {
    if (!storyId || !chapterId) {
      toast.error('请先选择一个章节');
      return;
    }
    const requested = Math.min(charsPerLoop, targetChars);
    const allowed = await hasAutoWriteQuota(requested);
    if (!allowed) {
      onShowUpgrade('自动续写配额已用完');
      return;
    }
    try {
      const result = await autoWrite({
        story_id: storyId,
        chapter_id: chapterId,
        target_chars: targetChars,
        chars_per_loop: charsPerLoop,
      });
      setAutoWriteTaskId(result.task_id);
      setIsAutoWriting(true);
      setProgress({ current: 0, target: targetChars, percentage: 0, loop: 0 });
      toast.success('自动续写已开始');
    } catch (err: any) {
      const msg = err?.message || String(err);
      if (msg.includes('配额') || msg.includes('次数已用完')) {
        onShowUpgrade('自动续写配额已用完');
      } else {
        toast.error(`启动失败：${msg}`);
      }
    }
  }, [storyId, chapterId, targetChars, charsPerLoop, hasAutoWriteQuota, onShowUpgrade]);

  const handleStopAutoWrite = useCallback(async () => {
    if (autoWriteTaskId) {
      await autoWriteCancel(autoWriteTaskId);
    }
    setIsAutoWriting(false);
    setAutoWriteTaskId(null);
    toast('自动续写已停止');
  }, [autoWriteTaskId]);

  const handleAutoRevise = useCallback(async () => {
    if (!storyId) {
      toast.error('请先选择一个故事');
      return;
    }
    const textLen = (selectedText || editorContent || '').length;
    const allowed = await hasAutoReviseQuota(textLen);
    if (!allowed) {
      onShowUpgrade('自动修改配额已用完');
      return;
    }
    setIsAutoRevising(true);
    try {
      const result = await autoRevise({
        story_id: storyId,
        chapter_id: chapterId || undefined,
        scope: reviseScope,
        selected_text: selectedText || undefined,
        revision_type: reviseType,
      });
      toast.success('自动修改完成！');
      onReviseResult?.(result.revised_text);
    } catch (err: any) {
      const msg = err?.message || String(err);
      if (msg.includes('配额') || msg.includes('次数已用完')) {
        onShowUpgrade('自动修改配额已用完');
      } else {
        toast.error(`修改失败：${msg}`);
      }
    } finally {
      setIsAutoRevising(false);
    }
  }, [storyId, chapterId, reviseScope, reviseType, selectedText, editorContent, hasAutoReviseQuota, onShowUpgrade, onReviseResult]);

  const maxCharsPerCall = isPro ? 999999 : 1000;

  return (
    <div className="wensi-panel">
      {/* 顶部工具栏 */}
      <div className="wensi-toolbar">
        <div className="wensi-tabs">
          <button
            onClick={() => setActiveTab(activeTab === 'write' ? 'none' : 'write')}
            className={cn(
              'wensi-tab',
              activeTab === 'write' && 'wensi-tab-active',
              isAutoWriting && 'wensi-tab-running'
            )}
            disabled={isAutoWriting}
          >
            <Zap className="w-3.5 h-3.5" />
            <span>自动续写</span>
            {isAutoWriting && <Loader2 className="w-3 h-3 animate-spin" />}
          </button>
          <button
            onClick={() => setActiveTab(activeTab === 'revise' ? 'none' : 'revise')}
            className={cn(
              'wensi-tab',
              activeTab === 'revise' && 'wensi-tab-active',
              isAutoRevising && 'wensi-tab-running'
            )}
            disabled={isAutoRevising}
          >
            <Wand2 className="w-3.5 h-3.5" />
            <span>自动修改</span>
            {isAutoRevising && <Loader2 className="w-3 h-3 animate-spin" />}
          </button>
        </div>
        <div className="wensi-quota" title="今日配额">
          <span className="wensi-quota-text">{quotaText}</span>
        </div>
      </div>

      {/* 自动续写设置面板 */}
      {activeTab === 'write' && (
        <div className="wensi-section">
          <div className="wensi-row">
            <label className="wensi-label">目标字数</label>
            <input
              type="number"
              value={targetChars}
              onChange={(e) => setTargetChars(Math.max(100, Math.min(500000, Number(e.target.value))))}
              className="wensi-input"
              min={100}
              max={500000}
              step={100}
              disabled={isAutoWriting}
            />
            <label className="wensi-label">每次</label>
            <input
              type="number"
              value={charsPerLoop}
              onChange={(e) => {
                const v = Math.max(100, Math.min(maxCharsPerCall, Number(e.target.value)));
                setCharsPerLoop(v);
              }}
              className="wensi-input"
              min={100}
              max={maxCharsPerCall}
              step={100}
              disabled={isAutoWriting}
            />
            <span className="wensi-unit">字</span>
            {!isPro && charsPerLoop > 1000 && (
              <span className="wensi-hint">免费版每次最多 1000 字</span>
            )}
          </div>

          {/* 进度条 */}
          {isAutoWriting && (
            <div className="wensi-progress-area">
              <div className="wensi-progress-bar-bg">
                <div
                  className="wensi-progress-bar-fill"
                  style={{ width: `${progress.percentage}%` }}
                />
              </div>
              <div className="wensi-progress-text">
                {progress.percentage}% · {progress.current}/{progress.target} 字 · 第 {progress.loop} 轮
              </div>
            </div>
          )}

          <div className="wensi-actions">
            {!isAutoWriting ? (
              <button onClick={handleStartAutoWrite} className="wensi-btn-primary">
                <Play className="w-3.5 h-3.5" />
                开始续写
              </button>
            ) : (
              <button onClick={handleStopAutoWrite} className="wensi-btn-danger">
                <Square className="w-3.5 h-3.5" />
                停止续写
              </button>
            )}
          </div>
        </div>
      )}

      {/* 自动修改设置面板 */}
      {activeTab === 'revise' && (
        <div className="wensi-section">
          <div className="wensi-row">
            <label className="wensi-label">范围</label>
            <select
              value={reviseScope}
              onChange={(e) => setReviseScope(e.target.value as any)}
              className="wensi-select"
              disabled={isAutoRevising}
            >
              <option value="chapter">当前章节</option>
              <option value="selection">选中部分</option>
              <option value="full">全文</option>
            </select>
            <label className="wensi-label">类型</label>
            <select
              value={reviseType}
              onChange={(e) => setReviseType(e.target.value)}
              className="wensi-select"
              disabled={isAutoRevising}
            >
              <option value="comprehensive">综合修改</option>
              <option value="style">优化文风</option>
              <option value="plot">强化情节</option>
              <option value="dialogue">生动对话</option>
              <option value="description">感官描写</option>
            </select>
          </div>
          <div className="wensi-actions">
            <button
              onClick={handleAutoRevise}
              disabled={isAutoRevising}
              className="wensi-btn-primary"
            >
              {isAutoRevising ? (
                <><Loader2 className="w-3.5 h-3.5 animate-spin" /> 修改中...</>
              ) : (
                <><Wand2 className="w-3.5 h-3.5" /> 开始修改</>
              )}
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default WenSiPanel;
