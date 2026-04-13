/**
 * 模型服务层
 * 
 * 处理与LLM模型的API通信
 * 支持多模态、语言和Embedding模型
 */

import { ModelConfig, getChatModel, getEmbeddingModel } from '@/config/models';

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface ChatCompletionRequest {
  model: string;
  messages: ChatMessage[];
  max_tokens?: number;
  temperature?: number;
  stream?: boolean;
}

export interface ChatCompletionResponse {
  id: string;
  choices: {
    index: number;
    message: ChatMessage;
    finish_reason: string;
  }[];
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export interface EmbeddingRequest {
  model: string;
  input: string | string[];
}

export interface EmbeddingResponse {
  data: {
    index: number;
    embedding: number[];
    object: string;
  }[];
  model: string;
  usage: {
    prompt_tokens: number;
    total_tokens: number;
  };
}

class ModelService {
  private currentModelId: string;
  private abortController: AbortController | null = null;

  constructor() {
    this.currentModelId = 'qwen35'; // 默认使用 Qwen 3.5
  }

  // 设置当前使用的模型
  setCurrentModel(modelId: string) {
    this.currentModelId = modelId;
  }

  // 获取当前模型配置
  getCurrentModel(): ModelConfig {
    return getChatModel(this.currentModelId);
  }

  // 获取当前模型ID
  getCurrentModelId(): string {
    return this.currentModelId;
  }

  // 检查模型连接状态
  async checkModelStatus(modelId?: string): Promise<'connected' | 'disconnected' | 'connecting'> {
    const config = modelId ? getChatModel(modelId) : this.getCurrentModel();
    
    try {
      const response = await fetch(`${config.baseUrl}/models`, {
        method: 'GET',
        headers: this.buildHeaders(config),
        signal: AbortSignal.timeout(5000),
      });
      
      if (response.ok) {
        return 'connected';
      }
      return 'disconnected';
    } catch (error) {
      console.warn('Model status check failed:', error);
      return 'disconnected';
    }
  }

  // 发送聊天请求
  async chat(
    messages: ChatMessage[],
    options?: {
      maxTokens?: number;
      temperature?: number;
      stream?: boolean;
      onStream?: (chunk: string) => void;
    }
  ): Promise<ChatCompletionResponse> {
    const config = this.getCurrentModel();
    
    // 取消之前的请求
    if (this.abortController) {
      this.abortController.abort();
    }
    this.abortController = new AbortController();

    const requestBody: ChatCompletionRequest = {
      model: config.id,
      messages,
      max_tokens: options?.maxTokens || config.maxTokens || 8192,
      temperature: options?.temperature || config.temperature || 0.8,
      stream: options?.stream || false,
    };

    try {
      const response = await fetch(`${config.baseUrl}/chat/completions`, {
        method: 'POST',
        headers: {
          ...this.buildHeaders(config),
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestBody),
        signal: this.abortController.signal,
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`API请求失败: ${response.status} - ${error}`);
      }

      // 处理流式响应
      if (options?.stream && options.onStream) {
        return await this.handleStreamResponse(response, options.onStream);
      }

      const data: ChatCompletionResponse = await response.json();
      return data;
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error('请求已取消');
      }
      throw error;
    }
  }

  // 处理流式响应
  private async handleStreamResponse(
    response: Response,
    onStream: (chunk: string) => void
  ): Promise<ChatCompletionResponse> {
    const reader = response.body?.getReader();
    if (!reader) {
      throw new Error('无法读取响应流');
    }

    const decoder = new TextDecoder();
    let fullContent = '';
    let finalResponse: ChatCompletionResponse | null = null;

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value, { stream: true });
        const lines = chunk.split('\n').filter(line => line.trim() !== '');

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6);
            if (data === '[DONE]') continue;

            try {
              const parsed = JSON.parse(data);
              const content = parsed.choices?.[0]?.delta?.content || '';
              if (content) {
                fullContent += content;
                onStream(content);
              }
              
              // 保存最后一个响应作为最终结果
              if (parsed.choices?.[0]?.finish_reason) {
                finalResponse = parsed;
              }
            } catch (e) {
              console.warn('解析流数据失败:', e);
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }

    if (!finalResponse) {
      // 构造一个默认响应
      return {
        id: 'stream-response',
        choices: [{
          index: 0,
          message: { role: 'assistant', content: fullContent },
          finish_reason: 'stop',
        }],
        usage: { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 },
      };
    }

    return finalResponse;
  }

  // 获取文本嵌入向量
  async getEmbedding(text: string | string[]): Promise<number[] | number[][]> {
    const config = getEmbeddingModel();
    
    const requestBody: EmbeddingRequest = {
      model: config.id,
      input: text,
    };

    const response = await fetch(`${config.baseUrl}/embeddings`, {
      method: 'POST',
      headers: {
        ...this.buildHeaders(config),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Embedding请求失败: ${response.status} - ${error}`);
    }

    const data: EmbeddingResponse = await response.json();
    
    if (Array.isArray(text)) {
      return data.data.map(d => d.embedding);
    }
    return data.data[0].embedding;
  }

  // 取消当前请求
  abortCurrentRequest() {
    if (this.abortController) {
      this.abortController.abort();
      this.abortController = null;
    }
  }

  // 构建请求头
  private buildHeaders(config: ModelConfig): Record<string, string> {
    const headers: Record<string, string> = {};
    
    if (config.useApiKey && config.apiKey) {
      headers['Authorization'] = `Bearer ${config.apiKey}`;
    }
    
    return headers;
  }
}

// 导出单例
export const modelService = new ModelService();

// Hook for React components
export function useModelService() {
  return modelService;
}
