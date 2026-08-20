import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import AgentInterruptionModal from '../AgentInterruptionModal';

const activeRunError = {
  code: 'VALIDATION_FAILED',
  message: '该故事已有进行中的创作任务',
  severity: 'UserAction' as const,
  data: { field: 'active_run' },
};

describe('AgentInterruptionModal — 进行中的创作任务', () => {
  it('不盖住纸面：进行中的续写不是需要用户处理的中断', () => {
    const { container } = render(
      <AgentInterruptionModal isOpen onClose={vi.fn()} error={activeRunError} />
    );

    expect(container.firstChild).toBeNull();
    expect(screen.queryByText('正在续写中')).not.toBeInTheDocument();
    expect(screen.queryByText('需要您先处理')).not.toBeInTheDocument();
    expect(screen.queryByText('前往设置')).not.toBeInTheDocument();
  });
});
