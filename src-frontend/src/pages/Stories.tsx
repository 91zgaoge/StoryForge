import { useState } from 'react';
import { Plus, BookOpen, Download, Trash2 } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useStories, useCreateStory, useDeleteStory } from '@/hooks/useStories';
import { ExportDialog } from '@/components/ExportDialog';
import { formatDate, truncateText } from '@/utils/format';

export function Stories() {
  const { data: stories = [], isLoading } = useStories();
  const createStory = useCreateStory();
  const deleteStory = useDeleteStory();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [exportStory, setExportStory] = useState<{ id: string; title: string } | null>(null);

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

  if (isLoading) {
    return <div className="p-8">加载中...</div>;
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

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {stories.map((story) => (
          <Card key={story.id} hover className="group cursor-pointer">
            <CardContent className="p-6">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-cinema-gold/10 flex items-center justify-center">
                  <BookOpen className="w-6 h-6 text-cinema-gold" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-display text-lg font-semibold text-white truncate">
                    {story.title}
                  </h3>
                  <p className="text-sm text-gray-400 mt-1">
                    {story.genre || '未分类'}
                  </p>
                  {story.description && (
                    <p className="text-sm text-gray-500 mt-2 line-clamp-2">
                      {truncateText(story.description, 100)}
                    </p>
                  )}
                  <p className="text-xs text-gray-600 mt-3">
                    创建于 {formatDate(story.created_at)}
                  </p>
                </div>
              </div>
              
              <div className="mt-4 pt-4 border-t border-cinema-700 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setExportStory({ id: story.id, title: story.title })}
                >
                  <Download className="w-4 h-4 mr-1" />
                  导出
                </Button>
                <Button variant="ghost" size="sm">编辑</Button>
                <Button
                  variant="danger"
                  size="sm"
                  onClick={() => deleteStory.mutate(story.id)}
                >
                  <Trash2 className="w-4 h-4 mr-1" />
                  删除
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
        
        {stories.length === 0 && (
          <div className="col-span-full text-center py-12">
            <p className="text-gray-500">还没有故事，开始创作吧！</p>
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
