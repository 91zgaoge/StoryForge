import {
  LayoutDashboard,
  BookOpen,
  Users,
  Clapperboard,
  Network,
  Settings,
  Sparkles,
} from 'lucide-react';
import { useAppStore } from '@/stores/appStore';
import { cn } from '@/utils/cn';

export const NAV_ITEMS = [
  { view: 'dashboard' as const, icon: LayoutDashboard, label: '仪表盘' },
  { view: 'stories' as const, icon: BookOpen, label: '故事' },
  { view: 'characters' as const, icon: Users, label: '角色' },
  { view: 'scenes' as const, icon: Clapperboard, label: '场景' },
  { view: 'knowledge-graph' as const, icon: Network, label: '知识图谱' },
  { view: 'settings' as const, icon: Settings, label: '设置' },
];

interface StudioNavRailProps {
  activeView?: string;
}

export function StudioNavRail({ activeView = 'dashboard' }: StudioNavRailProps) {
  const setCurrentView = useAppStore(s => s.setCurrentView);

  return (
    <nav className="w-16 flex-shrink-0 bg-cinema-900 border-r border-borderSubtle flex flex-col items-center py-4 gap-4">
      <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-cinema-gold to-cinema-gold-dark flex items-center justify-center shadow-panel">
        <Sparkles className="w-5 h-5 text-cinema-900" />
      </div>

      <div className="flex-1 flex flex-col items-center gap-2 w-full px-2">
        {NAV_ITEMS.map(item => {
          const Icon = item.icon;
          const isActive = item.view === activeView;
          return (
            <button
              key={item.view}
              type="button"
              aria-label={item.label}
              aria-current={isActive ? 'page' : undefined}
              title={item.label}
              onClick={() => setCurrentView(item.view)}
              className={cn(
                'w-full aspect-square rounded-panel flex items-center justify-center transition-colors',
                'hover:bg-cinema-800/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cinema-gold/50',
                isActive
                  ? 'bg-cinema-gold/10 text-cinema-gold'
                  : 'text-cinema-gold/60 hover:text-cinema-gold'
              )}
            >
              <Icon className="w-5 h-5" />
            </button>
          );
        })}
      </div>
    </nav>
  );
}
