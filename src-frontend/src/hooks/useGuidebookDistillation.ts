import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { loggedInvoke } from '@/services/tauri';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import type {
  GuidebookListItem,
  GuidebookResult,
  GuidebookStatusResponse,
  DistillationProgressEvent,
  MethodologyStep,
} from '@/types/guidebook-distillation';

const GUIDEBOOKS_KEY = 'guidebooks';
const RESULT_KEY = 'guidebook-result';
const DISTILL_STATUS_KEY = 'guidebook-distill-status';

export function useUploadGuidebook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (filePath: string) => {
      return await loggedInvoke<string>('upload_guidebook', { file_path: filePath });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
    },
  });
}

export function useGuidebooks() {
  return useQuery({
    queryKey: [GUIDEBOOKS_KEY],
    queryFn: async () => {
      return await loggedInvoke<GuidebookListItem[]>('list_guidebooks');
    },
  });
}

export function useGuidebookDistillationStatus(guidebookId: string | null) {
  const [liveStatus, setLiveStatus] = useState<GuidebookStatusResponse | null>(null);

  useEffect(() => {
    if (!guidebookId) return;
    let unlisten: (() => void) | undefined;
    const setup = async () => {
      unlisten = await listen<DistillationProgressEvent>(
        'guidebook-distillation-progress',
        event => {
          if (event.payload.guidebook_id === guidebookId) {
            setLiveStatus({
              guidebook_id: guidebookId,
              status: event.payload.status,
              progress: event.payload.progress,
              current_step: event.payload.current_step,
              error: null,
            });
          }
        }
      );
    };
    setup();
    return () => {
      if (unlisten) unlisten();
    };
  }, [guidebookId]);

  const query = useQuery({
    queryKey: [DISTILL_STATUS_KEY, guidebookId],
    queryFn: async () => {
      if (!guidebookId) return null;
      return await loggedInvoke<GuidebookStatusResponse>('get_guidebook_distillation_status', {
        guidebook_id: guidebookId,
      });
    },
    refetchInterval: query => {
      const data = query.state.data;
      if (!data) return false;
      return ['pending', 'extracting', 'distilling', 'merging'].includes(data.status)
        ? 3000
        : false;
    },
    enabled: !!guidebookId,
  });

  return liveStatus ?? query.data ?? null;
}

export function useGuidebookResult(guidebookId: string | null) {
  return useQuery({
    queryKey: [RESULT_KEY, guidebookId],
    queryFn: async () => {
      if (!guidebookId) return null;
      return await loggedInvoke<GuidebookResult>('get_guidebook_result', {
        guidebook_id: guidebookId,
      });
    },
    enabled: !!guidebookId,
  });
}

export function useDeleteGuidebook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (guidebookId: string) => {
      await loggedInvoke<void>('delete_guidebook', { guidebook_id: guidebookId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
    },
  });
}

export function useCancelGuidebookDistillation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (guidebookId: string) => {
      await loggedInvoke<void>('cancel_guidebook_distillation', {
        guidebook_id: guidebookId,
      });
    },
    onSuccess: (_, guidebookId) => {
      queryClient.invalidateQueries({ queryKey: [DISTILL_STATUS_KEY, guidebookId] });
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
    },
  });
}

export function useUpdateCustomMethodology() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (input: {
      id: string;
      name?: string;
      description?: string;
      steps?: MethodologyStep[];
      enabled?: boolean;
    }) => {
      const { id, ...rest } = input;
      await loggedInvoke<void>('update_custom_methodology', { id, ...rest });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [RESULT_KEY] });
      queryClient.invalidateQueries({ queryKey: ['all-methodologies'] });
    },
  });
}

export function useDeleteCustomMethodology() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      await loggedInvoke<void>('delete_custom_methodology', { id });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
      queryClient.invalidateQueries({ queryKey: ['all-methodologies'] });
    },
  });
}
