import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQuery } from '@tanstack/react-query';
import toast from 'react-hot-toast';

export interface SearchResult {
  id: string;
  text: string;
  score: number;
  chapter_id: string;
  chapter_number: number;
}

export interface VectorSearchRequest {
  story_id: string;
  query: string;
  top_k?: number;
}

export function useVectorSearch() {
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const search = useCallback(async (req: VectorSearchRequest) => {
    if (!req.query.trim()) {
      setResults([]);
      return;
    }

    setIsLoading(true);
    try {
      const data = await invoke<SearchResult[]>('search_similar', {
        storyId: req.story_id,
        query: req.query,
        topK: req.top_k || 5,
      });
      setResults(data);
    } catch (error) {
      toast.error('搜索失败: ' + (error as Error).message);
      setResults([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const clearResults = useCallback(() => {
    setResults([]);
  }, []);

  return {
    results,
    isLoading,
    search,
    clearResults,
  };
}

// FTS5 + Hybrid Search hooks
export function useTextSearchVectors(storyId: string | null, query: string, top_k: number = 5) {
  return useQuery({
    queryKey: ['text-search-vectors', storyId, query, top_k],
    queryFn: () => (storyId ? invoke<SearchResult[]>('text_search_vectors', { storyId, query, topK: top_k }) : Promise.resolve([])),
    enabled: !!storyId && query.trim().length > 0,
  });
}

export function useHybridSearchVectors(storyId: string | null, query: string, top_k: number = 5) {
  return useQuery({
    queryKey: ['hybrid-search-vectors', storyId, query, top_k],
    queryFn: () => (storyId ? invoke<SearchResult[]>('hybrid_search_vectors', { storyId, query, topK: top_k }) : Promise.resolve([])),
    enabled: !!storyId && query.trim().length > 0,
  });
}
