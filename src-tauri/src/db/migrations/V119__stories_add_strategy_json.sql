-- V119: stories 表新增 strategy_json —— 持久化向导选中的创作策略四元组
-- （beat_card_ids / story_engine_ids / pressure_relationship_id /
--  emotional_payoff / conflict_arena），JSON 文本。NULL 表示旧数据，
-- build_selected_strategy 对 NULL 走既有启发式推断，行为不变。
ALTER TABLE stories ADD COLUMN strategy_json TEXT;
