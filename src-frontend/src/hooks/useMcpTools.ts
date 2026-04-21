import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import toast from 'react-hot-toast';
import { callMcpTool, disconnectMcpServer } from '@/services/tauri';

export interface McpTool {
  name: string;
  description?: string;
  parameters?: Record<string, unknown>;
  source?: 'builtin' | 'external';
}

export interface ExternalServer {
  id: string;
  name: string;
  command: string;
  args: string;
  env?: string;
}

export function useMcpTools() {
  const [tools, setTools] = useState<McpTool[]>([]);
  const [externalTools, setExternalTools] = useState<McpTool[]>([]);
  const [connectedServer, setConnectedServer] = useState<ExternalServer | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);

  const listTools = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await invoke<McpTool[]>('list_mcp_tools');
      setTools(data.map((t) => ({ ...t, source: 'builtin' as const })));
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

  const connectServer = useCallback(async (config: ExternalServer) => {
    setIsConnecting(true);
    try {
      const env: Record<string, string> = config.env ? JSON.parse(config.env) : {};
      const serverConfig = {
        id: config.id || crypto.randomUUID(),
        name: config.name,
        command: config.command,
        args: config.args.split(' ').filter(Boolean),
        env,
      };
      const data = await invoke<McpTool[]>('connect_mcp_server', { config: serverConfig });
      setExternalTools(data.map((t) => ({ ...t, source: 'external' as const })));
      setConnectedServer(config);
      toast.success(`已连接到 ${config.name}，发现 ${data.length} 个工具`);
      return data;
    } catch (error) {
      toast.error('连接外部服务器失败: ' + (error as Error).message);
      setExternalTools([]);
      setConnectedServer(null);
      throw error;
    } finally {
      setIsConnecting(false);
    }
  }, []);

  const callExternalTool = useCallback(async (toolName: string, args: Record<string, unknown>) => {
    if (!connectedServer) throw new Error('未连接外部服务器');
    try {
      const result = await callMcpTool(connectedServer.id, toolName, args);
      return result;
    } catch (error) {
      toast.error('调用外部工具失败: ' + (error as Error).message);
      throw error;
    }
  }, [connectedServer]);

  const disconnectServer = useCallback(async () => {
    if (connectedServer) {
      try {
        await disconnectMcpServer(connectedServer.id);
      } catch {
        // ignore
      }
    }
    setExternalTools([]);
    setConnectedServer(null);
    toast.success('已断开外部服务器连接');
  }, [connectedServer]);

  return {
    tools,
    externalTools,
    allTools: [...tools, ...externalTools],
    isLoading,
    isConnecting,
    connectedServer,
    listTools,
    executeTool,
    connectServer,
    callExternalTool,
    disconnectServer,
  };
}
