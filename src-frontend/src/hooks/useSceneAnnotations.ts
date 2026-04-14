import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { SceneAnnotation } from '@/types/v3';
import {
  createSceneAnnotation,
  getSceneAnnotations,
  getStoryUnresolvedAnnotations,
  updateSceneAnnotation,
  resolveSceneAnnotation,
  unresolveSceneAnnotation,
  deleteSceneAnnotation,
} from '@/services/tauri';

export function useSceneAnnotations(sceneId: string | null) {
  return useQuery({
    queryKey: ['scene-annotations', sceneId],
    queryFn: () => (sceneId ? getSceneAnnotations(sceneId) : Promise.resolve([])),
    enabled: !!sceneId,
  });
}

export function useStoryUnresolvedAnnotations(storyId: string | null) {
  return useQuery({
    queryKey: ['story-unresolved-annotations', storyId],
    queryFn: () => (storyId ? getStoryUnresolvedAnnotations(storyId) : Promise.resolve([])),
    enabled: !!storyId,
  });
}

export function useCreateSceneAnnotation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: createSceneAnnotation,
    onSuccess: (_, vars) => {
      queryClient.invalidateQueries({ queryKey: ['scene-annotations', vars.scene_id] });
      queryClient.invalidateQueries({ queryKey: ['story-unresolved-annotations', vars.story_id] });
    },
  });
}

export function useUpdateSceneAnnotation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ annotationId, content }: { annotationId: string; content: string }) =>
      updateSceneAnnotation(annotationId, content),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scene-annotations'] });
    },
  });
}

export function useResolveSceneAnnotation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: resolveSceneAnnotation,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scene-annotations'] });
      queryClient.invalidateQueries({ queryKey: ['story-unresolved-annotations'] });
    },
  });
}

export function useUnresolveSceneAnnotation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: unresolveSceneAnnotation,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scene-annotations'] });
      queryClient.invalidateQueries({ queryKey: ['story-unresolved-annotations'] });
    },
  });
}

export function useDeleteSceneAnnotation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteSceneAnnotation,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scene-annotations'] });
      queryClient.invalidateQueries({ queryKey: ['story-unresolved-annotations'] });
    },
  });
}

export const ANNOTATION_TYPE_LABELS: Record<SceneAnnotation['annotation_type'], string> = {
  note: '笔记',
  todo: '待办',
  warning: '注意',
  idea: '灵感',
};

export const ANNOTATION_TYPE_COLORS: Record<SceneAnnotation['annotation_type'], string> = {
  note: 'bg-blue-500',
  todo: 'bg-orange-500',
  warning: 'bg-red-500',
  idea: 'bg-purple-500',
};
