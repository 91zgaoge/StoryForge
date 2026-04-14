// CINEMA-AI Frontend Types
export * from './llm';
export * from './v3';

export interface User {
  id: string;
  name: string;
  email?: string;
  avatar?: string;
}

export interface Story {
  id: string;
  title: string;
  description?: string;
  genre?: string;
  tone?: string;
  pacing?: string;
  character_count?: number;
  chapter_count?: number;
  created_at: string;
  updated_at: string;
}

export interface Character {
  id: string;
  story_id: string;
  name: string;
  background?: string;
  personality?: string;
  goals?: string;
  created_at: string;
  updated_at: string;
}

export interface Chapter {
  id: string;
  story_id: string;
  title: string;
  outline?: string;
  content?: string;
  chapter_number: number;
  status: 'draft' | 'outline' | 'completed';
  word_count?: number;
  created_at: string;
  updated_at: string;
}

export type SkillCategory = 
  | 'writing' 
  | 'analysis' 
  | 'character' 
  | 'world_building' 
  | 'style' 
  | 'plot' 
  | 'export' 
  | 'integration' 
  | 'custom';

export interface Skill {
  id: string;
  name: string;
  description?: string;
  category: SkillCategory;
  version: string;
  author?: string;
  enabled: boolean;
  builtin: boolean;
}

export interface McpServer {
  id: string;
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  enabled: boolean;
  tools?: McpTool[];
}

export interface McpTool {
  name: string;
  description?: string;
  input_schema?: Record<string, unknown>;
}

export interface LlmConfig {
  provider: 'openai' | 'anthropic' | 'ollama';
  api_key?: string;
  model: string;
  temperature: number;
  max_tokens: number;
  base_url?: string;
}

export interface AppSettings {
  llm: LlmConfig;
  theme: 'dark' | 'light' | 'system';
  language: string;
  auto_save: boolean;
}

export interface DashboardState {
  current_story?: Story;
  stories_count: number;
  characters_count: number;
  chapters_count: number;
}

export interface CreateStoryRequest {
  title: string;
  description?: string;
  genre?: string;
}

export interface CreateCharacterRequest {
  story_id: string;
  name: string;
  background?: string;
}

export interface UpdateChapterRequest {
  title?: string;
  outline?: string;
  content?: string;
}

export interface SimilarityResult {
  id: string;
  text: string;
  score: number;
  chapter_id: string;
  chapter_number: number;
}

export interface VectorSearchRequest {
  story_id: string;
  query: string;
  top_k?: number;
}

export type ViewType = 
  | 'dashboard' 
  | 'stories' 
  | 'characters' 
  | 'scenes' 
  | 'knowledge-graph'
  | 'skills' 
  | 'mcp' 
  | 'settings';

// ===== Intent Engine Types =====

export type IntentType =
  | 'text_generate'
  | 'text_rewrite'
  | 'plot_suggest'
  | 'character_check'
  | 'world_consistency'
  | 'style_shift'
  | 'memory_ingest'
  | 'visual_generate'
  | 'scene_reorder'
  | 'outline_expand'
  | 'unknown';

export type ExecutionMode = 'serial' | 'parallel';

export type FeedbackType =
  | 'direct_apply'
  | 'suggestion_card'
  | 'diff_preview'
  | 'system_notice'
  | 'visual_highlight';

export interface IntentTarget {
  target_type?: string | null;
  id?: string | null;
  name?: string | null;
}

export interface Intent {
  intent_type: IntentType;
  target: IntentTarget;
  constraints: string[];
  required_agents: string[];
  execution_mode: ExecutionMode;
  feedback_type: FeedbackType;
}

export interface IntentParseRequest {
  user_input: string;
}

export interface AgentStepResult {
  agent_name: string;
  success: boolean;
  result?: {
    content: string;
    score?: number;
    suggestions: string[];
  };
  error?: string;
}

export interface IntentExecutionResult {
  intent_type: IntentType;
  feedback_type: FeedbackType;
  execution_mode: ExecutionMode;
  steps: AgentStepResult[];
  summary: string;
}
