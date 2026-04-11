import { useState } from 'react';
import { Sidebar } from '@/components/Sidebar';
import { Dashboard } from '@/pages/Dashboard';
import { Stories } from '@/pages/Stories';
import { Characters } from '@/pages/Characters';
import { Chapters } from '@/pages/Chapters';
import { Skills } from '@/pages/Skills';
import { Mcp } from '@/pages/Mcp';
import { Settings } from '@/pages/Settings';
import type { ViewType } from '@/types';

function App() {
  const [currentView, setCurrentView] = useState<ViewType>('dashboard');

  const renderView = () => {
    switch (currentView) {
      case 'dashboard': return <Dashboard />;
      case 'stories': return <Stories />;
      case 'characters': return <Characters />;
      case 'chapters': return <Chapters />;
      case 'skills': return <Skills />;
      case 'mcp': return <Mcp />;
      case 'settings': return <Settings />;
      default: return <Dashboard />;
    }
  };

  return (
    <div className="flex h-screen bg-cinema-950 film-grain">
      <Sidebar currentView={currentView} onNavigate={setCurrentView} />
      <main className="flex-1 overflow-auto">
        {renderView()}
      </main>
    </div>
  );
}

export default App;
