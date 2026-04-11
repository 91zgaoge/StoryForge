import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import toast from 'react-hot-toast';

export interface McpTool {
  name: string;
  description?: string;
  parameters?: Record<string, unknown>;
}

export function useMcpTools() {
  const [tools, setTools] = useState<McpTool[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const listTools = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await invoke<McpTool[]>('list_mcp_tools');
      setTools(data);
    } catch (error) {
      toast.error('获取工具列表失败: ' + (error as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const executeTool = useCallback(async (toolName: string, args: Record<string, unknown>) => {
    try {
      const result = await invoke<unknown>('execute_mcp_tool', {
        toolName,
        arguments: args,
      });
      return result;
    } catch (error) {
      toast.error('执行工具失败: ' + (error as Error).message);
      throw error;
    }
  }, []);

  return {
    tools,
    isLoading,
    listTools,
    executeTool,
  };
}
