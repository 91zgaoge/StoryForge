import { useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useAppStore } from '@/stores/appStore';
import { healthCheck, listStories } from '@services/tauri';

// This component handles data loading separately from rendering
// to prevent React infinite loop issues
export function DataLoader() {
  const setStories = useAppStore((s) => s.setStories);
  const setError = useAppStore((s) => s.setError);
  const setIsLoading = useAppStore((s) => s.setIsLoading);
  const hasAttemptedRef = useRef(false);

  // First check if Tauri is available
  const { data: health, isSuccess: isHealthOk } = useQuery({
    queryKey: ['health'],
    queryFn: healthCheck,
    retry: 2,
    retryDelay: 1000,
    staleTime: 30000,
    // Only run once on mount
    refetchOnWindowFocus: false,
  });

  // Only load stories after health check passes
  const { data: stories, error, isLoading } = useQuery({
    queryKey: ['stories'],
    queryFn: listStories,
    // Only enable after health check is successful
    enabled: isHealthOk,
    retry: 1,
    retryDelay: 500,
    staleTime: 60000,
    refetchOnWindowFocus: false,
  });

  // Sync loading state to store
  useEffect(() => {
    setIsLoading(isLoading);
  }, [isLoading, setIsLoading]);

  // Sync error to store
  useEffect(() => {
    if (error) {
      setError((error as Error).message);
    }
  }, [error, setError]);

  // Sync stories to store - only once per unique data
  useEffect(() => {
    if (stories && !hasAttemptedRef.current) {
      hasAttemptedRef.current = true;
      setStories(stories);
    }
  }, [stories, setStories]);

  // This component doesn't render anything visible
  return null;
}
