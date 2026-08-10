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

const { subscriptionState, dialogOpenMock, retryMock } = vi.hoisted(() => ({
  subscriptionState: { isPro: false },
  dialogOpenMock: vi.fn(),
  retryMock: vi.fn(),
}));

vi.mock('@/hooks/useSubscription', () => ({
  useSubscription: () => ({
    isPro: subscriptionState.isPro,
    fetchStatus: () => Promise.resolve(),
  }),
}));

vi.mock('@/hooks/useGuidebookDistillation', () => ({
  useGuidebooks: vi.fn(() => ({ data: [], isLoading: false })),
  useUploadGuidebook: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useDeleteGuidebook: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useGuidebookDistillationStatus: () => ({ data: null }),
  useCancelGuidebookDistillation: () => ({ mutateAsync: vi.fn() }),
  useRetryGuidebookDistillation: () => ({ mutateAsync: retryMock, isPending: false }),
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

describe('GuidebookCard - 失败重试（v0.36.0）', () => {
  beforeEach(() => {
    retryMock.mockReset();
  });

  it('失败状态的卡片显示重试按钮，点击调用 retry', async () => {
    // 直接渲染卡片列表：mock useGuidebooks 返回一条 failed 记录
    const failedBook = {
      id: 'g-fail',
      title: '失败的书',
      author: null,
      subject: null,
      word_count: 1000,
      file_format: 'txt',
      methodology_id: null,
      status: 'failed',
      progress: 0,
      created_at: '2026-08-09',
    };
    vi.mocked((await import('@/hooks/useGuidebookDistillation')).useGuidebooks).mockReturnValue({
      data: [failedBook],
      isLoading: false,
    } as never);

    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    const btn = screen.getByText('重试提炼');
    await user.click(btn);
    expect(retryMock).toHaveBeenCalledWith('g-fail');
  });

  it('completed 状态的卡片不显示重试按钮', async () => {
    const doneBook = {
      id: 'g-done',
      title: '完成的书',
      author: null,
      subject: null,
      word_count: 1000,
      file_format: 'txt',
      methodology_id: 'custom_x',
      status: 'completed',
      progress: 100,
      created_at: '2026-08-09',
    };
    vi.mocked((await import('@/hooks/useGuidebookDistillation')).useGuidebooks).mockReturnValue({
      data: [doneBook],
      isLoading: false,
    } as never);

    render(<GuidebookDistillationPanel />, { wrapper });
    expect(screen.queryByText('重试提炼')).not.toBeInTheDocument();
  });
});
