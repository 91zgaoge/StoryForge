import { describe, it, expect, vi, beforeEach } from 'vitest';
import { applyWizardToStory } from '../applyWizardToStory';
import { loggedInvoke } from '@/services/tauri';

vi.mock('@/services/tauri', () => ({
  loggedInvoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(loggedInvoke);

const baseStory = {
  id: 'story-1',
  title: '测试故事',
  genre: '末世',
  style_dna_id: null,
  genre_profile_id: null,
  methodology_id: null,
} as any;

const baseWizardData = {
  worldBuilding: { concept: '废土', history: '百年战争' },
  characters: [],
  writingStyle: { name: '冷峻' },
  firstScene: { title: '开场' },
  genreInput: '末世',
} as any;

beforeEach(() => {
  mockedInvoke.mockReset();
  mockedInvoke.mockResolvedValue({
    story: baseStory,
    world_building: {},
    writing_style: {},
    first_scene: {},
    characters: [],
    ingested_entities: 0,
    ingested_relations: 0,
  } as any);
});

describe('applyWizardToStory 策略四元组落库', () => {
  it('向导选中的四元组全部传给后端', async () => {
    await applyWizardToStory(baseStory, {
      ...baseWizardData,
      selectedStrategy: {
        style_dna_ids: [],
        genre_profile_id: 'gp1',
        methodology_id: 'snowflake',
        beat_card_ids: ['beat_a'],
        story_engine_ids: ['engine_x', 'engine_y'],
        pressure_relationship_id: 'rel_debt',
        emotional_payoff: '爽',
        conflict_arena: '公开审查',
      },
    });
    expect(mockedInvoke).toHaveBeenCalledWith(
      'apply_wizard_to_story',
      expect.objectContaining({
        beat_card_ids: ['beat_a'],
        story_engine_ids: ['engine_x', 'engine_y'],
        pressure_relationship_id: 'rel_debt',
        emotional_payoff: '爽',
        conflict_arena: '公开审查',
      })
    );
  });

  it('未选策略时四元组传 null（不污染旧数据）', async () => {
    await applyWizardToStory(baseStory, baseWizardData);
    expect(mockedInvoke).toHaveBeenCalledWith(
      'apply_wizard_to_story',
      expect.objectContaining({
        beat_card_ids: null,
        story_engine_ids: null,
        pressure_relationship_id: null,
        emotional_payoff: null,
        conflict_arena: null,
      })
    );
  });
});
