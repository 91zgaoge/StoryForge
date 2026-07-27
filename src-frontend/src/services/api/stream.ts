import { loggedInvoke } from './core';
// ==================== LLM Stream ====================

export const llmGenerateStream = (params: {
  request_id: string;
  prompt: string;
  context?: string;
  max_tokens?: number;
  temperature?: number;
}) => loggedInvoke<void>('llm_generate_stream', { request: params });

export const llmCancelGeneration = (requestId: string) =>
  loggedInvoke<void>('llm_cancel_generation', { request_id: requestId });
// Input hint — LLM智能输入建议
export const getInputHint = (currentContent?: string) =>
  loggedInvoke<string>('get_input_hint', { current_content: currentContent });

// v0.30.27: Logline 幽灵提示--支持上下文感知；传入 story_id / chapter_number 时
// 后端会结合故事大纲、场景大纲、角色与当前正文生成后缀。
export const generateLoglineHint = (
  userInput: string,
  storyId?: string | null,
  chapterNumber?: number | null
) =>
  loggedInvoke<string | null>('generate_logline_hint', {
    user_input: userInput,
    story_id: storyId ?? null,
    chapter_number: chapterNumber ?? null,
  });
