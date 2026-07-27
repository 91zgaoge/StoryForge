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
});
