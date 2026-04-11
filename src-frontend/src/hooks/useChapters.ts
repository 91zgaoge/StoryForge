import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { 
  getStoryChapters, 
  getChapter,
  createChapter,
  updateChapter, 
  deleteChapter 
} from '@services/tauri';
import type { Chapter } from '@/types/index';
import toast from 'react-hot-toast';

const CHAPTERS_KEY = 'chapters';

export function useChapters(storyId: string | null) {
  return useQuery({
    queryKey: [CHAPTERS_KEY, storyId],
    queryFn: () => storyId ? getStoryChapters(storyId) : Promise.resolve([]),
    enabled: !!storyId,
  });
}

export function useChapter(id: string | null) {
  return useQuery({
    queryKey: [CHAPTERS_KEY, 'detail', id],
    queryFn: () => id ? getChapter(id) : Promise.resolve(null),
    enabled: !!id,
  });
}

export function useCreateChapter() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: createChapter,
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ 
        queryKey: [CHAPTERS_KEY, variables.story_id] 
      });
      toast.success('章节创建成功');
    },
    onError: (error: Error) => {
      toast.error('创建失败: ' + error.message);
    },
  });
}

export function useUpdateChapter() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, updates }: { id: string; updates: Partial<Chapter> }) => 
      updateChapter(id, updates),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [CHAPTERS_KEY] });
      toast.success('章节更新成功');
    },
    onError: (error: Error) => {
      toast.error('更新失败: ' + error.message);
    },
  });
}

export function useDeleteChapter() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: deleteChapter,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [CHAPTERS_KEY] });
      toast.success('章节已删除');
    },
    onError: (error: Error) => {
      toast.error('删除失败: ' + error.message);
    },
  });
}
