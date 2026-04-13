import React, { useState } from 'react';
import { 
  Sparkles, 
  ChevronRight, 
  ChevronLeft, 
  Globe, 
  Users, 
  PenTool,
  BookOpen,
  Check,
  RefreshCw
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import type { 
  WorldBuilding, 
  CharacterProfileOption, 
  WritingStyle,
  ConflictType 
} from '@/types';

type WizardStep = 
  | 'genre_input' 
  | 'generating_world' 
  | 'selecting_world' 
  | 'generating_characters' 
  | 'selecting_characters' 
  | 'generating_style' 
  | 'selecting_style' 
  | 'completed';

interface NovelCreationWizardProps {
  onComplete: (data: {
    worldBuilding: WorldBuilding;
    characters: CharacterProfileOption[];
    writingStyle: WritingStyle;
  }) => void;
  onCancel: () => void;
}

// 示例选项（实际应该来自AI生成）
const SAMPLE_WORLD_OPTIONS = [
  {
    id: 'world_1',
    concept: '灵气复苏的现代社会，古老修仙门派与现代科技文明碰撞融合',
    rules: [
      { name: '灵气复苏', description: '近十年全球灵气浓度急剧上升，普通人可以修炼' },
      { name: '科技修真', description: '现代科学与修仙法门结合，产生独特修炼体系' },
    ],
    history: '2020年，一场突如其来的灵气风暴改变了世界...',
    cultures: [
      { name: '修真学院', description: '融合现代教育的修仙门派' },
      { name: '科技联盟', description: '反对修仙、坚持纯科技路线的组织' },
    ],
  },
  {
    id: 'world_2',
    concept: '赛博朋克风格的星际帝国，人类意识可以上传到机械躯体',
    rules: [
      { name: '意识上传', description: '人类可以将意识转移到机械身体，获得永生' },
      { name: '星际殖民', description: '人类已殖民数百个星球，形成庞大帝国' },
    ],
    history: '公元3045年，人类突破意识上传技术...',
    cultures: [
      { name: '机械教', description: '崇拜纯机械化的宗教组织' },
      { name: '原生派', description: '坚持保持人类肉体的传统派别' },
    ],
  },
  {
    id: 'world_3',
    concept: '架空历史的东方玄幻，龙脉决定王朝兴衰',
    rules: [
      { name: '龙脉之力', description: '每个王朝都依托龙脉建立，龙脉枯竭则王朝覆灭' },
      { name: '天命所归', description: '真正的帝王可以沟通天地，获得天命加持' },
    ],
    history: '大周王朝已延续八百年，龙脉日渐衰弱...',
    cultures: [
      { name: '钦天监', description: '观测龙脉、预测国运的官方机构' },
      { name: '隐龙阁', description: '暗中维护龙脉的神秘组织' },
    ],
  },
];

const SAMPLE_CHARACTER_SETS = [
  [
    { name: '林墨', personality: '冷静理智，内心执着', background: '普通大学生，意外觉醒灵根', goals: '寻找失踪的父亲' },
    { name: '苏雨晴', personality: '热情开朗，有些冲动', background: '修真世家大小姐', goals: '证明自己的能力' },
    { name: '陈老', personality: '深不可测，亦正亦邪', background: '隐世高人', goals: '培养传人' },
  ],
  [
    { name: '韩冰', personality: '冷酷果断，重情重义', background: '退役特种兵', goals: '保护家人' },
    { name: '赵天', personality: '野心勃勃，不择手段', background: '黑道太子', goals: '统一地下世界' },
    { name: '白灵', personality: '温柔善良，医术高超', background: '神秘医女', goals: '救治众生' },
  ],
];

const SAMPLE_STYLES = [
  { 
    name: '热血激昂', 
    description: '充满激情和力量感的叙事风格',
    tone: '积极向上，热血沸腾',
    sample: '风卷起他的衣角，那双眼中燃烧着不灭的火焰。今天，他要让所有人知道，什么是真正的强者！'
  },
  { 
    name: '细腻深沉', 
    description: '注重内心描写和氛围营造',
    tone: '内敛深沉，余韵悠长',
    sample: '雨丝轻轻落在窗台上，他望着窗外模糊的景色，心中泛起说不清道不明的情绪。'
  },
  { 
    name: '快节奏悬疑', 
    description: '紧凑刺激，充满悬念',
    tone: '紧张刺激，扣人心弦',
    sample: '门开了。他的心猛然提到嗓子眼——里面空无一人，只有桌上那张写着"下一个就是你"的纸条。'
  },
];

export function NovelCreationWizard({ onComplete, onCancel }: NovelCreationWizardProps) {
  const [step, setStep] = useState<WizardStep>('genre_input');
  const [genreInput, setGenreInput] = useState('');
  const [selectedWorld, setSelectedWorld] = useState<number | null>(null);
  const [selectedCharacters, setSelectedCharacters] = useState<number | null>(null);
  const [selectedStyle, setSelectedStyle] = useState<number | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);

  const handleStartGeneration = () => {
    if (!genreInput.trim()) return;
    setStep('generating_world');
    setIsGenerating(true);
    // 模拟生成过程
    setTimeout(() => {
      setIsGenerating(false);
      setStep('selecting_world');
    }, 2000);
  };

  const handleSelectWorld = (index: number) => {
    setSelectedWorld(index);
    setStep('generating_characters');
    setIsGenerating(true);
    setTimeout(() => {
      setIsGenerating(false);
      setStep('selecting_characters');
    }, 2000);
  };

  const handleSelectCharacters = (index: number) => {
    setSelectedCharacters(index);
    setStep('generating_style');
    setIsGenerating(true);
    setTimeout(() => {
      setIsGenerating(false);
      setStep('selecting_style');
    }, 2000);
  };

  const handleSelectStyle = (index: number) => {
    setSelectedStyle(index);
    setStep('completed');
  };

  const handleComplete = () => {
    // 构造最终数据
    const worldData = SAMPLE_WORLD_OPTIONS[selectedWorld!];
    const characterData = SAMPLE_CHARACTER_SETS[selectedCharacters!];
    const styleData = SAMPLE_STYLES[selectedStyle!];

    onComplete({
      worldBuilding: {
        id: '',
        story_id: '',
        concept: worldData.concept,
        rules: worldData.rules.map((r, i) => ({
          id: `rule_${i}`,
          name: r.name,
          description: r.description,
          rule_type: 'Custom' as const,
          importance: 8,
        })),
        history: worldData.history,
        cultures: worldData.cultures.map(c => ({
          name: c.name,
          description: c.description,
          customs: [],
          values: [],
        })),
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
      characters: characterData.map((c, i) => ({
        id: `char_${i}`,
        name: c.name,
        personality: c.personality,
        background: c.background,
        goals: c.goals,
        voice_style: '沉稳内敛', // 默认声音风格
      })),
      writingStyle: {
        id: '',
        story_id: '',
        name: styleData.name,
        description: styleData.description,
        tone: styleData.tone,
        pacing: 'Normal',
        custom_rules: [],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    });
  };

  const renderGenreInput = () => (
    <div className="space-y-6">
      <div className="text-center">
        <h2 className="text-2xl font-bold text-white mb-2">创建你的小说</h2>
        <p className="text-gray-400">告诉AI你想写什么类型的小说</p>
      </div>

      <div className="relative">
        <textarea
          value={genreInput}
          onChange={(e) => setGenreInput(e.target.value)}
          placeholder="小说类型：玄幻...商战...或随便定"
          className="w-full h-32 px-4 py-4 bg-cinema-800 border border-cinema-700 rounded-xl text-white placeholder-gray-500 focus:border-cinema-gold focus:outline-none resize-none text-lg"
        />
        <div className="absolute bottom-3 right-3 text-xs text-gray-500">
          {genreInput.length} 字
        </div>
      </div>

      <div className="flex justify-between">
        <Button variant="ghost" onClick={onCancel}>取消</Button>
        <Button 
          variant="primary" 
          onClick={handleStartGeneration}
          disabled={!genreInput.trim()}
          isLoading={isGenerating}
        >
          <Sparkles className="w-4 h-4 mr-2" />
          开始创作
        </Button>
      </div>
    </div>
  );

  const renderGenerating = (message: string) => (
    <div className="text-center py-12">
      <div className="relative w-20 h-20 mx-auto mb-6">
        <div className="absolute inset-0 border-4 border-cinema-700 rounded-full" />
        <div className="absolute inset-0 border-4 border-cinema-gold rounded-full border-t-transparent animate-spin" />
        <Sparkles className="absolute inset-0 m-auto w-8 h-8 text-cinema-gold" />
      </div>
      <h3 className="text-xl font-semibold text-white mb-2">{message}</h3>
      <p className="text-gray-400">AI正在发挥创意...</p>
    </div>
  );

  const renderWorldSelection = () => (
    <div className="space-y-6">
      <div className="text-center">
        <h2 className="text-2xl font-bold text-white mb-2">选择世界观</h2>
        <p className="text-gray-400">双击可编辑，点击选择</p>
      </div>

      <div className="grid gap-4">
        {SAMPLE_WORLD_OPTIONS.map((world, index) => (
          <Card
            key={world.id}
            hover
            className={`cursor-pointer transition-all ${
              selectedWorld === index ? 'ring-2 ring-cinema-gold' : ''
            }`}
            onClick={() => handleSelectWorld(index)}
          >
            <CardContent className="p-5">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-cinema-gold/10 flex items-center justify-center flex-shrink-0">
                  <Globe className="w-6 h-6 text-cinema-gold" />
                </div>
                <div className="flex-1">
                  <h3 className="font-semibold text-white mb-2">{world.concept}</h3>
                  <div className="space-y-2">
                    <div>
                      <span className="text-xs text-gray-500">核心规则：</span>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {world.rules.map((rule, i) => (
                          <span key={i} className="px-2 py-0.5 text-xs bg-cinema-800 rounded text-gray-300">
                            {rule.name}
                          </span>
                        ))}
                      </div>
                    </div>
                    <p className="text-sm text-gray-400 line-clamp-2">{world.history}</p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="flex justify-between">
        <Button variant="ghost" onClick={() => setStep('genre_input')}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          返回
        </Button>
        <Button variant="ghost" onClick={() => {}}>
          <RefreshCw className="w-4 h-4 mr-1" />
          重新生成
        </Button>
      </div>
    </div>
  );

  const renderCharacterSelection = () => (
    <div className="space-y-6">
      <div className="text-center">
        <h2 className="text-2xl font-bold text-white mb-2">选择角色谱</h2>
        <p className="text-gray-400">双击可编辑角色详情</p>
      </div>

      <div className="grid gap-4">
        {SAMPLE_CHARACTER_SETS.map((set, index) => (
          <Card
            key={index}
            hover
            className={`cursor-pointer transition-all ${
              selectedCharacters === index ? 'ring-2 ring-cinema-gold' : ''
            }`}
            onClick={() => handleSelectCharacters(index)}
          >
            <CardContent className="p-5">
              <div className="flex items-center gap-3 mb-4">
                <Users className="w-5 h-5 text-cinema-gold" />
                <span className="font-medium text-white">角色组合 {index + 1}</span>
              </div>
              <div className="grid gap-3">
                {set.map((char, i) => (
                  <div key={i} className="flex items-start gap-3 p-3 bg-cinema-800/50 rounded-lg">
                    <div className="w-8 h-8 rounded-full bg-cinema-700 flex items-center justify-center flex-shrink-0">
                      <span className="text-sm font-medium text-cinema-gold">{char.name[0]}</span>
                    </div>
                    <div>
                      <p className="font-medium text-white">{char.name}</p>
                      <p className="text-sm text-gray-400">{char.personality}</p>
                      <p className="text-xs text-gray-500 mt-1">{char.goals}</p>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="flex justify-between">
        <Button variant="ghost" onClick={() => setStep('selecting_world')}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          返回
        </Button>
      </div>
    </div>
  );

  const renderStyleSelection = () => (
    <div className="space-y-6">
      <div className="text-center">
        <h2 className="text-2xl font-bold text-white mb-2">选择文字风格</h2>
        <p className="text-gray-400">这将影响AI续写的文风</p>
      </div>

      <div className="grid gap-4">
        {SAMPLE_STYLES.map((style, index) => (
          <Card
            key={index}
            hover
            className={`cursor-pointer transition-all ${
              selectedStyle === index ? 'ring-2 ring-cinema-gold' : ''
            }`}
            onClick={() => handleSelectStyle(index)}
          >
            <CardContent className="p-5">
              <div className="flex items-center gap-3 mb-3">
                <PenTool className="w-5 h-5 text-cinema-gold" />
                <span className="font-semibold text-white">{style.name}</span>
              </div>
              <p className="text-sm text-gray-400 mb-3">{style.description}</p>
              <div className="p-3 bg-cinema-800/50 rounded-lg">
                <p className="text-xs text-gray-500 mb-1">示例：</p>
                <p className="text-sm text-gray-300 italic">"{style.sample}"</p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="flex justify-between">
        <Button variant="ghost" onClick={() => setStep('selecting_characters')}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          返回
        </Button>
      </div>
    </div>
  );

  const renderCompleted = () => (
    <div className="text-center py-12 space-y-6">
      <div className="w-20 h-20 mx-auto rounded-full bg-green-500/20 flex items-center justify-center">
        <Check className="w-10 h-10 text-green-500" />
      </div>
      
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">创作准备完成！</h2>
        <p className="text-gray-400">你的小说世界已经构建好了</p>
      </div>

      <div className="grid grid-cols-3 gap-4 max-w-lg mx-auto">
        <div className="p-4 bg-cinema-800 rounded-lg">
          <Globe className="w-6 h-6 text-cinema-gold mx-auto mb-2" />
          <p className="text-sm text-white">世界观</p>
          <p className="text-xs text-gray-500">已设置</p>
        </div>
        <div className="p-4 bg-cinema-800 rounded-lg">
          <Users className="w-6 h-6 text-cinema-gold mx-auto mb-2" />
          <p className="text-sm text-white">角色谱</p>
          <p className="text-xs text-gray-500">已设置</p>
        </div>
        <div className="p-4 bg-cinema-800 rounded-lg">
          <PenTool className="w-6 h-6 text-cinema-gold mx-auto mb-2" />
          <p className="text-sm text-white">文字风格</p>
          <p className="text-xs text-gray-500">已设置</p>
        </div>
      </div>

      <div className="flex justify-center gap-4">
        <Button variant="primary" onClick={handleComplete}>
          <BookOpen className="w-4 h-4 mr-2" />
          开始写作
        </Button>
      </div>
    </div>
  );

  return (
    <div className="fixed inset-0 bg-cinema-950/95 flex items-center justify-center p-8 z-50">
      <div className="w-full max-w-2xl">
        {/* Progress */}
        <div className="flex items-center justify-center gap-2 mb-8">
          {['genre', 'world', 'characters', 'style', 'done'].map((s, i) => {
            const stepIndex = ['genre_input', 'selecting_world', 'selecting_characters', 'selecting_style', 'completed'].indexOf(step);
            const isActive = i <= stepIndex;
            return (
              <React.Fragment key={s}>
                <div className={`
                  w-8 h-8 rounded-full flex items-center justify-center text-sm
                  ${isActive ? 'bg-cinema-gold text-cinema-900' : 'bg-cinema-800 text-gray-500'}
                `}>
                  {i + 1}
                </div>
                {i < 4 && (
                  <div className={`
                    w-8 h-0.5
                    ${i < stepIndex ? 'bg-cinema-gold' : 'bg-cinema-800'}
                  `} />
                )}
              </React.Fragment>
            );
          })}
        </div>

        {/* Content */}
        <Card className="min-h-[500px]">
          <CardContent className="p-8">
            {step === 'genre_input' && renderGenreInput()}
            {(step === 'generating_world' || step === 'generating_characters' || step === 'generating_style') && 
              renderGenerating('AI生成中...')
            }
            {step === 'selecting_world' && renderWorldSelection()}
            {step === 'selecting_characters' && renderCharacterSelection()}
            {step === 'selecting_style' && renderStyleSelection()}
            {step === 'completed' && renderCompleted()}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
