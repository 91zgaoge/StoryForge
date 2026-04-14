import { useState, useEffect } from 'react';
import { Sidebar } from '@/components/Sidebar';
import { Dashboard } from '@/pages/Dashboard';
import { Stories } from '@/pages/Stories';
import { Characters } from '@/pages/Characters';
import { Scenes } from '@/pages/Scenes';
import { KnowledgeGraph } from '@/pages/KnowledgeGraph';
import { Skills } from '@/pages/Skills';
import { Mcp } from '@/pages/Mcp';
import { Settings } from '@/pages/Settings';
import { DataLoader } from '@/components/DataLoader';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { ConnectionStatus } from '@/components/ConnectionStatus';
import { FrontstageLauncher } from '@/components/FrontstageLauncher';
import { UpdateNotification } from '@/components/updater';
import { useUpdater } from '@/hooks/useUpdater';
import type { ViewType } from '@/types';

function App() {
  const [currentView, setCurrentView] = useState<ViewType>('dashboard');
  const [isFrontstageOpen, setIsFrontstageOpen] = useState(false);

  // 自动更新检测
  const {
    currentVersion,
    hasUpdate,
    latestVersion,
    updateInfo,
    isInstalling,
    error,
    checkUpdate,
    installUpdate,
    dismissUpdate,
  } = useUpdater(true);

  // Check if we're in frontstage mode (via URL or window label)
  useEffect(() => {
    const checkFrontstage = () => {
      const url = window.location.href;
      const isFrontstage = url.includes('frontstage') ||
                          (window as any).__TAURI__?.window?.label === 'frontstage';
      setIsFrontstageOpen(isFrontstage);
    };

    checkFrontstage();
  }, []);

  const renderView = () => {
    switch (currentView) {
      case 'dashboard': return <Dashboard />;
      case 'stories': return <Stories />;
      case 'characters': return <Characters />;
      case 'scenes': return <Scenes />;
      case 'knowledge-graph': return <KnowledgeGraph />;
      case 'skills': return <Skills />;
      case 'mcp': return <Mcp />;
      case 'settings': return <Settings />;
      default: return <Dashboard />;
    }
  };

  return (
    <ErrorBoundary>
      <div className="flex h-screen bg-cinema-950 film-grain">
        <DataLoader />
        <ConnectionStatus />
        <UpdateNotification
          isOpen={hasUpdate}
          currentVersion={currentVersion}
          latestVersion={latestVersion}
          updateInfo={updateInfo}
          isInstalling={isInstalling}
          error={error}
          onInstall={installUpdate}
          onDismiss={dismissUpdate}
          onCheck={checkUpdate}
        />
        <FrontstageLauncher
          isOpen={isFrontstageOpen}
          onToggle={() => setIsFrontstageOpen(!isFrontstageOpen)}
        />
        <Sidebar currentView={currentView} onNavigate={setCurrentView} />
        <main className="flex-1 overflow-auto">
          {renderView()}
        </main>
      </div>
    </ErrorBoundary>
  );
}

export default App;
