import { Wand2, Check, X } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import type { SkillCategory } from '@/types';

const categories: { id: SkillCategory; label: string; color: string }[] = [
  { id: 'writing', label: '写作', color: 'text-blue-400' },
  { id: 'analysis', label: '分析', color: 'text-purple-400' },
  { id: 'character', label: '角色', color: 'text-pink-400' },
  { id: 'plot', label: '情节', color: 'text-orange-400' },
  { id: 'style', label: '风格', color: 'text-green-400' },
];

const mockSkills = [
  { id: '1', name: '文风增强器', description: '提升文字表现力', category: 'style', enabled: true, builtin: true },
  { id: '2', name: '情节反转', description: '生成意想不到的转折', category: 'plot', enabled: false, builtin: true },
  { id: '3', name: '角色声音', description: '保持角色一致性', category: 'character', enabled: true, builtin: true },
];

export function Skills() {
  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div>
        <h1 className="font-display text-3xl font-bold text-white">技能工坊</h1>
        <p className="text-gray-400">管理和配置AI辅助技能</p>
      </div>

      {/* Categories */}
      <div className="flex flex-wrap gap-2">
        <Button variant="primary" size="sm">全部</Button>
        {categories.map((cat) => (
          <Button key={cat.id} variant="secondary" size="sm">
            {cat.label}
          </Button>
        ))}
      </div>

      {/* Skills Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {mockSkills.map((skill) => (
          <Card key={skill.id} hover>
            <CardContent className="p-6">
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-lg bg-cinema-800 flex items-center justify-center">
                    <Wand2 className="w-5 h-5 text-cinema-gold" />
                  </div>
                  <div>
                    <h3 className="font-display font-semibold text-white">{skill.name}</h3>
                    <p className="text-xs text-gray-500">{skill.category}</p>
                  </div>
                </div>
                
                <button
                  className={`w-10 h-6 rounded-full transition-colors relative ${
                    skill.enabled ? 'bg-cinema-gold' : 'bg-cinema-700'
                  }`}
                >
                  <span
                    className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-all ${
                      skill.enabled ? 'left-5' : 'left-1'
                    }`}
                  />
                </button>
              </div>
              
              <p className="text-sm text-gray-400 mt-3">{skill.description}</p>
              
              {skill.builtin && (
                <span className="inline-block mt-3 text-xs px-2 py-1 rounded bg-cinema-gold/10 text-cinema-gold">
                  内置
                </span>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
