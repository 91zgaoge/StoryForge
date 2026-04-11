import { useState, useEffect } from 'react';
import { FileText, Plus, ChevronRight, Save, Trash2, Search, Users } from 'lucide-react';
import toast from 'react-hot-toast';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useAppStore } from '@/stores/appStore';
import { useChapters, useCreateChapter, useUpdateChapter, useDeleteChapter } from '@/hooks/useChapters';
import { MonacoEditor } from '@/components/Editor';
import { VectorSearch } from '@/components/VectorSearch';
import { useCollaboration } from '@/hooks/useCollaboration';
import type { Chapter } from '@/types/index';

export function Chapters() {
  const currentStory = useAppStore((s) => s.currentStory);
  const currentUser = useAppStore((s) => s.currentUser);
  const { data: chapters = [], isLoading } = useChapters(currentStory?.id || null);
  const [selectedChapter, setSelectedChapter] = useState<Chapter | null>(null);
  const [editedTitle, setEditedTitle] = useState('');
  const [editedContent, setEditedContent] = useState('');
  const [editedOutline, setEditedOutline] = useState('');
  const [activeTab, setActiveTab] = useState<'content' | 'outline'>('content');
  const [showSearchPanel, setShowSearchPanel] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const createChapter = useCreateChapter();
  const updateChapter = useUpdateChapter();
  const deleteChapter = useDeleteChapter();

  // Collaboration hook
  const collab = useCollaboration({
    storyId: currentStory?.id || '',
    chapterId: selectedChapter?.id || '',
    userId: currentUser?.id || '',
    userName: currentUser?.name || 'Anonymous',
    onRemoteOperation: (op) => {
      // Apply remote operation to editor
      console.log('Remote operation:', op);
    },
    onUserJoined: (user) => {
      toast.success(`${user.user_name} 加入编辑`);
    },
    onUserLeft: (user) => {
      toast(`${user.user_name} 离开编辑`, { icon: '👋' });
    },
  });

  // Update edited content when chapter changes
  useEffect(() => {
    if (selectedChapter) {
      setEditedTitle(selectedChapter.title || '');
      setEditedContent(selectedChapter.content || '');
      setEditedOutline(selectedChapter.outline || '');
    }
  }, [selectedChapter]);

  const handleSave = () => {
    if (selectedChapter) {
      updateChapter.mutate({
        id: selectedChapter.id,
        updates: { title: editedTitle, content: editedContent, outline: editedOutline },
      });
    }
  };

  const handleCreate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!currentStory) return;

    const form = e.currentTarget;
    const formData = new FormData(form);

    createChapter.mutate({
      story_id: currentStory.id,
      chapter_number: chapters.length + 1,
      title: formData.get('title') as string || undefined,
      outline: formData.get('outline') as string || undefined,
    }, {
      onSuccess: () => {
        setIsModalOpen(false);
        form.reset();
      },
    });
  };

  const handleDelete = (chapterId: string) => {
    if (confirm('确定要删除这个章节吗？')) {
      deleteChapter.mutate(chapterId);
      if (selectedChapter?.id === chapterId) {
        setSelectedChapter(null);
      }
    }
  };

  if (!currentStory) {
    return (
      <div className="p-8 flex items-center justify-center h-full">
        <Card>
          <CardContent className="p-8 text-center">
            <FileText className="w-12 h-12 text-gray-600 mx-auto mb-4" />
            <h2 className="font-display text-xl font-semibold text-white mb-2">先选择一个故事</h2>
            <p className="text-gray-400">在故事库中选择一个故事来管理章节</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold text-white">章节工坊</h1>
          <p className="text-gray-400">{currentStory.title} - 共 {chapters.length} 章</p>
        </div>
        <div className="flex items-center gap-3">
          {selectedChapter && (
            <>
              <Button
                variant={collab.isConnected ? 'primary' : 'ghost'}
                size="sm"
                onClick={() => collab.isConnected ? collab.disconnect() : collab.connect()}
                className="gap-2"
              >
                <Users className="w-4 h-4" />
                {collab.isConnected ? '协作中' : '开启协作'}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowSearchPanel(!showSearchPanel)}
                className="gap-2"
              >
                <Search className="w-4 h-4" />
                智能搜索
              </Button>
            </>
          )}
          <Button variant="primary" onClick={() => setIsModalOpen(true)}>
            <Plus className="w-4 h-4" />
            新建章节
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Chapter List */}
        <div className="space-y-3 max-h-[calc(100vh-200px)] overflow-y-auto">
          {isLoading ? (
            <div className="text-center py-8 text-gray-500">加载中...</div>
          ) : chapters.length === 0 ? (
            <div className="text-center py-8 text-gray-500">还没有章节，创建一个新的吧！</div>
          ) : (
            chapters.map((chapter) => (
              <div
                key={chapter.id}
                onClick={() => setSelectedChapter(chapter)}
                className={`w-full text-left p-4 rounded-xl border transition-all cursor-pointer group ${
                  selectedChapter?.id === chapter.id
                    ? 'bg-cinema-gold/10 border-cinema-gold/30'
                    : 'bg-cinema-850/50 border-cinema-700/50 hover:border-cinema-700'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <p className="text-sm text-gray-500">第 {chapter.chapter_number} 章</p>
                    <h4 className="font-display text-white truncate">
                      {chapter.title || '未命名章节'}
                    </h4>
                    {chapter.word_count !== undefined && chapter.word_count > 0 && (
                      <p className="text-xs text-gray-600 mt-1">{chapter.word_count} 字</p>
                    )}
                  </div>
                  <div className="flex items-center gap-2">
                    <ChevronRight className={`w-4 h-4 text-gray-600 transition-transform ${
                      selectedChapter?.id === chapter.id ? 'rotate-90' : ''
                    }`} />
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(chapter.id);
                      }}
                      className="p-1.5 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-red-500/20 text-red-400 transition-all"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Editor Area */}
        <Card className="lg:col-span-2 min-h-[500px] relative">
          <CardContent className="p-6 h-full flex flex-col">
            {selectedChapter ? (
              <>
                <div className="flex items-center justify-between mb-4">
                  <input
                    type="text"
                    value={editedTitle}
                    onChange={(e) => setEditedTitle(e.target.value)}
                    placeholder="章节标题"
                    className="text-2xl font-display bg-transparent border-none outline-none text-white placeholder-gray-600 flex-1"
                  />
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={handleSave}
                    isLoading={updateChapter.isPending}
                    className="gap-2"
                  >
                    <Save className="w-4 h-4" />
                    保存
                  </Button>
                </div>
                {/* Tabs */}
                <div className="flex gap-2 mb-4">
                  <button
                    onClick={() => setActiveTab('content')}
                    className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                      activeTab === 'content'
                        ? 'bg-cinema-gold/20 text-cinema-gold'
                        : 'text-gray-400 hover:text-white'
                    }`}
                  >
                    正文
                  </button>
                  <button
                    onClick={() => setActiveTab('outline')}
                    className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                      activeTab === 'outline'
                        ? 'bg-cinema-gold/20 text-cinema-gold'
                        : 'text-gray-400 hover:text-white'
                    }`}
                  >
                    大纲
                  </button>
                </div>

                <div className="flex-1 min-h-0 border border-cinema-800 rounded-xl overflow-hidden">
                  {activeTab === 'content' ? (
                    <MonacoEditor
                      value={editedContent}
                      onChange={setEditedContent}
                      onSave={handleSave}
                      placeholder="开始写作..."
                    />
                  ) : (
                    <MonacoEditor
                      value={editedOutline}
                      onChange={setEditedOutline}
                      onSave={handleSave}
                      placeholder="本章大纲..."
                    />
                  )}
                </div>
                <div className="mt-4 flex items-center justify-between text-sm text-gray-500">
                  <span>{editedContent.length} 字符</span>
                  <span>最后更新: {new Date(selectedChapter.updated_at).toLocaleString()}</span>
                </div>
              </>
            ) : (
              <div className="h-full flex items-center justify-center text-gray-500">
                <div className="text-center">
                  <FileText className="w-12 h-12 mx-auto mb-4 opacity-50" />
                  <p>选择一个章节开始编辑</p>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Vector Search Panel */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="font-display text-lg font-semibold text-white flex items-center gap-2">
              <Search className="w-5 h-5 text-cinema-gold" />
              智能搜索
            </h2>
            <button
              onClick={() => setShowSearchPanel(!showSearchPanel)}
              className="text-sm text-gray-400 hover:text-white transition-colors"
            >
              {showSearchPanel ? '收起' : '展开'}
            </button>
          </div>

          {showSearchPanel && currentStory && (
            <VectorSearch storyId={currentStory.id} />
          )}

          {/* Collaboration Panel */}
          {collab.isConnected && (
            <div className="mt-6">
              <h2 className="font-display text-lg font-semibold text-white flex items-center gap-2 mb-3">
                <Users className="w-5 h-5 text-cinema-gold" />
                协作者
              </h2>
              <div className="space-y-2">
                {collab.participants.map((p) => (
                  <div key={p.user_id} className="flex items-center gap-2 text-sm">
                    <div className="w-2 h-2 rounded-full bg-green-500" />
                    <span className="text-gray-300">{p.user_name}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Create Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card className="w-full max-w-md mx-4">
            <CardContent className="p-6">
              <h2 className="font-display text-xl font-bold text-white mb-4">新建章节</h2>

              <form onSubmit={handleCreate} className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">章节标题</label>
                  <input
                    name="title"
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                    placeholder={`第 ${chapters.length + 1} 章`}
                  />
                </div>

                <div>
                  <label className="block text-sm text-gray-400 mb-1">大纲/概要</label>
                  <textarea
                    name="outline"
                    rows={3}
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none resize-none"
                    placeholder="本章主要内容..."
                  />
                </div>

                <div className="flex gap-3 pt-4">
                  <Button type="button" variant="ghost" onClick={() => setIsModalOpen(false)}>
                    取消
                  </Button>
                  <Button
                    type="submit"
                    variant="primary"
                    isLoading={createChapter.isPending}
                  >
                    创建
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}
