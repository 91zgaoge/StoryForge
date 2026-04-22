/**
 * CreationWizard - 小说创建向导页面
 *
 * 分步向导，每步提供 AI 生成选项供用户选择：
 * 1. 创意输入
 * 2. 世界观选择
 * 3. 角色谱选择
 * 4. 文风选择
 * 5. 首个场景生成
 */

import React, { useState, useCallback, useRef } from 'react';
import { cn } from '@/utils/cn';
import {
  Sparkles,
  ChevronRight,
  ChevronLeft,
  Globe,
  Users,
  PenTool,
  BookOpen,
  Check,
  Lightbulb,
  ChevronDown,
  ChevronUp,
  Edit3,
  RefreshCw,
  Palette,
  FileText,
  Wand2,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { StreamOutput } from '@/components/StreamOutput';
import {
  generateWorldBuildingOptions,
  generateCharacterProfiles,
  generateWritingStyles,
  generateFirstScene,
  createStoryWithWizard,
  listStyleDnas,
  analyzeStyleSample,
} from '@/services/tauri';
import type {
  WorldBuildingOption,
  CharacterProfileOption,
  WritingStyleOption,
  SceneProposal,
} from '@/types/v3';
import type { StyleDNA } from '@/types/index';
import { useAppStore } from '@/stores/appStore';

// 兼容 listStyleDnas 返回的简化类型
interface StyleDnaItem {
  id: string;
  name: string;
  author?: string;
  is_builtin: boolean;
}
import toast from 'react-hot-toast';
import { useQuery, useQueryClient } from '@tanstack/react-query';

interface WizardData {
  genreInput: string;
  worldBuilding: WorldBuildingOption | null;
  characters: CharacterProfileOption[] | null;
  writingStyle: WritingStyleOption | null;
  firstScene: SceneProposal | null;
}

type WizardStep =
  | 'input'
  | 'world'
  | 'characters'
  | 'style'
  | 'scene'
  | 'confirm';

const STEP_ORDER: WizardStep[] = ['input', 'world', 'characters', 'style', 'scene', 'confirm'];

const STEP_CONFIG: Record<
  WizardStep,
  { label: string; icon: React.ElementType; description: string }
> = {
  input: { label: '创意输入', icon: Lightbulb, description: '描述你的故事创意' },
  world: { label: '世界观', icon: Globe, description: '选择或自定义世界观' },
  characters: { label: '角色谱', icon: Users, description: '选择核心角色配置' },
  style: { label: '文风', icon: PenTool, description: '选择文字风格' },
  scene: { label: '首个场景', icon: BookOpen, description: '确认首个场景' },
  confirm: { label: '确认创作', icon: Check, description: '汇总并开始写作' },
};

function countWords(text: string): number {
  const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
  const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
  return chineseChars + englishWords;
}

export function CreationWizard() {
  const setCurrentView = useAppStore((s) => s.setCurrentView);
  const [currentStep, setCurrentStep] = useState<WizardStep>('input');
  const [data, setData] = useState<WizardData>({
    genreInput: '',
    worldBuilding: null,
    characters: null,
    writingStyle: null,
    firstScene: null,
  });
  const [isGenerating, setIsGenerating] = useState(false);
  const [generationStep, setGenerationStep] = useState('');
  const [typewriterText, setTypewriterText] = useState('');
  const [isTypewriterRunning, setIsTypewriterRunning] = useState(false);
  const [typewriterProgress, setTypewriterProgress] = useState(0);
  const typewriterRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // 各步骤选项数据
  const [worldOptions, setWorldOptions] = useState<WorldBuildingOption[]>([]);
  const [characterSets, setCharacterSets] = useState<CharacterProfileOption[][]>([]);
  const [styleOptions, setStyleOptions] = useState<WritingStyleOption[]>([]);

  // 自定义编辑状态
  const [customWorld, setCustomWorld] = useState<WorldBuildingOption | null>(null);
  const [customCharacters, setCustomCharacters] = useState<CharacterProfileOption[] | null>(null);
  const [customStyle, setCustomStyle] = useState<WritingStyleOption | null>(null);
  const [customScene, setCustomScene] = useState<SceneProposal | null>(null);

  // StyleDNA
  const queryClient = useQueryClient();
  const { data: styleDnas = [] } = useQuery({
    queryKey: ['style-dnas'],
    queryFn: listStyleDnas,
    staleTime: 5 * 60 * 1000,
  });
  const [selectedStyleDna, setSelectedStyleDna] = useState<StyleDnaItem | null>(null);
  const [showStyleSampleInput, setShowStyleSampleInput] = useState(false);
  const [styleSampleText, setStyleSampleText] = useState('');
  const [isAnalyzingStyle, setIsAnalyzingStyle] = useState(false);
  const [styleTab, setStyleTab] = useState<'ai' | 'dna'>('ai');

  // 折叠状态
  const [collapsedSteps, setCollapsedSteps] = useState<Set<WizardStep>>(new Set());

  const getStepIndex = (step: WizardStep) => STEP_ORDER.indexOf(step);
  const isStepCompleted = (step: WizardStep) => getStepIndex(step) < getStepIndex(currentStep);
  const isStepAvailable = (step: WizardStep) => getStepIndex(step) <= getStepIndex(currentStep);

  const updateData = useCallback((partial: Partial<WizardData>) => {
    setData((prev) => ({ ...prev, ...partial }));
  }, []);

  const stopTypewriter = useCallback(() => {
    if (typewriterRef.current) {
      clearInterval(typewriterRef.current);
      typewriterRef.current = null;
    }
    setIsTypewriterRunning(false);
    setTypewriterProgress(100);
  }, []);

  const startTypewriter = useCallback((fullText: string, onComplete?: () => void) => {
    stopTypewriter();
    setTypewriterText('');
    setIsTypewriterRunning(true);
    setTypewriterProgress(0);

    let index = 0;
    const total = fullText.length;
    typewriterRef.current = setInterval(() => {
      index += 2;
      if (index >= total) {
        setTypewriterText(fullText);
        setTypewriterProgress(100);
        stopTypewriter();
        onComplete?.();
      } else {
        setTypewriterText(fullText.slice(0, index));
        setTypewriterProgress(Math.round((index / total) * 100));
      }
    }, 12);
  }, [stopTypewriter]);

  const goToStep = useCallback((step: WizardStep) => {
    if (!isStepAvailable(step)) return;
    setCurrentStep(step);
    setCollapsedSteps((prev) => {
      const next = new Set(prev);
      next.delete(step);
      return next;
    });
  }, [isStepAvailable]);

  const goNext = useCallback(() => {
    const idx = getStepIndex(currentStep);
    if (idx < STEP_ORDER.length - 1) {
      setCollapsedSteps((prev) => new Set(prev).add(currentStep));
      setCurrentStep(STEP_ORDER[idx + 1]);
    }
  }, [currentStep]);

  const goBack = useCallback(() => {
    const idx = getStepIndex(currentStep);
    if (idx > 0) {
      setCurrentStep(STEP_ORDER[idx - 1]);
      setCollapsedSteps((prev) => {
        const next = new Set(prev);
        next.delete(STEP_ORDER[idx - 1]);
        return next;
      });
    }
  }, [currentStep]);

  // ===== 第2步：生成世界观 =====
  const handleGenerateWorlds = useCallback(async () => {
    if (!data.genreInput.trim()) {
      toast.error('请先输入故事创意');
      return;
    }
    setIsGenerating(true);
    setGenerationStep('正在生成世界观选项...');
    try {
      const options = await generateWorldBuildingOptions(data.genreInput.trim());
      setWorldOptions(options);
      updateData({ worldBuilding: options[0] || null });
      goNext();
    } catch (error) {
      console.error('Failed to generate worlds:', error);
      toast.error('生成世界观失败，请重试');
    } finally {
      setIsGenerating(false);
      setGenerationStep('');
    }
  }, [data.genreInput, updateData, goNext]);

  // ===== 第3步：生成角色 =====
  const handleGenerateCharacters = useCallback(async () => {
    const world = customWorld || data.worldBuilding;
    if (!world) {
      toast.error('请先选择世界观');
      return;
    }
    setIsGenerating(true);
    setGenerationStep('正在生成角色谱...');
    try {
      const sets = await generateCharacterProfiles(world);
      setCharacterSets(sets);
      updateData({ characters: sets[0] || null });
      goNext();
    } catch (error) {
      console.error('Failed to generate characters:', error);
      toast.error('生成角色失败，请重试');
    } finally {
      setIsGenerating(false);
      setGenerationStep('');
    }
  }, [data.worldBuilding, customWorld, updateData, goNext]);

  // ===== 第4步：生成文风 =====
  const handleGenerateStyles = useCallback(async () => {
    const world = customWorld || data.worldBuilding;
    if (!world) {
      toast.error('请先选择世界观');
      return;
    }
    setIsGenerating(true);
    setGenerationStep('正在生成文风选项...');
    try {
      const styles = await generateWritingStyles(data.genreInput.trim(), world);
      setStyleOptions(styles);
      updateData({ writingStyle: styles[0] || null });
      goNext();
    } catch (error) {
      console.error('Failed to generate styles:', error);
      toast.error('生成文风失败，请重试');
    } finally {
      setIsGenerating(false);
      setGenerationStep('');
    }
  }, [data.genreInput, data.worldBuilding, customWorld, updateData, goNext]);

  // ===== 第5步：生成首个场景 =====
  const handleGenerateScene = useCallback(async () => {
    const world = customWorld || data.worldBuilding;
    const chars = customCharacters || data.characters;
    const style = customStyle || data.writingStyle;
    if (!world || !chars || !style) {
      toast.error('请先完成前面的步骤');
      return;
    }
    setIsGenerating(true);
    setGenerationStep('正在生成首个场景...');
    try {
      const scene = await generateFirstScene(world, chars, style);
      setCustomScene(scene);
      updateData({ firstScene: scene });
      startTypewriter(scene.content, () => {
        setIsGenerating(false);
        setGenerationStep('');
      });
      goNext();
    } catch (error) {
      console.error('Failed to generate scene:', error);
      toast.error('生成首个场景失败，请重试');
      setIsGenerating(false);
      setGenerationStep('');
    }
  }, [data.worldBuilding, data.characters, data.writingStyle, customWorld, customCharacters, customStyle, updateData, startTypewriter, goNext]);

  // ===== 完成：创建故事 =====
  const [isCreating, setIsCreating] = useState(false);
  const handleComplete = useCallback(async () => {
    const world = customWorld || data.worldBuilding;
    const chars = customCharacters || data.characters;
    const style = customStyle || data.writingStyle;
    const scene = customScene || data.firstScene;
    if (!world || !chars || !style || !scene) {
      toast.error('信息不完整，无法创建故事');
      return;
    }
    setIsCreating(true);
    try {
      const result = await createStoryWithWizard({
        title: scene.title || '未命名故事',
        description: data.genreInput,
        genre: data.genreInput.slice(0, 20),
        world_building: world,
        characters: chars,
        writing_style: style,
        first_scene: scene,
      });
      toast.success(`故事「${result.story.title}」创建成功！`);
      // 跳转到场景页面
      setCurrentView('scenes');
    } catch (error) {
      console.error('Failed to create story:', error);
      toast.error('创建故事失败');
    } finally {
      setIsCreating(false);
    }
  }, [data, customWorld, customCharacters, customStyle, customScene]);

  // ===== 重新生成 =====
  const handleRegenerate = useCallback(
    async (targetStep: WizardStep) => {
      switch (targetStep) {
        case 'world':
          await handleGenerateWorlds();
          break;
        case 'characters':
          await handleGenerateCharacters();
          break;
        case 'style':
          await handleGenerateStyles();
          break;
        case 'scene':
          stopTypewriter();
          setTypewriterText('');
          await handleGenerateScene();
          break;
      }
    },
    [handleGenerateWorlds, handleGenerateCharacters, handleGenerateStyles, handleGenerateScene, stopTypewriter]
  );

  // ===== 侧边栏汇总项 =====
  const SummaryItem = ({
    step,
    label,
    value,
  }: {
    step: WizardStep;
    label: string;
    value: string | React.ReactNode;
  }) => {
    const completed = isStepCompleted(step) || currentStep === step;
    return (
      <button
        onClick={() => goToStep(step)}
        className={cn(
          'w-full text-left p-3 rounded-lg transition-colors border',
          completed
            ? 'bg-cinema-800/50 border-cinema-700 hover:bg-cinema-800'
            : 'bg-transparent border-transparent opacity-50 cursor-not-allowed'
        )}
        disabled={!completed}
      >
        <div className="text-xs text-gray-500">{label}</div>
        <div className="text-sm text-gray-300 truncate mt-0.5">
          {value || <span className="text-gray-600">未选择</span>}
        </div>
      </button>
    );
  };

  // ===== 各步骤渲染 =====

  const renderStepInput = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-xl font-bold text-white mb-2 flex items-center gap-2">
          <Lightbulb className="w-5 h-5 text-cinema-gold" />
          你的故事创意是什么？
        </h3>
        <p className="text-gray-400 text-sm">
          用一句话描述你想写的故事，AI 将基于此为你生成完整的世界观、角色和文风。
        </p>
      </div>

      <div className="relative">
        <textarea
          value={data.genreInput}
          onChange={(e) => updateData({ genreInput: e.target.value })}
          placeholder="例如：一个修仙者穿越到现代都市，在灵气复苏的背景下重新修炼..."
          className="w-full h-40 px-4 py-4 bg-cinema-800 border border-cinema-700 rounded-xl text-white placeholder-gray-500 focus:border-cinema-gold focus:outline-none resize-none text-base leading-relaxed"
        />
        <div className="absolute bottom-3 right-3 text-xs text-gray-500">
          {countWords(data.genreInput)} 字
        </div>
      </div>

      <div className="flex justify-end">
        <Button
          variant="primary"
          onClick={handleGenerateWorlds}
          disabled={!data.genreInput.trim() || isGenerating}
          isLoading={isGenerating}
        >
          <Sparkles className="w-4 h-4 mr-2" />
          {isGenerating ? generationStep : '开始创作'}
        </Button>
      </div>
    </div>
  );

  const renderStepWorld = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-bold text-white flex items-center gap-2">
          <Globe className="w-5 h-5 text-cinema-gold" />
          选择世界观
        </h3>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => handleRegenerate('world')}
          disabled={isGenerating}
        >
          <RefreshCw className="w-3.5 h-3.5 mr-1" />
          重新生成
        </Button>
      </div>

      <div className="grid gap-4">
        {worldOptions.map((world, index) => (
          <Card
            key={world.id || index}
            hover
            className={cn(
              'cursor-pointer transition-all',
              (customWorld?.id || data.worldBuilding?.id) === world.id
                ? 'ring-2 ring-cinema-gold'
                : ''
            )}
            onClick={() => {
              setCustomWorld(null);
              updateData({ worldBuilding: world });
            }}
          >
            <CardContent className="p-5">
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 rounded-lg bg-cinema-gold/10 flex items-center justify-center flex-shrink-0">
                  <Globe className="w-5 h-5 text-cinema-gold" />
                </div>
                <div className="flex-1 min-w-0">
                  <h4 className="font-semibold text-white mb-1">{world.concept}</h4>
                  <div className="flex flex-wrap gap-1.5 mb-2">
                    {world.rules.map((rule, i) => (
                      <span
                        key={i}
                        className="px-2 py-0.5 text-[11px] bg-cinema-700 rounded text-gray-300"
                      >
                        {rule.name}
                      </span>
                    ))}
                  </div>
                  <p className="text-sm text-gray-400 line-clamp-2">{world.history}</p>
                </div>
                {(customWorld?.id || data.worldBuilding?.id) === world.id && (
                  <Check className="w-5 h-5 text-cinema-gold flex-shrink-0" />
                )}
              </div>
            </CardContent>
          </Card>
        ))}

        {/* 自定义选项 */}
        <Card
          hover
          className={cn(
            'cursor-pointer transition-all',
            customWorld ? 'ring-2 ring-cinema-gold' : ''
          )}
          onClick={() => {
            if (!customWorld) {
              setCustomWorld({
                id: `custom-${Date.now()}`,
                concept: '自定义世界观',
                rules: [{ id: '1', name: '自定义规则', rule_type: 'Custom', importance: 5 }],
                cultures: [{ name: '自定义文化', description: '', customs: [], values: [] }],
              });
              updateData({ worldBuilding: null });
            }
          }}
        >
          <CardContent className="p-5">
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 rounded-lg bg-cinema-700 flex items-center justify-center flex-shrink-0">
                <Edit3 className="w-5 h-5 text-gray-400" />
              </div>
              <div className="flex-1">
                <h4 className="font-semibold text-white">自定义世界观</h4>
                <p className="text-sm text-gray-400">点击后手动编辑世界观细节</p>
              </div>
              {customWorld && <Check className="w-5 h-5 text-cinema-gold" />}
            </div>
            {customWorld && (
              <div className="mt-4 space-y-3 border-t border-cinema-700 pt-3">
                <input
                  value={customWorld.concept}
                  onChange={(e) =>
                    setCustomWorld({ ...customWorld, concept: e.target.value })
                  }
                  className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                  placeholder="世界观名称"
                />
                <textarea
                  value={customWorld.history || ''}
                  onChange={(e) =>
                    setCustomWorld({ ...customWorld, history: e.target.value })
                  }
                  rows={3}
                  className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-none"
                  placeholder="世界观背景描述..."
                />
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="flex justify-between">
        <Button variant="ghost" onClick={goBack}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          上一步
        </Button>
        <Button
          variant="primary"
          onClick={handleGenerateCharacters}
          disabled={(!data.worldBuilding && !customWorld) || isGenerating}
          isLoading={isGenerating}
        >
          下一步：角色谱
          <ChevronRight className="w-4 h-4 ml-1" />
        </Button>
      </div>
    </div>
  );

  const renderStepCharacters = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-bold text-white flex items-center gap-2">
          <Users className="w-5 h-5 text-cinema-gold" />
          选择角色谱
        </h3>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => handleRegenerate('characters')}
          disabled={isGenerating}
        >
          <RefreshCw className="w-3.5 h-3.5 mr-1" />
          重新生成
        </Button>
      </div>

      <div className="grid gap-4">
        {characterSets.map((set, index) => (
          <Card
            key={index}
            hover
            className={cn(
              'cursor-pointer transition-all',
              (customCharacters ? false : data.characters === set)
                ? 'ring-2 ring-cinema-gold'
                : ''
            )}
            onClick={() => {
              setCustomCharacters(null);
              updateData({ characters: set });
            }}
          >
            <CardContent className="p-5">
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 rounded-lg bg-cinema-gold/10 flex items-center justify-center flex-shrink-0">
                  <Users className="w-5 h-5 text-cinema-gold" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex flex-wrap gap-2 mb-2">
                    {set.map((char) => (
                      <span
                        key={char.id}
                        className="px-2 py-0.5 rounded-md bg-cinema-700 text-gray-300 text-xs"
                      >
                        {char.name}
                      </span>
                    ))}
                  </div>
                  <div className="space-y-1">
                    {set.map((char) => (
                      <p key={char.id} className="text-sm text-gray-400">
                        <span className="text-gray-300">{char.name}：</span>
                        {char.personality} · {char.goals}
                      </p>
                    ))}
                  </div>
                </div>
                {(customCharacters ? false : data.characters === set) && (
                  <Check className="w-5 h-5 text-cinema-gold flex-shrink-0" />
                )}
              </div>
            </CardContent>
          </Card>
        ))}

        {/* 自定义角色 */}
        <Card
          hover
          className={cn(
            'cursor-pointer transition-all',
            customCharacters ? 'ring-2 ring-cinema-gold' : ''
          )}
          onClick={() => {
            if (!customCharacters) {
              setCustomCharacters([
                {
                  id: `custom-char-${Date.now()}`,
                  name: '主角',
                  personality: '',
                  background: '',
                  goals: '',
                  voice_style: '',
                },
              ]);
              updateData({ characters: null });
            }
          }}
        >
          <CardContent className="p-5">
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 rounded-lg bg-cinema-700 flex items-center justify-center flex-shrink-0">
                <Edit3 className="w-5 h-5 text-gray-400" />
              </div>
              <div className="flex-1">
                <h4 className="font-semibold text-white">自定义角色</h4>
                <p className="text-sm text-gray-400">手动配置角色</p>
              </div>
              {customCharacters && <Check className="w-5 h-5 text-cinema-gold" />}
            </div>
            {customCharacters && (
              <div className="mt-4 space-y-3 border-t border-cinema-700 pt-3">
                {customCharacters.map((char, idx) => (
                  <div key={char.id} className="space-y-2">
                    <input
                      value={char.name}
                      onChange={(e) => {
                        const next = [...customCharacters];
                        next[idx] = { ...char, name: e.target.value };
                        setCustomCharacters(next);
                      }}
                      className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                      placeholder="角色名"
                    />
                    <input
                      value={char.personality}
                      onChange={(e) => {
                        const next = [...customCharacters];
                        next[idx] = { ...char, personality: e.target.value };
                        setCustomCharacters(next);
                      }}
                      className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                      placeholder="性格特点"
                    />
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="flex justify-between">
        <Button variant="ghost" onClick={goBack}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          上一步
        </Button>
        <Button
          variant="primary"
          onClick={handleGenerateStyles}
          disabled={(!data.characters && !customCharacters) || isGenerating}
          isLoading={isGenerating}
        >
          下一步：文风
          <ChevronRight className="w-4 h-4 ml-1" />
        </Button>
      </div>
    </div>
  );

  const renderStepStyle = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-bold text-white flex items-center gap-2">
          <PenTool className="w-5 h-5 text-cinema-gold" />
          选择文风
        </h3>
      </div>

      {/* 标签切换 */}
      <div className="flex gap-2">
        <button
          onClick={() => setStyleTab('ai')}
          className={cn(
            'px-3 py-1.5 rounded-lg text-sm transition-colors',
            styleTab === 'ai'
              ? 'bg-cinema-gold/20 text-cinema-gold border border-cinema-gold/30'
              : 'bg-cinema-800 text-gray-400 border border-transparent hover:bg-cinema-700'
          )}
        >
          <Wand2 className="w-3.5 h-3.5 inline mr-1" />
          AI 生成文风
        </button>
        <button
          onClick={() => setStyleTab('dna')}
          className={cn(
            'px-3 py-1.5 rounded-lg text-sm transition-colors',
            styleTab === 'dna'
              ? 'bg-cinema-gold/20 text-cinema-gold border border-cinema-gold/30'
              : 'bg-cinema-800 text-gray-400 border border-transparent hover:bg-cinema-700'
          )}
        >
          <Palette className="w-3.5 h-3.5 inline mr-1" />
          风格 DNA
        </button>
      </div>

      {styleTab === 'ai' && (
        <>
          <div className="grid gap-4">
            {styleOptions.map((style, index) => (
              <Card
                key={style.id || index}
                hover
                className={cn(
                  'cursor-pointer transition-all',
                  (customStyle ? false : data.writingStyle?.id === style.id)
                    ? 'ring-2 ring-cinema-gold'
                    : ''
                )}
                onClick={() => {
                  setCustomStyle(null);
                  setSelectedStyleDna(null);
                  updateData({ writingStyle: style });
                }}
              >
                <CardContent className="p-5">
                  <div className="flex items-start gap-4">
                    <div className="w-10 h-10 rounded-lg bg-cinema-gold/10 flex items-center justify-center flex-shrink-0">
                      <PenTool className="w-5 h-5 text-cinema-gold" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h4 className="font-semibold text-white mb-1">{style.name}</h4>
                      <p className="text-sm text-gray-400 mb-2">{style.description}</p>
                      <p className="text-xs text-gray-500 italic line-clamp-2">
                        "{style.sample_text}"
                      </p>
                    </div>
                    {(customStyle ? false : data.writingStyle?.id === style.id) && (
                      <Check className="w-5 h-5 text-cinema-gold flex-shrink-0" />
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}

            {/* 自定义文风 */}
            <Card
              hover
              className={cn(
                'cursor-pointer transition-all',
                customStyle ? 'ring-2 ring-cinema-gold' : ''
              )}
              onClick={() => {
                if (!customStyle) {
                  setCustomStyle({
                    id: `custom-style-${Date.now()}`,
                    name: '自定义文风',
                    description: '',
                    tone: '',
                    pacing: '',
                    vocabulary_level: '',
                    sentence_structure: '',
                    sample_text: '',
                  });
                  updateData({ writingStyle: null });
                }
              }}
            >
              <CardContent className="p-5">
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 rounded-lg bg-cinema-700 flex items-center justify-center flex-shrink-0">
                    <Edit3 className="w-5 h-5 text-gray-400" />
                  </div>
                  <div className="flex-1">
                    <h4 className="font-semibold text-white">自定义文风</h4>
                    <p className="text-sm text-gray-400">手动配置文风参数</p>
                  </div>
                  {customStyle && <Check className="w-5 h-5 text-cinema-gold" />}
                </div>
                {customStyle && (
                  <div className="mt-4 space-y-3 border-t border-cinema-700 pt-3">
                    <input
                      value={customStyle.name}
                      onChange={(e) =>
                        setCustomStyle({ ...customStyle, name: e.target.value })
                      }
                      className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                      placeholder="文风名称"
                    />
                    <textarea
                      value={customStyle.description}
                      onChange={(e) =>
                        setCustomStyle({ ...customStyle, description: e.target.value })
                      }
                      rows={3}
                      className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-none"
                      placeholder="文风描述..."
                    />
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
          <div className="flex justify-end">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handleRegenerate('style')}
              disabled={isGenerating}
            >
              <RefreshCw className="w-3.5 h-3.5 mr-1" />
              重新生成文风
            </Button>
          </div>
        </>
      )}

      {styleTab === 'dna' && (
        <div className="space-y-4">
          <div className="grid gap-2">
            {styleDnas.map((dna) => (
              <button
                key={dna.id}
                onClick={() => {
                  setSelectedStyleDna(dna);
                  setCustomStyle(null);
                  updateData({
                    writingStyle: {
                      id: dna.id,
                      name: dna.name,
                      description: dna.author ? `模仿 ${dna.author} 的写作风格` : '自定义风格',
                      tone: '',
                      pacing: '',
                      vocabulary_level: '',
                      sentence_structure: '',
                      sample_text: '',
                    },
                  });
                }}
                className={cn(
                  'w-full p-3 rounded-lg text-left transition-colors border',
                  selectedStyleDna?.id === dna.id
                    ? 'bg-cinema-gold/20 border-cinema-gold/50'
                    : 'bg-cinema-800 border-transparent hover:bg-cinema-700'
                )}
              >
                <div className="flex items-center gap-2">
                  <span className="font-medium text-white">{dna.name}</span>
                  {dna.is_builtin && (
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-cinema-gold/20 text-cinema-gold">
                      内置
                    </span>
                  )}
                </div>
                {dna.author && (
                  <div className="text-xs text-gray-400">作者：{dna.author}</div>
                )}
              </button>
            ))}
          </div>

          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowStyleSampleInput(true)}
          >
            <Sparkles className="w-3.5 h-3.5 mr-1 text-cinema-gold" />
            从文本生成风格
          </Button>

          {showStyleSampleInput && (
            <Card>
              <CardContent className="p-4 space-y-3">
                <p className="text-sm text-gray-400">
                  粘贴一段你喜欢的文字（300-3000字），AI 将分析其风格特征。
                </p>
                <textarea
                  value={styleSampleText}
                  onChange={(e) => setStyleSampleText(e.target.value)}
                  rows={5}
                  className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-none"
                  placeholder="在此粘贴文本样例..."
                />
                <div className="flex justify-end gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      setShowStyleSampleInput(false);
                      setStyleSampleText('');
                    }}
                  >
                    取消
                  </Button>
                  <Button
                    variant="primary"
                    size="sm"
                    isLoading={isAnalyzingStyle}
                    disabled={styleSampleText.length < 30}
                    onClick={async () => {
                      setIsAnalyzingStyle(true);
                      try {
                        const result = await analyzeStyleSample(styleSampleText);
                        toast.success(`风格「${result.name}」生成成功！`);
                        queryClient.invalidateQueries({ queryKey: ['style-dnas'] });
                        setShowStyleSampleInput(false);
                        setStyleSampleText('');
                        setSelectedStyleDna(result);
                        updateData({
                          writingStyle: {
                            id: result.id,
                            name: result.name,
                            description: result.author
                              ? `模仿 ${result.author} 的写作风格`
                              : '自定义风格',
                            tone: '',
                            pacing: '',
                            vocabulary_level: '',
                            sentence_structure: '',
                            sample_text: '',
                          },
                        });
                      } catch (err: any) {
                        toast.error(`风格生成失败: ${err?.message || String(err)}`);
                      } finally {
                        setIsAnalyzingStyle(false);
                      }
                    }}
                  >
                    生成风格
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      <div className="flex justify-between">
        <Button variant="ghost" onClick={goBack}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          上一步
        </Button>
        <Button
          variant="primary"
          onClick={handleGenerateScene}
          disabled={(!data.writingStyle && !customStyle) || isGenerating}
          isLoading={isGenerating}
        >
          下一步：首个场景
          <ChevronRight className="w-4 h-4 ml-1" />
        </Button>
      </div>
    </div>
  );

  const renderStepScene = () => {
    const scene = customScene || data.firstScene;
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h3 className="text-xl font-bold text-white flex items-center gap-2">
            <BookOpen className="w-5 h-5 text-cinema-gold" />
            首个场景
          </h3>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleRegenerate('scene')}
            disabled={isGenerating}
          >
            <RefreshCw className="w-3.5 h-3.5 mr-1" />
            重新生成
          </Button>
        </div>

        {scene && (
          <div className="space-y-4">
            <div className="bg-cinema-800/50 rounded-xl p-4 border border-cinema-700">
              <h4 className="font-semibold text-white mb-2">{scene.title}</h4>
              <div className="grid grid-cols-2 gap-2 text-xs text-gray-400 mb-3">
                <div>
                  <span className="text-gray-500">场景地点：</span>
                  {scene.setting_location}
                </div>
                <div>
                  <span className="text-gray-500">时间：</span>
                  {scene.setting_time}
                </div>
                <div>
                  <span className="text-gray-500">氛围：</span>
                  {scene.setting_atmosphere}
                </div>
                <div>
                  <span className="text-gray-500">冲突：</span>
                  {scene.conflict_type}
                </div>
              </div>
            </div>

            <StreamOutput
              text={typewriterText || scene.content}
              isStreaming={isTypewriterRunning || isGenerating}
              progress={typewriterProgress}
              streamType="simulated"
              onStop={stopTypewriter}
              title="场景正文"
              showToolbar
            />

            <Card
              hover
              className={cn(
                'cursor-pointer transition-all',
                customScene && customScene !== data.firstScene
                  ? 'ring-2 ring-cinema-gold'
                  : ''
              )}
              onClick={() => {
                if (!customScene || customScene === data.firstScene) {
                  const base = data.firstScene!;
                  setCustomScene({ ...base });
                }
              }}
            >
              <CardContent className="p-4">
                <div className="flex items-center gap-3">
                  <Edit3 className="w-4 h-4 text-gray-400" />
                  <span className="text-sm text-gray-300">手动编辑场景内容</span>
                </div>
                {customScene && customScene !== data.firstScene && (
                  <textarea
                    value={customScene.content}
                    onChange={(e) =>
                      setCustomScene({ ...customScene, content: e.target.value })
                    }
                    rows={6}
                    className="w-full mt-3 px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-none"
                  />
                )}
              </CardContent>
            </Card>
          </div>
        )}

        <div className="flex justify-between">
          <Button variant="ghost" onClick={goBack}>
            <ChevronLeft className="w-4 h-4 mr-1" />
            上一步
          </Button>
          <Button variant="primary" onClick={goNext}>
            确认并汇总
            <ChevronRight className="w-4 h-4 ml-1" />
          </Button>
        </div>
      </div>
    );
  };

  const renderStepConfirm = () => {
    const world = customWorld || data.worldBuilding;
    const chars = customCharacters || data.characters;
    const style = customStyle || data.writingStyle;
    const scene = customScene || data.firstScene;

    return (
      <div className="space-y-6">
        <div className="text-center">
          <div className="relative w-20 h-20 mx-auto mb-4">
            <div className="absolute inset-0 bg-cinema-gold/20 rounded-full animate-ping" />
            <div className="relative w-20 h-20 bg-cinema-gold/10 rounded-full flex items-center justify-center">
              <Check className="w-10 h-10 text-cinema-gold" />
            </div>
          </div>
          <h3 className="text-2xl font-bold text-white mb-1">创作准备完成！</h3>
          <p className="text-gray-400">确认以下信息后即可开始写作</p>
        </div>

        <div className="bg-cinema-800/50 rounded-xl p-5 space-y-4 border border-cinema-700">
          <div className="flex items-start gap-3">
            <Lightbulb className="w-5 h-5 text-cinema-gold mt-0.5 flex-shrink-0" />
            <div>
              <div className="text-xs text-gray-500">创意</div>
              <div className="text-sm text-gray-300">{data.genreInput}</div>
            </div>
          </div>
          <div className="border-t border-cinema-700" />
          <div className="flex items-start gap-3">
            <Globe className="w-5 h-5 text-cinema-gold mt-0.5 flex-shrink-0" />
            <div>
              <div className="text-xs text-gray-500">世界观</div>
              <div className="text-sm text-gray-300">{world?.concept}</div>
            </div>
          </div>
          <div className="border-t border-cinema-700" />
          <div className="flex items-start gap-3">
            <Users className="w-5 h-5 text-cinema-gold mt-0.5 flex-shrink-0" />
            <div>
              <div className="text-xs text-gray-500">角色</div>
              <div className="text-sm text-gray-300">
                {chars?.map((c) => c.name).join('、')}
              </div>
            </div>
          </div>
          <div className="border-t border-cinema-700" />
          <div className="flex items-start gap-3">
            <PenTool className="w-5 h-5 text-cinema-gold mt-0.5 flex-shrink-0" />
            <div>
              <div className="text-xs text-gray-500">文风</div>
              <div className="text-sm text-gray-300">{style?.name}</div>
            </div>
          </div>
          <div className="border-t border-cinema-700" />
          <div className="flex items-start gap-3">
            <BookOpen className="w-5 h-5 text-cinema-gold mt-0.5 flex-shrink-0" />
            <div>
              <div className="text-xs text-gray-500">首个场景</div>
              <div className="text-sm text-gray-300">{scene?.title}</div>
            </div>
          </div>
        </div>

        <div className="flex justify-center gap-4">
          <Button variant="ghost" onClick={() => goToStep('scene')}>
            <ChevronLeft className="w-4 h-4 mr-1" />
            返回修改
          </Button>
          <Button
            variant="primary"
            onClick={handleComplete}
            isLoading={isCreating}
            disabled={!world || !chars || !style || !scene}
          >
            <Sparkles className="w-4 h-4 mr-2" />
            {isCreating ? '创建中...' : '开始写作'}
          </Button>
        </div>
      </div>
    );
  };

  const renderStepContent = (step: WizardStep) => {
    switch (step) {
      case 'input':
        return renderStepInput();
      case 'world':
        return renderStepWorld();
      case 'characters':
        return renderStepCharacters();
      case 'style':
        return renderStepStyle();
      case 'scene':
        return renderStepScene();
      case 'confirm':
        return renderStepConfirm();
    }
  };

  const toggleCollapse = (step: WizardStep) => {
    if (step === currentStep) return;
    setCollapsedSteps((prev) => {
      const next = new Set(prev);
      if (next.has(step)) next.delete(step);
      else next.add(step);
      return next;
    });
  };

  return (
    <div className="h-full flex animate-fade-in">
      {/* 主内容区 */}
      <div className="flex-1 overflow-auto p-8">
        <div className="max-w-3xl mx-auto">
          <h1 className="font-display text-3xl font-bold text-white mb-2">
            AI 创作向导
          </h1>
          <p className="text-gray-400 mb-8">
            通过 5 个步骤，让 AI 帮你构建一个完整的故事世界
          </p>

          {/* 步骤进度条 */}
          <div className="mb-8">
            <div className="flex items-center justify-between mb-2">
              {STEP_ORDER.map((step, idx) => {
                const config = STEP_CONFIG[step];
                const Icon = config.icon;
                const isActive = step === currentStep;
                const isDone = isStepCompleted(step);
                return (
                  <React.Fragment key={step}>
                    <button
                      onClick={() => goToStep(step)}
                      className={cn(
                        'flex flex-col items-center gap-1 transition-colors',
                        isActive ? 'text-cinema-gold' : isDone ? 'text-gray-300' : 'text-gray-600'
                      )}
                      disabled={!isStepAvailable(step)}
                    >
                      <div
                        className={cn(
                          'w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold border-2 transition-colors',
                          isActive
                            ? 'bg-cinema-gold/20 border-cinema-gold text-cinema-gold'
                            : isDone
                            ? 'bg-cinema-gold/10 border-cinema-gold/50 text-cinema-gold'
                            : 'bg-cinema-800 border-cinema-700 text-gray-600'
                        )}
                      >
                        {isDone ? <Check className="w-4 h-4" /> : <Icon className="w-4 h-4" />}
                      </div>
                      <span className="text-[10px] hidden sm:block">{config.label}</span>
                    </button>
                    {idx < STEP_ORDER.length - 1 && (
                      <div
                        className={cn(
                          'flex-1 h-0.5 mx-2',
                          isDone ? 'bg-cinema-gold/50' : 'bg-cinema-800'
                        )}
                      />
                    )}
                  </React.Fragment>
                );
              })}
            </div>
          </div>

          {/* 当前步骤内容 */}
          <Card className="mb-8">
            <CardContent className="p-6">
              {renderStepContent(currentStep)}
            </CardContent>
          </Card>
        </div>
      </div>

      {/* 右侧汇总栏 */}
      <aside className="w-72 bg-cinema-900 border-l border-cinema-800 overflow-y-auto hidden xl:block">
        <div className="p-5">
          <h3 className="font-semibold text-white mb-4 flex items-center gap-2">
            <FileText className="w-4 h-4 text-cinema-gold" />
            创作汇总
          </h3>

          <div className="space-y-2">
            <SummaryItem
              step="input"
              label="创意"
              value={
                data.genreInput ? (
                  <span className="line-clamp-2">{data.genreInput}</span>
                ) : null
              }
            />
            <SummaryItem
              step="world"
              label="世界观"
              value={(customWorld || data.worldBuilding)?.concept}
            />
            <SummaryItem
              step="characters"
              label="角色"
              value={(customCharacters || data.characters)
                ?.map((c) => c.name)
                .join('、')}
            />
            <SummaryItem
              step="style"
              label="文风"
              value={(customStyle || data.writingStyle)?.name}
            />
            <SummaryItem
              step="scene"
              label="首个场景"
              value={(customScene || data.firstScene)?.title}
            />
          </div>

          {/* 折叠的步骤列表 */}
          <div className="mt-6 pt-6 border-t border-cinema-800">
            <h4 className="text-xs text-gray-500 mb-3">步骤导航</h4>
            <div className="space-y-1">
              {STEP_ORDER.map((step) => {
                const config = STEP_CONFIG[step];
                const Icon = config.icon;
                const isActive = step === currentStep;
                const done = isStepCompleted(step);
                const collapsed = collapsedSteps.has(step);

                return (
                  <div key={step}>
                    <button
                      onClick={() => {
                        if (done) toggleCollapse(step);
                        goToStep(step);
                      }}
                      className={cn(
                        'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors',
                        isActive
                          ? 'bg-cinema-gold/10 text-cinema-gold'
                          : done
                          ? 'text-gray-300 hover:bg-cinema-800'
                          : 'text-gray-600 cursor-not-allowed'
                      )}
                      disabled={!isStepAvailable(step)}
                    >
                      <Icon className="w-4 h-4 flex-shrink-0" />
                      <span className="flex-1 text-left">{config.label}</span>
                      {done && (
                        <>
                          <Check className="w-3 h-3 text-cinema-gold flex-shrink-0" />
                          {collapsed && step !== currentStep && (
                            <ChevronDown className="w-3 h-3 flex-shrink-0" />
                          )}
                          {!collapsed && step !== currentStep && (
                            <ChevronUp className="w-3 h-3 flex-shrink-0" />
                          )}
                        </>
                      )}
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </aside>
    </div>
  );
}

export default CreationWizard;
