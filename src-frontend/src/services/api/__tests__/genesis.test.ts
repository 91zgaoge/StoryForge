import { describe, it, expect, vi } from 'vitest';
import { createCharacterRelationship, updateCharacterRelationship } from '../genesis';

// Mock Tauri invoke（与 services/__tests__/settings.test.ts 同一模式）
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('createCharacterRelationship', () => {
  it('sends source_character_id/target_character_id and emotional fields (not character_a_id)', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({});
    await createCharacterRelationship({
      story_id: 's1',
      source_character_id: 'char-a',
      target_character_id: 'char-b',
      relationship_type: '师徒',
      emotional_bond: '欺骗',
      emotional_intensity: 0.9,
      reverse_emotional_bond: '崇拜',
      reverse_emotional_intensity: 0.7,
    });
    expect(invoke).toHaveBeenCalledWith(
      'create_character_relationship',
      expect.objectContaining({
        source_character_id: 'char-a',
        target_character_id: 'char-b',
        emotional_bond: '欺骗',
        emotional_intensity: 0.9,
        reverse_emotional_bond: '崇拜',
        reverse_emotional_intensity: 0.7,
      })
    );
    const args = vi.mocked(invoke).mock.calls[0][1] as Record<string, unknown>;
    expect(args).not.toHaveProperty('character_a_id');
    expect(args).not.toHaveProperty('character_b_id');
  });
});

describe('updateCharacterRelationship', () => {
  it('forwards emotional fields to update_character_relationship', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);
    await updateCharacterRelationship('rel-1', {
      relationship_type: '仇敌',
      emotional_bond: '憎恨',
      reverse_emotional_intensity: 0.4,
    });
    expect(invoke).toHaveBeenCalledWith(
      'update_character_relationship',
      expect.objectContaining({
        relationship_id: 'rel-1',
        relationship_type: '仇敌',
        emotional_bond: '憎恨',
        reverse_emotional_intensity: 0.4,
      })
    );
  });
});
