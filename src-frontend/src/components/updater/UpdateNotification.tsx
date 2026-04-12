import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Download, 
  X, 
  RefreshCw, 
  AlertCircle,
  Sparkles,
  ChevronRight
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { cn } from '@/utils/cn';
import type { UpdateInfo } from '@/hooks/useUpdater';

interface UpdateNotificationProps {
  isOpen: boolean;
  currentVersion: string;
  latestVersion: string | null;
  updateInfo: UpdateInfo | null;
  isInstalling: boolean;
  error: string | null;
  onInstall: () => void;
  onDismiss: () => void;
  onCheck: () => void;
  className?: string;
}

export const UpdateNotification: React.FC<UpdateNotificationProps> = ({
  isOpen,
  currentVersion,
  latestVersion,
  updateInfo,
  isInstalling,
  error,
  onInstall,
  onDismiss,
  onCheck,
  className,
}) => {
  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0, y: -50, scale: 0.95 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: -50, scale: 0.95 }}
          transition={{ duration: 0.3, ease: [0.4, 0, 0.2, 1] }}
          className={cn(
            "fixed top-4 right-4 z-50 w-96",
            className
          )}
        >
          <div className="bg-white rounded-xl shadow-2xl border border-terracotta/20 overflow-hidden">
            {/* Header */}
            <div className="bg-gradient-to-r from-terracotta to-terracotta/80 px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Sparkles className="w-5 h-5 text-white" />
                <span className="font-serif text-white font-medium">发现新版本</span>
              </div>
              <button
                onClick={onDismiss}
                className="text-white/80 hover:text-white transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* Content */}
            <div className="p-4 space-y-4">
              {/* Version Info */}
              <div className="flex items-center gap-3">
                <div className="flex-1">
                  <div className="text-sm text-stone-500">当前版本</div>
                  <div className="font-mono text-stone-700">v{currentVersion}</div>
                </div>
                <ChevronRight className="w-5 h-5 text-stone-400" />
                <div className="flex-1">
                  <div className="text-sm text-stone-500">最新版本</div>
                  <div className="font-mono text-terracotta font-medium">
                    v{latestVersion || '...'}
                  </div>
                </div>
              </div>

              {/* Update Notes */}
              {updateInfo?.notes && (
                <div className="bg-stone-50 rounded-lg p-3 max-h-32 overflow-y-auto">
                  <div className="text-sm font-medium text-stone-700 mb-1">更新内容</div>
                  <div className="text-sm text-stone-600 whitespace-pre-wrap">
                    {updateInfo.notes}
                  </div>
                </div>
              )}

              {/* Error */}
              {error && (
                <div className="flex items-start gap-2 text-red-600 bg-red-50 p-3 rounded-lg">
                  <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
                  <div className="text-sm">{error}</div>
                </div>
              )}

              {/* Actions */}
              <div className="flex gap-2">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={onCheck}
                  disabled={isInstalling}
                  className="flex-1"
                >
                  <RefreshCw className="w-4 h-4 mr-2" />
                  刷新
                </Button>
                <Button
                  size="sm"
                  onClick={onInstall}
                  disabled={isInstalling}
                  className="flex-1 bg-terracotta hover:bg-terracotta/90 text-white"
                >
                  {isInstalling ? (
                    <>
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                      安装中...
                    </>
                  ) : (
                    <>
                      <Download className="w-4 h-4 mr-2" />
                      立即更新
                    </>
                  )}
                </Button>
              </div>

              {/* Note */}
              <p className="text-xs text-stone-400 text-center">
                更新将在后台下载，完成后自动重启应用
              </p>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

export default UpdateNotification;
