/**
 * Settings Service
 * 
 * 与后端通信管理应用设置
 */

import { invoke } from '@tauri-apps/api/core';
import type { 
  AppSettings, 
  ModelConfig, 
  AgentModelMapping,
  SettingsExport 
} from '@/types/llm';

// 获取完整设置
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings');
}

// 保存设置
export async function saveSettings(settings: Partial<AppSettings>): Promise<void> {
  return invoke('save_settings', { settings });
}

// 导出设置
export async function exportSettings(): Promise<SettingsExport> {
  return invoke<SettingsExport>('export_settings');
}

// 导入设置
export async function importSettings(data: SettingsExport): Promise<void> {
  return invoke('import_settings', { data });
}

// 获取所有模型配置
export async function getModels(): Promise<ModelConfig[]> {
  return invoke<ModelConfig[]>('get_models');
}

// 创建模型配置
export async function createModel(config: Omit<ModelConfig, 'id'>): Promise<ModelConfig> {
  return invoke<ModelConfig>('create_model', { config });
}

// 更新模型配置
export async function updateModel(id: string, config: Partial<ModelConfig>): Promise<void> {
  return invoke('update_model', { id, config });
}

// 删除模型配置
export async function deleteModel(id: string): Promise<void> {
  return invoke('delete_model', { id });
}

// 设置激活的模型
export async function setActiveModel(type: ModelConfig['type'], modelId: string): Promise<void> {
  return invoke('set_active_model', { type, modelId });
}

// 获取Agent模型映射
export async function getAgentMappings(): Promise<AgentModelMapping[]> {
  return invoke<AgentModelMapping[]>('get_agent_mappings');
}

// 更新Agent模型映射
export async function updateAgentMapping(mapping: AgentModelMapping): Promise<void> {
  return invoke('update_agent_mapping', { mapping });
}

// 测试模型连接
export async function testModelConnection(modelId: string): Promise<{ success: boolean; latency: number; error?: string }> {
  return invoke('test_model_connection', { modelId });
}

// 获取模型提供商列表
export function getModelProviders(): Array<{ id: string; name: string; requiresApiKey: boolean; supports: ModelConfig['type'][] }> {
  return [
    { id: 'openai', name: 'OpenAI', requiresApiKey: true, supports: ['chat', 'embedding', 'multimodal', 'image'] },
    { id: 'anthropic', name: 'Anthropic', requiresApiKey: true, supports: ['chat', 'multimodal'] },
    { id: 'azure', name: 'Azure OpenAI', requiresApiKey: true, supports: ['chat', 'embedding'] },
    { id: 'ollama', name: 'Ollama (Local)', requiresApiKey: false, supports: ['chat', 'embedding'] },
    { id: 'deepseek', name: 'DeepSeek', requiresApiKey: true, supports: ['chat'] },
    { id: 'qwen', name: '通义千问', requiresApiKey: true, supports: ['chat', 'multimodal'] },
    { id: 'moonshot', name: 'Moonshot', requiresApiKey: true, supports: ['chat'] },
    { id: 'zhipu', name: '智谱AI', requiresApiKey: true, supports: ['chat', 'multimodal'] },
    { id: 'custom', name: 'Custom', requiresApiKey: false, supports: ['chat', 'embedding'] },
  ];
}

// 获取提供商默认模型
export function getProviderDefaultModels(provider: string): string[] {
  const defaults: Record<string, string[]> = {
    openai: ['gpt-4', 'gpt-4-turbo-preview', 'gpt-3.5-turbo', 'text-embedding-3-small', 'dall-e-3'],
    anthropic: ['claude-3-opus-20240229', 'claude-3-sonnet-20240229', 'claude-3-haiku-20240307'],
    azure: ['gpt-4', 'gpt-35-turbo', 'text-embedding-ada-002'],
    ollama: ['llama2', 'mistral', 'codellama', 'nomic-embed-text'],
    deepseek: ['deepseek-chat', 'deepseek-coder'],
    qwen: ['qwen-turbo', 'qwen-plus', 'qwen-max'],
    moonshot: ['moonshot-v1-8k', 'moonshot-v1-32k', 'moonshot-v1-128k'],
    zhipu: ['glm-4', 'glm-3-turbo'],
    custom: ['custom-model'],
  };
  return defaults[provider] || ['custom-model'];
}
