import { invoke } from '@tauri-apps/api/core';
import type { 
  Story, Character, Chapter, Skill, McpServer, McpTool,
  DashboardState, CreateStoryRequest, CreateCharacterRequest, 
  UpdateChapterRequest, LlmConfig, SimilarityResult, VectorSearchRequest,
  Intent, IntentParseRequest, IntentExecutionResult
} from '@/types/index';
import type { StoryGraph, Entity, Relation } from '@/types/v3';

// Health Check
export const healthCheck = () => 
  invoke<{ status: string; timestamp: string; version: string }>('health_check');

// Dashboard
export const getDashboardState = () => 
  invoke<DashboardState>('get_state');

// Stories
export const listStories = () => 
  invoke<Story[]>('list_stories');

export const createStory = (req: CreateStoryRequest) =>
  invoke<Story>('create_story', { ...req });

export const updateStory = (id: string, updates: Partial<Story>) =>
  invoke<void>('update_story', { id, ...updates });

export const deleteStory = (id: string) =>
  invoke<void>('delete_story', { id });

// Characters
export const getStoryCharacters = (storyId: string) => 
  invoke<Character[]>('get_story_characters', { storyId });

export const createCharacter = (req: CreateCharacterRequest) =>
  invoke<Character>('create_character', { ...req });

export const updateCharacter = (id: string, updates: Partial<Character>) => 
  invoke<void>('update_character', { id, ...updates });

export const deleteCharacter = (id: string) => 
  invoke<void>('delete_character', { id });

// Chapters
export const getStoryChapters = (storyId: string) => 
  invoke<Chapter[]>('get_story_chapters', { storyId });

export const getChapter = (id: string) => 
  invoke<Chapter | null>('get_chapter', { id });

export const updateChapter = (id: string, updates: UpdateChapterRequest) => 
  invoke<void>('update_chapter', { id, ...updates });

export const deleteChapter = (id: string) =>
  invoke<void>('delete_chapter', { id });

export const createChapter = (req: { story_id: string; chapter_number: number; title?: string; outline?: string; content?: string }) =>
  invoke<Chapter>('create_chapter', { ...req });

// Skills
export const getSkills = () => 
  invoke<Skill[]>('get_skills');

export const getSkillsByCategory = (category: string) => 
  invoke<Skill[]>('get_skills_by_category', { category });

export const importSkill = (path: string) => 
  invoke<Skill>('import_skill', { path });

export const enableSkill = (skillId: string) => 
  invoke<void>('enable_skill', { skillId });

export const disableSkill = (skillId: string) => 
  invoke<void>('disable_skill', { skillId });

export const uninstallSkill = (skillId: string) => 
  invoke<void>('uninstall_skill', { skillId });

export const executeSkill = (skillId: string, params: Record<string, unknown>) => 
  invoke<unknown>('execute_skill', { skillId, params });

// MCP
export const connectMcpServer = (config: McpServer) => 
  invoke<McpTool[]>('connect_mcp_server', { config });

export const callMcpTool = (config: McpServer, toolName: string, args: unknown) => 
  invoke<unknown>('call_mcp_tool', { config, toolName, arguments: args });

// Vector Search (NEW - LanceDB)
export const searchSimilar = (req: VectorSearchRequest) =>
  invoke<SimilarityResult[]>('search_similar', { ...req });

export const embedChapter = (chapterId: string, content: string) =>
  invoke<void>('embed_chapter', { chapterId, content });

// Settings
export const getConfig = () => 
  invoke<LlmConfig>('get_config_command');

export const updateConfig = (config: { llm: LlmConfig }) => 
  invoke<void>('update_config', config);

// Intent Engine
export const parseIntent = (req: IntentParseRequest) =>
  invoke<Intent>('parse_intent', { user_input: req.user_input });

export const executeIntent = (intent: Intent, storyId: string) =>
  invoke<IntentExecutionResult>('execute_intent', { intent, story_id: storyId });

// Knowledge Graph
export const getStoryGraph = (storyId: string) =>
  invoke<StoryGraph>('get_story_graph', { story_id: storyId });

export const createEntity = (storyId: string, name: string, entityType: string, attributes: Record<string, unknown>) =>
  invoke<Entity>('create_entity', { story_id: storyId, name, entity_type: entityType, attributes });

export const createRelation = (storyId: string, sourceId: string, targetId: string, relationType: string, strength: number) =>
  invoke<Relation>('create_relation', { story_id: storyId, source_id: sourceId, target_id: targetId, relation_type: relationType, strength });
