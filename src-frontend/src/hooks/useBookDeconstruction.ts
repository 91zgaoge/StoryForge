import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import type {
  ReferenceBookSummary,
  BookAnalysisResult,
  AnalysisStatusResponse,
  BookAnalysisProgressEvent,
} from '@/types/book-deconstruction';

const BOOKS_KEY = 'reference-books';
const ANALYSIS_KEY = 'book-analysis';
const STATUS_KEY = 'analysis-status';

// ==================== 上传书籍 ====================

export function useUploadBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (filePath: string) => {
      const bookId: string = await invoke('upload_book', { filePath });
      return bookId;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [BOOKS_KEY] });
    },
  });
}

// ==================== 分析状态 ====================

export function useBookAnalysisStatus(bookId: string | null) {
  const [liveStatus, setLiveStatus] = useState<AnalysisStatusResponse | null>(null);

  // 监听实时进度事件
  useEffect(() => {
    if (!bookId) return;

    let unlisten: (() => void) | undefined;

    const setup = async () => {
      unlisten = await listen<BookAnalysisProgressEvent>('book-analysis-progress', (event) => {
        if (event.payload.book_id === bookId) {
          setLiveStatus({
            book_id: bookId,
            status: event.payload.status,
            progress: event.payload.progress,
            current_step: event.payload.current_step,
            error: undefined,
          });
        }
      });
    };

    setup();
    return () => {
      if (unlisten) unlisten();
    };
  }, [bookId]);

  // 轮询作为 fallback
  const query = useQuery({
    queryKey: [STATUS_KEY, bookId],
    queryFn: async () => {
      if (!bookId) return null;
      const status: AnalysisStatusResponse = await invoke('get_analysis_status', { bookId });
      return status;
    },
    refetchInterval: (query) => {
      const data = query.state.data;
      if (!data) return false;
      return data.status === 'pending' || data.status === 'extracting' || data.status === 'analyzing'
        ? 3000
        : false;
    },
    enabled: !!bookId,
  });

  return liveStatus ?? query.data ?? null;
}

// ==================== 分析结果 ====================

export function useBookAnalysis(bookId: string | null) {
  return useQuery({
    queryKey: [ANALYSIS_KEY, bookId],
    queryFn: async () => {
      if (!bookId) return null;
      const result: BookAnalysisResult = await invoke('get_book_analysis', { bookId });
      return result;
    },
    enabled: !!bookId,
  });
}

// ==================== 书籍列表 ====================

export function useReferenceBooks() {
  return useQuery({
    queryKey: [BOOKS_KEY],
    queryFn: async () => {
      const books: ReferenceBookSummary[] = await invoke('list_reference_books');
      return books;
    },
  });
}

// ==================== 删除书籍 ====================

export function useDeleteBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (bookId: string) => {
      await invoke('delete_reference_book', { bookId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [BOOKS_KEY] });
    },
  });
}

// ==================== 转为故事 ====================

export function useConvertToStory() {
  return useMutation({
    mutationFn: async (bookId: string) => {
      const storyId: string = await invoke('convert_book_to_story', { bookId });
      return storyId;
    },
  });
}
