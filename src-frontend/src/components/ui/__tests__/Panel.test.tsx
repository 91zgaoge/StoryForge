import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Panel } from '../Panel';

describe('Panel', () => {
  it('renders title and children', () => {
    render(<Panel title="Settings">content</Panel>);
    expect(screen.getByText('Settings')).toBeInTheDocument();
    expect(screen.getByText('content')).toBeInTheDocument();
  });

  it('collapses on click', () => {
    render(
      <Panel title="Advanced" collapsible>
        secret
      </Panel>
    );
    const content = screen.getByText('secret').parentElement;
    fireEvent.click(screen.getByText('Advanced'));
    expect(content).toHaveClass('max-h-0');
    expect(content).toHaveClass('opacity-0');
  });

  it('starts collapsed when defaultOpen is false', () => {
    render(
      <Panel title="Hidden" collapsible defaultOpen={false}>
        secret
      </Panel>
    );
    const content = screen.getByText('secret').parentElement;
    expect(content).toHaveClass('max-h-0');
    expect(content).toHaveClass('opacity-0');
  });

  it('does not collapse when not collapsible', () => {
    render(
      <Panel title="Static" collapsible={false}>
        secret
      </Panel>
    );
    const content = screen.getByText('secret').parentElement;
    expect(content).toHaveClass('max-h-[1000px]');
    expect(content).toHaveClass('opacity-100');

    fireEvent.click(screen.getByText('Static'));
    expect(content).toHaveClass('max-h-[1000px]');
    expect(content).toHaveClass('opacity-100');
  });

  it('re-opens after being collapsed', () => {
    render(
      <Panel title="Toggle" collapsible>
        secret
      </Panel>
    );
    const content = screen.getByText('secret').parentElement;
    const header = screen.getByText('Toggle');

    fireEvent.click(header);
    expect(content).toHaveClass('max-h-0');
    expect(content).toHaveClass('opacity-0');

    fireEvent.click(header);
    expect(content).toHaveClass('max-h-[1000px]');
    expect(content).toHaveClass('opacity-100');
  });

  it('exposes accessibility attributes on the collapsible header', () => {
    render(
      <Panel title="Accessible" collapsible>
        secret
      </Panel>
    );
    const button = screen.getByRole('button', { name: 'Accessible' });
    expect(button).toHaveAttribute('aria-expanded', 'true');
    const content = screen.getByText('secret').parentElement;
    expect(button).toHaveAttribute('aria-controls', content?.id);

    fireEvent.click(button);
    expect(button).toHaveAttribute('aria-expanded', 'false');
  });
});
