import { useEffect, useState } from 'react';
import { Plug, Plus, TestTube, Play, Wrench } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useMcpTools } from '@/hooks/useMcpTools';

export function Mcp() {
  const { tools, isLoading, listTools, executeTool } = useMcpTools();
  const [selectedTool, setSelectedTool] = useState<string | null>(null);
  const [toolArgs, setToolArgs] = useState('{}');
  const [toolResult, setToolResult] = useState<unknown>(null);
  const [isExecuting, setIsExecuting] = useState(false);

  useEffect(() => {
    listTools();
  }, [listTools]);

  const handleExecute = async () => {
    if (!selectedTool) return;
    setIsExecuting(true);
    try {
      const args = JSON.parse(toolArgs);
      const result = await executeTool(selectedTool, args);
      setToolResult(result);
    } catch (e) {
      setToolResult({ error: 'Invalid JSON arguments' });
    } finally {
      setIsExecuting(false);
    }
  };

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold text-white">MCP 工具</h1>
          <p className="text-gray-400">内置 Model Context Protocol 工具</p>
        </div>
        <Button variant="primary" onClick={listTools} isLoading={isLoading}>
          <Plus className="w-4 h-4" />
          刷新工具
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Tools List */}
        <div className="space-y-4">
          <h2 className="font-display text-lg font-semibold text-white flex items-center gap-2">
            <Wrench className="w-5 h-5 text-cinema-gold" />
            可用工具 ({tools.length})
          </h2>

          {tools.map((tool) => (
            <Card
              key={tool.name}
              className={`cursor-pointer transition-colors ${
                selectedTool === tool.name ? 'border-cinema-gold' : ''
              }`}
              onClick={() => setSelectedTool(tool.name)}
            >
              <CardContent className="p-4">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-lg bg-cinema-800 flex items-center justify-center">
                    <Plug className="w-5 h-5 text-cinema-gold" />
                  </div>
                  <div className="flex-1">
                    <h3 className="font-medium text-white">{tool.name}</h3>
                    {tool.description && (
                      <p className="text-sm text-gray-400">{tool.description}</p>
                    )}
                  </div>
                  {selectedTool === tool.name && (
                    <Play className="w-4 h-4 text-cinema-gold" />
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Tool Execution */}
        <div className="space-y-4">
          <h2 className="font-display text-lg font-semibold text-white flex items-center gap-2">
            <TestTube className="w-5 h-5 text-cinema-gold" />
            工具执行
          </h2>

          {selectedTool ? (
            <Card>
              <CardContent className="p-4 space-y-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-2">
                    参数 (JSON)
                  </label>
                  <textarea
                    value={toolArgs}
                    onChange={(e) => setToolArgs(e.target.value)}
                    rows={6}
                    className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white font-mono text-sm focus:border-cinema-gold focus:outline-none"
                    placeholder='{"key": "value"}'
                  />
                </div>

                <Button
                  variant="primary"
                  onClick={handleExecute}
                  isLoading={isExecuting}
                  className="w-full gap-2"
                >
                  <Play className="w-4 h-4" />
                  执行 {selectedTool}
                </Button>

                {toolResult !== null && (
                  <div className="mt-4">
                    <label className="block text-sm text-gray-400 mb-2">结果:</label>
                    <pre className="bg-cinema-900 p-3 rounded-lg text-xs text-gray-300 overflow-auto max-h-60">
                      {JSON.stringify(toolResult, null, 2)}
                    </pre>
                  </div>
                )}
              </CardContent>
            </Card>
          ) : (
            <Card>
              <CardContent className="p-8 text-center text-gray-500">
                <Wrench className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>选择一个工具开始执行</p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
