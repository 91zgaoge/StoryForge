import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, expect, it, vi } from 'vitest';
import React from 'react';

vi.mock('@/services/tauri', () => ({
  loggedInvoke: vi.fn(),
}));

import { loggedInvoke } from '@/services/tauri';
import { useAllMethodologies } from '../useMethodologies';

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
    {children}
  </QueryClientProvider>
);

describe('useAllMethodologies', () => {
  it('调用 list_all_methodologies 并返回清单', async () => {
    vi.mocked(loggedInvoke).mockResolvedValue([
      {
        id: '',
        name: '无（自由创作）',
        description: '',
        max_steps: 1,
        is_custom: false,
        source_book: null,
        enabled: true,
      },
      {
        id: 'snowflake',
        name: '雪花写作法',
        description: 'd',
        max_steps: 10,
        is_custom: false,
        source_book: null,
        enabled: true,
      },
      {
        id: 'custom_x',
        name: '冲突驱动法',
        description: 'd',
        max_steps: 3,
        is_custom: true,
        source_book: '故事技巧',
        enabled: true,
      },
    ]);
    const { result } = renderHook(() => useAllMethodologies(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(loggedInvoke).toHaveBeenCalledWith('list_all_methodologies');
    expect(result.current.data).toHaveLength(3);
    expect(result.current.data?.[2].is_custom).toBe(true);
  });
});
