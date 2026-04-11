/**
 * Settings Page - 工作室配置
 * 
 * 支持多类型LLM配置管理：
 * - Chat/Completion 模型（文本生成）
 * - Embedding 模型（向量嵌入）
 * - Multimodal 模型（多模态）
 * - Image 模型（图像生成）
 * 
 * 功能：
 * - 添加/编辑/删除模型配置
 * - 设置默认模型
 * - Agent模型映射
 * - 设置导出/导入
 */

import { useState } from 'react';
import { 
  Settings2, Key, Globe, Database, 
  Plus, Trash2, Edit2, Download, Upload,
  Check, X, Bot, Sparkles, Image, MessageSquare,
  RefreshCw
} from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useSettings, useModels, useExportSettings, useImportSettings } from '@/hooks/useSettings';
import { useUpdater } from '@/hooks/useUpdater';
import { useForm } from 'react-hook-form';
import { cn } from '@/utils/cn';
import type { ModelConfig, ModelType, LlmProvider } from '@/types/llm';
import { getModelProviders, getProviderDefaultModels } from '@/services/settings';

type TabType = 'chat' | 'embedding' | 'multimodal' | 'image' | 'agents' | 'general';

export function Settings() {
  const [activeTab, setActiveTab] = useState<TabType>('chat');
  const [showAddModal, setShowAddModal] = useState(false);
  const [editingModel, setEditingModel] = useState<ModelConfig | null>(null);
  
  const { data: settings, isLoading: settingsLoading } = useSettings();
  const { data: models = [], isLoading: modelsLoading } = useModels();
  const exportSettings = useExportSettings();
  const importSettings = useImportSettings();
  
  const isLoading = settingsLoading || modelsLoading;
  
  // 按类型过滤模型
  const filteredModels = models.filter(m => m.type === activeTab);
  
  // 处理设置导入
  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      importSettings.mutate(file);
    }
  };
  
  return (
    <div className="p-8 space-y-6 animate-fade-in">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold text-white">工作室配置</h1>
          <p className="text-gray-400">配置LLM模型和全局设置</p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="ghost" onClick={() => exportSettings.mutate()} isLoading={exportSettings.isPending}>
            <Download className="w-4 h-4 mr-2" />
            导出设置
          </Button>
          <label className="cursor-pointer inline-flex items-center gap-2 px-4 py-2 text-gray-400 hover:text-white hover:bg-cinema-800/50 rounded-xl transition-all">
            <input type="file" accept=".json" className="hidden" onChange={handleImport} />
            {importSettings.isPending ? (
              <span className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
            ) : (
              <Upload className="w-4 h-4" />
            )}
            导入设置
          </label>
        </div>
      </div>
      
      {/* 标签页 */}
      <div className="flex items-center gap-2 border-b border-cinema-800 pb-4 overflow-x-auto">
        <TabButton 
          active={activeTab === 'chat'} 
          onClick={() => setActiveTab('chat')}
          icon={<MessageSquare className="w-4 h-4" />}
          label="聊天模型"
        />
        <TabButton 
          active={activeTab === 'embedding'} 
          onClick={() => setActiveTab('embedding')}
          icon={<Database className="w-4 h-4" />}
          label="嵌入模型"
        />
        <TabButton 
          active={activeTab === 'multimodal'} 
          onClick={() => setActiveTab('multimodal')}
          icon={<Sparkles className="w-4 h-4" />}
          label="多模态"
        />
        <TabButton 
          active={activeTab === 'image'} 
          onClick={() => setActiveTab('image')}
          icon={<Image className="w-4 h-4" />}
          label="图像生成"
        />
        <TabButton 
          active={activeTab === 'agents'} 
          onClick={() => setActiveTab('agents')}
          icon={<Bot className="w-4 h-4" />}
          label="Agent配置"
        />
        <TabButton 
          active={activeTab === 'general'} 
          onClick={() => setActiveTab('general')}
          icon={<Settings2 className="w-4 h-4" />}
          label="通用设置"
        />
      </div>
      
      {/* 内容区域 */}
      {isLoading ? (
        <div className="text-center py-12 text-gray-500">加载中...</div>
      ) : (
        <>
          {activeTab === 'chat' && (
            <ModelList 
              type="chat" 
              models={filteredModels}
              onAdd={() => setShowAddModal(true)}
              onEdit={setEditingModel}
            />
          )}
          {activeTab === 'embedding' && (
            <ModelList 
              type="embedding" 
              models={filteredModels}
              onAdd={() => setShowAddModal(true)}
              onEdit={setEditingModel}
            />
          )}
          {activeTab === 'multimodal' && (
            <ModelList 
              type="multimodal" 
              models={filteredModels}
              onAdd={() => setShowAddModal(true)}
              onEdit={setEditingModel}
            />
          )}
          {activeTab === 'image' && (
            <ModelList 
              type="image" 
              models={filteredModels}
              onAdd={() => setShowAddModal(true)}
              onEdit={setEditingModel}
            />
          )}
          {activeTab === 'agents' && <AgentConfig />}
          {activeTab === 'general' && <GeneralSettings />}
        </>
      )}
      
      {/* 添加/编辑模态框 */}
      {(showAddModal || editingModel) && (
        <ModelModal 
          type={activeTab as ModelType}
          model={editingModel}
          onClose={() => {
            setShowAddModal(false);
            setEditingModel(null);
          }}
        />
      )}
    </div>
  );
}

// 标签按钮组件
function TabButton({ active, onClick, icon, label }: { 
  active: boolean; 
  onClick: () => void; 
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors whitespace-nowrap',
        active 
          ? 'bg-cinema-gold text-black' 
          : 'text-gray-400 hover:text-white hover:bg-cinema-800'
      )}
    >
      {icon}
      {label}
    </button>
  );
}

// 模型列表组件
function ModelList({ 
  type, 
  models,
  onAdd,
  onEdit,
}: { 
  type: ModelType;
  models: ModelConfig[];
  onAdd: () => void;
  onEdit: (model: ModelConfig) => void;
}) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-white">
          {type === 'chat' && '聊天模型配置'}
          {type === 'embedding' && '嵌入模型配置'}
          {type === 'multimodal' && '多模态模型配置'}
          {type === 'image' && '图像生成模型配置'}
        </h2>
        <Button variant="primary" onClick={onAdd}>
          <Plus className="w-4 h-4 mr-2" />
          添加模型
        </Button>
      </div>
      
      {models.length === 0 ? (
        <Card>
          <CardContent className="p-12 text-center">
            <Database className="w-16 h-16 text-gray-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-white mb-2">暂无模型配置</h3>
            <p className="text-gray-500 mb-4">点击上方按钮添加第一个模型配置</p>
            <Button variant="primary" onClick={onAdd}>
              <Plus className="w-4 h-4 mr-2" />
              添加模型
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {models.map(model => (
            <ModelCard 
              key={model.id} 
              model={model} 
              onEdit={() => onEdit(model)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// 模型卡片组件
function ModelCard({ model, onEdit }: { model: ModelConfig; onEdit: () => void }) {
  const isDefault = model.is_default;
  const requiresApiKey = model.provider !== 'ollama';
  const hasApiKey = model.api_key && model.api_key !== '***' && model.api_key.length > 0;
  
  return (
    <Card className={cn(isDefault && 'border-cinema-gold')}> 
      <CardContent className="p-5">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-4">
            {/* 提供商图标 */}
            <div className="w-12 h-12 rounded-xl bg-cinema-800 flex items-center justify-center">
              {model.provider === 'openai' && <span className="text-green-400 font-bold text-lg">O</span>}
              {model.provider === 'anthropic' && <span className="text-orange-400 font-bold text-lg">A</span>}
              {model.provider === 'ollama' && <span className="text-blue-400 font-bold text-lg">L</span>}
              {model.provider === 'azure' && <span className="text-blue-500 font-bold text-lg">Az</span>}
              {!['openai', 'anthropic', 'ollama', 'azure'].includes(model.provider) && (
                <Globe className="w-6 h-6 text-gray-400" />
              )}
            </div>
            
            <div>
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-white text-lg">{model.name}</h3>
                {isDefault && (
                  <span className="px-2 py-0.5 bg-cinema-gold/20 text-cinema-gold text-xs rounded-full">
                    默认
                  </span>
                )}
                {!model.enabled && (
                  <span className="px-2 py-0.5 bg-red-500/20 text-red-400 text-xs rounded-full">
                    禁用
                  </span>
                )}
              </div>
              <p className="text-sm text-gray-500">
                {model.provider} · {model.model}
              </p>
              {model.description && (
                <p className="text-sm text-gray-400 mt-1">{model.description}</p>
              )}
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            {requiresApiKey && !hasApiKey && (
              <span className="flex items-center gap-1 text-amber-400 text-sm">
                <Key className="w-4 h-4" />
                需配置API Key
              </span>
            )}
            <Button variant="ghost" size="sm" onClick={onEdit}>
              <Edit2 className="w-4 h-4" />
            </Button>
          </div>
        </div>
        
        {/* 能力标签 */}
        {'capabilities' in model && model.capabilities && (
          <div className="flex flex-wrap gap-2 mt-4">
            {model.capabilities.map(cap => (
              <span 
                key={cap}
                className="px-2 py-1 bg-cinema-800 text-gray-400 text-xs rounded-lg"
              >
                {cap}
              </span>
            ))}
          </div>
        )}
        
        {'dimensions' in model && (
          <div className="mt-4 text-sm text-gray-500">
            维度: {model.dimensions} · 最大输入: {model.max_input_tokens} tokens
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// 模型添加/编辑模态框
function ModelModal({ 
  type, 
  model,
  onClose,
}: { 
  type: ModelType;
  model: ModelConfig | null;
  onClose: () => void;
}) {
  const defaultValues = {
    name: '',
    provider: 'openai' as LlmProvider,
    model: '',
    api_key: '',
    api_base: '',
    temperature: 0.7,
    max_tokens: 4096,
    dimensions: 1536,
    is_default: false,
    enabled: true,
  };
  
  const { register, handleSubmit, watch } = useForm({
    defaultValues: model ? { ...defaultValues, ...model } : defaultValues
  });
  
  const provider = watch('provider');
  const providers = getModelProviders().filter(p => p.supports.includes(type));
  const defaultModels = getProviderDefaultModels(provider);
  const requiresApiKey = providers.find(p => p.id === provider)?.requiresApiKey ?? true;
  
  const onSubmit = (data: any) => {
    console.log('Submit:', data);
    // TODO: 调用创建/更新API
    onClose();
  };
  
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-full max-w-2xl max-h-[90vh] overflow-auto">
        <form onSubmit={handleSubmit(onSubmit)}>
          <CardContent className="p-6 space-y-6">
            <div className="flex items-center justify-between">
              <h2 className="font-display text-xl font-bold text-white">
                {model ? '编辑模型' : '添加模型'}
              </h2>
              <button type="button" onClick={onClose} className="text-gray-400 hover:text-white">
                <X className="w-5 h-5" />
              </button>
            </div>
            
            {/* 基本配置 */}
            <div className="grid grid-cols-2 gap-4">
              <div className="col-span-2">
                <label className="block text-sm text-gray-400 mb-1">名称 *</label>
                <input
                  {...register('name', { required: true })}
                  className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                  placeholder="例如: GPT-4"
                />
              </div>
              
              <div>
                <label className="block text-sm text-gray-400 mb-1">提供商 *</label>
                <select
                  {...register('provider', { required: true })}
                  className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                >
                  {providers.map(p => (
                    <option key={p.id} value={p.id}>{p.name}</option>
                  ))}
                </select>
              </div>
              
              <div>
                <label className="block text-sm text-gray-400 mb-1">模型 *</label>
                <input
                  {...register('model', { required: true })}
                  list="model-suggestions"
                  className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                  placeholder="例如: gpt-4"
                />
                <datalist id="model-suggestions">
                  {defaultModels.map(m => <option key={m} value={m} />)}
                </datalist>
              </div>
            </div>
            
            {/* API配置 */}
            <div className="space-y-4">
              <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wider">API配置</h3>
              
              {requiresApiKey && (
                <div>
                  <label className="block text-sm text-gray-400 mb-1">API Key</label>
                  <input
                    {...register('api_key')}
                    type="password"
                    className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                    placeholder="sk-..."
                  />
                  <p className="text-xs text-gray-500 mt-1">API Key将被安全存储</p>
                </div>
              )}
              
              <div>
                <label className="block text-sm text-gray-400 mb-1">API Base (可选)</label>
                <input
                  {...register('api_base')}
                  className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                  placeholder="https://api.openai.com/v1"
                />
              </div>
            </div>
            
            {/* 模型参数 */}
            {(type === 'chat' || type === 'multimodal') && (
              <div className="space-y-4">
                <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wider">模型参数</h3>
                
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">Temperature</label>
                    <input
                      {...register('temperature', { valueAsNumber: true })}
                      type="number"
                      step="0.1"
                      min="0"
                      max="2"
                      className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">Max Tokens</label>
                    <input
                      {...register('max_tokens', { valueAsNumber: true })}
                      type="number"
                      className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                    />
                  </div>
                </div>
              </div>
            )}
            
            {type === 'embedding' && (
              <div className="space-y-4">
                <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wider">嵌入参数</h3>
                
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">Dimensions</label>
                    <input
                      {...register('dimensions' as const, { valueAsNumber: true })}
                      type="number"
                      className="w-full px-4 py-2 bg-cinema-800 border border-cinema-700 rounded-xl text-white focus:border-cinema-gold focus:outline-none"
                    />
                  </div>
                </div>
              </div>
            )}
            
            {/* 选项 */}
            <div className="flex items-center gap-6">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  {...register('is_default')}
                  type="checkbox"
                  className="w-4 h-4 rounded border-cinema-700 bg-cinema-800 text-cinema-gold"
                />
                <span className="text-sm text-gray-300">设为默认</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  {...register('enabled')}
                  type="checkbox"
                  className="w-4 h-4 rounded border-cinema-700 bg-cinema-800 text-cinema-gold"
                />
                <span className="text-sm text-gray-300">启用</span>
              </label>
            </div>
            
            {/* 按钮 */}
            <div className="flex justify-end gap-3 pt-4 border-t border-cinema-800">
              <Button type="button" variant="ghost" onClick={onClose}>
                取消
              </Button>
              <Button type="submit" variant="primary">
                {model ? '保存' : '创建'}
              </Button>
            </div>
          </CardContent>
        </form>
      </Card>
    </div>
  );
}

// Agent配置组件
function AgentConfig() {
  return (
    <Card>
      <CardContent className="p-8 text-center">
        <Bot className="w-16 h-16 text-gray-600 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-white mb-2">Agent模型映射</h3>
        <p className="text-gray-500">为不同的Agent配置专用的LLM模型</p>
        <p className="text-sm text-gray-600 mt-4">功能开发中...</p>
      </CardContent>
    </Card>
  );
}

// 通用设置组件
function GeneralSettings() {
  const { 
    currentVersion, 
    hasUpdate, 
    latestVersion, 
    isChecking, 
    isInstalling,
    checkUpdate, 
    installUpdate 
  } = useUpdater(false);

  return (
    <div className="space-y-6">
      {/* 版本信息 */}
      <Card>
        <CardContent className="p-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-16 h-16 rounded-xl bg-gradient-to-br from-terracotta to-terracotta/60 flex items-center justify-center">
                <span className="text-white font-serif text-2xl font-bold">草</span>
              </div>
              <div>
                <h3 className="text-lg font-medium text-white">StoryForge (草苔)</h3>
                <p className="text-gray-400">当前版本: v{currentVersion}</p>
                {hasUpdate && (
                  <p className="text-terracotta text-sm">
                    新版本可用: v{latestVersion}
                  </p>
                )}
              </div>
            </div>
            <div className="flex gap-2">
              {hasUpdate ? (
                <Button 
                  variant="primary" 
                  onClick={installUpdate}
                  disabled={isInstalling}
                >
                  {isInstalling ? (
                    <>
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                      安装中...
                    </>
                  ) : (
                    <>
                      <Download className="w-4 h-4 mr-2" />
                      立即更新
                    </>
                  )}
                </Button>
              ) : (
                <Button 
                  variant="secondary" 
                  onClick={checkUpdate}
                  disabled={isChecking}
                >
                  {isChecking ? (
                    <>
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                      检查中...
                    </>
                  ) : (
                    <>
                      <RefreshCw className="w-4 h-4 mr-2" />
                      检查更新
                    </>
                  )}
                </Button>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 其他设置 */}
      <Card>
        <CardContent className="p-8 text-center">
          <Settings2 className="w-16 h-16 text-gray-600 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-white mb-2">通用设置</h3>
          <p className="text-gray-500">主题、语言、自动保存等全局配置</p>
          <p className="text-sm text-gray-600 mt-4">功能开发中...</p>
        </CardContent>
      </Card>
    </div>
  );
}
