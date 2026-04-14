import React, { useEffect, useState } from 'react';
import { KnowledgeGraphView } from '@/components/KnowledgeGraph';
import { getStoryGraph, getRetentionReport } from '@/services/tauri';
import { useAppStore } from '@/stores/appStore';
import type { StoryGraph, RetentionReport } from '@/types/v3';
import { Network, RefreshCw, Activity, AlertTriangle, CheckCircle, Brain, Archive } from 'lucide-react';
import toast from 'react-hot-toast';

type TabType = 'graph' | 'memory';

const LEVEL_COLORS: Record<string, string> = {
  critical: 'bg-red-500',
  high: 'bg-orange-500',
  medium: 'bg-yellow-500',
  low: 'bg-blue-500',
  forgotten: 'bg-gray-500',
};

const LEVEL_LABELS: Record<string, string> = {
  critical: '关键',
  high: '高优先级',
  medium: '中等',
  low: '低优先级',
  forgotten: '已遗忘',
};

export const KnowledgeGraph: React.FC = () => {
  const currentStory = useAppStore((s) => s.currentStory);
  const [activeTab, setActiveTab] = useState<TabType>('graph');
  const [graphData, setGraphData] = useState<StoryGraph | null>(null);
  const [report, setReport] = useState<RetentionReport | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const loadData = async () => {
    if (!currentStory) return;
    setIsLoading(true);
    try {
      const [graph, retention] = await Promise.all([
        getStoryGraph(currentStory.id),
        getRetentionReport(currentStory.id),
      ]);
      setGraphData(graph);
      setReport(retention);
    } catch (error) {
      console.error('Failed to load knowledge data:', error);
      toast.error('加载知识数据失败');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadData();
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

  const renderMemoryHealth = () => {
    if (!report) return null;

    const hasForgotten = report.forgotten_entities.length > 0;
    const hasCritical = report.critical_entities.length > 0;

    return (
      <div className="h-full overflow-y-auto p-6">
        <div className="max-w-4xl mx-auto space-y-6">
          {/* Summary Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-cinema-900/80 border border-cinema-800 rounded-xl p-4">
              <div className="flex items-center gap-3 mb-2">
                <Brain className="w-5 h-5 text-cinema-gold" />
                <span className="text-sm text-gray-400">总实体数</span>
              </div>
              <p className="text-2xl font-bold text-white">{report.total_entities}</p>
            </div>
            <div className="bg-cinema-900/80 border border-cinema-800 rounded-xl p-4">
              <div className="flex items-center gap-3 mb-2">
                <Activity className="w-5 h-5 text-cinema-gold" />
                <span className="text-sm text-gray-400">平均优先级</span>
              </div>
              <p className="text-2xl font-bold text-white">{(report.avg_priority * 100).toFixed(1)}%</p>
            </div>
            <div className="bg-cinema-900/80 border border-cinema-800 rounded-xl p-4">
              <div className="flex items-center gap-3 mb-2">
                {hasForgotten ? (
                  <AlertTriangle className="w-5 h-5 text-red-500" />
                ) : (
                  <CheckCircle className="w-5 h-5 text-green-500" />
                )}
                <span className="text-sm text-gray-400">系统状态</span>
              </div>
              <p className="text-lg font-semibold text-white">
                {hasForgotten ? '需要关注' : '状态良好'}
              </p>
            </div>
          </div>

          {/* Recommendation */}
          <div className="bg-cinema-900/80 border border-cinema-800 rounded-xl p-5">
            <h3 className="text-lg font-semibold text-white mb-2 flex items-center gap-2">
              <Archive className="w-5 h-5 text-cinema-gold" />
              自动归档建议
            </h3>
            <p className="text-gray-300 leading-relaxed">{report.recommended_action}</p>
          </div>

          {/* Priority Distribution */}
          <div className="bg-cinema-900/80 border border-cinema-800 rounded-xl p-5">
            <h3 className="text-lg font-semibold text-white mb-4">优先级分布</h3>
            <div className="space-y-3">
              {Object.entries(report.level_distribution)
                .sort(([a], [b]) => {
                  const order = ['critical', 'high', 'medium', 'low', 'forgotten'];
                  return order.indexOf(a) - order.indexOf(b);
                })
                .map(([level, count]) => {
                  const percentage = report.total_entities > 0
                    ? (count / report.total_entities) * 100
                    : 0;
                  return (
                    <div key={level}>
                      <div className="flex items-center justify-between text-sm mb-1">
                        <span className="flex items-center gap-2 text-gray-300">
                          <span className={cn('w-2.5 h-2.5 rounded-full', LEVEL_COLORS[level] || 'bg-gray-500')} />
                          {LEVEL_LABELS[level] || level}
                        </span>
                        <span className="text-gray-400">
                          {count} ({percentage.toFixed(1)}%)
                        </span>
                      </div>
                      <div className="h-2 bg-cinema-800 rounded-full overflow-hidden">
                        <div
                          className={cn('h-full rounded-full transition-all', LEVEL_COLORS[level] || 'bg-gray-500')}
                          style={{ width: `${percentage}%` }}
                        />
                      </div>
                    </div>
                  );
                })}
            </div>
          </div>

          {/* Critical & Forgotten Lists */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {hasCritical && (
              <div className="bg-cinema-900/80 border border-cinema-800 rounded-xl p-5">
                <h3 className="text-sm font-semibold text-red-400 uppercase tracking-wider mb-3">
                  关键实体 ({report.critical_entities.length})
                </h3>
                <div className="flex flex-wrap gap-2">
                  {report.critical_entities.map((name) => (
                    <span
                      key={name}
                      className="px-2.5 py-1 rounded-lg bg-red-500/10 text-red-300 text-sm border border-red-500/20"
                    >
                      {name}
                    </span>
                  ))}
                </div>
              </div>
            )}
            {hasForgotten && (
              <div className="bg-cinema-900/80 border border-cinema-800 rounded-xl p-5">
                <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
                  建议归档 ({report.forgotten_entities.length})
                </h3>
                <div className="flex flex-wrap gap-2">
                  {report.forgotten_entities.map((name) => (
                    <span
                      key={name}
                      className="px-2.5 py-1 rounded-lg bg-gray-500/10 text-gray-400 text-sm border border-gray-500/20"
                    >
                      {name}
                    </span>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    );
  };

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
        <div className="flex items-center gap-3">
          {/* Tabs */}
          <div className="flex items-center bg-cinema-800 rounded-lg p-1">
            <button
              onClick={() => setActiveTab('graph')}
              className={cn(
                'px-3 py-1.5 rounded-md text-sm font-medium transition-colors',
                activeTab === 'graph' ? 'bg-cinema-700 text-white' : 'text-gray-400 hover:text-white'
              )}
            >
              图谱
            </button>
            <button
              onClick={() => setActiveTab('memory')}
              className={cn(
                'px-3 py-1.5 rounded-md text-sm font-medium transition-colors',
                activeTab === 'memory' ? 'bg-cinema-700 text-white' : 'text-gray-400 hover:text-white'
              )}
            >
              记忆健康
            </button>
          </div>
          <button
            onClick={loadData}
            disabled={isLoading}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-cinema-800 hover:bg-cinema-700 text-gray-300 transition-colors disabled:opacity-50"
          >
            <RefreshCw className={cn('w-4 h-4', isLoading && 'animate-spin')} />
            <span className="text-sm">刷新</span>
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 relative">
        {isLoading && !graphData ? (
          <div className="h-full flex items-center justify-center text-gray-500">
            <div className="text-center">
              <RefreshCw className="w-10 h-10 animate-spin mx-auto mb-3 text-cinema-gold" />
              <p>正在构建知识图谱...</p>
            </div>
          </div>
        ) : activeTab === 'graph' ? (
          graphData ? (
            <KnowledgeGraphView
              entities={graphData.entities}
              relations={graphData.relations}
            />
          ) : null
        ) : (
          renderMemoryHealth()
        )}
      </div>
    </div>
  );
};

function cn(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(' ');
}

export default KnowledgeGraph;
