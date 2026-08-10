import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CharacterRelationshipForm } from '../CharacterRelationshipForm';

// mock 创建 mutation hook（组件只用到 mutate/isPending）
vi.mock('@/hooks/useCharacterRelationships', () => ({
  useCreateCharacterRelationship: () => ({
    mutate: vi.fn(),
    isPending: false,
  }),
}));

describe('CharacterRelationshipForm', () => {
  it('renders emotional bond fields', () => {
    render(
      <CharacterRelationshipForm storyId="s1" characters={[]} isOpen={true} onClose={() => {}} />
    );
    expect(screen.getByText(/A对B的情感/)).toBeInTheDocument();
    expect(screen.getByText(/B对A的情感/)).toBeInTheDocument();
  });
});
