import { useState } from 'react';
import {
  BookOpen,
  Upload,
  Trash2,
  Loader2,
  CheckCircle2,
  AlertCircle,
  Square,
  Save,
  Plus,
} from 'lucide-react';
import toast from 'react-hot-toast';
import {
  useGuidebooks,
  useUploadGuidebook,
  useDeleteGuidebook,
  useGuidebookDistillationStatus,
  useCancelGuidebookDistillation,
  useGuidebookResult,
  useUpdateCustomMethodology,
  useDeleteCustomMethodology,
} from '@/hooks/useGuidebookDistillation';
import type {
  CustomMethodology,
  GuidebookListItem,
  MethodologyStep,
} from '@/types/guidebook-distillation';
import { cn } from '@/utils/cn';
import { extractMessage } from '@/utils/errorHandler';

const ACTIVE_STATUSES = ['pending', 'extracting', 'distilling', 'merging'];

const STATUS_LABELS: Record<string, string> = {
  pending: '等待中',
  extracting: '提取中',
  distilling: '提炼中',
  merging: '合并中',
  completed: '已完成',
  failed: '失败',
  cancelled: '已取消',
};

function formatWordCount(count?: number | null) {
  if (!count) return '';
  if (count >= 10000) return `${(count / 10000).toFixed(1)}万字`;
  return `${count}字`;
}

interface GuidebookCardProps {
  guidebook: GuidebookListItem;
  selected: boolean;
  onSelect: () => void;
  onDelete: () => void;
}

function GuidebookCard({ guidebook, selected, onSelect, onDelete }: GuidebookCardProps) {
  const isActive = ACTIVE_STATUSES.includes(guidebook.status);
  const liveStatus = useGuidebookDistillationStatus(isActive ? guidebook.id : null);
  const cancelMutation = useCancelGuidebookDistillation();

  const status = liveStatus?.status ?? guidebook.status;
  const progress = liveStatus?.progress ?? guidebook.progress;
  const currentStep = liveStatus?.current_step;

  const handleCancel = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!confirm('确定要取消当前提炼吗？已处理的部分不会被保存。')) return;
    try {
      await cancelMutation.mutateAsync(guidebook.id);
      toast.success('提炼已取消');
    } catch (error) {
      toast.error(`取消失败: ${extractMessage(error)}`);
    }
  };

  return (
    <div
      onClick={onSelect}
      className={cn(
        'p-4 rounded-xl cursor-pointer transition-all border',
        selected
          ? 'bg-cinema-gold/10 border-cinema-gold/30'
          : 'bg-cinema-900 border-cinema-800 hover:border-cinema-700'
      )}
    >
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-lg bg-cinema-800 flex items-center justify-center flex-shrink-0">
          <BookOpen className="w-5 h-5 text-cinema-gold" />
        </div>
        <div className="flex-1 min-w-0">
          <h4 className="text-sm font-medium text-white truncate">{guidebook.title}</h4>
          <div className="flex items-center gap-2 mt-1">
            <span className="text-xs text-gray-500">{guidebook.author || '未知作者'}</span>
            {guidebook.word_count && (
              <span className="text-xs text-gray-600">{formatWordCount(guidebook.word_count)}</span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          {status === 'completed' && <CheckCircle2 className="w-4 h-4 text-green-500" />}
          {status === 'failed' && <AlertCircle className="w-4 h-4 text-red-500" />}
          {status === 'cancelled' && <AlertCircle className="w-4 h-4 text-orange-500" />}
          {ACTIVE_STATUSES.includes(status) && (
            <Loader2 className="w-4 h-4 text-cinema-gold animate-spin" />
          )}
          <span className="text-xs text-gray-500">{STATUS_LABELS[status] || status}</span>
          {!ACTIVE_STATUSES.includes(status) && (
            <button
              onClick={e => {
                e.stopPropagation();
                onDelete();
              }}
              className="p-1.5 rounded-lg hover:bg-red-500/10 text-gray-500 hover:text-red-400 transition-colors"
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>

      {ACTIVE_STATUSES.includes(status) && (
        <div className="mt-3">
          <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
            <span>{currentStep || '正在提炼...'}</span>
            <span className="font-mono">{progress}%</span>
          </div>
          <div className="w-full h-2 bg-cinema-800 rounded-full overflow-hidden">
            <div
              className="h-full transition-all duration-500 rounded-full bg-gradient-to-r from-cinema-gold to-cinema-gold-dark"
              style={{ width: `${progress}%` }}
            />
          </div>
          <button
            onClick={handleCancel}
            disabled={cancelMutation.isPending}
            className="mt-2 flex items-center gap-1.5 px-3 py-1 rounded-lg border border-red-500/30 text-red-400 hover:bg-red-500/10 transition-colors text-xs disabled:opacity-50"
          >
            <Square className="w-3 h-3" />
            {cancelMutation.isPending ? '正在取消...' : '取消提炼'}
          </button>
        </div>
      )}
    </div>
  );
}

interface MethodologyEditorProps {
  methodology: CustomMethodology;
}

function MethodologyEditor({ methodology }: MethodologyEditorProps) {
  const [name, setName] = useState(methodology.name);
  const [description, setDescription] = useState(methodology.description ?? '');
  const [enabled, setEnabled] = useState(methodology.enabled);
  const [steps, setSteps] = useState<
    Array<{ title: string; instruction: string; checklist: string }>
  >(
    methodology.steps.map(s => ({
      title: s.title,
      instruction: s.instruction,
      checklist: s.checklist.join('\n'),
    }))
  );

  const updateMutation = useUpdateCustomMethodology();
  const deleteMutation = useDeleteCustomMethodology();

  const handleSave = async () => {
    const payload: MethodologyStep[] = steps.map(s => ({
      title: s.title,
      instruction: s.instruction,
      checklist: s.checklist
        .split('\n')
        .map(line => line.trim())
        .filter(line => line.length > 0),
    }));
    try {
      await updateMutation.mutateAsync({
        id: methodology.id,
        name,
        description,
        steps: payload,
        enabled,
      });
      toast.success('方法论已保存');
    } catch (error) {
      toast.error(`保存失败: ${extractMessage(error)}`);
    }
  };

  const handleDelete = async () => {
    if (!confirm('确定要删除该方法论吗？引用它的故事将恢复为无方法论。')) return;
    try {
      await deleteMutation.mutateAsync(methodology.id);
      toast.success('方法论已删除');
    } catch (error) {
      toast.error(`删除失败: ${extractMessage(error)}`);
    }
  };

  const updateStep = (idx: number, patch: Partial<(typeof steps)[number]>) => {
    setSteps(prev => prev.map((s, i) => (i === idx ? { ...s, ...patch } : s)));
  };

  return (
    <div className="mt-4 p-4 rounded-xl bg-cinema-900 border border-cinema-800 space-y-4">
      <h3 className="text-sm font-medium text-white">提炼出的方法论</h3>

      <div>
        <label className="block text-xs text-gray-400 mb-1">名称</label>
        <input
          type="text"
          value={name}
          onChange={e => setName(e.target.value)}
          className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
        />
      </div>

      <div>
        <label className="block text-xs text-gray-400 mb-1">描述</label>
        <textarea
          value={description}
          onChange={e => setDescription(e.target.value)}
          rows={2}
          className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-y"
        />
      </div>

      <div className="space-y-3">
        <label className="block text-xs text-gray-400">步骤（{steps.length}）</label>
        {steps.map((step, idx) => (
          <div
            key={idx}
            className="p-3 rounded-lg bg-cinema-800/50 border border-cinema-700 space-y-2"
          >
            <div className="flex items-center gap-2">
              <span className="text-xs text-gray-500 w-12 shrink-0">第 {idx + 1} 步</span>
              <input
                type="text"
                value={step.title}
                onChange={e => updateStep(idx, { title: e.target.value })}
                placeholder="步骤标题"
                className="flex-1 px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-white text-sm focus:border-cinema-gold focus:outline-none"
              />
              <button
                onClick={() => setSteps(prev => prev.filter((_, i) => i !== idx))}
                className="p-1.5 rounded hover:bg-red-500/10 text-gray-500 hover:text-red-400 transition-colors"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
            <textarea
              value={step.instruction}
              onChange={e => updateStep(idx, { instruction: e.target.value })}
              placeholder="步骤指令"
              rows={2}
              className="w-full px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-white text-sm focus:border-cinema-gold focus:outline-none resize-y"
            />
            <textarea
              value={step.checklist}
              onChange={e => updateStep(idx, { checklist: e.target.value })}
              placeholder="检查清单（一行一条）"
              rows={2}
              className="w-full px-2 py-1.5 bg-cinema-800 border border-cinema-700 rounded text-white text-xs font-mono focus:border-cinema-gold focus:outline-none resize-y"
            />
          </div>
        ))}
        <button
          onClick={() => setSteps(prev => [...prev, { title: '', instruction: '', checklist: '' }])}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-cinema-700 text-gray-400 hover:text-white hover:border-cinema-600 transition-colors text-xs"
        >
          <Plus className="w-3.5 h-3.5" />
          添加步骤
        </button>
      </div>

      <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
        <input
          type="checkbox"
          checked={enabled}
          onChange={e => setEnabled(e.target.checked)}
          className="rounded border-cinema-700 bg-cinema-800"
        />
        启用该方法论（启用后可在故事设置中选用）
      </label>

      <div className="flex items-center justify-end gap-3 pt-3 border-t border-cinema-800">
        <button
          onClick={handleDelete}
          disabled={deleteMutation.isPending}
          className="flex items-center gap-1.5 px-3 py-2 rounded-lg border border-red-500/30 text-red-400 hover:bg-red-500/10 transition-colors text-sm disabled:opacity-50"
        >
          <Trash2 className="w-3.5 h-3.5" />
          删除方法论
        </button>
        <button
          onClick={handleSave}
          disabled={updateMutation.isPending}
          className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-cinema-gold/20 text-cinema-gold hover:bg-cinema-gold/30 transition-colors text-sm disabled:opacity-50"
        >
          {updateMutation.isPending ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <Save className="w-3.5 h-3.5" />
          )}
          保存
        </button>
      </div>
    </div>
  );
}

function GuidebookResultView({ guidebookId }: { guidebookId: string }) {
  const { data, isLoading } = useGuidebookResult(guidebookId);

  if (isLoading) {
    return <div className="mt-4 text-center text-gray-500 text-sm">加载结果中...</div>;
  }

  if (!data?.methodology) {
    return <div className="mt-4 text-center text-gray-500 text-sm">该指导书尚未提炼出方法论</div>;
  }

  return <MethodologyEditor key={data.methodology.id} methodology={data.methodology} />;
}

export function GuidebookDistillationPanel() {
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const { data: guidebooks, isLoading } = useGuidebooks();
  const uploadMutation = useUploadGuidebook();
  const deleteMutation = useDeleteGuidebook();

  const handleUpload = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{ name: '指导书', extensions: ['txt', 'pdf', 'epub'] }],
      });
      if (selected && typeof selected === 'string') {
        const guidebookId = await uploadMutation.mutateAsync(selected);
        setSelectedId(guidebookId);
        toast.success('上传成功，开始提炼...');
      }
    } catch (error) {
      toast.error(`上传失败: ${extractMessage(error)}`);
    }
  };

  const handleDelete = async (guidebookId: string) => {
    if (!confirm('确定要删除这本指导书吗？提炼出的方法论会保留但失去来源关联。')) return;
    try {
      await deleteMutation.mutateAsync(guidebookId);
      if (selectedId === guidebookId) {
        setSelectedId(null);
      }
      toast.success('删除成功');
    } catch (error) {
      toast.error(`删除失败: ${extractMessage(error)}`);
    }
  };

  return (
    <div className="p-6 overflow-auto h-full">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-lg font-bold text-white">指导书提炼</h2>
          <p className="text-sm text-gray-500 mt-1">
            上传故事创作指导书（txt/pdf/epub），自动提炼为可调用的创作方法论
          </p>
        </div>
        <button
          onClick={handleUpload}
          disabled={uploadMutation.isPending}
          className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-cinema-gold/20 text-cinema-gold text-sm hover:bg-cinema-gold/30 transition-colors disabled:opacity-50"
        >
          {uploadMutation.isPending ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Upload className="w-4 h-4" />
          )}
          上传指导书
        </button>
      </div>

      {isLoading ? (
        <div className="text-center py-12 text-gray-500">加载中...</div>
      ) : guidebooks && guidebooks.length > 0 ? (
        <div className="grid grid-cols-1 gap-3 max-w-3xl">
          {guidebooks.map(guidebook => (
            <div key={guidebook.id}>
              <GuidebookCard
                guidebook={guidebook}
                selected={selectedId === guidebook.id}
                onSelect={() => setSelectedId(selectedId === guidebook.id ? null : guidebook.id)}
                onDelete={() => handleDelete(guidebook.id)}
              />
              {selectedId === guidebook.id &&
                guidebook.status === 'completed' &&
                guidebook.methodology_id && <GuidebookResultView guidebookId={guidebook.id} />}
            </div>
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-16 text-gray-500">
          <div className="w-16 h-16 rounded-full bg-cinema-800 flex items-center justify-center mb-4">
            <BookOpen className="w-8 h-8" />
          </div>
          <p className="text-sm">暂无指导书，点击右上角上传</p>
        </div>
      )}
    </div>
  );
}
