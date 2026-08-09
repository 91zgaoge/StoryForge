import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { UpgradeModal } from '../UpgradeModal';

const { authState, loginMock, devUpgradeMock } = vi.hoisted(() => ({
  authState: { isLoggedIn: false, isWaitingForOAuth: false },
  loginMock: vi.fn(),
  devUpgradeMock: vi.fn(),
}));

vi.mock('@/stores/useAuthStore', () => ({
  useAuthStore: () => ({
    isLoggedIn: authState.isLoggedIn,
    isWaitingForOAuth: authState.isWaitingForOAuth,
    login: loginMock,
  }),
}));

vi.mock('@/services/tauri', () => ({
  devUpgradeSubscription: (...args: unknown[]) => devUpgradeMock(...args),
}));

const renderModal = (props: Partial<React.ComponentProps<typeof UpgradeModal>> = {}) =>
  render(<UpgradeModal isOpen onClose={vi.fn()} featureName="指导书提炼" {...props} />);

describe('UpgradeModal - 登录引导（Task 6）', () => {
  beforeEach(() => {
    authState.isLoggedIn = false;
    authState.isWaitingForOAuth = false;
    loginMock.mockReset();
    devUpgradeMock.mockReset();
  });

  it('未登录：显示登录引导区与「暂不登录，仅本设备升级」', () => {
    renderModal();
    expect(screen.getByText(/登录后升级，Pro 跟随账号/)).toBeInTheDocument();
    expect(screen.getByText('Google 登录')).toBeInTheDocument();
    expect(screen.getByText('GitHub 登录')).toBeInTheDocument();
    expect(screen.getByText('暂不登录，仅本设备升级')).toBeInTheDocument();
    expect(screen.queryByText('立即升级')).not.toBeInTheDocument();
  });

  it("未登录：点「暂不登录，仅本设备升级」调 devUpgradeSubscription('pro')", async () => {
    devUpgradeMock.mockResolvedValue({ tier: 'pro' });
    const user = userEvent.setup();
    renderModal();

    await user.click(screen.getByText('暂不登录，仅本设备升级'));
    expect(devUpgradeMock).toHaveBeenCalledWith('pro');
  });

  it('已登录：不显示登录引导，「立即升级」可用', async () => {
    authState.isLoggedIn = true;
    devUpgradeMock.mockResolvedValue({ tier: 'pro' });
    const user = userEvent.setup();
    renderModal();

    expect(screen.queryByText(/登录后升级，Pro 跟随账号/)).not.toBeInTheDocument();
    expect(screen.queryByText('暂不登录，仅本设备升级')).not.toBeInTheDocument();

    await user.click(screen.getByText('立即升级'));
    expect(devUpgradeMock).toHaveBeenCalledWith('pro');
  });
});
