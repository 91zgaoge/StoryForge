import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { GuidebookDistillationPanel } from '../GuidebookDistillationPanel';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

const { subscriptionState, dialogOpenMock } = vi.hoisted(() => ({
  subscriptionState: { isPro: false },
  dialogOpenMock: vi.fn(),
}));

vi.mock('@/hooks/useSubscription', () => ({
  useSubscription: () => ({
    isPro: subscriptionState.isPro,
    fetchStatus: () => Promise.resolve(),
  }),
}));

vi.mock('@/hooks/useGuidebookDistillation', () => ({
  useGuidebooks: () => ({ data: [], isLoading: false }),
  useUploadGuidebook: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useDeleteGuidebook: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useGuidebookDistillationStatus: () => ({ data: null }),
  useCancelGuidebookDistillation: () => ({ mutateAsync: vi.fn() }),
  useGuidebookResult: () => ({ data: null, isLoading: false }),
  useUpdateCustomMethodology: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useDeleteCustomMethodology: () => ({ mutateAsync: vi.fn(), isPending: false }),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: (...args: unknown[]) => dialogOpenMock(...args),
}));

vi.mock('react-hot-toast', () => ({ default: { success: vi.fn(), error: vi.fn() } }));

describe('GuidebookDistillationPanel - 免费可用（v0.36.0）', () => {
  beforeEach(() => {
    subscriptionState.isPro = false;
    dialogOpenMock.mockReset();
  });

  it('Free 用户无 Pro 徽标与升级横幅', () => {
    render(<GuidebookDistillationPanel />, { wrapper });
    expect(screen.queryByText('Pro')).not.toBeInTheDocument();
    expect(screen.queryByText(/指导书提炼为 Pro 功能/)).not.toBeInTheDocument();
    expect(screen.queryByText('升级 Pro')).not.toBeInTheDocument();
  });

  it('Free 用户点击上传直接打开文件对话框', async () => {
    dialogOpenMock.mockResolvedValue(null);
    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    await user.click(screen.getByText('上传指导书'));
    expect(dialogOpenMock).toHaveBeenCalledTimes(1);
  });
});
