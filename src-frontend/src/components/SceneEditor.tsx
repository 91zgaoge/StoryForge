import React, { useState, useEffect } from 'react';
import { 
  Target, 
  Zap, 
  Users, 
  MapPin, 
  Clock, 
  Sparkles,
  Save,
  X
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import type { Scene, ConflictType, CharacterConflict } from '@/types';
import { getConflictTypeLabel, getConflictTypeColor } from '@/hooks/useScenes';

interface SceneEditorProps {
  scene: Scene | null;
  characters: { id: string; name: string; personality?: string }[];
  onSave: (updates: Partial<Scene>) => void;
  onCancel: () => void;
}

const CONFLICT_TYPES: ConflictType[] = [
  'ManVsMan',
  'ManVsSelf',
  'ManVsSociety',
  'ManVsNature',
  'ManVsTechnology',
  'ManVsFate',
  'ManVsSupernatural',
];

export function SceneEditor({ scene, characters, onSave, onCancel }: SceneEditorProps) {
  const [formData, setFormData] = useState<Partial<Scene>>({});
  const [activeTab, setActiveTab] = useState<'basic' | 'drama' | 'content'>('basic');

  useEffect(() => {
    if (scene) {
      setFormData({
        title: scene.title,
        dramatic_goal: scene.dramatic_goal,
        external_pressure: scene.external_pressure,
        conflict_type: scene.conflict_type,
        characters_present: scene.characters_present,
        character_conflicts: scene.character_conflicts,
        setting_location: scene.setting_location,
        setting_time: scene.setting_time,
        setting_atmosphere: scene.setting_atmosphere,
        content: scene.content,
      });
    }
  }, [scene]);

  if (!scene) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        选择一个场景进行编辑
      </div>
    );
  }

  const handleSave = () => {
    onSave(formData);
  };

  const toggleCharacter = (charId: string) => {
    const current = formData.characters_present || [];
    if (current.includes(charId)) {
      setFormData({
        ...formData,
        characters_present: current.filter(id => id !== charId),
      });
    } else {
      setFormData({
        ...formData,
        characters_present: [...current, charId],
      });
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-white">
          编辑场景 #{scene.sequence_number}
        </h2>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="sm" onClick={onCancel}>
            <X className="w-4 h-4 mr-1" />
            取消
          </Button>
          <Button variant="primary" size="sm" onClick={handleSave}>
            <Save className="w-4 h-4 mr-1" />
            保存
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-4 p-1 bg-cinema-800 rounded-lg">
        {[
          { id: 'basic', label: '基本信息', icon: Target },
          { id: 'drama', label: '戏剧结构', icon: Zap },
          { id: 'content', label: '内容', icon: Sparkles },
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as typeof activeTab)}
            className={`
              flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors
              ${activeTab === tab.id 
                ? 'bg-cinema-gold text-cinema-900' 
                : 'text-gray-400 hover:text-white hover:bg-cinema-700'
              }
            `}
          >
            <tab.icon className="w-4 h-4" />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto space-y-4">
        {/* Basic Info Tab */}
        {activeTab === 'basic' && (
          <>
            {/* Title */}
            <div>
              <label className="block text-sm text-gray-400 mb-1">场景标题</label>
              <input
                type="text"
                value={formData.title || ''}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                placeholder={`场景 ${scene.sequence_number}`}
                className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white focus:border-cinema-gold focus:outline-none"
              />
            </div>

            {/* Setting */}
            <Card>
              <CardContent className="p-4 space-y-3">
                <h3 className="font-medium text-white flex items-center gap-2">
                  <MapPin className="w-4 h-4 text-cinema-gold" />
                  场景设置
                </h3>
                
                <div>
                  <label className="block text-xs text-gray-400 mb-1">地点</label>
                  <input
                    type="text"
                    value={formData.setting_location || ''}
                    onChange={(e) => setFormData({ ...formData, setting_location: e.target.value })}
                    placeholder="例如：长安城、太空站..."
                    className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                  />
                </div>
                
                <div>
                  <label className="block text-xs text-gray-400 mb-1">时间</label>
                  <input
                    type="text"
                    value={formData.setting_time || ''}
                    onChange={(e) => setFormData({ ...formData, setting_time: e.target.value })}
                    placeholder="例如：黄昏、2145年..."
                    className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                  />
                </div>
                
                <div>
                  <label className="block text-xs text-gray-400 mb-1">氛围</label>
                  <input
                    type="text"
                    value={formData.setting_atmosphere || ''}
                    onChange={(e) => setFormData({ ...formData, setting_atmosphere: e.target.value })}
                    placeholder="例如：紧张、神秘、温馨..."
                    className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none"
                  />
                </div>
              </CardContent>
            </Card>

            {/* Characters */}
            <Card>
              <CardContent className="p-4">
                <h3 className="font-medium text-white flex items-center gap-2 mb-3">
                  <Users className="w-4 h-4 text-cinema-gold" />
                  出场角色
                </h3>
                
                <div className="grid grid-cols-2 gap-2">
                  {characters.map((char) => (
                    <button
                      key={char.id}
                      onClick={() => toggleCharacter(char.id)}
                      className={`
                        flex items-center gap-2 p-2 rounded-lg text-left text-sm transition-colors
                        ${(formData.characters_present || []).includes(char.id)
                          ? 'bg-cinema-gold/20 border border-cinema-gold/50 text-white'
                          : 'bg-cinema-800 border border-transparent text-gray-300 hover:bg-cinema-700'
                        }
                      `}
                    >
                      <div className={`
                        w-2 h-2 rounded-full
                        ${(formData.characters_present || []).includes(char.id) ? 'bg-cinema-gold' : 'bg-gray-600'}
                      `} />
                      <div>
                        <div className="font-medium">{char.name}</div>
                        {char.personality && (
                          <div className="text-xs text-gray-500 truncate">{char.personality}</div>
                        )}
                      </div>
                    </button>
                  ))}
                </div>

                {characters.length === 0 && (
                  <p className="text-sm text-gray-500 text-center py-4">
                    还没有创建角色
                  </p>
                )}
              </CardContent>
            </Card>
          </>
        )}

        {/* Drama Tab */}
        {activeTab === 'drama' && (
          <>
            {/* Dramatic Goal */}
            <Card>
              <CardContent className="p-4">
                <h3 className="font-medium text-white flex items-center gap-2 mb-3">
                  <Target className="w-4 h-4 text-cinema-gold" />
                  戏剧目标
                </h3>
                <p className="text-xs text-gray-500 mb-2">
                  这个场景要完成什么？推动什么情节？
                </p>
                <textarea
                  value={formData.dramatic_goal || ''}
                  onChange={(e) => setFormData({ ...formData, dramatic_goal: e.target.value })}
                  placeholder="例如：主角发现真相，反派暴露野心..."
                  rows={3}
                  className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-none"
                />
              </CardContent>
            </Card>

            {/* External Pressure */}
            <Card>
              <CardContent className="p-4">
                <h3 className="font-medium text-white flex items-center gap-2 mb-3">
                  <Zap className="w-4 h-4 text-cinema-gold" />
                  外部压迫
                </h3>
                <p className="text-xs text-gray-500 mb-2">
                  什么力量在给角色施压？（环境、反派、事件等）
                </p>
                <textarea
                  value={formData.external_pressure || ''}
                  onChange={(e) => setFormData({ ...formData, external_pressure: e.target.value })}
                  placeholder="例如：暴雨将至，追兵逼近，时间紧迫..."
                  rows={3}
                  className="w-full px-3 py-2 bg-cinema-800 border border-cinema-700 rounded-lg text-white text-sm focus:border-cinema-gold focus:outline-none resize-none"
                />
              </CardContent>
            </Card>

            {/* Conflict Type */}
            <Card>
              <CardContent className="p-4">
                <h3 className="font-medium text-white mb-3">冲突类型</h3>
                <div className="grid grid-cols-2 gap-2">
                  {CONFLICT_TYPES.map((type) => (
                    <button
                      key={type}
                      onClick={() => setFormData({ ...formData, conflict_type: type })}
                      className={`
                        flex items-center gap-2 p-3 rounded-lg text-left transition-colors
                        ${formData.conflict_type === type
                          ? 'bg-cinema-gold/20 border border-cinema-gold/50'
                          : 'bg-cinema-800 border border-transparent hover:bg-cinema-700'
                        }
                      `}
                    >
                      <div
                        className="w-3 h-3 rounded-full"
                        style={{ backgroundColor: getConflictTypeColor(type) }}
                      />
                      <span className={`
                        text-sm
                        ${formData.conflict_type === type ? 'text-white' : 'text-gray-300'}
                      `}>
                        {getConflictTypeLabel(type)}
                      </span>
                    </button>
                  ))}
                </div>
              </CardContent>
            </Card>
          </>
        )}

        {/* Content Tab */}
        {activeTab === 'content' && (
          <div>
            <label className="block text-sm text-gray-400 mb-2">场景内容</label>
            <textarea
              value={formData.content || ''}
              onChange={(e) => setFormData({ ...formData, content: e.target.value })}
              placeholder="开始写作..."
              rows={20}
              className="w-full px-4 py-3 bg-cinema-800 border border-cinema-700 rounded-lg text-white focus:border-cinema-gold focus:outline-none resize-none font-serif leading-relaxed"
            />
          </div>
        )}
      </div>
    </div>
  );
}
