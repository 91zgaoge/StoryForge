import { useEffect, useState, useMemo, useCallback } from 'react';
import { Wand2, Play, Trash2, Loader2, AlertCircle } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { getSkills, enableSkill, disableSkill, uninstallSkill, executeSkill } from '@/services/tauri';
import type { Skill, SkillCategory } from '@/types';

const categories: { id: SkillCategory | 'all'; label: string; color: string }[] = [
  { id: 'all', label: '全部', color: 'text-white' },
  { id: 'writing', label: '写作', color: 'text-blue-400' },
  { id: 'analysis', label: '分析', color: 'text-purple-400' },
  { id: 'character', label: '角色', color: 'text-pink-400' },
  { id: 'plot', label: '情节', color: 'text-orange-400' },
  { id: 'style', label: '风格', color: 'text-green-400' },
  { id: 'world_building', label: '世界观', color: 'text-cyan-400' },
  { id: 'export', label: '导出', color: 'text-yellow-400' },
  { id: 'integration', label: '集成', color: 'text-indigo-400' },
  { id: 'custom', label: '自定义', color: 'text-gray-400' },
];

const categoryLabelMap: Record<string, string> = {
  writing: '写作',
  analysis: '分析',
  character: '角色',
  plot: '情节',
  style: '风格',
  world_building: '世界观',
  export: '导出',
  integration: '集成',
  custom: '自定义',
};

export function Skills() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<SkillCategory | 'all'>('all');
  const [togglingId, setTogglingId] = useState<string | null>(null);
  const [executingId, setExecutingId] = useState<string | null>(null);
  const [executionResult, setExecutionResult] = useState<{ skillName: string; result: unknown } | null>(null);

  const fetchSkills = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await getSkills();
      setSkills(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSkills();
  }, [fetchSkills]);

  const filteredSkills = useMemo(() => {
    if (selectedCategory === 'all') return skills;
    return skills.filter((s) => s.category === selectedCategory);
  }, [skills, selectedCategory]);

  const handleToggle = useCallback(async (skill: Skill) => {
    try {
      setTogglingId(skill.id);
      if (skill.is_enabled) {
        await disableSkill(skill.id);
      } else {
        await enableSkill(skill.id);
      }
      await fetchSkills();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setTogglingId(null);
    }
  }, [fetchSkills]);

  const handleExecute = useCallback(async (skill: Skill) => {
    try {
      setExecutingId(skill.id);
      const params: Record<string, unknown> = {};

      for (const param of skill.parameters) {
        if (param.required) {
          const value = window.prompt(`${param.description} (${param.name})`);
          if (value === null) {
            setExecutingId(null);
            return;
          }
          params[param.name] = value;
        }
      }

      const result = await executeSkill(skill.id, params);
      setExecutionResult({ skillName: skill.name, result });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setExecutingId(null);
    }
  }, []);

  const handleUninstall = useCallback(async (skill: Skill) => {
    if (!window.confirm(`确定要卸载技能「${skill.name}」吗？`)) return;
    try {
      await uninstallSkill(skill.id);
      await fetchSkills();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [fetchSkills]);

  const isBuiltin = (skill: Skill) => skill.path === 'builtin';

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div>
        <h1 className="font-display text-3xl font-bold text-white">技能工坊</h1>
        <p className="text-gray-400">管理和配置AI辅助技能</p>
      </div>

      {error && (
        <div className="flex items-center gap-2 text-red-400 bg-red-950/30 px-4 py-3 rounded-lg border border-red-900/50">
          <AlertCircle className="w-5 h-5" />
          <span className="text-sm">{error}</span>
          <button onClick={() => setError(null)} className="ml-auto text-xs underline">关闭</button>
        </div>
      )}

      {executionResult && (
        <div className="bg-cinema-800/50 border border-cinema-700 rounded-lg p-4 space-y-2">
          <div className="flex items-center justify-between">
            <h4 className="font-semibold text-white">「{executionResult.skillName}」执行结果</h4>
            <button onClick={() => setExecutionResult(null)} className="text-xs text-gray-400 hover:text-white">关闭</button>
          </div>
          <pre className="text-xs text-gray-300 bg-cinema-900/80 rounded p-3 overflow-auto max-h-48">
            {JSON.stringify(executionResult.result, null, 2)}
          </pre>
        </div>
      )}

      {/* Categories */}
      <div className="flex flex-wrap gap-2">
        {categories.map((cat) => (
          <Button
            key={cat.id}
            variant={selectedCategory === cat.id ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => setSelectedCategory(cat.id)}
          >
            {cat.label}
          </Button>
        ))}
      </div>

      {/* Skills Grid */}
      {loading ? (
        <div className="flex items-center justify-center py-20 text-gray-400">
          <Loader2 className="w-6 h-6 animate-spin mr-2" />
          加载技能中...
        </div>
      ) : filteredSkills.length === 0 ? (
        <div className="text-center py-20 text-gray-500">
          <Wand2 className="w-12 h-12 mx-auto mb-4 opacity-30" />
          <p>该分类下暂无技能</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredSkills.map((skill) => (
            <Card key={skill.id} hover>
              <CardContent className="p-6">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-cinema-800 flex items-center justify-center">
                      <Wand2 className="w-5 h-5 text-cinema-gold" />
                    </div>
                    <div>
                      <h3 className="font-display font-semibold text-white">{skill.name}</h3>
                      <p className="text-xs text-gray-500">{categoryLabelMap[skill.category] ?? skill.category}</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleExecute(skill)}
                      disabled={executingId === skill.id}
                      className="w-8 h-8 rounded-full bg-cinema-700 hover:bg-cinema-600 flex items-center justify-center transition-colors disabled:opacity-50"
                      title="执行"
                    >
                      {executingId === skill.id ? (
                        <Loader2 className="w-4 h-4 text-white animate-spin" />
                      ) : (
                        <Play className="w-4 h-4 text-white" />
                      )}
                    </button>

                    <button
                      onClick={() => handleToggle(skill)}
                      disabled={togglingId === skill.id}
                      className={`w-10 h-6 rounded-full transition-colors relative disabled:opacity-50 ${
                        skill.is_enabled ? 'bg-cinema-gold' : 'bg-cinema-700'
                      }`}
                      title={skill.is_enabled ? '禁用' : '启用'}
                    >
                      <span
                        className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-all ${
                          skill.is_enabled ? 'left-5' : 'left-1'
                        }`}
                      />
                    </button>
                  </div>
                </div>

                <p className="text-sm text-gray-400 mt-3">{skill.description}</p>

                <div className="flex items-center gap-2 mt-4">
                  {isBuiltin(skill) && (
                    <span className="text-xs px-2 py-1 rounded bg-cinema-gold/10 text-cinema-gold">
                      内置
                    </span>
                  )}
                  <span className="text-xs px-2 py-1 rounded bg-cinema-700 text-gray-300">
                    {skill.runtime_type}
                  </span>
                  {!isBuiltin(skill) && (
                    <button
                      onClick={() => handleUninstall(skill)}
                      className="ml-auto text-xs flex items-center gap-1 text-red-400 hover:text-red-300"
                      title="卸载"
                    >
                      <Trash2 className="w-3 h-3" />
                      卸载
                    </button>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
