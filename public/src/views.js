// CINEMA-AI Views
// All UI view components

const Views = {
    // Dashboard View
    dashboard(data) {
        const stats = data || { stories_count: 0, characters_count: 0, chapters_count: 0, current_story: null };
        return `
            <div class="fade-in">
                <h2 class="text-3xl font-bold mb-8">📊 仪表盘</h2>

                <!-- Stats Cards -->
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                        <div class="flex items-center justify-between mb-4">
                            <i data-lucide="book-open" class="w-8 h-8 text-cinema-accent"></i>
                            <span class="text-3xl font-bold">${stats.stories_count}</span>
                        </div>
                        <div class="text-gray-400">故事</div>
                    </div>
                    <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                        <div class="flex items-center justify-between mb-4">
                            <i data-lucide="users" class="w-8 h-8 text-cinema-success"></i>
                            <span class="text-3xl font-bold">${stats.characters_count}</span>
                        </div>
                        <div class="text-gray-400">角色</div>
                    </div>
                    <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                        <div class="flex items-center justify-between mb-4">
                            <i data-lucide="file-text" class="w-8 h-8 text-cinema-warning"></i>
                            <span class="text-3xl font-bold">${stats.chapters_count}</span>
                        </div>
                        <div class="text-gray-400">章节</div>
                    </div>
                    <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                        <div class="flex items-center justify-between mb-4">
                            <i data-lucide="activity" class="w-8 h-8 text-purple-500"></i>
                            <span class="text-3xl font-bold text-green-400">95%</span>
                        </div>
                        <div class="text-gray-400">一致性</div>
                    </div>
                </div>

                <!-- Quick Actions -->
                <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700 mb-8">
                    <h3 class="text-xl font-semibold mb-4">快速操作</h3>
                    <div class="flex flex-wrap gap-4">
                        <button onclick="app.navigate('stories')" class="flex items-center gap-2 bg-cinema-accent hover:bg-blue-600 px-4 py-2 rounded-lg transition">
                            <i data-lucide="plus" class="w-4 h-4"></i>
                            新建故事
                        </button>
                        <button onclick="app.navigate('chapters')" class="flex items-center gap-2 bg-cinema-700 hover:bg-cinema-600 px-4 py-2 rounded-lg transition">
                            <i data-lucide="pen-tool" class="w-4 h-4"></i>
                            开始写作
                        </button>
                        <button onclick="app.navigate('skills')" class="flex items-center gap-2 bg-cinema-700 hover:bg-cinema-600 px-4 py-2 rounded-lg transition">
                            <i data-lucide="zap" class="w-4 h-4"></i>
                            技能管理
                        </button>
                        <button onclick="app.navigate('settings')" class="flex items-center gap-2 bg-cinema-700 hover:bg-cinema-600 px-4 py-2 rounded-lg transition">
                            <i data-lucide="settings" class="w-4 h-4"></i>
                            系统设置
                        </button>
                    </div>
                </div>

                <!-- Recent Activity -->
                <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                    <h3 class="text-xl font-semibold mb-4">最近活动</h3>
                    <div class="space-y-3">
                        <div class="flex items-center gap-3 text-gray-400">
                            <i data-lucide="clock" class="w-4 h-4"></i>
                            <span>欢迎使用 CINEMA-AI v2.0</span>
                        </div>
                    </div>
                </div>
            </div>
        `;
    },

    // Stories List View
    storiesList(stories) {
        return `
            <div class="fade-in">
                <div class="flex justify-between items-center mb-8">
                    <h2 class="text-3xl font-bold">📚 故事管理</h2>
                    <button onclick="app.showModal('createStory')" class="flex items-center gap-2 bg-cinema-accent hover:bg-blue-600 px-4 py-2 rounded-lg transition">
                        <i data-lucide="plus" class="w-4 h-4"></i>
                        新建故事
                    </button>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    ${stories.map(s => `
                        <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700 hover:border-cinema-accent transition cursor-pointer" onclick="app.selectStory('${s.id}')">
                            <h3 class="text-xl font-semibold mb-2">${s.title}</h3>
                            <p class="text-gray-400 text-sm mb-4 line-clamp-2">${s.description || '暂无描述'}</p>
                            <div class="flex items-center gap-4 text-sm text-gray-500">
                                <span class="flex items-center gap-1">
                                    <i data-lucide="tag" class="w-3 h-3"></i>
                                    ${s.genre || '未分类'}
                                </span>
                                <span class="flex items-center gap-1">
                                    <i data-lucide="calendar" class="w-3 h-3"></i>
                                    ${new Date(s.created_at).toLocaleDateString()}
                                </span>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    // Create Story Modal
    createStoryModal() {
        return `
            <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onclick="if(event.target===this)app.closeModal()">
                <div class="bg-cinema-800 rounded-xl p-6 w-full max-w-lg border border-cinema-700" onclick="event.stopPropagation()">
                    <h3 class="text-xl font-semibold mb-4">新建故事</h3>
                    <form id="create-story-form" onsubmit="app.handleCreateStory(event)">
                        <div class="mb-4">
                            <label class="block text-gray-400 mb-2">标题 *</label>
                            <input type="text" name="title" required class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none">
                        </div>
                        <div class="mb-4">
                            <label class="block text-gray-400 mb-2">类型</label>
                            <select name="genre" class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none">
                                <option value="">选择类型</option>
                                <option value="fantasy">奇幻</option>
                                <option value="sci-fi">科幻</option>
                                <option value="mystery">悬疑</option>
                                <option value="romance">言情</option>
                                <option value="thriller">惊悚</option>
                                <option value="wuxia">武侠</option>
                                <option value="xianxia">仙侠</option>
                                <option value="urban">都市</option>
                            </select>
                        </div>
                        <div class="mb-4">
                            <label class="block text-gray-400 mb-2">基调</label>
                            <select name="tone" class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none">
                                <option value="">选择基调</option>
                                <option value="dark">暗黑</option>
                                <option value="light">轻松</option>
                                <option value="serious">严肃</option>
                                <option value="humorous">幽默</option>
                                <option value="epic">史诗</option>
                            </select>
                        </div>
                        <div class="mb-6">
                            <label class="block text-gray-400 mb-2">简介</label>
                            <textarea name="description" rows="3" class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none resize-none"></textarea>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button type="button" onclick="app.closeModal()" class="px-4 py-2 text-gray-400 hover:text-white transition">取消</button>
                            <button type="submit" class="bg-cinema-accent hover:bg-blue-600 px-6 py-2 rounded-lg transition">创建</button>
                        </div>
                    </form>
                </div>
            </div>
        `;
    },

    // Characters View
    characters(characters) {
        return `
            <div class="fade-in">
                <div class="flex justify-between items-center mb-8">
                    <h2 class="text-3xl font-bold">👥 角色管理</h2>
                    <button onclick="app.showModal('createCharacter')" class="flex items-center gap-2 bg-cinema-accent hover:bg-blue-600 px-4 py-2 rounded-lg transition">
                        <i data-lucide="plus" class="w-4 h-4"></i>
                        新建角色
                    </button>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    ${characters.map(c => `
                        <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                            <div class="flex items-start justify-between mb-4">
                                <div>
                                    <h3 class="text-xl font-semibold">${c.name}</h3>
                                    <span class="text-sm text-gray-500">${c.role || '配角'}</span>
                                </div>
                                <button onclick="app.editCharacter('${c.id}')" class="text-gray-400 hover:text-cinema-accent">
                                    <i data-lucide="edit-2" class="w-4 h-4"></i>
                                </button>
                            </div>
                            <p class="text-gray-400 text-sm mb-4">${c.background || '暂无背景'}</p>
                            ${c.personality ? `
                                <div class="mb-3">
                                    <span class="text-xs text-gray-500 uppercase">性格</span>
                                    <p class="text-sm">${c.personality}</p>
                                </div>
                            ` : ''}
                            ${c.traits?.length ? `
                                <div class="flex flex-wrap gap-2">
                                    ${c.traits.map(t => `<span class="bg-purple-900/50 text-purple-300 text-xs px-2 py-1 rounded">${t}</span>`).join('')}
                                </div>
                            ` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    // Chapters View
    chapters(chapters) {
        return `
            <div class="fade-in">
                <div class="flex justify-between items-center mb-8">
                    <h2 class="text-3xl font-bold">📝 章节创作</h2>
                    <button onclick="app.showModal('createChapter')" class="flex items-center gap-2 bg-cinema-accent hover:bg-blue-600 px-4 py-2 rounded-lg transition">
                        <i data-lucide="plus" class="w-4 h-4"></i>
                        新建章节
                    </button>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <!-- Chapter List -->
                    <div class="lg:col-span-1 bg-cinema-800 rounded-xl border border-cinema-700 overflow-hidden">
                        <div class="p-4 border-b border-cinema-700">
                            <h3 class="font-semibold">章节列表</h3>
                        </div>
                        <div class="max-h-[600px] overflow-y-auto">
                            ${chapters.map((c, i) => `
                                <div class="p-4 border-b border-cinema-700 hover:bg-cinema-700 cursor-pointer transition" onclick="app.selectChapter('${c.id}')">
                                    <div class="flex items-center justify-between">
                                        <span class="text-sm text-gray-500">第 ${i + 1} 章</span>
                                        <span class="text-xs px-2 py-1 rounded ${c.status === 'completed' ? 'bg-green-900 text-green-300' : 'bg-yellow-900 text-yellow-300'}">${c.status === 'completed' ? '已完成' : '写作中'}</span>
                                    </div>
                                    <h4 class="font-medium mt-1">${c.title || '未命名章节'}</h4>
                                </div>
                            `).join('')}
                        </div>
                    </div>

                    <!-- Editor -->
                    <div class="lg:col-span-2 bg-cinema-800 rounded-xl border border-cinema-700 p-6">
                        <div id="chapter-editor">
                            <div class="text-center text-gray-500 py-20">
                                <i data-lucide="file-text" class="w-16 h-16 mx-auto mb-4 opacity-50"></i>
                                <p>选择一个章节开始编辑</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;
    },

    // Skills View
    skills(skills) {
        const categories = {
            writing: '写作辅助',
            analysis: '分析工具',
            character: '角色管理',
            plot: '情节设计',
            style: '文风处理',
            export: '导出格式',
            integration: '外部集成',
            custom: '自定义'
        };

        return `
            <div class="fade-in">
                <div class="flex justify-between items-center mb-8">
                    <h2 class="text-3xl font-bold">⚡ 技能管理</h2>
                    <div class="flex gap-3">
                        <button onclick="app.showModal('importSkill')" class="flex items-center gap-2 bg-cinema-700 hover:bg-cinema-600 px-4 py-2 rounded-lg transition">
                            <i data-lucide="download" class="w-4 h-4"></i>
                            导入技能
                        </button>
                        <button onclick="app.navigate('mcp')" class="flex items-center gap-2 bg-cinema-accent hover:bg-blue-600 px-4 py-2 rounded-lg transition">
                            <i data-lucide="plug" class="w-4 h-4"></i>
                            MCP 配置
                        </button>
                    </div>
                </div>

                <!-- Category Tabs -->
                <div class="flex gap-2 mb-6 overflow-x-auto pb-2">
                    <button onclick="app.filterSkills('all')" class="px-4 py-2 rounded-lg bg-cinema-accent text-white whitespace-nowrap">全部</button>
                    ${Object.entries(categories).map(([key, label]) => `
                        <button onclick="app.filterSkills('${key}')" class="px-4 py-2 rounded-lg bg-cinema-800 hover:bg-cinema-700 transition whitespace-nowrap">${label}</button>
                    `).join('')}
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    ${skills.map(s => `
                        <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700 ${s.is_enabled ? '' : 'opacity-60'}">
                            <div class="flex items-start justify-between mb-4">
                                <div class="flex items-center gap-3">
                                    <div class="w-10 h-10 rounded-lg bg-cinema-700 flex items-center justify-center">
                                        <i data-lucide="${s.manifest.category === 'writing' ? 'pen-tool' : s.manifest.category === 'analysis' ? 'bar-chart-2' : 'zap'}" class="w-5 h-5 text-cinema-accent"></i>
                                    </div>
                                    <div>
                                        <h3 class="font-semibold">${s.manifest.name}</h3>
                                        <span class="text-xs text-gray-500">${categories[s.manifest.category] || s.manifest.category}</span>
                                    </div>
                                </div>
                                <label class="relative inline-flex items-center cursor-pointer">
                                    <input type="checkbox" ${s.is_enabled ? 'checked' : ''} onchange="app.toggleSkill('${s.manifest.id}', this.checked)" class="sr-only peer">
                                    <div class="w-11 h-6 bg-cinema-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-cinema-accent"></div>
                                </label>
                            </div>
                            <p class="text-gray-400 text-sm mb-4">${s.manifest.description}</p>
                            <div class="flex items-center justify-between text-xs text-gray-500">
                                <span>v${s.manifest.version}</span>
                                <span>${s.manifest.author}</span>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    // MCP Configuration View
    mcpConfig(servers) {
        return `
            <div class="fade-in">
                <div class="flex justify-between items-center mb-8">
                    <h2 class="text-3xl font-bold">🔌 MCP 配置</h2>
                    <button onclick="app.showModal('addMcpServer')" class="flex items-center gap-2 bg-cinema-accent hover:bg-blue-600 px-4 py-2 rounded-lg transition">
                        <i data-lucide="plus" class="w-4 h-4"></i>
                        添加服务器
                    </button>
                </div>

                <div class="bg-cinema-800 rounded-xl border border-cinema-700 p-6 mb-6">
                    <h3 class="text-lg font-semibold mb-2">关于 MCP</h3>
                    <p class="text-gray-400 text-sm">Model Context Protocol (MCP) 允许你连接外部工具和服务到 CINEMA-AI。配置 MCP 服务器后，你可以在技能系统中使用这些外部能力。</p>
                </div>

                <div class="space-y-4">
                    ${servers.map(s => `
                        <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                            <div class="flex items-start justify-between">
                                <div class="flex items-center gap-4">
                                    <div class="w-12 h-12 rounded-lg bg-cinema-700 flex items-center justify-center">
                                        <i data-lucide="server" class="w-6 h-6 text-cinema-accent"></i>
                                    </div>
                                    <div>
                                        <h3 class="font-semibold">${s.name}</h3>
                                        <p class="text-sm text-gray-500">${s.command} ${s.args.join(' ')}</p>
                                    </div>
                                </div>
                                <div class="flex gap-2">
                                    <button onclick="app.testMcpServer('${s.id}')" class="p-2 text-gray-400 hover:text-cinema-accent transition" title="测试连接">
                                        <i data-lucide="activity" class="w-4 h-4"></i>
                                    </button>
                                    <button onclick="app.deleteMcpServer('${s.id}')" class="p-2 text-gray-400 hover:text-red-400 transition" title="删除">
                                        <i data-lucide="trash-2" class="w-4 h-4"></i>
                                    </button>
                                </div>
                            </div>
                            ${s.tools?.length ? `
                                <div class="mt-4 pt-4 border-t border-cinema-700">
                                    <span class="text-xs text-gray-500 uppercase">可用工具</span>
                                    <div class="flex flex-wrap gap-2 mt-2">
                                        ${s.tools.map(t => `<span class="bg-cinema-700 text-xs px-2 py-1 rounded">${t.name}</span>`).join('')}
                                    </div>
                                </div>
                            ` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    // Settings View
    settings(config) {
        return `
            <div class="fade-in">
                <h2 class="text-3xl font-bold mb-8">⚙️ 系统设置</h2>

                <div class="max-w-2xl space-y-6">
                    <!-- LLM Settings -->
                    <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                        <h3 class="text-xl font-semibold mb-4 flex items-center gap-2">
                            <i data-lucide="brain" class="w-5 h-5 text-cinema-accent"></i>
                            LLM 配置
                        </h3>
                        <form onsubmit="app.saveSettings(event)">
                            <div class="mb-4">
                                <label class="block text-gray-400 mb-2">提供商</label>
                                <select name="provider" class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none">
                                    <option value="openai" ${config?.provider === 'openai' ? 'selected' : ''}>OpenAI</option>
                                    <option value="anthropic" ${config?.provider === 'anthropic' ? 'selected' : ''}>Anthropic</option>
                                    <option value="ollama" ${config?.provider === 'ollama' ? 'selected' : ''}>Ollama (本地)</option>
                                </select>
                            </div>
                            <div class="mb-4">
                                <label class="block text-gray-400 mb-2">API Key</label>
                                <input type="password" name="api_key" value="${config?.api_key || ''}" class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none">
                            </div>
                            <div class="mb-4">
                                <label class="block text-gray-400 mb-2">模型</label>
                                <select name="model" class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none">
                                    <option value="gpt-4" ${config?.model === 'gpt-4' ? 'selected' : ''}>GPT-4</option>
                                    <option value="gpt-4-turbo" ${config?.model === 'gpt-4-turbo' ? 'selected' : ''}>GPT-4 Turbo</option>
                                    <option value="claude-3-opus" ${config?.model === 'claude-3-opus' ? 'selected' : ''}>Claude 3 Opus</option>
                                    <option value="claude-3-sonnet" ${config?.model === 'claude-3-sonnet' ? 'selected' : ''}>Claude 3 Sonnet</option>
                                </select>
                            </div>
                            <div class="grid grid-cols-2 gap-4 mb-6">
                                <div>
                                    <label class="block text-gray-400 mb-2">Temperature</label>
                                    <input type="number" name="temperature" value="${config?.temperature || 0.7}" min="0" max="2" step="0.1" class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none">
                                </div>
                                <div>
                                    <label class="block text-gray-400 mb-2">Max Tokens</label>
                                    <input type="number" name="max_tokens" value="${config?.max_tokens || 4096}" class="w-full bg-cinema-900 border border-cinema-700 rounded-lg px-4 py-2 focus:border-cinema-accent focus:outline-none">
                                </div>
                            </div>
                            <button type="submit" class="w-full bg-cinema-accent hover:bg-blue-600 py-2 rounded-lg transition">保存设置</button>
                        </form>
                    </div>

                    <!-- Export Settings -->
                    <div class="bg-cinema-800 rounded-xl p-6 border border-cinema-700">
                        <h3 class="text-xl font-semibold mb-4 flex items-center gap-2">
                            <i data-lucide="download" class="w-5 h-5 text-cinema-success"></i>
                            导出设置
                        </h3>
                        <div class="space-y-3">
                            <label class="flex items-center gap-3">
                                <input type="checkbox" checked class="w-4 h-4 rounded border-cinema-700 bg-cinema-900 text-cinema-accent">
                                <span>包含大纲</span>
                            </label>
                            <label class="flex items-center gap-3">
                                <input type="checkbox" checked class="w-4 h-4 rounded border-cinema-700 bg-cinema-900 text-cinema-accent">
                                <span>包含元数据</span>
                            </label>
                            <label class="flex items-center gap-3">
                                <input type="checkbox" class="w-4 h-4 rounded border-cinema-700 bg-cinema-900 text-cinema-accent">
                                <span>包含角色信息</span>
                            </label>
                        </div>
                    </div>
                </div>
            </div>
        `;
    },

    // Sidebar
    sidebar(currentView) {
        const items = [
            { id: 'dashboard', icon: 'layout-dashboard', label: '仪表盘' },
            { id: 'stories', icon: 'book-open', label: '故事管理' },
            { id: 'characters', icon: 'users', label: '角色管理' },
            { id: 'chapters', icon: 'file-text', label: '章节创作' },
            { id: 'skills', icon: 'zap', label: '技能管理' },
            { id: 'settings', icon: 'settings', label: '系统设置' },
        ];

        return `
            <aside class="w-64 bg-cinema-800 border-r border-cinema-700 flex flex-col">
                <div class="p-6 border-b border-cinema-700">
                    <h1 class="text-xl font-bold text-cinema-accent flex items-center gap-2">
                        <i data-lucide="film" class="w-6 h-6"></i>
                        CINEMA-AI
                    </h1>
                    <p class="text-xs text-gray-500 mt-1">v2.0.0-alpha</p>
                </div>
                <nav class="flex-1 p-4 space-y-1">
                    ${items.map(item => `
                        <button onclick="app.navigate('${item.id}')"
                            class="w-full flex items-center gap-3 px-4 py-3 rounded-lg transition ${currentView === item.id ? 'bg-cinema-accent text-white' : 'text-gray-400 hover:bg-cinema-700 hover:text-white'}">
                            <i data-lucide="${item.icon}" class="w-5 h-5"></i>
                            ${item.label}
                        </button>
                    `).join('')}
                </nav>
                <div class="p-4 border-t border-cinema-700">
                    <div class="bg-cinema-900 rounded-lg p-3">
                        <div class="text-xs text-gray-500 mb-1">当前故事</div>
                        <div class="font-medium truncate" id="current-story-name">未选择</div>
                    </div>
                </div>
            </aside>
        `;
    },

    // Toast Notification
    toast(message, type = 'info') {
        const colors = {
            info: 'bg-cinema-accent',
            success: 'bg-green-600',
            warning: 'bg-yellow-600',
            error: 'bg-red-600'
        };

        const id = 'toast-' + Date.now();
        const el = document.createElement('div');
        el.id = id;
        el.className = `${colors[type]} text-white px-6 py-3 rounded-lg shadow-lg flex items-center gap-3 fade-in`;
        el.innerHTML = `
            <i data-lucide="${type === 'success' ? 'check-circle' : type === 'error' ? 'x-circle' : 'info'}" class="w-5 h-5"></i>
            <span>${message}</span>
        `;

        document.getElementById('toast-container').appendChild(el);
        lucide.createIcons();

        setTimeout(() => {
            el.style.opacity = '0';
            el.style.transform = 'translateX(100%)';
            el.style.transition = 'all 0.3s ease';
            setTimeout(() => el.remove(), 300);
        }, 3000);
    }
};