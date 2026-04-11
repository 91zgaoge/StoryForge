import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Wifi, WifiOff, Loader2 } from 'lucide-react';

export function ConnectionStatus() {
  const [status, setStatus] = useState<'checking' | 'connected' | 'disconnected'>('checking');
  const [retryCount, setRetryCount] = useState(0);

  const checkConnection = useCallback(async () => {
    setStatus('checking');
    try {
      await invoke('health_check');
      setStatus('connected');
    } catch (error) {
      console.log('Connection check failed:', error);
      if (retryCount < 3) {
        setTimeout(() => setRetryCount(c => c + 1), 1000);
      } else {
        setStatus('disconnected');
      }
    }
  }, [retryCount]);

  useEffect(() => {
    checkConnection();
  }, [checkConnection]);

  if (status === 'connected') {
    return null;
  }

  return (
    <div className="fixed top-4 left-1/2 -translate-x-1/2 z-50">
      <div className={`
        flex items-center gap-3 px-4 py-3 rounded-xl shadow-lg border
        ${status === 'checking' 
          ? 'bg-cinema-800/90 border-cinema-600 text-cinema-gold' 
          : 'bg-red-900/90 border-red-700 text-red-200'}
        backdrop-blur-sm
      `}>
        {status === 'checking' ? (
          <>
            <Loader2 className="w-5 h-5 animate-spin" />
            <span className="text-sm font-medium">正在连接服务...</span>
          </>
        ) : (
          <>
            <WifiOff className="w-5 h-5" />
            <div className="flex flex-col">
              <span className="text-sm font-medium">无法连接到本地服务</span>
              <span className="text-xs opacity-75">请确保应用已正确启动</span>
            </div>
            <button
              onClick={() => {
                setRetryCount(0);
                checkConnection();
              }}
              className="ml-2 px-3 py-1 bg-red-800 hover:bg-red-700 rounded-lg text-xs font-medium transition-colors"
            >
              重试
            </button>
          </>
        )}
      </div>
    </div>
  );
}
