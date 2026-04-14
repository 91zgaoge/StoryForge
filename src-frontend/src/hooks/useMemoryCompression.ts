import { useMutation } from '@tanstack/react-query';
import { compressContent, compressScene } from '@/services/tauri';

export function useCompressContent() {
  return useMutation({
    mutationFn: compressContent,
  });
}

export function useCompressScene() {
  return useMutation({
    mutationFn: compressScene,
  });
}
