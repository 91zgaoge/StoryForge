import { useQuery } from '@tanstack/react-query';
import { loggedInvoke } from '@/services/tauri';
import type { MethodologyInfo } from '@/types/guidebook-distillation';

/** 全量方法论清单：无 + 5 内置 + 自定义（含禁用项，前端自行过滤或标记） */
export function useAllMethodologies() {
  return useQuery({
    queryKey: ['all-methodologies'],
    queryFn: async () => {
      return await loggedInvoke<MethodologyInfo[]>('list_all_methodologies');
    },
  });
}
