import { act, renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import React from 'react';

vi.mock('@/services/tauri', () => ({
  loggedInvoke: vi.fn(),
}));

type Handler = (event: { payload: unknown }) => void;
let progressHandler: Handler | null = null;
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (_name: string, cb: Handler) => {
    progressHandler = cb;
    return () => {};
  }),
}));

import { loggedInvoke } from '@/services/tauri';
import { useGuidebookDistillationStatus } from '../useGuidebookDistillation';

const makeWrapper = (client: QueryClient) => {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
};

const evt = (status: string, progress: number) => ({
  guidebook_id: 'g1',
  status,
  progress,
  current_step: `step-${status}`,
  message: null,
  active_threads: 0,
});

describe('useGuidebookDistillationStatus 终态事件（72% 卡死修复回归）', () => {
  beforeEach(() => {
    progressHandler = null;
    vi.clearAllMocks();
    vi.mocked(loggedInvoke).mockResolvedValue({
      guidebook_id: 'g1',
      status: 'merging',
      progress: 72,
      current_step: '正在分类合并创作资产...',
      error: null,
    });
  });

  it('merging 72% 后收到 failed 终态事件：状态不再卡死并刷新列表', async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const invalidateSpy = vi.spyOn(client, 'invalidateQueries');
    const { result } = renderHook(() => useGuidebookDistillationStatus('g1'), {
      wrapper: makeWrapper(client),
    });

    await waitFor(() => expect(progressHandler).not.toBeNull());
    act(() => progressHandler!({ payload: evt('merging', 72) }));
    expect(result.current?.status).toBe('merging');
    expect(result.current?.progress).toBe(72);

    // 后端失败时只写 DB 不发事件曾导致卡片永远停在 72%（liveStatus 优先于轮询）
    act(() => progressHandler!({ payload: evt('failed', 0) }));
    expect(result.current?.status).toBe('failed');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['guidebooks'] });
  });

  it('completed 终态事件刷新列表', async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const invalidateSpy = vi.spyOn(client, 'invalidateQueries');
    const { result } = renderHook(() => useGuidebookDistillationStatus('g1'), {
      wrapper: makeWrapper(client),
    });

    await waitFor(() => expect(progressHandler).not.toBeNull());
    act(() => progressHandler!({ payload: evt('completed', 100) }));
    expect(result.current?.status).toBe('completed');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['guidebooks'] });
  });

  it('非本卡片 guidebook 的事件不影响状态', async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const { result } = renderHook(() => useGuidebookDistillationStatus('g1'), {
      wrapper: makeWrapper(client),
    });

    await waitFor(() => expect(progressHandler).not.toBeNull());
    act(() => progressHandler!({ payload: { ...evt('failed', 0), guidebook_id: 'other-book' } }));
    // liveStatus 未设置，回落到 query.data（mock 的 merging）
    await waitFor(() => expect(result.current?.status).toBe('merging'));
  });
});
