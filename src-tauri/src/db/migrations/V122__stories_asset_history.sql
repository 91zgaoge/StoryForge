-- v0.34.0 弹性扩张：资产选用历史（JSON 数组 [{chapter, ids}]，用于资产轮换排除）
ALTER TABLE stories ADD COLUMN asset_history_json TEXT;
