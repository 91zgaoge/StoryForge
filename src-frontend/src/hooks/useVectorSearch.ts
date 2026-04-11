import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
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
