import { useState } from 'react';
import { useAppStore } from '@/stores/appStore';
import { useCharacters, useCreateCharacter, useDeleteCharacter } from '@/hooks/useCharacters';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Users, Plus, Trash2 } from 'lucide-react';

export function Characters() {
  const currentStory = useAppStore((s) => s.currentStory);
  const { data: characters = [] } = useCharacters(currentStory?.id || null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const createCharacter = useCreateCharacter();
  const deleteCharacter = useDeleteCharacter();

  const handleCreate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!currentStory) return;

    const form = e.currentTarget;
    const formData = new FormData(form);

    createCharacter.mutate({
      story_id: currentStory.id,
      name: formData.get('name') as string,
      background: formData.get('background') as string || undefined,
    }, {
      onSuccess: () => {
        setIsModalOpen(false);
        form.reset();
      },
    });
  };

  const handleDelete = (id: string) => {
    if (confirm('确定要删除这个角色吗？')) {
      deleteCharacter.mutate(id);
    }
  };

  if (!currentStory) {
    return (
      <div className="p-8 flex items-center justify-center h-full">
        <Card>
          <CardContent className="p-8 text-center">
            <Users className="w-12 h-12 text-gray-600 mx-auto mb-4" />
            <h2 className="font-display text-xl font-semibold text-white mb-2">先选择一个故事</h2>
            <p className="text-gray-400">在故事库中选择一个故事来管理角色</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold text-white">角色管理</h1>
          <p className="text-gray-400">{currentStory.title} - 共 {characters.length} 个角色</p>
        </div>
        <Button variant="primary" onClick={() => setIsModalOpen(true)}>
          <Plus className="w-4 h-4" />
          添加角色
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {characters.map((char) => (
          <Card key={char.id} hover className="group">
            <CardContent className="p-6">
              <div className="flex items-center gap-4">
                <div className="w-14 h-14 rounded-full bg-cinema-velvet/20 flex items-center justify-center text-cinema-velvet font-display text-xl">
                  {char.name.charAt(0)}
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-display text-lg font-semibold text-white truncate">{char.name}</h3>
                  {char.personality && (
                    <p className="text-sm text-gray-400 mt-1 line-clamp-1">{char.personality}</p>
                  )}
                </div>
                <button
                  onClick={() => handleDelete(char.id)}
                  className="p-2 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-red-500/20 text-red-400 transition-all"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
              {char.background && (
                <p className="mt-4 text-sm text-gray-500 line-clamp-2">{char.background}</p>
              )}
            </CardContent>
          </Card>
        ))}

        {characters.length === 0 && (
          <div className="col-span-full text-center py-12">
            <Users className="w-16 h-16 text-gray-700 mx-auto mb-4" />
            <p className="text-gray-500">还没有角色，添加一个吧！</p>
          </div>
        )}
      </div>

      {/* Create Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card className="w-full max-w-md mx-4">
            <CardContent className="p-6">
              <h2 className="font-display text-xl font-bold text-white mb-4">添加角色</h2>

              <form onSubmit={handleCreate} className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">角色名称 *</label>
                  <input
                    name="name"
                    required
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                    placeholder="输入角色名称"
                  />
                </div>

                <div>
                  <label className="block text-sm text-gray-400 mb-1">背景故事</label>
                  <textarea
                    name="background"
                    rows={3}
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none resize-none"
                    placeholder="角色的背景故事..."
                  />
                </div>

                <div className="flex gap-3 pt-4">
                  <Button type="button" variant="ghost" onClick={() => setIsModalOpen(false)}>
                    取消
                  </Button>
                  <Button
                    type="submit"
                    variant="primary"
                    isLoading={createCharacter.isPending}
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
