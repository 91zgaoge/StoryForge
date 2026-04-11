import { useQuery, useMutation } from '@tanstack/react-query';
import { searchSimilar, embedChapter } from '@/services/tauri';
import type { VectorSearchRequest, SimilarityResult } from '@/types';
import toast from 'react-hot-toast';

const VECTOR_KEY = 'vector';

export function useVectorSearch() {
  return useMutation({
    mutationFn: async (req: VectorSearchRequest): Promise<SimilarityResult[]> => {
      return await searchSimilar(req);
    },
    onError: (error: Error) => {
      toast.error('搜索失败: ' + error.message);
    },
  });
}

export function useEmbedChapter() {
  return useMutation({
    mutationFn: ({ chapterId, content }: { chapterId: string; content: string }) => 
      embedChapter(chapterId, content),
    onSuccess: () => {
      toast.success('章节已向量化');
    },
    onError: (error: Error) => {
      toast.error('向量化失败: ' + error.message);
    },
  });
}
