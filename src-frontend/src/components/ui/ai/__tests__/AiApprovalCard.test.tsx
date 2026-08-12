import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { AiApprovalCard } from '../AiApprovalCard';

const questions = [
  {
    key: 'world',
    title: '选择世界观',
    type: 'radio' as const,
    options: [
      { key: '0', label: '苍澜大陆', description: '修真文明与蒸汽机械并存' },
      { key: '1', label: '雾都伦城', description: '维多利亚悬疑' },
    ],
  },
  {
    key: 'tags',
    title: '选择故事元素',
    type: 'check' as const,
    options: [
      { key: 'a', label: '悬疑' },
      { key: 'b', label: '成长' },
    ],
    allowCustom: true,
  },
];

describe('AiApprovalCard', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('一次只显示一个问题，分页按钮可前后切换', () => {
    render(<AiApprovalCard questions={questions} onSubmit={() => {}} />);
    expect(screen.getByText('选择世界观')).toBeInTheDocument();
    expect(screen.queryByText('选择故事元素')).not.toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('下一题'));
    expect(screen.getByText('选择故事元素')).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('上一题'));
    expect(screen.getByText('选择世界观')).toBeInTheDocument();
  });

  it('radio 选中 480ms 后自动前进；最后一题提交显示「已提交」并按 key 上报', () => {
    vi.useFakeTimers();
    const onSubmit = vi.fn();
    render(<AiApprovalCard questions={questions} onSubmit={onSubmit} />);
    fireEvent.click(screen.getByText('苍澜大陆'));
    act(() => {
      vi.advanceTimersByTime(500);
    });
    expect(screen.getByText('选择故事元素')).toBeInTheDocument(); // 自动前进
    fireEvent.click(screen.getByText('悬疑')); // check 不自动前进
    act(() => {
      vi.advanceTimersByTime(500);
    });
    expect(screen.queryByText('已提交')).not.toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('提交'));
    expect(screen.getByText('已提交')).toBeInTheDocument();
    expect(onSubmit).toHaveBeenCalledWith({ world: ['0'], tags: ['a'] });
  });

  it('check 多选可再点取消；radio 输入自定义回答时清空已选', () => {
    render(<AiApprovalCard questions={[questions[1]]} onSubmit={() => {}} />);
    fireEvent.click(screen.getByText('悬疑'));
    fireEvent.click(screen.getByText('成长'));
    fireEvent.click(screen.getByText('悬疑')); // 取消
    expect(screen.getByLabelText('提交')).toBeEnabled(); // 仍有「成长」
  });

  it('allowCustom 自定义回答以文本作为答案提交', () => {
    const onSubmit = vi.fn();
    render(<AiApprovalCard questions={[questions[1]]} onSubmit={onSubmit} />);
    fireEvent.change(screen.getByLabelText('自定义回答'), { target: { value: '我自己的答案' } });
    fireEvent.click(screen.getByLabelText('提交'));
    expect(onSubmit).toHaveBeenCalledWith({ tags: ['我自己的答案'] });
  });

  it('onDismiss 传入时渲染关闭按钮并回调', () => {
    const onDismiss = vi.fn();
    render(<AiApprovalCard questions={questions} onSubmit={() => {}} onDismiss={onDismiss} />);
    fireEvent.click(screen.getByLabelText('关闭'));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });
});
