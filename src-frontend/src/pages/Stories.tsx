import { useState, useRef, useEffect } from 'react';
import { Plus, BookOpen, Download, Trash2, Edit3, ArrowRight, Check, X, FolderOpen, Sparkles, Loader2, Palette, ChevronDown, Wand2 } from 'lucide-react';
import { useWorkflowProgress } from '@/hooks/useWorkflowProgress';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useStories, useCreateStory, useDeleteStory, useUpdateStory } from '@/hooks/useStories';
import { useAppStore } from '@/stores/appStore';
import { ExportDialog } from '@/components/ExportDialog';
import { formatDate, truncateText } from '@/utils/format';
import type { Story } from '@/types/index';
import toast from 'react-hot-toast';
import { runCreationWorkflow, listStyleDnas, setStoryStyleDna, analyzeStyleSample } from '@/services/tauri';
import { useQuery, useQueryClient } from '@tanstack/react-query';

export function Stories() {
  const { data: stories = [], isLoading } = useStories();
  const createStory = useCreateStory();
  const deleteStory = useDeleteStory();
  const updateStory = useUpdateStory();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [exportStory, setExportStory] = useState<{ id: string; title: string } | null>(null);
  const [editingStory, setEditingStory] = useState<Story | null>(null);
  const [editForm, setEditForm] = useState({ title: '', description: '', genre: '', methodology_id: '', methodology_step: 1 });
  const [creatingStoryId, setCreatingStoryId] = useState<string | null>(null);
  const [styleDnaModalStory, setStyleDnaModalStory] = useState<Story | null>(null);
  const [showStyleSampleInput, setShowStyleSampleInput] = useState(false);
  const [styleSampleText, setStyleSampleText] = useState('');
  const [isAnalyzingStyle, setIsAnalyzingStyle] = useState(false);
  const queryClient = useQueryClient();
  const [creationMode, setCreationMode] = useState<'ai_only' | 'ai_draft_human_edit' | 'human_draft_ai_polish'>('ai_only');
  const { progress, isActive: isWorkflowActive, startListening, stopListening } = useWorkflowProgress();
  const [showAiMenu, setShowAiMenu] = useState<string | null>(null);
  const aiMenuRef = useRef<HTMLDivElement>(null);

  const { data: styleDnas = [] } = useQuery({
    queryKey: ['style-dnas'],
    queryFn: listStyleDnas,
    staleTime: 5 * 60 * 1000,
  });

  const currentStory = useAppStore((s) => s.currentStory);
  const setCurrentStory = useAppStore((s) => s.setCurrentStory);
  const setCurrentView = useAppStore((s) => s.setCurrentView);

  const handleCreate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const formData = new FormData(form);

    createStory.mutate({
      title: formData.get('title') as string,
      description: formData.get('description') as string,
      genre: formData.get('genre') as string,
    }, {
      onSuccess: () => {
        setIsModalOpen(false);
        form.reset();
      },
    });
  };

  const handleSelectStory = (story: Story) => {
    setCurrentStory(story);
    toast.success(`已选择 "${story.title}"`);
  };

  const handleContinueStory = (story: Story) => {
    setCurrentStory(story);
    setCurrentView('scenes');
  };

  const handleEditClick = (story: Story, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingStory(story);
    setEditForm({
      title: story.title,
      description: story.description || '',
      genre: story.genre || '',
      methodology_id: story.methodology_id || '',
      methodology_step: story.methodology_step || 1,
    });
  };

  const handleEditSave = () => {
    if (!editingStory) return;

    updateStory.mutate({
      id: editingStory.id,
      updates: {
        title: editForm.title,
        description: editForm.description || undefined,
        genre: editForm.genre || undefined,
        methodology_id: editForm.methodology_id || undefined,
        methodology_step: editForm.methodology_id ? editForm.methodology_step : undefined,
      },
    }, {
      onSuccess: () => {
        setEditingStory(null);
      },
    });
  };

  const handleEditCancel = () => {
    setEditingStory(null);
    setEditForm({ title: '', description: '', genre: '', methodology_id: '', methodology_step: 1 });
  };

  const handleDelete = (storyId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm('确定要删除这个故事吗？此操作不可撤销。')) {
      deleteStory.mutate(storyId);
      if (currentStory?.id === storyId) {
        setCurrentStory(null);
      }
    }
  };

  const handleQuickCreate = async (story: Story, e: React.MouseEvent) => {
    e.stopPropagation();
    setShowAiMenu(null);
    setCreatingStoryId(story.id);
    startListening();
    try {
      const result = await runCreationWorkflow(story.id, creationMode, story.description || story.title);
      if (result.success) {
        if (creationMode === 'ai_draft_human_edit') {
          toast.success(`AI 初稿已完成！请在幕前编辑后继续。已完成 ${result.completed_phases.length} 个阶段`);
        } else if (creationMode === 'human_draft_ai_polish') {
          toast.success(`AI 润色完成！已完成 ${result.completed_phases.length} 个阶段`);
        } else {
          toast.success(`一键创作完成！已完成 ${result.completed_phases.length} 个阶段`);
        }
      } else {
        if (creationMode === 'ai_draft_human_edit' && result.current_phase === '写作') {
          toast.success(`AI 初稿已生成，请切换到幕前编辑`);
        } else {
          toast.error(`创作未完成: ${result.error || '未知错误'}`);
        }
      }
    } catch (err: any) {
      toast.error(`创作失败: ${err?.message || String(err)}`);
    } finally {
      setCreatingStoryId(null);
      stopListening();
    }
  };

  const handleWizardCreate = (story: Story, e: React.MouseEvent) => {
    e.stopPropagation();
    setShowAiMenu(null);
    setCurrentStory(story);
    setCurrentView('creation-wizard');
  };

  if (isLoading) {
    return (
      <div className="p-8 flex items-center justify-center h-full">
        <div className="loading-reel" />
      </div>
    );
  }

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold text-white">故事库</h1>
          <p className="text-gray-400">管理和创作你的故事</p>
        </div>
        <Button variant="primary" onClick={() => setIsModalOpen(true)}>
          <Plus className="w-4 h-4" />
          新建故事
        </Button>
      </div>

      {/* Current Story Indicator */}
      {currentStory && (
        <div className="p-4 rounded-xl bg-cinema-gold/10 border border-cinema-gold/30 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-cinema-gold/20 flex items-center justify-center">
              <BookOpen className="w-5 h-5 text-cinema-gold" />
            </div>
            <div>
              <p className="text-sm text-cinema-gold">当前编辑</p>
              <p className="font-display font-semibold text-white">{currentStory.title}</p>
            </div>
          </div>
          <Button variant="ghost" size="sm" onClick={() => setCurrentView('scenes')}>
            继续创作
            <ArrowRight className="w-4 h-4 ml-1" />
          </Button>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {stories.map((story) => (
          <Card
            key={story.id}
            hover
            className={`group cursor-pointer transition-all ${
              currentStory?.id === story.id ? 'ring-2 ring-cinema-gold/50' : ''
            }`}
            onClick={() => handleSelectStory(story)}
          >
            <CardContent className="p-6">
              {editingStory?.id === story.id ? (
                // Edit Mode
                <div className="space-y-3" onClick={(e) => e.stopPropagation()}>
                  <input
                    type="text"
                    value={editForm.title}
                    onChange={(e) => setEditForm({ ...editForm, title: e.target.value })}
                    className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                    placeholder="标题"
                  />
                  <select
                    value={editForm.genre}
                    onChange={(e) => setEditForm({ ...editForm, genre: e.target.value })}
                    className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                  >
                    <option value="">选择类型</option>
                    <option value="科幻">科幻</option>
                    <option value="奇幻">奇幻</option>
                    <option value="悬疑">悬疑</option>
                    <option value="言情">言情</option>
                    <option value="历史">历史</option>
                    <option value="武侠">武侠</option>
                    <option value="现代">现代</option>
                    <option value="其他">其他</option>
                  </select>
                  <select
                    value={editForm.methodology_id}
                    onChange={(e) => setEditForm({ ...editForm, methodology_id: e.target.value })}
                    className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                  >
                    <option value="">选择创作方法论（可选）</option>
                    <option value="snowflake">雪花法</option>
                    <option value="scene_beat">场景节拍</option>
                    <option value="hero_journey">英雄之旅</option>
                    <option value="character_depth">人物深度</option>
                  </select>
                  {editForm.methodology_id === 'snowflake' && (
                    <select
                      value={editForm.methodology_step}
                      onChange={(e) => setEditForm({ ...editForm, methodology_step: Number(e.target.value) })}
                      className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                    >
                      {Array.from({ length: 10 }, (_, i) => (
                        <option key={i + 1} value={i + 1}>
                          步骤 {i + 1}: {['一句话概括', '一段式概括', '人物概述', '一页纸大纲', '人物详细背景', '四页纸大纲', '人物完整档案', '场景清单', '场景扩展', '初稿写作'][i]}
                        </option>
                      ))}
                    </select>
                  )}
                  <textarea
                    value={editForm.description}
                    onChange={(e) => setEditForm({ ...editForm, description: e.target.value })}
                    rows={2}
                    className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-none"
                    placeholder="描述"
                  />
                  <div className="flex gap-2">
                    <Button variant="ghost" size="sm" onClick={handleEditCancel}>
                      <X className="w-4 h-4" />
                    </Button>
                    <Button variant="primary" size="sm" onClick={handleEditSave}>
                      <Check className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              ) : (
                // View Mode
                <>
                  <div className="flex items-start gap-4">
                    <div className="w-12 h-12 rounded-xl bg-cinema-gold/10 flex items-center justify-center">
                      <BookOpen className="w-6 h-6 text-cinema-gold" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="font-display text-lg font-semibold text-white truncate">
                        {story.title}
                      </h3>
                      <p className="text-sm text-gray-400 mt-1">
                        {story.genre || '未分类'} · {story.chapter_count || 0} 章
                      </p>
                      {story.description && (
                        <p className="text-sm text-gray-500 mt-2 line-clamp-2">
                          {truncateText(story.description, 100)}
                        </p>
                      )}
                      <p className="text-xs text-gray-600 mt-3">
                        更新于 {formatDate(story.updated_at)}
                      </p>
                    </div>
                  </div>

                  <div className="mt-4 pt-4 border-t border-cinema-700 flex flex-wrap gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Button
                      variant="primary"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleContinueStory(story);
                      }}
                    >
                      <FolderOpen className="w-4 h-4 mr-1" />
                      打开
                    </Button>
                    <div className="relative" ref={showAiMenu === story.id ? aiMenuRef : undefined}>
                      <Button
                        variant="ghost"
                        size="sm"
                        disabled={creatingStoryId === story.id}
                        onClick={(e) => {
                          e.stopPropagation();
                          setShowAiMenu(showAiMenu === story.id ? null : story.id);
                        }}
                        title="AI 创作菜单"
                      >
                        {creatingStoryId === story.id ? (
                          <Loader2 className="w-4 h-4 mr-1 animate-spin" />
                        ) : (
                          <Sparkles className="w-4 h-4 mr-1 text-cinema-gold" />
                        )}
                        AI 创作
                        <ChevronDown className="w-3 h-3 ml-1" />
                      </Button>

                      {showAiMenu === story.id && (
                        <div className="absolute right-0 mt-1 w-56 bg-cinema-800 border border-cinema-700 rounded-xl shadow-xl z-50 overflow-hidden">
                          <div className="p-2 space-y-1">
                            <select
                              value={creationMode}
                              onChange={(e) => setCreationMode(e.target.value as typeof creationMode)}
                              onClick={(e) => e.stopPropagation()}
                              className="w-full px-2 py-1.5 bg-cinema-900 border border-cinema-700 rounded-lg text-xs text-white focus:border-cinema-gold focus:outline-none"
                              title="选择创作模式"
                            >
                              <option value="ai_only">AI 全自动</option>
                              <option value="ai_draft_human_edit">AI 初稿 + 我精修</option>
                              <option value="human_draft_ai_polish">我初稿 + AI 润色</option>
                            </select>
                            <button
                              onClick={(e) => handleQuickCreate(story, e)}
                              className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-gray-300 hover:bg-cinema-700 transition-colors text-left"
                            >
                              <Sparkles className="w-4 h-4 text-cinema-gold" />
                              <div>
                                <div className="text-white text-sm">快速创作</div>
                                <div className="text-[10px] text-gray-500">
                                  {creationMode === 'ai_only' ? 'AI 全自动生成' : creationMode === 'ai_draft_human_edit' ? 'AI 初稿 + 我精修' : '我初稿 + AI 润色'}
                                </div>
                              </div>
                            </button>
                            <button
                              onClick={(e) => handleWizardCreate(story, e)}
                              className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-gray-300 hover:bg-cinema-700 transition-colors text-left"
                            >
                              <Wand2 className="w-4 h-4 text-cinema-gold" />
                              <div>
                                <div className="text-white text-sm">向导创作</div>
                                <div className="text-[10px] text-gray-500">分步选择 AI 生成选项</div>
                              </div>
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        setStyleDnaModalStory(story);
                      }}
                      title="选择写作风格 DNA"
                    >
                      <Palette className="w-4 h-4 mr-1" />
                      风格
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        setExportStory({ id: story.id, title: story.title });
                      }}
                    >
                      <Download className="w-4 h-4 mr-1" />
                      导出
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => handleEditClick(story, e)}
                    >
                      <Edit3 className="w-4 h-4 mr-1" />
                      编辑
                    </Button>
                    <Button
                      variant="danger"
                      size="sm"
                      onClick={(e) => handleDelete(story.id, e)}
                    >
                      <Trash2 className="w-4 h-4 mr-1" />
                      删除
                    </Button>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        ))}

        {stories.length === 0 && (
          <div className="col-span-full">
            <Card className="py-12">
              <CardContent className="text-center">
                <BookOpen className="w-16 h-16 text-cinema-700 mx-auto mb-4" />
                <h3 className="font-display text-xl font-semibold text-white mb-2">
                  开始你的创作之旅
                </h3>
                <p className="text-gray-500 max-w-md mx-auto mb-6">
                  你还没有创建任何故事。点击"新建故事"开始创作吧！
                </p>
                <Button variant="primary" onClick={() => setIsModalOpen(true)}>
                  <Plus className="w-4 h-4 mr-2" />
                  创建第一个故事
                </Button>
              </CardContent>
            </Card>
          </div>
        )}
      </div>

      {/* Create Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card className="w-full max-w-md mx-4">
            <CardContent className="p-6">
              <h2 className="font-display text-xl font-bold text-white mb-4">新建故事</h2>
              
              <form onSubmit={handleCreate} className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">标题</label>
                  <input
                    name="title"
                    required
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                  />
                </div>
                
                <div>
                  <label className="block text-sm text-gray-400 mb-1">类型</label>
                  <select
                    name="genre"
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                  >
                    <option value="">选择类型</option>
                    <option value="科幻">科幻</option>
                    <option value="奇幻">奇幻</option>
                    <option value="悬疑">悬疑</option>
                    <option value="言情">言情</option>
                    <option value="历史">历史</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm text-gray-400 mb-1">描述</label>
                  <textarea
                    name="description"
                    rows={3}
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none resize-none"
                  />
                </div>
                
                <div className="flex gap-3 pt-4">
                  <Button type="button" variant="ghost" onClick={() => setIsModalOpen(false)}>
                    取消
                  </Button>
                  <Button 
                    type="submit" 
                    variant="primary"
                    isLoading={createStory.isPending}
                  >
                    创建
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Export Dialog */}
      {exportStory && (
        <ExportDialog
          storyId={exportStory.id}
          storyTitle={exportStory.title}
          isOpen={!!exportStory}
          onClose={() => setExportStory(null)}
        />
      )}

      {/* Workflow Progress Modal */}
      {isWorkflowActive && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
          <Card className="w-full max-w-md mx-4">
            <CardContent className="p-6 space-y-4">
              <div className="flex items-center gap-3">
                <Loader2 className="w-6 h-6 text-cinema-gold animate-spin" />
                <div>
                  <h2 className="font-display text-lg font-bold text-white">AI 一键创作中</h2>
                  <p className="text-sm text-gray-400">{progress?.message || '准备中...'}</p>
                </div>
              </div>

              {/* Progress Bar */}
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-cinema-gold font-medium">{progress?.phase || '启动'}</span>
                  <span className="text-gray-400 font-mono">{Math.round((progress?.progress ?? 0) * 100)}%</span>
                </div>
                <div className="h-2 bg-cinema-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-cinema-gold rounded-full transition-all duration-500"
                    style={{ width: `${Math.round((progress?.progress ?? 0) * 100)}%` }}
                  />
                </div>
              </div>

              {/* Phase Indicators */}
              <div className="flex items-center justify-between text-xs">
                {['构思', '大纲', '写作', '审阅', '入库'].map((phase, idx) => {
                  const thresholds = [0.0, 0.15, 0.5, 0.7, 1.0];
                  const currentProgress = progress?.progress ?? 0;
                  const isActive = currentProgress >= thresholds[idx] && (idx === 4 || currentProgress < thresholds[idx + 1]);
                  const isDone = currentProgress >= thresholds[idx] + (idx < 4 ? 0.15 : 0);
                  return (
                    <div key={phase} className="flex flex-col items-center gap-1">
                      <div
                        className={`w-2 h-2 rounded-full ${
                          isDone ? 'bg-cinema-gold' : isActive ? 'bg-cinema-gold/60 animate-pulse' : 'bg-cinema-700'
                        }`}
                      />
                      <span className={isDone || isActive ? 'text-cinema-gold' : 'text-gray-600'}>{phase}</span>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* StyleDNA Selector Modal */}
      {styleDnaModalStory && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card className="w-full max-w-lg mx-4">
            <CardContent className="p-6">
              <h2 className="font-display text-xl font-bold text-white mb-2">
                选择写作风格
              </h2>
              <p className="text-sm text-gray-400 mb-4">
                为「{styleDnaModalStory.title}」选择一种风格 DNA，AI 创作时将模仿该风格。
              </p>
              <div className="space-y-2 max-h-80 overflow-y-auto">
                <button
                  onClick={async () => {
                    await setStoryStyleDna(styleDnaModalStory.id, null);
                    setStyleDnaModalStory(null);
                    toast.success('已清除风格设置');
                  }}
                  className={`w-full p-3 rounded-lg text-left transition-colors border ${
                    !styleDnaModalStory.style_dna_id
                      ? 'bg-cinema-gold/20 border-cinema-gold/50'
                      : 'bg-cinema-800 border-transparent hover:bg-cinema-700'
                  }`}
                >
                  <div className="font-medium text-white">默认风格</div>
                  <div className="text-xs text-gray-400">不指定特定风格，使用通用创作风格</div>
                </button>
                {styleDnas.map((dna) => (
                  <button
                    key={dna.id}
                    onClick={async () => {
                      await setStoryStyleDna(styleDnaModalStory.id, dna.id);
                      setStyleDnaModalStory(null);
                      toast.success(`已设置风格：${dna.name}`);
                    }}
                    className={`w-full p-3 rounded-lg text-left transition-colors border ${
                      styleDnaModalStory.style_dna_id === dna.id
                        ? 'bg-cinema-gold/20 border-cinema-gold/50'
                        : 'bg-cinema-800 border-transparent hover:bg-cinema-700'
                    }`}
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
              <div className="flex justify-between mt-4">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowStyleSampleInput(true)}
                >
                  <Sparkles className="w-4 h-4 mr-1 text-cinema-gold" />
                  从文本生成风格
                </Button>
                <Button variant="ghost" onClick={() => setStyleDnaModalStory(null)}>
                  取消
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Style Sample Input Modal */}
      {showStyleSampleInput && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card className="w-full max-w-lg mx-4">
            <CardContent className="p-6 space-y-4">
              <h2 className="font-display text-xl font-bold text-white">
                从文本样例生成风格
              </h2>
              <p className="text-sm text-gray-400">
                粘贴一段你喜欢的文字（300-3000字），AI 将分析其风格特征并生成风格 DNA。
              </p>
              <textarea
                value={styleSampleText}
                onChange={(e) => setStyleSampleText(e.target.value)}
                rows={8}
                placeholder="在此粘贴文本样例..."
                className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-none"
              />
              <div className="flex justify-end gap-2">
                <Button variant="ghost" onClick={() => {
                  setShowStyleSampleInput(false);
                  setStyleSampleText('');
                }}>
                  取消
                </Button>
                <Button
                  variant="primary"
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
        </div>
      )}
    </div>
  );
}
