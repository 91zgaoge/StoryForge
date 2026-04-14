import React, { useEffect, useState } from 'react';
import { KnowledgeGraphView } from '@/components/KnowledgeGraph';
import { getStoryGraph } from '@/services/tauri';
import { useAppStore } from '@/stores/appStore';
import type { StoryGraph } from '@/types/v3';
import { Network, RefreshCw } from 'lucide-react';
import toast from 'react-hot-toast';

export const KnowledgeGraph: React.FC = () => {
  const currentStory = useAppStore((s) => s.currentStory);
  const [graphData, setGraphData] = useState<StoryGraph | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const loadGraph = async () => {
    if (!currentStory) return;
    setIsLoading(true);
    try {
      const data = await getStoryGraph(currentStory.id);
      setGraphData(data);
    } catch (error) {
      console.error('Failed to load knowledge graph:', error);
      toast.error('加载知识图谱失败');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadGraph();
  }, [currentStory?.id]);

  if (!currentStory) {
    return (
      <div className="h-full flex items-center justify-center text-gray-500">
        <div className="text-center">
          <Network className="w-16 h-16 mx-auto mb-4 text-cinema-800" />
          <p className="text-lg">请先选择一个故事</p>
          <p className="text-sm text-gray-600 mt-2">在故事库中选择一部小说以查看其知识图谱</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="px-6 py-4 border-b border-cinema-800 flex items-center justify-between bg-cinema-900/50">
        <div>
          <h1 className="text-xl font-bold text-white flex items-center gap-2">
            <Network className="w-5 h-5 text-cinema-gold" />
            知识图谱
          </h1>
          <p className="text-sm text-gray-500 mt-0.5">
            {currentStory.title} · {graphData ? `${graphData.entities.length} 实体 · ${graphData.relations.length} 关系` : '加载中...'}
          </p>
        </div>
        <button
          onClick={loadGraph}
          disabled={isLoading}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-cinema-800 hover:bg-cinema-700 text-gray-300 transition-colors disabled:opacity-50"
        >
          <RefreshCw className={cn('w-4 h-4', isLoading && 'animate-spin')} />
          <span className="text-sm">刷新</span>
        </button>
      </div>

      {/* Graph View */}
      <div className="flex-1 relative">
        {isLoading && !graphData ? (
          <div className="h-full flex items-center justify-center text-gray-500">
            <div className="text-center">
              <RefreshCw className="w-10 h-10 animate-spin mx-auto mb-3 text-cinema-gold" />
              <p>正在构建知识图谱...</p>
            </div>
          </div>
        ) : graphData ? (
          <KnowledgeGraphView
            entities={graphData.entities}
            relations={graphData.relations}
          />
        ) : null}
      </div>
    </div>
  );
};

function cn(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(' ');
}

export default KnowledgeGraph;
