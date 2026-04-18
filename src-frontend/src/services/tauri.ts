import { invoke } from '@tauri-apps/api/core';
import type { 
  Story, Character, Chapter, Skill, McpServer, McpTool,
  DashboardState, CreateStoryRequest, CreateCharacterRequest, 
  UpdateChapterRequest, LlmConfig, SimilarityResult, VectorSearchRequest,
  Intent, IntentParseRequest, IntentExecutionResult
} from '@/types/index';
import type { StoryGraph, Entity, Relation, RetentionReport, ArchiveResult, WorldBuildingOption, CharacterProfileOption, WritingStyleOption, SceneProposal, SceneAnnotation, TextAnnotation, ParagraphCommentary, AgentResult, VectorSearchResult, StorySummary } from '@/types/v3';
import type { WizardCreationResult } from '@/types/index';
import type { AppSettings } from '@/types/llm';

// Health Check
export const healthCheck = () => 
  invoke<{ status: string; timestamp: string; version: string }>('health_check');

// Dashboard
/** @deprecated 暂时保留 — Dashboard 直接调用 `get_state` */
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

/** @deprecated 暂时保留 — Skills 页面使用前端本地分类筛选 */
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
/** @deprecated 暂时保留 — 待 MCP 外部服务器 UI 完成后启用 */
export const connectMcpServer = (config: McpServer) => 
  invoke<McpTool[]>('connect_mcp_server', { config });

/** @deprecated 暂时保留 — 待 MCP 外部服务器 UI 完成后启用 */
export const callMcpTool = (config: McpServer, toolName: string, args: unknown) => 
  invoke<unknown>('call_mcp_tool', { config, toolName, arguments: args });

// Vector Search (NEW - LanceDB)
export const searchSimilar = (req: VectorSearchRequest) =>
  invoke<SimilarityResult[]>('search_similar', { storyId: req.story_id, query: req.query, topK: req.top_k });

/** @deprecated 暂时保留 — 待章节嵌入手动触发功能完成后启用 */
export const embedChapter = (chapterId: string, content: string) =>
  invoke<void>('embed_chapter', { chapterId, content });

// Settings (兼容旧接口，内部映射到 get_settings / save_settings)
export const getConfig = async () => {
  const settings = await invoke<AppSettings>('get_settings');
  const chatModel = settings.models.chat?.find((m: any) => m.id === settings.active_models.chat) || settings.models.chat?.[0];
  if (!chatModel) {
    throw new Error('No chat model configured');
  }
  return {
    provider: chatModel.provider || 'custom',
    api_key: chatModel.api_key || '',
    model: chatModel.model || '',
    temperature: chatModel.temperature ?? 0.8,
    max_tokens: chatModel.max_tokens ?? 4096,
    base_url: chatModel.api_base || '',
  } as LlmConfig;
};

export const updateConfig = async (config: { llm: LlmConfig }) => {
  const settings = await invoke<AppSettings>('get_settings');
  const chatModel = settings.models.chat?.find((m: any) => m.id === settings.active_models.chat) || settings.models.chat?.[0];
  if (chatModel) {
    chatModel.provider = config.llm.provider;
    chatModel.api_key = config.llm.api_key || '';
    chatModel.model = config.llm.model;
    chatModel.temperature = config.llm.temperature;
    chatModel.max_tokens = config.llm.max_tokens;
    chatModel.api_base = config.llm.base_url;
  }
  await invoke('save_settings', { settings });
};

// Intent Engine
export const parseIntent = (req: IntentParseRequest) =>
  invoke<Intent>('parse_intent', { user_input: req.user_input });

export const executeIntent = (intent: Intent, storyId: string) =>
  invoke<IntentExecutionResult>('execute_intent', { intent, story_id: storyId });

// Knowledge Graph
export const getStoryGraph = (storyId: string) =>
  invoke<StoryGraph>('get_story_graph', { story_id: storyId });

export const getRetentionReport = (storyId: string) =>
  invoke<RetentionReport>('get_retention_report', { story_id: storyId });

export const archiveForgottenEntities = (storyId: string) =>
  invoke<ArchiveResult>('archive_forgotten_entities', { story_id: storyId });

export const restoreArchivedEntity = (entityId: string) =>
  invoke<Entity>('restore_archived_entity', { entity_id: entityId });

export const getArchivedEntities = (storyId: string) =>
  invoke<Entity[]>('get_archived_entities', { story_id: storyId });

/** @deprecated 暂时保留 — 待知识图谱手动创建实体功能完成后启用 */
export const createEntity = (storyId: string, name: string, entityType: string, attributes: Record<string, unknown>) =>
  invoke<Entity>('create_entity', { story_id: storyId, name, entity_type: entityType, attributes });

export const updateEntity = (entityId: string, updates: { name?: string; attributes?: Record<string, unknown> }) =>
  invoke<Entity>('update_entity', { entity_id: entityId, name: updates.name, attributes: updates.attributes });

// Novel Creation Wizard
export const generateWorldBuildingOptions = (userInput: string) =>
  invoke<WorldBuildingOption[]>('generate_world_building_options', { user_input: userInput });

export const generateCharacterProfiles = (worldBuilding: WorldBuildingOption) =>
  invoke<CharacterProfileOption[][]>('generate_character_profiles', { world_building: worldBuilding });

export const generateWritingStyles = (genre: string, worldBuilding: WorldBuildingOption) =>
  invoke<WritingStyleOption[]>('generate_writing_styles', { genre, world_building: worldBuilding });

export const generateFirstScene = (worldBuilding: WorldBuildingOption, characters: CharacterProfileOption[], writingStyle: WritingStyleOption) =>
  invoke<SceneProposal>('generate_first_scene', { world_building: worldBuilding, characters, writing_style: writingStyle });

export const createStoryWithWizard = (params: {
  title: string;
  description?: string;
  genre?: string;
  world_building: WorldBuildingOption;
  characters: CharacterProfileOption[];
  writing_style: WritingStyleOption;
  first_scene: SceneProposal;
}) =>
  invoke<import('@/types/index').WizardCreationResult>('create_story_with_wizard', params);

/** @deprecated 暂时保留 — 待知识图谱手动创建关系功能完成后启用 */
export const createRelation = (storyId: string, sourceId: string, targetId: string, relationType: string, strength: number) =>
  invoke<Relation>('create_relation', { story_id: storyId, source_id: sourceId, target_id: targetId, relation_type: relationType, strength });

// Scene Annotations
export const createSceneAnnotation = (params: { scene_id: string; story_id: string; content: string; annotation_type: string }) =>
  invoke<SceneAnnotation>('create_scene_annotation', params);

export const getSceneAnnotations = (sceneId: string) =>
  invoke<SceneAnnotation[]>('get_scene_annotations', { scene_id: sceneId });

export const getStoryUnresolvedAnnotations = (storyId: string) =>
  invoke<SceneAnnotation[]>('get_story_unresolved_annotations', { story_id: storyId });

export const updateSceneAnnotation = (annotationId: string, content: string) =>
  invoke<number>('update_scene_annotation', { annotation_id: annotationId, content });

export const resolveSceneAnnotation = (annotationId: string) =>
  invoke<number>('resolve_scene_annotation', { annotation_id: annotationId });

export const unresolveSceneAnnotation = (annotationId: string) =>
  invoke<number>('unresolve_scene_annotation', { annotation_id: annotationId });

export const deleteSceneAnnotation = (annotationId: string) =>
  invoke<number>('delete_scene_annotation', { annotation_id: annotationId });

// Text Inline Annotations
export const createTextAnnotation = (params: { story_id: string; scene_id?: string; chapter_id?: string; content: string; annotation_type: string; from_pos: number; to_pos: number }) =>
  invoke<TextAnnotation>('create_text_annotation', params);

export const getTextAnnotationsByChapter = (chapterId: string) =>
  invoke<TextAnnotation[]>('get_text_annotations_by_chapter', { chapter_id: chapterId });

export const getTextAnnotationsByScene = (sceneId: string) =>
  invoke<TextAnnotation[]>('get_text_annotations_by_scene', { scene_id: sceneId });

export const updateTextAnnotation = (annotationId: string, content: string) =>
  invoke<number>('update_text_annotation', { annotation_id: annotationId, content });

export const resolveTextAnnotation = (annotationId: string) =>
  invoke<number>('resolve_text_annotation', { annotation_id: annotationId });

export const unresolveTextAnnotation = (annotationId: string) =>
  invoke<number>('unresolve_text_annotation', { annotation_id: annotationId });

export const deleteTextAnnotation = (annotationId: string) =>
  invoke<number>('delete_text_annotation', { annotation_id: annotationId });

// Commentator Agent
export const generateParagraphCommentaries = (params: { story_id: string; story_title: string; genre: string; text: string }) =>
  invoke<string>('generate_paragraph_commentaries', params);

// Vector Search
export const textSearchVectors = (storyId: string, query: string, top_k?: number) =>
  invoke<VectorSearchResult[]>('text_search_vectors', { storyId, query, topK: top_k });

export const hybridSearchVectors = (storyId: string, query: string, top_k?: number) =>
  invoke<VectorSearchResult[]>('hybrid_search_vectors', { storyId, query, topK: top_k });

// Writer Agent (正文助手)
export const writerAgentExecute = (params: {
  story_id: string;
  chapter_number?: number;
  current_content: string;
  selected_text?: string;
  instruction: string;
}) =>
  invoke<{ content: string; story_id?: string; chapter_id?: string }>('writer_agent_execute', { request: params });

// Memory Compressor
export const compressContent = (params: { story_id: string; content: string; target_ratio?: number }) =>
  invoke<AgentResult>('compress_content', params);

export const compressScene = (params: { scene_id: string; target_ratio?: number }) =>
  invoke<AgentResult>('compress_scene', params);

// Knowledge Distillation
export const distillStoryKnowledge = (storyId: string) =>
  invoke<StorySummary>('distill_story_knowledge', { story_id: storyId });

export const getStorySummaries = (storyId: string) =>
  invoke<StorySummary[]>('get_story_summaries', { story_id: storyId });

export const updateStorySummary = (summaryId: string, content: string) =>
  invoke<number>('update_story_summary', { summary_id: summaryId, content });

export const deleteStorySummary = (summaryId: string) =>
  invoke<number>('delete_story_summary', { summary_id: summaryId });


// ==================== Subscription (Freemium) ====================

export interface SubscriptionStatus {
  user_id: string;
  tier: string;
  status: string;
  daily_used: number;
  daily_limit: number;
  quota_resets_at: string;
  expires_at?: string;
}

export interface QuotaCheckResult {
  allowed: boolean;
  remaining: number;
  daily_limit: number;
  daily_used: number;
  resets_at: string;
  message?: string;
}

export const getSubscriptionStatus = () =>
  invoke<SubscriptionStatus>('get_subscription_status');

export const checkAiQuota = () =>
  invoke<QuotaCheckResult>('check_ai_quota');

export const devUpgradeSubscription = (tier: string) =>
  invoke<SubscriptionStatus>('dev_upgrade_subscription', { tier });

export const devDowngradeSubscription = () =>
  invoke<SubscriptionStatus>('dev_downgrade_subscription');

// Window communication
export const notifyFrontstageDataRefresh = (entity: string) =>
  invoke<void>('notify_frontstage_data_refresh', { entity });
