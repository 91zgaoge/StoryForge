import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import toast from 'react-hot-toast';

export type ExportFormat = 'markdown' | 'pdf' | 'epub' | 'html' | 'txt' | 'json';

export interface ExportOptions {
  story_id: string;
  format: ExportFormat;
  include_metadata?: boolean;
  include_outline?: boolean;
  include_characters?: boolean;
}

export interface ExportResult {
  file_path: string;
  content?: string;
}

async function exportStory(options: ExportOptions): Promise<ExportResult> {
  return invoke<ExportResult>('export_story', { options });
}

export function useExport() {
  return useMutation({
    mutationFn: exportStory,
    onSuccess: (data) => {
      // Trigger file download
      if (!data.content) {
        toast.error('导出内容为空');
        return;
      }
      const blob = new Blob([data.content], { type: 'text/plain;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = data.file_path.split('\\').pop()?.split('/').pop() || 'export.txt';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      toast.success(`导出成功: ${data.file_path}`);
    },
    onError: (error: Error) => {
      toast.error('导出失败: ' + error.message);
    },
  });
}
