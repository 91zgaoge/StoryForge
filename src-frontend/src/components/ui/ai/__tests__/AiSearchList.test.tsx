import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiSearchList } from '../AiSearchList';

describe('AiSearchList', () => {
  it('渲染输入框（placeholder 与 aria-label）', () => {
    render(
      <AiSearchList value="" onChange={() => {}} placeholder="搜索提示词…" ariaLabel="搜索提示词" />
    );
    expect(screen.getByLabelText('搜索提示词')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('搜索提示词…')).toBeInTheDocument();
  });

  it('输入调用 onChange(新值)', () => {
    const onChange = vi.fn();
    render(<AiSearchList value="" onChange={onChange} />);
    fireEvent.change(screen.getByLabelText('搜索'), { target: { value: '写' } });
    expect(onChange).toHaveBeenCalledWith('写');
  });

  it('有值时渲染清除按钮，点击调用 onChange(空串)', () => {
    const onChange = vi.fn();
    render(<AiSearchList value="写作" onChange={onChange} />);
    fireEvent.click(screen.getByRole('button', { name: '清除搜索' }));
    expect(onChange).toHaveBeenCalledWith('');
  });

  it('空值时不渲染清除按钮与计数行', () => {
    render(<AiSearchList value="" onChange={() => {}} resultCount={3} />);
    expect(screen.queryByRole('button', { name: '清除搜索' })).not.toBeInTheDocument();
    expect(screen.queryByTestId('ai-search-count')).not.toBeInTheDocument();
  });

  it('有值且提供 resultCount>0 时渲染计数行', () => {
    render(<AiSearchList value="写作" onChange={() => {}} resultCount={5} />);
    expect(screen.getByTestId('ai-search-count')).toHaveTextContent('搜索 “写作” 找到 5 条结果');
  });

  it('resultCount=0 时渲染空态而非计数行', () => {
    render(
      <AiSearchList
        value="zzz"
        onChange={() => {}}
        resultCount={0}
        emptyText="未找到匹配的提示词"
        emptyHint="尝试调整搜索条件"
      />
    );
    expect(screen.getByTestId('ai-search-empty')).toBeInTheDocument();
    expect(screen.getByText('未找到匹配的提示词')).toBeInTheDocument();
    expect(screen.getByText('尝试调整搜索条件')).toBeInTheDocument();
    expect(screen.queryByTestId('ai-search-count')).not.toBeInTheDocument();
  });
});
