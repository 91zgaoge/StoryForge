import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import AgentInterruptionModal from '../AgentInterruptionModal';

const activeRunError = {
  code: 'VALIDATION_FAILED',
  message: '该故事已有进行中的创作任务',
  severity: 'UserAction' as const,
  data: { field: 'active_run' },
};

describe('AgentInterruptionModal — 进行中的创作任务', () => {
  it('不把用户打发到设置，而是说明正在续写', () => {
    render(
      <AgentInterruptionModal
        isOpen
        onClose={vi.fn()}
        error={activeRunError}
        onOpenBackstage={vi.fn()}
        onCancelGeneration={vi.fn()}
      />
    );

    expect(screen.getByText('正在续写中')).toBeInTheDocument();
    expect(screen.getByText(/不用去设置/)).toBeInTheDocument();
    expect(screen.queryByText('前往设置')).not.toBeInTheDocument();
    expect(screen.queryByText('需要您先处理')).not.toBeInTheDocument();
    expect(screen.getByText('知道了')).toBeInTheDocument();
    expect(screen.getByText('取消当前续写')).toBeInTheDocument();
  });

  it('取消当前续写会调用 onCancelGeneration 并关闭', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onCancelGeneration = vi.fn();
    render(
      <AgentInterruptionModal
        isOpen
        onClose={onClose}
        error={activeRunError}
        onOpenBackstage={vi.fn()}
        onCancelGeneration={onCancelGeneration}
      />
    );

    await user.click(screen.getByText('取消当前续写'));
    expect(onCancelGeneration).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
