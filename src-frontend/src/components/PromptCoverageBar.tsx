/**
 * v0.26.40: 展示 WriteTimeBundle 资产→prompt 覆盖率。
 * 数据来自 generation_traces 中 name=prompt_coverage 步骤的 details。
 */

import { AiContextCards } from '@/components/ui/ai/AiContextCards';

export type PromptCoverageDetails = {
  contract_redlines?: boolean;
  runtime_contract?: boolean;
  core_characters?: boolean;
  scene_outline?: boolean;
  pending_foreshadowings?: boolean;
  overdue_foreshadowings?: boolean;
  genre_antipatterns?: boolean;
  style_slice?: boolean;
  style_dna_summary?: boolean;
  methodology_extension?: boolean;
  related_entity_summaries?: boolean;
  reference_scene_fewshots?: boolean;
  filled_slots?: number;
  total_slots?: number;
};

const SLOT_LABELS: { key: keyof PromptCoverageDetails; label: string }[] = [
  { key: 'contract_redlines', label: '合同红线' },
  { key: 'runtime_contract', label: '运行时合同' },
  { key: 'core_characters', label: '角色' },
  { key: 'scene_outline', label: '场景大纲' },
  { key: 'pending_foreshadowings', label: '伏笔' },
  { key: 'genre_antipatterns', label: '体裁反模式' },
  { key: 'style_dna_summary', label: '风格' },
  { key: 'methodology_extension', label: '方法论' },
  { key: 'related_entity_summaries', label: 'KG摘要' },
  { key: 'reference_scene_fewshots', label: '拆书参考' },
];

function isFilled(details: PromptCoverageDetails, key: keyof PromptCoverageDetails): boolean {
  if (key === 'pending_foreshadowings') {
    return !!(details.pending_foreshadowings || details.overdue_foreshadowings);
  }
  if (key === 'style_dna_summary') {
    return !!(details.style_dna_summary || details.style_slice);
  }
  return !!details[key];
}

export function PromptCoverageBar({ details }: { details: PromptCoverageDetails }) {
  const total = details.total_slots ?? 10;
  const filled = details.filled_slots ?? SLOT_LABELS.filter(s => isFilled(details, s.key)).length;
  const pct = total > 0 ? Math.round((filled / total) * 100) : 0;

  return (
    <div className="mt-2 space-y-2" data-testid="prompt-coverage-bar">
      <div className="flex items-center justify-between text-xs text-gray-400">
        <span>资产→prompt 覆盖率</span>
        <span className="text-cinema-gold">
          {filled}/{total}（{pct}%）
        </span>
      </div>
      <div className="h-1.5 rounded-full bg-cinema-800 overflow-hidden">
        <div
          className="h-full rounded-full bg-cinema-gold/80 transition-all"
          style={{ width: `${pct}%` }}
        />
      </div>
      <AiContextCards
        title="上下文槽位"
        count={filled}
        items={SLOT_LABELS.map(slot => {
          const on = isFilled(details, slot.key);
          return {
            key: slot.key,
            title: slot.label,
            source: {
              label: on ? '已注入 prompt' : '未注入',
              badge: on ? '✓' : '✗',
              tone: on ? ('green' as const) : ('neutral' as const),
            },
          };
        })}
      />
    </div>
  );
}

export function extractPromptCoverage(
  steps: { name: string; details?: unknown }[]
): PromptCoverageDetails | null {
  const step = steps.find(s => s.name === 'prompt_coverage');
  if (!step?.details || typeof step.details !== 'object') return null;
  return step.details as PromptCoverageDetails;
}
