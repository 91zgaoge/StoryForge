import { useState } from 'react';
import { Plus, BookOpen, Download, Trash2, Edit3, ArrowRight, Check, X, FolderOpen } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useStories, useCreateStory, useDeleteStory, useUpdateStory } from '@/hooks/useStories';
import { useAppStore } from '@/stores/appStore';
import { ExportDialog } from '@/components/ExportDialog';
import { formatDate, truncateText } from '@/utils/format';
import type { Story } from '@/types/index';
import toast from 'react-hot-toast';

export function Stories() {
  const { data: stories = [], isLoading } = useStories();
  const createStory = useCreateStory();
  const deleteStory = useDeleteStory();
  const updateStory = useUpdateStory();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [exportStory, setExportStory] = useState<{ id: string; title: string } | null>(null);
  const [editingStory, setEditingStory] = useState<Story | null>(null);
  const [editForm, setEditForm] = useState({ title: '', description: '', genre: '' });

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
    setCurrentView('chapters');
  };

  const handleEditClick = (story: Story, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingStory(story);
    setEditForm({
      title: story.title,
      description: story.description || '',
      genre: story.genre || '',
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
      },
    }, {
      onSuccess: () => {
        setEditingStory(null);
      },
    });
  };

  const handleEditCancel = () => {
    setEditingStory(null);
    setEditForm({ title: '', description: '', genre: '' });
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
          <Button variant="ghost" size="sm" onClick={() => setCurrentView('chapters')}>
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
    </div>
  );
}
