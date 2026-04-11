import { 
  LayoutDashboard, BookOpen, Users, FileText, 
  Wand2, Plug, Settings, Film 
} from 'lucide-react';
import { cn } from '@/utils/cn';
import type { ViewType } from '@/types';

interface SidebarProps {
  currentView: ViewType;
  onNavigate: (view: ViewType) => void;
}

const navItems: { id: ViewType; label: string; icon: React.ElementType }[] = [
  { id: 'dashboard', label: '仪表盘', icon: LayoutDashboard },
  { id: 'stories', label: '故事', icon: BookOpen },
  { id: 'characters', label: '角色', icon: Users },
  { id: 'chapters', label: '章节', icon: FileText },
  { id: 'skills', label: '技能', icon: Wand2 },
  { id: 'mcp', label: 'MCP', icon: Plug },
  { id: 'settings', label: '设置', icon: Settings },
];

export function Sidebar({ currentView, onNavigate }: SidebarProps) {
  return (
    <aside className="w-20 lg:w-64 bg-cinema-900 border-r border-cinema-800 flex flex-col">
      <div className="p-4 flex items-center justify-center lg:justify-start gap-3 border-b border-cinema-800">
        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-cinema-gold to-cinema-gold-dark flex items-center justify-center">
          <Film className="w-5 h-5 text-cinema-900" />
        </div>
        <span className="hidden lg:block font-display text-xl font-bold text-white">
          CINEMA-AI
        </span>
      </div>

      <nav className="flex-1 p-3 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = currentView === item.id;
          
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={cn(
                'w-full flex items-center gap-3 px-3 py-3 rounded-xl transition-all duration-200',
                'hover:bg-cinema-800',
                isActive && 'bg-cinema-gold/10 text-cinema-gold border border-cinema-gold/20',
                !isActive && 'text-gray-400'
              )}
            >
              <Icon className={cn('w-5 h-5', isActive && 'text-cinema-gold')} />
              <span className="hidden lg:block font-medium">{item.label}</span>
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
