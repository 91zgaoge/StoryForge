import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Sidebar } from '@/components/Sidebar';
import { Dashboard } from '@/pages/Dashboard';
import { Stories } from '@/pages/Stories';
import { Characters } from '@/pages/Characters';
import { Scenes } from '@/pages/Scenes';
import { KnowledgeGraph } from '@/pages/KnowledgeGraph';
import { Skills } from '@/pages/Skills';
import { Mcp } from '@/pages/Mcp';
import { Settings } from '@/pages/Settings';
import { BookDeconstruction } from '@/pages/BookDeconstruction';
import { Tasks } from '@/pages/Tasks';
import { Foreshadowing } from '@/pages/Foreshadowing';
import { CreationWizard } from '@/pages/CreationWizard';
import { DataLoader } from '@/components/DataLoader';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { ConnectionStatus } from '@/components/ConnectionStatus';
import { FrontstageLauncher } from '@/components/FrontstageLauncher';
import { UpdateNotification } from '@/components/updater';
import { useUpdater } from '@/hooks/useUpdater';
import type { ViewType } from '@/types';
import toast from 'react-hot-toast';

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

  // 监听 backstage-update 事件（幕前 → 幕后联动）
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      try {
        unlisten = await listen('backstage-update', (event: any) => {
          const { type, payload } = event.payload || {};
          switch (type) {
            case 'ContentChanged':
              toast('幕前内容已更新', { icon: '📝' });
              break;
            case 'GenerationRequested':
              toast('幕前请求生成内容', { icon: '✨' });
              break;
            case 'FrontstageClosed':
              setIsFrontstageOpen(false);
              break;
            case 'FrontstageFocused':
              setIsFrontstageOpen(true);
              break;
            case 'DataRefresh':
              const entity = payload?.entity || 'data';
              toast(`幕后${entity}已更新`, { icon: '🔄' });
              window.dispatchEvent(new CustomEvent('backstage-data-refreshed', { detail: entity }));
              break;
          }
        });
      } catch (e) {
        console.error('Failed to setup backstage listener:', e);
      }
    };

    setupListener();
    return () => {
      if (unlisten) unlisten();
    };
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
      case 'book-deconstruction': return <BookDeconstruction />;
      case 'tasks': return <Tasks />;
      case 'foreshadowing': return <Foreshadowing />;
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
