import { useState } from 'react';
import { Plus, BookOpen, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { StoryTimeline } from '@/components/StoryTimeline';
import { SceneEditor } from '@/components/SceneEditor';
import { useScenes, useCreateScene, useUpdateScene, useDeleteScene, useReorderScenes } from '@/hooks/useScenes';
import { useCharacters } from '@/hooks/useCharacters';
import { useAppStore } from '@/stores/appStore';
import type { Scene } from '@/types';
import toast from 'react-hot-toast';

export function Scenes() {
  const currentStory = useAppStore((s) => s.currentStory);
  const [selectedSceneId, setSelectedSceneId] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);

  const { data: scenes = [], isLoading } = useScenes(currentStory?.id || null);
  const { data: characters = [] } = useCharacters(currentStory?.id || null);
  
  const createScene = useCreateScene();
  const updateScene = useUpdateScene();
  const deleteScene = useDeleteScene();
  const reorderScenes = useReorderScenes();

  const selectedScene = scenes.find((s) => s.id === selectedSceneId) || null;

  const handleCreateScene = () => {
    if (!currentStory) {
      toast.error('请先选择一个故事');
      return;
    }

    const nextSequence = scenes.length > 0 
      ? Math.max(...scenes.map(s => s.sequence_number)) + 1 
      : 1;

    createScene.mutate(
      {
        storyId: currentStory.id,
        sequenceNumber: nextSequence,
        title: `场景 ${nextSequence}`,
      },
      {
        onSuccess: (newScene) => {
          toast.success('场景创建成功');
          setSelectedSceneId(newScene.id);
          setIsEditing(true);
        },
      }
    );
  };

  const handleSelectScene = (scene: Scene) => {
    setSelectedSceneId(scene.id);
    setIsEditing(false);
  };

  const handleEditScene = (scene: Scene) => {
    setSelectedSceneId(scene.id);
    setIsEditing(true);
  };

  const handleSaveScene = (updates: Partial<Scene>) => {
    if (!selectedScene || !currentStory) return;

    updateScene.mutate(
      {
        sceneId: selectedScene.id,
        storyId: currentStory.id,
        updates: {
          title: updates.title,
          dramatic_goal: updates.dramatic_goal,
          external_pressure: updates.external_pressure,
          conflict_type: updates.conflict_type,
          characters_present: updates.characters_present,
          character_conflicts: updates.character_conflicts,
          content: updates.content,
          setting_location: updates.setting_location,
          setting_time: updates.setting_time,
          setting_atmosphere: updates.setting_atmosphere,
        },
      },
      {
        onSuccess: () => {
          toast.success('场景已保存');
          setIsEditing(false);
        },
      }
    );
  };

  const handleDeleteScene = (sceneId: string) => {
    if (!currentStory) return;
    
    if (confirm('确定要删除这个场景吗？此操作不可撤销。')) {
      deleteScene.mutate(
        { sceneId, storyId: currentStory.id },
        {
          onSuccess: () => {
            toast.success('场景已删除');
            if (selectedSceneId === sceneId) {
              setSelectedSceneId(null);
              setIsEditing(false);
            }
          },
        }
      );
    }
  };

  const handleReorderScenes = (sceneIds: string[]) => {
    if (!currentStory) return;
    
    reorderScenes.mutate({
      storyId: currentStory.id,
      sceneIds,
    });
  };

  if (!currentStory) {
    return (
      <div className="p-8 flex flex-col items-center justify-center h-full text-center">
        <BookOpen className="w-16 h-16 text-cinema-700 mb-4" />
        <h2 className="font-display text-xl font-semibold text-white mb-2">
          还没有选择故事
        </h2>
        <p className="text-gray-500 max-w-md mb-6">
          请先选择一个故事，然后开始创建场景
        </p>
        <Button 
          variant="primary" 
          onClick={() => useAppStore.getState().setCurrentView('stories')}
        >
          去故事库
        </Button>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="p-8 flex items-center justify-center h-full">
        <div className="loading-reel" />
      </div>
    );
  }

  return (
    <div className="h-full flex">
      {/* Left Panel - Timeline */}
      <div className="w-1/2 min-w-[400px] max-w-[600px] border-r border-cinema-700 bg-cinema-900/50">
        <div className="h-full p-6 overflow-auto">
          <StoryTimeline
            scenes={scenes}
            currentSceneId={selectedSceneId}
            characters={characters}
            onSelectScene={handleSelectScene}
            onCreateScene={handleCreateScene}
            onDeleteScene={handleDeleteScene}
            onReorderScenes={handleReorderScenes}
            onEditScene={handleEditScene}
          />
        </div>
      </div>

      {/* Right Panel - Editor */}
      <div className="flex-1 bg-cinema-950">
        {isEditing && selectedScene ? (
          <div className="h-full p-6">
            <SceneEditor
              scene={selectedScene}
              characters={characters}
              onSave={handleSaveScene}
              onCancel={() => setIsEditing(false)}
            />
          </div>
        ) : selectedScene ? (
          <div className="h-full flex flex-col">
            {/* Scene Preview */}
            <div className="flex-1 p-8 overflow-auto">
              <div className="max-w-3xl mx-auto">
                <h1 className="text-2xl font-bold text-white mb-4">
                  {selectedScene.title || `场景 ${selectedScene.sequence_number}`}
                </h1>
                
                {/* Scene Meta */}
                <div className="flex flex-wrap gap-2 mb-6">
                  {selectedScene.conflict_type && (
                    <span className="px-3 py-1 text-sm rounded-full bg-cinema-800 text-gray-300">
                      冲突: {selectedScene.conflict_type}
                    </span>
                  )}
                  {selectedScene.setting_location && (
                    <span className="px-3 py-1 text-sm rounded-full bg-cinema-800 text-gray-300">
                      地点: {selectedScene.setting_location}
                    </span>
                  )}
                  {selectedScene.characters_present.length > 0 && (
                    <span className="px-3 py-1 text-sm rounded-full bg-cinema-800 text-gray-300">
                      {selectedScene.characters_present.length} 个角色
                    </span>
                  )}
                </div>

                {/* Drama Info */}
                {(selectedScene.dramatic_goal || selectedScene.external_pressure) && (
                  <div className="grid grid-cols-2 gap-4 mb-6">
                    {selectedScene.dramatic_goal && (
                      <div className="p-4 bg-cinema-800/50 rounded-lg">
                        <h3 className="text-sm font-medium text-cinema-gold mb-2">戏剧目标</h3>
                        <p className="text-sm text-gray-300">{selectedScene.dramatic_goal}</p>
                      </div>
                    )}
                    {selectedScene.external_pressure && (
                      <div className="p-4 bg-cinema-800/50 rounded-lg">
                        <h3 className="text-sm font-medium text-cinema-gold mb-2">外部压迫</h3>
                        <p className="text-sm text-gray-300">{selectedScene.external_pressure}</p>
                      </div>
                    )}
                  </div>
                )}

                {/* Content */}
                {selectedScene.content ? (
                  <div className="prose prose-invert max-w-none">
                    <div className="whitespace-pre-wrap text-gray-200 leading-relaxed font-serif">
                      {selectedScene.content}
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-12">
                    <AlertCircle className="w-12 h-12 text-cinema-700 mx-auto mb-4" />
                    <p className="text-gray-500 mb-4">这个场景还没有内容</p>
                    <Button 
                      variant="primary" 
                      onClick={() => setIsEditing(true)}
                    >
                      <Plus className="w-4 h-4 mr-2" />
                      开始写作
                    </Button>
                  </div>
                )}
              </div>
            </div>

            {/* Action Bar */}
            <div className="p-4 border-t border-cinema-700 bg-cinema-900">
              <div className="flex justify-center">
                <Button 
                  variant="primary" 
                  onClick={() => setIsEditing(true)}
                >
                  编辑场景
                </Button>
              </div>
            </div>
          </div>
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-center p-8">
            <BookOpen className="w-16 h-16 text-cinema-700 mb-4" />
            <h2 className="font-display text-xl font-semibold text-white mb-2">
              选择一个场景
            </h2>
            <p className="text-gray-500 max-w-md">
              从左侧选择一个场景查看详情，或创建新场景
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
