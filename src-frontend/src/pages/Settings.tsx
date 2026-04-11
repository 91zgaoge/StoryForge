import { Settings2, Key, Globe, Palette } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';

export function Settings() {
  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div>
        <h1 className="font-display text-3xl font-bold text-white">工作室配置</h1>
        <p className="text-gray-400">配置LLM和全局设置</p>
      </div>

      <div className="max-w-2xl space-y-6">
        {/* LLM Settings */}
        <Card>
          <CardContent className="p-6 space-y-4">
            <div className="flex items-center gap-3 mb-4">
              <Key className="w-5 h-5 text-cinema-gold" />
              <h2 className="font-display text-lg font-semibold text-white">LLM 配置</h2>
            </div>

            <div className="grid gap-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">提供商</label>
                <select className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none">
                  <option value="openai">OpenAI</option>
                  <option value="anthropic">Anthropic</option>
                  <option value="ollama">Ollama (本地)</option>
                </select>
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-1">API Key</label>
                <input
                  type="password"
                  placeholder="sk-..."
                  className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-1">模型</label>
                <select className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none">
                  <option value="gpt-4">GPT-4</option>
                  <option value="gpt-4-turbo">GPT-4 Turbo</option>
                  <option value="claude-3-opus">Claude 3 Opus</option>
                  <option value="claude-3-sonnet">Claude 3 Sonnet</option>
                </select>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Temperature</label>
                  <input
                    type="number"
                    step="0.1"
                    min="0"
                    max="2"
                    defaultValue="0.7"
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Max Tokens</label>
                  <input
                    type="number"
                    defaultValue="4096"
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                  />
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="flex justify-end">
          <Button variant="primary">保存设置</Button>
        </div>
      </div>
    </div>
  );
}
