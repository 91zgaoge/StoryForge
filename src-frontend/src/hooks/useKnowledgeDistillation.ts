import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  distillStoryKnowledge,
  getStorySummaries,
  updateStorySummary,
  deleteStorySummary,
} from '@/services/tauri';
import type { StorySummary } from '@/types/v3';

const STORY_SUMMARIES_KEY = 'storySummaries';

export function useStorySummaries(storyId: string | undefined) {
  return useQuery({
    queryKey: [STORY_SUMMARIES_KEY, storyId],
    queryFn: () => getStorySummaries(storyId!),
    enabled: !!storyId,
  });
}

export function useDistillStoryKnowledge() {
  const queryClient = useQueryClient();

  return useMutation<StorySummary, Error, string>({
    mutationFn: (storyId: string) => distillStoryKnowledge(storyId),
    onSuccess: (_, storyId) => {
      queryClient.invalidateQueries({ queryKey: [STORY_SUMMARIES_KEY, storyId] });
    },
  });
}

export function useUpdateStorySummary() {
  const queryClient = useQueryClient();

  return useMutation<number, Error, { summaryId: string; content: string; storyId: string }>({
    mutationFn: ({ summaryId, content }) => updateStorySummary(summaryId, content),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [STORY_SUMMARIES_KEY, variables.storyId] });
    },
  });
}

export function useDeleteStorySummary() {
  const queryClient = useQueryClient();

  return useMutation<number, Error, { summaryId: string; storyId: string }>({
    mutationFn: ({ summaryId }) => deleteStorySummary(summaryId),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [STORY_SUMMARIES_KEY, variables.storyId] });
    },
  });
}
