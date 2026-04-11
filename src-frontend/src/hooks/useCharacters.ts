import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { 
  getStoryCharacters, 
  createCharacter, 
  updateCharacter, 
  deleteCharacter 
} from '@services/tauri';
import type { CreateCharacterRequest, Character } from '@/types/index';
import toast from 'react-hot-toast';

const CHARACTERS_KEY = 'characters';

export function useCharacters(storyId: string | null) {
  return useQuery({
    queryKey: [CHARACTERS_KEY, storyId],
    queryFn: () => storyId ? getStoryCharacters(storyId) : Promise.resolve([]),
    enabled: !!storyId,
  });
}

export function useCreateCharacter() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: createCharacter,
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ 
        queryKey: [CHARACTERS_KEY, variables.story_id] 
      });
      toast.success('角色创建成功');
    },
    onError: (error: Error) => {
      toast.error('创建失败: ' + error.message);
    },
  });
}

export function useUpdateCharacter() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, updates }: { id: string; updates: Partial<Character> }) => 
      updateCharacter(id, updates),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [CHARACTERS_KEY] });
      toast.success('角色更新成功');
    },
    onError: (error: Error) => {
      toast.error('更新失败: ' + error.message);
    },
  });
}

export function useDeleteCharacter() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: deleteCharacter,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [CHARACTERS_KEY] });
      toast.success('角色已删除');
    },
    onError: (error: Error) => {
      toast.error('删除失败: ' + error.message);
    },
  });
}
