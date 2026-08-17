import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiPromptBar } from '../AiPromptBar';

const sources = [
  { key: 'char-lin', name: '林晚', desc: '角色' },
  { key: 'world-canglan', name: '苍澜大陆', desc: '世界观' },
];
const commands = [
  { key: 'auto_write', name: '/自动续写', desc: '从当前位置自动续写' },
  { key: 'auto_revise', name: '/审校', desc: '审校当前章节' },
];
const models = [
  { key: 'a', name: 'Model A', tag: 'Flagship' },
  { key: 'b', name: 'Model B' },
];

describe('AiPromptBar', () => {
  it('受控渲染：显示 value，输入触发 onChange', () => {
    const onChange = vi.fn();
    render(<AiPromptBar value="你好" onChange={onChange} onSend={() => {}} />);
    const ta = screen.getByPlaceholderText('输入任意指令…');
    expect(ta).toHaveValue('你好');
    fireEvent.change(ta, { target: { value: '你好！' } });
    expect(onChange).toHaveBeenCalledWith('你好！');
  });

  it('空输入发送禁用；有内容点击发送触发 onSend', () => {
    const onSend = vi.fn();
    const { rerender } = render(<AiPromptBar value="" onChange={() => {}} onSend={onSend} />);
    expect(screen.getByTitle('发送')).toBeDisabled();
    rerender(<AiPromptBar value="写一段" onChange={() => {}} onSend={onSend} />);
    fireEvent.click(screen.getByTitle('发送'));
    expect(onSend).toHaveBeenCalledTimes(1);
  });

  it('Enter 发送；IME 组合输入中 Enter 不发送', () => {
    const onSend = vi.fn();
    render(<AiPromptBar value="写一段" onChange={() => {}} onSend={onSend} />);
    const ta = screen.getByPlaceholderText('输入任意指令…');
    fireEvent.keyDown(ta, { key: 'Enter' });
    expect(onSend).toHaveBeenCalledTimes(1);
    // 中文输入法组合中（isComposing=true）Enter 仅上屏，不触发发送
    const composing = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true });
    Object.defineProperty(composing, 'isComposing', { value: true });
    ta.dispatchEvent(composing);
    expect(onSend).toHaveBeenCalledTimes(1);
  });

  it('输入 / 打开命令菜单，↓+Enter 选中插入命令文本（去掉 / 前缀）', () => {
    const onChange = vi.fn();
    render(<AiPromptBar value="/" onChange={onChange} onSend={() => {}} commands={commands} />);
    const ta = screen.getByPlaceholderText('输入任意指令…');
    expect(screen.getByTestId('ai-prompt-menu')).toBeInTheDocument();
    fireEvent.keyDown(ta, { key: 'ArrowDown' });
    fireEvent.keyDown(ta, { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith('审校 ');
  });

  it('输入 @ 且传入 sources 时打开数据源菜单；无 sources 不打开', () => {
    const { rerender } = render(<AiPromptBar value="@" onChange={() => {}} onSend={() => {}} />);
    expect(screen.queryByTestId('ai-prompt-menu')).not.toBeInTheDocument();
    rerender(<AiPromptBar value="@" onChange={() => {}} onSend={() => {}} sources={sources} />);
    expect(screen.getByTestId('ai-prompt-menu')).toBeInTheDocument();
    expect(screen.getByText('林晚')).toBeInTheDocument();
  });

  it('Esc 关闭菜单；菜单关闭后 onKeyDown 透传父级', () => {
    const onKeyDown = vi.fn();
    render(
      <AiPromptBar
        value="/"
        onChange={() => {}}
        onSend={() => {}}
        commands={commands}
        onKeyDown={onKeyDown}
      />
    );
    const ta = screen.getByPlaceholderText('输入任意指令…');
    fireEvent.keyDown(ta, { key: 'Escape' }); // 第一次：仅关菜单，不透传
    expect(screen.queryByTestId('ai-prompt-menu')).not.toBeInTheDocument();
    expect(onKeyDown).not.toHaveBeenCalled();
    fireEvent.keyDown(ta, { key: 'ArrowUp' }); // 菜单已关：透传
    expect(onKeyDown).toHaveBeenCalledTimes(1);
  });

  it('models 缺省时不渲染模型选择器；传入后切换模型触发 onModelChange 与扫光', () => {
    const onModelChange = vi.fn();
    const { rerender } = render(<AiPromptBar value="" onChange={() => {}} onSend={() => {}} />);
    expect(screen.queryByLabelText('选择模型')).not.toBeInTheDocument();
    rerender(
      <AiPromptBar
        value=""
        onChange={() => {}}
        onSend={() => {}}
        models={models}
        model="a"
        onModelChange={onModelChange}
      />
    );
    fireEvent.click(screen.getByLabelText('选择模型'));
    fireEvent.click(screen.getByText('Model B'));
    expect(onModelChange).toHaveBeenCalledWith('b');
    expect(screen.getByTestId('ai-sweep-overlay')).toBeInTheDocument();
  });

  it('flush 变体去掉内层边框，发送按钮仍可用', () => {
    const onSend = vi.fn();
    render(<AiPromptBar variant="flush" value="写一段" onChange={() => {}} onSend={onSend} />);
    expect(screen.getByTestId('ai-prompt-bar')).toHaveAttribute('data-variant', 'flush');
    const chrome = screen.getByTestId('ai-prompt-bar').querySelector('[data-chrome]');
    expect(chrome).toHaveAttribute('data-chrome', 'flush');
    expect(chrome).not.toHaveClass('border-ai-line');
    expect(chrome).not.toHaveClass('bg-ai-surface');
    expect(chrome).toHaveClass('bg-transparent');
    fireEvent.click(screen.getByTitle('发送'));
    expect(onSend).toHaveBeenCalledTimes(1);
  });

  it('textarea 去掉系统原生描边，避免 flush 后露出一圈线', () => {
    render(<AiPromptBar variant="flush" value="" onChange={() => {}} onSend={() => {}} />);
    const ta = screen.getByPlaceholderText('输入任意指令…');
    expect(ta.className).toMatch(/\bborder-0\b/);
    expect(ta.className).toMatch(/\bappearance-none\b/);
    expect(ta.className).toMatch(/\bshadow-none\b/);
  });

  it('trailingAction 传入时替换发送按钮', () => {
    render(
      <AiPromptBar
        value="x"
        onChange={() => {}}
        onSend={() => {}}
        trailingAction={<button title="取消生成">x</button>}
      />
    );
    expect(screen.getByTitle('取消生成')).toBeInTheDocument();
    expect(screen.queryByTitle('发送')).not.toBeInTheDocument();
  });

  it('发射键去掉系统原生按钮外观', () => {
    render(<AiPromptBar variant="flush" value="写一段" onChange={() => {}} onSend={() => {}} />);
    const send = screen.getByTitle('发送');
    expect(send.className).toMatch(/\bappearance-none\b/);
    expect(send.className).toMatch(/\bborder-0\b/);
    expect(send.className).toMatch(/\bshadow-none\b/);
  });

  it('有内容时发射键不用 --ai-ink 实心填充', () => {
    render(<AiPromptBar value="写一段" onChange={() => {}} onSend={() => {}} />);
    const send = screen.getByTitle('发送');
    const style = send.getAttribute('style') ?? '';
    expect(style).not.toMatch(/var\(--ai-ink\)/);
    expect(style).toMatch(/color-mix/);
    expect(style).toMatch(/18%/);
    expect(style).toMatch(/--ai-accent-ink/);
  });
});
