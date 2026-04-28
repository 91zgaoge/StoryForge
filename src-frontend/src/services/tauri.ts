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
  invoke<Character[]>('get_story_characters', { story_id: storyId });

export const createCharacter = (req: CreateCharacterRequest) =>
  invoke<Character>('create_character', { ...req });

export const updateCharacter = (id: string, updates: Partial<Character>) => 
  invoke<void>('update_character', { id, ...updates });

export const deleteCharacter = (id: string) => 
  invoke<void>('delete_character', { id });

// Chapters
export const getStoryChapters = (storyId: string) => 
  invoke<Chapter[]>('get_story_chapters', { story_id: storyId });

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

export const getSkill = (skillId: string) => 
  invoke<Skill>('get_skill', { skill_id: skillId });

/** @deprecated 暂时保留 — Skills 页面使用前端本地分类筛选 */
export const getSkillsByCategory = (category: string) => 
  invoke<Skill[]>('get_skills_by_category', { category });

export const importSkill = (path: string) => 
  invoke<Skill>('import_skill', { path });

export const enableSkill = (skillId: string) => 
  invoke<void>('enable_skill', { skill_id: skillId });

export const disableSkill = (skillId: string) => 
  invoke<void>('disable_skill', { skill_id: skillId });

export const uninstallSkill = (skillId: string) => 
  invoke<void>('uninstall_skill', { skill_id: skillId });

export const updateSkill = (skillId: string, manifest: Partial<Skill>) => 
  invoke<void>('update_skill', { skill_id: skillId, manifest });

export const executeSkill = (skillId: string, params: Record<string, unknown>) => 
  invoke<unknown>('execute_skill', { skill_id: skillId, params });

export const formatText = (content: string) =>
  invoke<string>('format_text', { content });

// MCP
/** @deprecated 暂时保留 — 待 MCP 外部服务器 UI 完成后启用 */
export const connectMcpServer = (config: McpServer) => 
  invoke<McpTool[]>('connect_mcp_server', { config });

export const callMcpTool = (serverId: string, toolName: string, args: unknown) => 
  invoke<unknown>('call_mcp_tool', { server_id: serverId, toolName, arguments: args });

export const disconnectMcpServer = (serverId: string) =>
  invoke<void>('disconnect_mcp_server', { server_id: serverId });

export const getMcpConnections = () =>
  invoke<Array<{ id: string; tools: number; resources: number }>>('get_mcp_connections');

export const runCreationWorkflow = (storyId: string, mode: string, initialInput: string) =>
  invoke<{
    success: boolean;
    current_phase: string;
    completed_phases: string[];
    output_preview?: string;
    quality_report?: unknown;
    error?: string;
  }>('run_creation_workflow', { story_id: storyId, mode, initial_input: initialInput });

export const listStyleDnas = () =>
  invoke<Array<{ id: string; name: string; author?: string; is_builtin: boolean }>>('list_style_dnas');

export const setStoryStyleDna = (storyId: string, styleDnaId: string | null) =>
  invoke<void>('set_story_style_dna', { story_id: storyId, style_dna_id: styleDnaId });

export const analyzeStyleSample = (text: string, name?: string) =>
  invoke<{ id: string; name: string; author?: string; is_builtin: boolean; is_user_created: boolean }>('analyze_style_sample', { text, name });

// Vector Search (NEW - LanceDB)
export const searchSimilar = (req: VectorSearchRequest) =>
  invoke<SimilarityResult[]>('search_similar', { story_id: req.story_id, query: req.query, top_k: req.top_k });

/** @deprecated 暂时保留 — 待章节嵌入手动触发功能完成后启用 */
export const embedChapter = (chapterId: string, content: string) =>
  invoke<void>('embed_chapter', { chapter_id: chapterId, content });

// Settings (兼容旧接口，内部映射到 get_settings / save_settings)
export const getConfig = async () => {
  const settings = await invoke<AppSettings>('get_settings');
  const chatModel = settings.models.chat?.find((m: any) => m.id === settings.active_models.chat)
    || settings.models.chat?.[0];
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
  const chatModel = settings.models.chat?.find((m: any) => m.id === settings.active_models.chat)
    || settings.models.chat?.[0];
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

// Smart Execute - Model-driven orchestration
export interface SmartExecuteRequest {
  user_input: string;
  current_content?: string;
}

export interface SmartExecuteResult {
  success: boolean;
  steps_completed: number;
  final_content?: string;
  messages: string[];
}

export const smartExecute = (req: SmartExecuteRequest) =>
  invoke<SmartExecuteResult>('smart_execute', { userInput: req.user_input, currentContent: req.current_content });

// Feedback Recording
export interface RecordFeedbackRequest {
  story_id: string;
  scene_id?: string;
  chapter_id?: string;
  feedback_type: 'accept' | 'reject' | 'modify';
  agent_type?: string;
  original_ai_text: string;
  final_text?: string;
}

export const recordFeedback = (req: RecordFeedbackRequest) =>
  invoke<void>('record_feedback', { ...req });

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
  invoke<VectorSearchResult[]>('text_search_vectors', { story_id: storyId, query, top_k });

export const hybridSearchVectors = (storyId: string, query: string, top_k?: number) =>
  invoke<VectorSearchResult[]>('hybrid_search_vectors', { story_id: storyId, query, top_k });

// Writer Agent (正文助手)
export const writerAgentExecute = (params: {
  story_id: string;
  chapter_number?: number;
  current_content: string;
  selected_text?: string;
  instruction: string;
}) =>
  invoke<{ content: string; story_id?: string; chapter_id?: string; task_id: string }>('writer_agent_execute', { request: params });



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


// ==================== LLM Stream ====================

export const llmGenerateStream = (params: {
  request_id: string;
  prompt: string;
  context?: string;
  max_tokens?: number;
  temperature?: number;
}) =>
  invoke<void>('llm_generate_stream', { request: params });

export const llmCancelGeneration = (requestId: string) =>
  invoke<void>('llm_cancel_generation', { request_id: requestId });

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

export const devUpgradeSubscription = (tier: string) =>
  invoke<SubscriptionStatus>('dev_upgrade_subscription', { tier });

export const devDowngradeSubscription = () =>
  invoke<SubscriptionStatus>('dev_downgrade_subscription');

// V2 Quota (按功能区分)
export interface QuotaDetail {
  auto_write_used: number;
  auto_write_limit: number;
  auto_revise_used: number;
  auto_revise_limit: number;
  max_chars_per_call: number;
}

export const getQuotaDetail = () =>
  invoke<QuotaDetail>('get_quota_detail');

export const checkAutoWriteQuota = (requestedChars: number) =>
  invoke<QuotaCheckResult>('check_auto_write_quota', { requestedChars });

export const checkAutoReviseQuota = (requestedChars: number) =>
  invoke<QuotaCheckResult>('check_auto_revise_quota', { requestedChars });

// ==================== 文思泉涌 ====================

export const autoWrite = (params: {
  story_id: string;
  chapter_id: string;
  target_chars: number;
  chars_per_loop: number;
}) =>
  invoke<{ task_id: string; actual_chars: number; loops: number; status: string }>('auto_write', { request: params });

export const autoWriteCancel = (taskId: string) =>
  invoke<void>('auto_write_cancel', { task_id: taskId });

export const autoRevise = (params: {
  story_id: string;
  chapter_id?: string;
  scope: string;
  selected_text?: string;
  revision_type: string;
}) =>
  invoke<{ task_id: string; revised_text: string; status: string }>('auto_revise', { request: params });

export const autoReviseCancel = (taskId: string) =>
  invoke<void>('auto_revise_cancel', { task_id: taskId });

// Window communication
export const notifyFrontstageDataRefresh = (entity: string) =>
  invoke<void>('notify_frontstage_data_refresh', { entity });

// Input hint — LLM智能输入建议
export const getInputHint = (currentContent?: string) =>
  invoke<string>('get_input_hint', { currentContent });
