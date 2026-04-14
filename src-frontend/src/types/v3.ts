// V3 架构类型定义

// ==================== 场景类型 ====================

export type ConflictType = 
  | 'ManVsMan'        // 人与人
  | 'ManVsSelf'       // 人与自我
  | 'ManVsSociety'    // 人与社会
  | 'ManVsNature'     // 人与自然
  | 'ManVsTechnology' // 人与科技
  | 'ManVsFate'       // 人与命运
  | 'ManVsSupernatural'; // 人与超自然

export interface CharacterConflict {
  character_a_id: string;
  character_b_id: string;
  conflict_nature: string;
  stakes: string;
}

export interface Scene {
  id: string;
  story_id: string;
  sequence_number: number;
  title?: string;
  
  // 戏剧结构
  dramatic_goal?: string;
  external_pressure?: string;
  conflict_type?: ConflictType;
  
  // 角色参与
  characters_present: string[];
  character_conflicts: CharacterConflict[];
  
  // 内容
  content?: string;
  
  // 场景设置
  setting_location?: string;
  setting_time?: string;
  setting_atmosphere?: string;
  
  // 关联
  previous_scene_id?: string;
  next_scene_id?: string;
  
  // 元数据
  model_used?: string;
  cost?: number;
  created_at: string;
  updated_at: string;
}

// ==================== 世界观类型 ====================

export type RuleType = 
  | 'Magic'       // 魔法规则
  | 'Technology'  // 科技规则
  | 'Social'      // 社会规则
  | 'Physical'    // 物理规则
  | 'Biological'  // 生物规则
  | 'Historical'  // 历史规则
  | 'Cultural'    // 文化规则
  | 'Custom';     // 自定义

export interface WorldRule {
  id: string;
  name: string;
  description?: string;
  rule_type: RuleType;
  importance: number; // 1-10
}

export interface Culture {
  name: string;
  description: string;
  customs: string[];
  values: string[];
}

export interface WorldBuilding {
  id: string;
  story_id: string;
  concept: string;
  rules: WorldRule[];
  history?: string;
  cultures: Culture[];
  created_at: string;
  updated_at: string;
}

// ==================== 场景设置类型 ====================

export type LocationType = 
  | 'City'
  | 'Building'
  | 'Nature'
  | 'Underground'
  | 'Underwater'
  | 'Space'
  | 'Dream'
  | 'Virtual'
  | { Custom: string };

export interface SensoryDetails {
  visual: string[];
  auditory: string[];
  olfactory: string[];
  tactile: string[];
  gustatory: string[];
}

export interface Setting {
  id: string;
  story_id: string;
  name: string;
  description?: string;
  location_type: LocationType;
  sensory_details: SensoryDetails;
  significance?: string;
  created_at: string;
}

// ==================== 文字风格类型 ====================

export interface WritingStyle {
  id: string;
  story_id: string;
  name?: string;
  description?: string;
  tone?: string;
  pacing?: string;
  vocabulary_level?: string;
  sentence_structure?: string;
  custom_rules: string[];
  created_at: string;
  updated_at: string;
}

// ==================== 工作室配置类型 ====================

export interface LlmProfile {
  id: string;
  name: string;
  provider: string;
  model: string;
  api_key?: string;
  base_url?: string;
  temperature: number;
  max_tokens: number;
}

export interface LlmStudioConfig {
  default_provider: string;
  default_model: string;
  generation_temperature: number;
  max_tokens: number;
  profiles: LlmProfile[];
}

export interface UiStudioConfig {
  frontstage_font_size: number;
  frontstage_font_family: string;
  frontstage_line_height: number;
  frontstage_paper_color: string;
  frontstage_text_color: string;
  backstage_theme: string;
  backstage_accent_color: string;
}

export type AgentBotType = 
  | 'WorldBuilding'  // 世界观助手
  | 'Character'      // 人物助手
  | 'WritingStyle'   // 文风助手
  | 'Plot'           // 情节助手
  | 'Scene'          // 场景助手
  | 'Memory';        // 记忆助手

export interface AgentBotConfig {
  id: string;
  name: string;
  agent_type: AgentBotType;
  enabled: boolean;
  llm_profile_id: string;
  system_prompt: string;
  custom_settings: Record<string, unknown>;
}

export interface StudioConfig {
  id: string;
  story_id: string;
  pen_name?: string;
  llm_config: LlmStudioConfig;
  ui_config: UiStudioConfig;
  agent_bots: AgentBotConfig[];
  frontstage_theme?: string;
  backstage_theme?: string;
  created_at: string;
  updated_at: string;
}

// ==================== 知识图谱类型 ====================

export type EntityType = 
  | 'Character'
  | 'Location'
  | 'Item'
  | 'Organization'
  | 'Concept'
  | 'Event';

export interface Entity {
  id: string;
  story_id: string;
  name: string;
  entity_type: EntityType;
  attributes: Record<string, unknown>;
  embedding?: number[];
  first_seen: string;
  last_updated: string;
  confidence_score?: number;
  access_count: number;
  last_accessed?: string;
  is_archived: boolean;
  archived_at?: string;
}

export type RelationType = 
  // 人际关系
  | 'Friend' | 'Enemy' | 'Family' | 'Lover' | 'Mentor' | 'Rival' | 'Ally'
  // 物品关系
  | 'LocatedAt' | 'BelongsTo' | 'Uses' | 'Owns' | 'Created' | 'Destroyed'
  // 组织关系
  | 'PartOf' | 'Leads' | 'MemberOf' | 'FounderOf'
  // 因果关系
  | 'Causes' | 'Enables' | 'Prevents' | 'ResultsIn'
  // 语义关系
  | 'SimilarTo' | 'OppositeOf' | 'RelatedTo' | 'EvolvesInto';

export interface Relation {
  id: string;
  story_id: string;
  source_id: string;
  target_id: string;
  relation_type: RelationType;
  strength: number; // 0-1
  evidence: string[];
  first_seen: string;
}

export interface StoryGraph {
  entities: Entity[];
  relations: Relation[];
}

// ==================== 请求/响应类型 ====================

export interface CreateSceneRequest {
  story_id: string;
  sequence_number: number;
  title?: string;
  dramatic_goal?: string;
  external_pressure?: string;
  conflict_type?: ConflictType;
  characters_present: string[];
  setting_location?: string;
  content?: string;
}

export interface UpdateSceneRequest {
  title?: string;
  dramatic_goal?: string;
  external_pressure?: string;
  conflict_type?: ConflictType;
  characters_present?: string[];
  character_conflicts?: CharacterConflict[];
  content?: string;
  setting_location?: string;
  setting_time?: string;
  setting_atmosphere?: string;
}

export interface WritingStyleUpdate {
  name?: string;
  description?: string;
  tone?: string;
  pacing?: string;
  vocabulary_level?: string;
  sentence_structure?: string;
  custom_rules?: string[];
}

export interface StudioExportRequest {
  story_id: string;
  include_world_building: boolean;
  include_characters: boolean;
  include_writing_style: boolean;
  include_scenes: boolean;
  include_llm_config: boolean;
  include_ui_config: boolean;
  include_agent_bots: boolean;
}

export interface StudioExportData {
  manifest: ExportManifest;
  story: import('./index').Story;
  world_building?: WorldBuilding;
  characters: import('./index').Character[];
  writing_style?: WritingStyle;
  scenes: Scene[];
  studio_config?: StudioConfig;
}

export interface ExportManifest {
  version: string;
  exported_at: string;
  story_id: string;
  story_title: string;
}

export interface ImportOptions {
  include_world_building: boolean;
  include_characters: boolean;
  include_writing_style: boolean;
  include_scenes: boolean;
  include_llm_config: boolean;
  include_ui_config: boolean;
  include_agent_bots: boolean;
  skip_existing: boolean;
  merge_existing: boolean;
}

// ==================== AI生成类型 ====================

export interface WorldBuildingOption {
  id: string;
  concept: string;
  rules: WorldRule[];
  history?: string;
  cultures: Culture[];
}

export interface CharacterProfileOption {
  id: string;
  name: string;
  personality: string;
  background: string;
  goals: string;
  voice_style: string;
}

export interface WritingStyleOption {
  id: string;
  name: string;
  description: string;
  tone: string;
  pacing: string;
  vocabulary_level: string;
  sentence_structure: string;
  sample_text: string;
}

export interface NovelCreationProgress {
  step: 'genre_input' | 'generating_world' | 'selecting_world' | 'generating_characters' | 'selecting_characters' | 'generating_style' | 'selecting_style' | 'generating_first_scene' | 'completed';
  message: string;
  progress: number; // 0-100
}

// ==================== 场景版本类型 (Phase 3.x) ====================

export type CreatorType = 'user' | 'ai' | 'system';

export interface SceneVersion {
  id: string;
  scene_id: string;
  version_number: number;
  
  // 内容快照
  title?: string;
  content?: string;
  dramatic_goal?: string;
  external_pressure?: string;
  conflict_type?: ConflictType;
  characters_present: string[];
  character_conflicts: CharacterConflict[];
  setting_location?: string;
  setting_time?: string;
  setting_atmosphere?: string;
  
  // 版本元数据
  word_count: number;
  change_summary: string;
  created_by: CreatorType;
  model_used?: string;
  confidence_score?: number;
  
  // 版本链
  previous_version_id?: string;
  superseded_by?: string;
  
  created_at: string;
}

export interface VersionDiff {
  from_version: number;
  to_version: number;
  content_diff?: TextDiff;
  title_changed: boolean;
  setting_changed: boolean;
  characters_changed: boolean;
  dramatic_goal_changed: boolean;
  word_count_delta: number;
  confidence_delta: number;
}

export interface TextDiff {
  added_lines: string[];
  removed_lines: string[];
  unchanged_percentage: number;
}

export interface VersionChainNode {
  version: SceneVersion;
  children: string[];
  depth: number;
}

export interface VersionStats {
  total_versions: number;
  avg_confidence: number;
  best_version_id?: string;
  best_version_number?: number;
  user_edits: number;
  ai_edits: number;
  system_edits: number;
  total_word_delta: number;
  first_version_at?: string;
  last_version_at?: string;
}

// ==================== 保留/遗忘曲线类型 (Phase 1.4) ====================

export type PriorityLevel = 'critical' | 'high' | 'medium' | 'low' | 'forgotten';

export interface RetentionScore {
  entity_id: string;
  entity_name: string;
  base_score: number;
  decayed_score: number;
  reinforced_score: number;
  final_priority: number;
  priority_level: PriorityLevel;
  days_since_last_access: number;
  access_count: number;
  estimated_retention_days: number;
}

export interface RetentionReport {
  total_entities: number;
  avg_priority: number;
  level_distribution: Record<string, number>;
  critical_entities: string[];
  forgotten_entities: string[];
  recommended_action: string;
}

export interface ArchiveResult {
  archived_count: number;
  archived_entities: string[];
  story_id: string;
}

export interface RetentionScore {
  entity_id: string;
  entity_name: string;
  base_score: number;
  decayed_score: number;
  reinforced_score: number;
  final_priority: number;
  priority_level: PriorityLevel;
  days_since_last_access: number;
  access_count: number;
  estimated_retention_days: number;
}
