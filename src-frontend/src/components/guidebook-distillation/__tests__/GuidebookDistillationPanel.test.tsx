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

const { subscriptionState, dialogOpenMock, devUpgradeMock } = vi.hoisted(() => ({
  subscriptionState: { isPro: false },
  dialogOpenMock: vi.fn(),
  devUpgradeMock: vi.fn(),
}));

vi.mock('@/hooks/useSubscription', () => ({
  useSubscription: () => ({
    isPro: subscriptionState.isPro,
    // 模拟刷新订阅状态：升级成功后后端已为 Pro
    fetchStatus: () => {
      subscriptionState.isPro = true;
      return Promise.resolve();
    },
  }),
}));

vi.mock('@/services/tauri', () => ({
  devUpgradeSubscription: (...args: unknown[]) => devUpgradeMock(...args),
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

describe('GuidebookDistillationPanel - Pro 门控（v0.33.7）', () => {
  beforeEach(() => {
    subscriptionState.isPro = false;
    dialogOpenMock.mockReset();
    devUpgradeMock.mockReset();
  });

  it('Free 用户看到 Pro 徽标与升级横幅', () => {
    render(<GuidebookDistillationPanel />, { wrapper });
    expect(screen.getByText('Pro')).toBeInTheDocument();
    expect(screen.getByText(/指导书提炼为 Pro 功能/)).toBeInTheDocument();
    expect(screen.getByText('升级 Pro')).toBeInTheDocument();
  });

  it('Free 用户点击上传不弹文件对话框，改为打开升级弹窗', async () => {
    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    await user.click(screen.getByText('上传指导书'));
    expect(dialogOpenMock).not.toHaveBeenCalled();
    expect(screen.getByText('「指导书提炼」需要 Pro')).toBeInTheDocument();
  });

  it('Free 用户点击横幅「升级 Pro」打开升级弹窗', async () => {
    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    await user.click(screen.getByText('升级 Pro'));
    expect(screen.getByText('「指导书提炼」需要 Pro')).toBeInTheDocument();
  });

  it('Pro 用户无徽标与横幅，点击上传会打开文件对话框', async () => {
    subscriptionState.isPro = true;
    dialogOpenMock.mockResolvedValue(null);
    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    expect(screen.queryByText(/指导书提炼为 Pro 功能/)).not.toBeInTheDocument();
    await user.click(screen.getByText('上传指导书'));
    expect(dialogOpenMock).toHaveBeenCalledTimes(1);
  });

  it('升级链路：弹窗点「立即升级」后横幅消失、上传入口解锁', async () => {
    devUpgradeMock.mockResolvedValue({ tier: 'pro' });
    dialogOpenMock.mockResolvedValue(null);
    const user = userEvent.setup();
    render(<GuidebookDistillationPanel />, { wrapper });

    // Free 态：横幅存在，上传被拦截
    await user.click(screen.getByText('升级 Pro'));
    expect(screen.getByText('「指导书提炼」需要 Pro')).toBeInTheDocument();

    await user.click(screen.getByText('立即升级'));
    expect(devUpgradeMock).toHaveBeenCalledWith('pro');

    // 升级后：弹窗关闭、横幅消失
    expect(screen.queryByText('「指导书提炼」需要 Pro')).not.toBeInTheDocument();
    expect(screen.queryByText(/指导书提炼为 Pro 功能/)).not.toBeInTheDocument();

    // 上传入口已解锁
    await user.click(screen.getByText('上传指导书'));
    expect(dialogOpenMock).toHaveBeenCalledTimes(1);
  });
});
