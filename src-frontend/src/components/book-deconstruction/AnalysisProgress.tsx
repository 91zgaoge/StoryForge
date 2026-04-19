import { Loader2 } from 'lucide-react';

interface AnalysisProgressProps {
  progress: number;
  currentStep: string;
}

const STEPS = [
  '正在提取文本信息...',
  '正在识别小说类型...',
  '正在分析世界观设定...',
  '正在拆解人物角色...',
  '正在生成章节概要...',
  '正在生成故事线...',
];

export function AnalysisProgress({ progress, currentStep }: AnalysisProgressProps) {
  const currentStepIndex = STEPS.findIndex((s) => currentStep.includes(s.replace(/\.\.\./, '')));
  const activeIndex = currentStepIndex >= 0 ? currentStepIndex : Math.floor((progress / 100) * STEPS.length);

  return (
    <div className="flex flex-col items-center justify-center p-8">
      <Loader2 className="w-10 h-10 text-cinema-gold animate-spin mb-4" />
      <h3 className="text-lg font-medium text-white mb-2">正在分析小说</h3>
      <p className="text-sm text-gray-400 mb-6">{currentStep}</p>

      {/* 进度条 */}
      <div className="w-full max-w-md h-2 bg-cinema-800 rounded-full overflow-hidden mb-6">
        <div
          className="h-full bg-gradient-to-r from-cinema-gold to-cinema-gold-dark transition-all duration-500"
          style={{ width: `${progress}%` }}
        />
      </div>

      {/* 步骤指示器 */}
      <div className="w-full max-w-md space-y-2">
        {STEPS.map((step, index) => {
          const isActive = index === activeIndex;
          const isCompleted = index < activeIndex;

          return (
            <div
              key={step}
              className={`flex items-center gap-3 text-sm ${
                isActive ? 'text-cinema-gold' : isCompleted ? 'text-green-500' : 'text-gray-600'
              }`}
            >
              <div
                className={`w-5 h-5 rounded-full flex items-center justify-center text-xs ${
                  isActive
                    ? 'bg-cinema-gold/20 text-cinema-gold'
                    : isCompleted
                    ? 'bg-green-500/20 text-green-500'
                    : 'bg-cinema-800 text-gray-600'
                }`}
              >
                {isCompleted ? '✓' : index + 1}
              </div>
              <span>{step}</span>
            </div>
          );
        })}
      </div>

      <p className="text-xs text-gray-600 mt-6">
        分析时间取决于小说长度和 LLM 响应速度，请耐心等待...
      </p>
    </div>
  );
}
