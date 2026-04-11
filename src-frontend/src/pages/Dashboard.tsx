import { useEffect } from 'react';
import { BookOpen, Users, FileText, Sparkles } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useStories } from '@/hooks/useStories';
import { useAppStore } from '@/stores/appStore';
import { formatNumber } from '@/utils/format';

export function Dashboard() {
  const { data: stories = [] } = useStories();
  const setStories = useAppStore((s) => s.setStories);
  const storiesList = useAppStore((s) => s.stories);

  useEffect(() => {
    setStories(stories);
  }, [stories, setStories]);

  // Calculate total characters and chapters across all stories
  const totalCharacters = storiesList.reduce((sum, s) => sum + (s.character_count || 0), 0);
  const totalChapters = storiesList.reduce((sum, s) => sum + (s.chapter_count || 0), 0);

  const stats = [
    { label: '故事', value: stories.length, icon: BookOpen, color: 'text-cinema-gold' },
    { label: '角色', value: totalCharacters, icon: Users, color: 'text-purple-400' },
    { label: '章节', value: totalChapters, icon: FileText, color: 'text-blue-400' },
  ];

  return (
    <div className="p-8 space-y-8 animate-fade-in">
      {/* Hero */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-br from-cinema-800 to-cinema-900 border border-cinema-700 p-8">
        <div className="absolute top-0 right-0 w-96 h-96 bg-cinema-gold/5 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2" />
        
        <div className="relative z-10">
          <h1 className="font-display text-4xl font-bold text-white mb-2">
            欢迎回到创作工作室
          </h1>
          <p className="text-gray-400 text-lg font-body italic max-w-2xl">
            "每一个伟大的故事，都始于一个勇敢的开始。"
          </p>
          <div className="mt-6 flex gap-4">
            <Button variant="primary" className="gap-2">
              <Sparkles className="w-4 h-4" />
              新建故事
            </Button>
            <Button variant="secondary">打开最近项目</Button>
          </div>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <Card key={stat.label} hover>
              <CardContent className="flex items-center gap-4">
                <div className={cn('p-3 rounded-xl bg-cinema-800', stat.color)}>
                  <Icon className="w-6 h-6" />
                </div>
                <div>
                  <p className="text-3xl font-display font-bold text-white">
                    {formatNumber(stat.value)}
                  </p>
                  <p className="text-sm text-gray-400">{stat.label}</p>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}

function cn(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(' ');
}
