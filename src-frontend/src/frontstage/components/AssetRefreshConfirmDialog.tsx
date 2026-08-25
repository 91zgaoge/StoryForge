/**
 * 按正文重写大纲：确认后才落库。取消废弃；重写再生成一轮。
 */
import React from 'react';
import { FilePenLine, X } from 'lucide-react';

export type AssetRefreshConfirmDraft = {
  instruction: string;
  storyId: string;
  sceneId?: string;
  overwriteManual?: boolean;
  storyOutline: string;
  sceneOutline: string;
  showStory: boolean;
  showScene: boolean;
};

export interface AssetRefreshConfirmDialogProps {
  draft: AssetRefreshConfirmDraft | null;
  rewriting?: boolean;
  saving?: boolean;
  onChange: (next: AssetRefreshConfirmDraft) => void;
  onConfirm: () => void;
  onCancel: () => void;
  onRewrite: () => void;
}

export const AssetRefreshConfirmDialog: React.FC<AssetRefreshConfirmDialogProps> = ({
  draft,
  rewriting = false,
  saving = false,
  onChange,
  onConfirm,
  onCancel,
  onRewrite,
}) => {
  if (!draft) return null;
  const busy = rewriting || saving;

  return (
    <div
      className="fixed inset-0 z-[10000] flex items-center justify-center"
      style={{ backgroundColor: 'rgba(0,0,0,0.55)' }}
      data-testid="asset-refresh-confirm"
      onClick={busy ? undefined : onCancel}
    >
      <div
        className="rounded-2xl p-6 max-w-2xl w-full mx-4 shadow-2xl relative flex flex-col"
        style={{
          background: 'var(--parchment, #f5f0e1)',
          border: '1px solid var(--warm-sand, #d4c5a9)',
          color: 'var(--charcoal, #2c2c2c)',
          maxHeight: '80vh',
        }}
        onClick={e => e.stopPropagation()}
      >
        <button
          className="absolute top-4 right-4 p-1 rounded-full opacity-70 hover:opacity-100 transition-opacity"
          onClick={onCancel}
          disabled={saving}
          aria-label="关闭"
        >
          <X size={18} />
        </button>

        <div className="flex items-center gap-3 mb-2">
          <div
            className="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center"
            style={{ background: 'rgba(91,140,90,0.12)' }}
          >
            <FilePenLine size={20} color="var(--accent, #5b8c5a)" />
          </div>
          <h3 className="text-lg font-bold" style={{ color: 'var(--charcoal, #2c2c2c)' }}>
            确认大纲
          </h3>
        </div>
        <p className="text-sm mb-4 opacity-70">确认后写入幕后；取消则废弃；重写会再生成一轮。</p>

        <div className="flex flex-col gap-3 mb-5 overflow-y-auto flex-1 min-h-0">
          {draft.showStory && (
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium">故事大纲</span>
              <textarea
                data-testid="asset-refresh-story-outline"
                className="rounded-lg p-3 text-sm leading-relaxed min-h-[120px] resize-y"
                style={{
                  background: 'rgba(255,255,255,0.55)',
                  border: '1px solid var(--warm-sand, #d4c5a9)',
                  color: 'inherit',
                }}
                value={draft.storyOutline}
                disabled={busy}
                onChange={e => onChange({ ...draft, storyOutline: e.target.value })}
              />
            </label>
          )}
          {draft.showScene && (
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium">场景大纲</span>
              <textarea
                data-testid="asset-refresh-scene-outline"
                className="rounded-lg p-3 text-sm leading-relaxed min-h-[120px] resize-y"
                style={{
                  background: 'rgba(255,255,255,0.55)',
                  border: '1px solid var(--warm-sand, #d4c5a9)',
                  color: 'inherit',
                }}
                value={draft.sceneOutline}
                disabled={busy}
                onChange={e => onChange({ ...draft, sceneOutline: e.target.value })}
              />
            </label>
          )}
          {rewriting && <p className="text-sm opacity-70">正在重写大纲…</p>}
        </div>

        <div className="flex justify-end gap-2">
          <button
            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            style={{
              background: 'transparent',
              border: '1px solid var(--warm-sand, #d4c5a9)',
            }}
            data-testid="asset-refresh-cancel"
            onClick={onCancel}
            disabled={saving}
          >
            取消
          </button>
          <button
            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            style={{
              background: 'rgba(91,140,90,0.12)',
              color: 'var(--accent, #5b8c5a)',
            }}
            onClick={onRewrite}
            disabled={busy}
          >
            重写
          </button>
          <button
            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors text-white"
            style={{ background: 'var(--accent, #5b8c5a)' }}
            onClick={onConfirm}
            disabled={busy}
          >
            {saving ? '保存中…' : '确认'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default AssetRefreshConfirmDialog;
