import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
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
      toast.success(`导出成功: ${data.file_path}`);
    },
    onError: (error: Error) => {
      toast.error('导出失败: ' + error.message);
    },
  });
}
