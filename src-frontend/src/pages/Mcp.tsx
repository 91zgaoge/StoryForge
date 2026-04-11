import { Plug, Plus, TestTube } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';

export function Mcp() {
  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold text-white">MCP 连接</h1>
          <p className="text-gray-400">配置 Model Context Protocol 服务器</p>
        </div>
        <Button variant="primary">
          <Plus className="w-4 h-4" />
          添加服务器
        </Button>
      </div>

      <div className="grid gap-6">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-xl bg-green-500/10 flex items-center justify-center">
                  <Plug className="w-6 h-6 text-green-400" />
                </div>
                <div>
                  <h3 className="font-display text-lg font-semibold text-white">示例服务器</h3>
                  <p className="text-sm text-gray-400">stdio / path/to/server</p>
                </div>
              </div>
              
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />
                <span className="text-sm text-green-400">已连接</span>
                <Button variant="ghost" size="sm">
                  <TestTube className="w-4 h-4" />
                  测试
                </Button>
              </div>
            </div>

            <div className="mt-4 pt-4 border-t border-cinema-700">
              <p className="text-sm text-gray-400 mb-2">可用工具:</p>
              <div className="flex flex-wrap gap-2">
                <span className="px-2 py-1 rounded bg-cinema-800 text-xs text-gray-300">search</span>
                <span className="px-2 py-1 rounded bg-cinema-800 text-xs text-gray-300">fetch</span>
                <span className="px-2 py-1 rounded bg-cinema-800 text-xs text-gray-300">analyze</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
