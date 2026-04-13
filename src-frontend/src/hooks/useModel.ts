/**
 * 模型管理 Hook
 * 
 * 用于管理当前模型、模型状态和模型切换
 */

import { useState, useEffect, useCallback } from 'react';
import { ModelConfig, getAvailableModels, getChatModel, DEFAULT_MODEL_ID } from '@/config/models';
import { modelService, ChatMessage } from '@/services/modelService';

export interface ModelState {
  currentModel: ModelConfig;
  status: 'connected' | 'disconnected' | 'connecting';
  availableModels: ModelConfig[];
}

export function useModel() {
  const [state, setState] = useState<ModelState>({
    currentModel: getChatModel(DEFAULT_MODEL_ID),
    status: 'connecting',
    availableModels: getAvailableModels(),
  });

  // 检查模型状态
  const checkStatus = useCallback(async () => {
    setState(prev => ({ ...prev, status: 'connecting' }));
    const status = await modelService.checkModelStatus();
    setState(prev => ({ ...prev, status }));
    return status;
  }, []);

  // 切换模型
  const switchModel = useCallback((modelId: string) => {
    const newModel = getChatModel(modelId);
    modelService.setCurrentModel(modelId);
    setState(prev => ({
      ...prev,
      currentModel: newModel,
      status: 'connecting',
    }));
    // 切换后检查新模型状态
    checkStatus();
  }, [checkStatus]);

  // 发送聊天消息
  const chat = useCallback(async (
    messages: ChatMessage[],
    options?: {
      stream?: boolean;
      onStream?: (chunk: string) => void;
    }
  ) => {
    return modelService.chat(messages, options);
  }, []);

  // 初始检查状态
  useEffect(() => {
    checkStatus();
    // 每30秒检查一次状态
    const interval = setInterval(checkStatus, 30000);
    return () => clearInterval(interval);
  }, [checkStatus]);

  return {
    ...state,
    checkStatus,
    switchModel,
    chat,
  };
}

export default useModel;
