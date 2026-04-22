import { describe, it, expect, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useSetActiveModel } from '../useSettings';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock react-hot-toast
vi.mock('react-hot-toast', () => ({
  default: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: 0 },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('useSetActiveModel', () => {
  it('should call invoke with correct arguments', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const { result } = renderHook(() => useSetActiveModel(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ type: 'chat', modelId: 'model-1' });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(invoke).toHaveBeenCalledWith('set_active_model', {
      model_type: 'chat',
      model_id: 'model-1',
    });
  });

  it('should handle multimodal model type', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const { result } = renderHook(() => useSetActiveModel(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ type: 'multimodal', modelId: 'gemma-4' });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(invoke).toHaveBeenCalledWith('set_active_model', {
      model_type: 'multimodal',
      model_id: 'gemma-4',
    });
  });

  it('should handle embedding model type', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const { result } = renderHook(() => useSetActiveModel(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ type: 'embedding', modelId: 'bge-m3' });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(invoke).toHaveBeenCalledWith('set_active_model', {
      model_type: 'embedding',
      model_id: 'bge-m3',
    });
  });

  it('should set error state on failure', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('Backend error'));

    const { result } = renderHook(() => useSetActiveModel(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ type: 'chat', modelId: 'model-1' });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toContain('Backend error');
  });
});
