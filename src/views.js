// CINEMA-AI Views
// Cinematic Design System - Distinctive, production-grade frontend

const Views = {
    // Dashboard View - Hero section with cinematic stats
    dashboard(data) {
        const stats = data || { stories_count: 0, characters_count: 0, chapters_count: 0, current_story: null };
        return `
            <div class="animate-fade-up">
                <!-- Hero Section -->
                <div class="mb-12 relative">
                    <div class="absolute inset-0 bg-gradient-to-r from-cinema-gold/5 via-transparent to-cinema-velvet/5 rounded-2xl"></div>
                    <div class="relative p-8">
                        <h1 class="font-display text-5xl font-bold text-white mb-4 text-glow">
                            创作宇宙
                        </h1>
                        <p class="font-body text-xl text-gray-400 italic max-w-2xl">
                            "每一个故事都是一扇通往无限可能的门"
                        </p>
                        <div class="gold-line w-32 mt-6"></div>
                    </div>
                </div>

                <!-- Cinematic Stats Cards -->
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
                    <div class="glass-cinema rounded-2xl p-6 gradient-border animate-fade-up stagger-1 hover:glow-gold transition-all duration-500 group">
                        <div class="flex items-center justify-between mb-4">
                            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-cinema-gold/20 to-cinema-rust/20 flex items-center justify-center group-hover:scale-110 transition-transform">
                                <i data-lucide="book-open" class="w-6 h-6 text-cinema-gold"></i>
                            </div>
                            <span class="font-display text-4xl font-bold text-cinema-gold">${stats.stories_count}</span>
                        </div>
                        <div class="font-body text-gray-400 text-sm uppercase tracking-wider">故事作品</div>
                        <div class="h-1 w-full bg-cinema-700 mt-4 rounded-full overflow-hidden">
                            <div class="h-full bg-gradient-to-r from-cinema-gold to-cinema-gold-light w-3/4 rounded-full"></div>
                        </div>
                    </div>

                    <div class="glass-cinema rounded-2xl p-6 gradient-border animate-fade-up stagger-2 hover:glow-gold transition-all duration-500 group">
                        <div class="flex items-center justify-between mb-4">
                            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-cinema-amber/20 to-cinema-rust/20 flex items-center justify-center group-hover:scale-110 transition-transform">
                                <i data-lucide="users" class="w-6 h-6 text-cinema-amber"></i>
                            </div>
                            <span class="font-display text-4xl font-bold text-cinema-amber">${stats.characters_count}</span>
                        </div>
                        <div class="font-body text-gray-400 text-sm uppercase tracking-wider">角色灵魂</div>
                        <div class="h-1 w-full bg-cinema-700 mt-4 rounded-full overflow-hidden">
                            <div class="h-full bg-gradient-to-r from-cinema-amber to-cinema-rust w-1/2 rounded-full"></div>
                        </div>
                    </div>

                    <div class="glass-cinema rounded-2xl p-6 gradient-border animate-fade-up stagger-3 hover:glow-gold transition-all duration-500 group">
                        <div class="flex items-center justify-between mb-4">
                            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-cinema-velvet/30 to-cinema-gold/20 flex items-center justify-center group-hover:scale-110 transition-transform">
                                <i data-lucide="file-text" class="w-6 h-6 text-cinema-velvet"></i>
                            </div>
                            <span class="font-display text-4xl font-bold text-cinema-velvet">${stats.chapters_count}</span>
                        </div>
                        <div class="font-body text-gray-400 text-sm uppercase tracking-wider">章节篇章</div>
                        <div class="h-1 w-full bg-cinema-700 mt-4 rounded-full overflow-hidden">
                            <div class="h-full bg-gradient-to-r from-cinema-velvet to-cinema-gold w-2/3 rounded-full"></div>
                        </div>
                    </div>

                    <div class="glass-cinema rounded-2xl p-6 gradient-border animate-fade-up stagger-4 hover:glow-gold transition-all duration-500 group">
                        <div class="flex items-center justify-between mb-4">
                            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-green-500/20 to-emerald-600/20 flex items-center justify-center group-hover:scale-110 transition-transform">
                                <i data-lucide="sparkles" class="w-6 h-6 text-green-400"></i>
                            </div>
                            <span class="font-display text-4xl font-bold text-green-400">95%</span>
                        </div>
                        <div class="font-body text-gray-400 text-sm uppercase tracking-wider">AI 一致性</div>
                        <div class="h-1 w-full bg-cinema-700 mt-4 rounded-full overflow-hidden">
                            <div class="h-full bg-gradient-to-r from-green-500 to-emerald-400 w-[95%] rounded-full"></div>
                        </div>
                    </div>
                </div>

                <!-- Cinematic Quick Actions -->
                <div class="glass-cinema rounded-2xl p-8 border border-cinema-gold/10 mb-12">
                    <h3 class="font-display text-2xl font-semibold mb-6 text-cinema-gold flex items-center gap-3">
                        <i data-lucide="clapperboard" class="w-6 h-6"></i>
                        开始创作
                    </h3>
                    <div class="flex flex-wrap gap-4">
                        <button onclick="app.navigate('stories')" class="group relative px-8 py-4 bg-gradient-to-r from-cinema-gold to-cinema-gold-dark rounded-xl font-body font-semibold text-cinema-950 hover:shadow-lg hover:shadow-cinema-gold/20 transition-all duration-300 overflow-hidden">
                            <span class="relative z-10 flex items-center gap-2">
                                <i data-lucide="plus" class="w-5 h-5"></i>
                                新建故事
                            </span>
                            <div class="absolute inset-0 bg-gradient-to-r from-cinema-gold-light to-cinema-gold opacity-0 group-hover:opacity-100 transition-opacity"></div>
                        </button>

                        <button onclick="app.navigate('chapters')" class="px-8 py-4 glass-cinema rounded-xl font-body font-medium text-cinema-gold border border-cinema-gold/30 hover:border-cinema-gold hover:bg-cinema-gold/5 transition-all duration-300 flex items-center gap-2">
                            <i data-lucide="pen-tool" class="w-5 h-5"></i>
                            继续写作
                        </button>

                        <button onclick="app.navigate('skills')" class="px-8 py-4 glass-cinema rounded-xl font-body font-medium text-gray-300 border border-cinema-600 hover:border-cinema-amber/50 hover:text-cinema-amber transition-all duration-300 flex items-center gap-2">
                            <i data-lucide="zap" class="w-5 h-5"></i>
                            技能配置
                        </button>

                        <button onclick="app.navigate('settings')" class="px-8 py-4 glass-cinema rounded-xl font-body font-medium text-gray-300 border border-cinema-600 hover:border-cinema-500 hover:text-white transition-all duration-300 flex items-center gap-2">
                            <i data-lucide="settings" class="w-5 h-5"></i>
                            系统设置
                        </button>
                    </div>
                </div>

                <!-- Recent Activity - Editorial Style -->
                <div class="glass-cinema rounded-2xl p-8 border border-cinema-500/20">
                    <h3 class="font-display text-xl font-semibold mb-6 text-gray-300 flex items-center gap-2">
                        <i data-lucide="clock" class="w-5 h-5 text-cinema-gold"></i>
                        最近活动
                    </h3>
                    <div class="space-y-4">
                        <div class="flex items-center gap-4 p-4 rounded-xl bg-cinema-850/50 border-l-2 border-cinema-gold">
                            <div class="w-2 h-2 rounded-full bg-cinema-gold animate-pulse"></div>
                            <span class="font-body text-gray-400">欢迎使用 CINEMA-AI v2.0 — 您的 AI 创作伙伴已就绪</span>
                        </div>
                    </div>
                </div>
            </div>
        `;
    },

    // Stories List View - Cinematic Grid
    storiesList(stories) {
        return `
            <div class="animate-fade-up">
                <!-- Header -->
                <div class="flex justify-between items-end mb-10">
                    <div>
                        <h2 class="font-display text-4xl font-bold text-white mb-2">故事典藏</h2>
                        <p class="font-body text-gray-400 italic">"每一个未诉说的故事，都是一扇等待开启的门"</p>
                        <div class="gold-line w-24 mt-4"></div>
                    </div>
                    <button onclick="app.showModal('createStory')" class="group px-6 py-3 bg-gradient-to-r from-cinema-gold to-cinema-gold-dark rounded-xl font-body font-semibold text-cinema-950 hover:shadow-lg hover:shadow-cinema-gold/20 transition-all duration-300 flex items-center gap-2">
                        <i data-lucide="plus" class="w-5 h-5 group-hover:rotate-90 transition-transform"></i>
                        新建故事
                    </button>
                </div>

                <!-- Cinematic Story Grid -->
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    ${stories.map((s, i) => `
                        <div class="group glass-cinema rounded-2xl p-6 border border-cinema-500/20 hover:border-cinema-gold/40 transition-all duration-500 cursor-pointer relative overflow-hidden animate-fade-up"
                           style="animation-delay: ${i * 0.1}s"
                           onclick="app.selectStory('${s.id}')">
                            <!-- Hover gradient overlay -->
                            <div class="absolute inset-0 bg-gradient-to-br from-cinema-gold/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500"></div>

                            <div class="relative">
                                <!-- Genre badge -->
                                <div class="flex justify-between items-start mb-4">
                                    <span class="font-mono text-xs px-3 py-1 rounded-full bg-cinema-800 text-cinema-gold border border-cinema-gold/20 uppercase tracking-wider">
                                        ${s.genre || '未分类'}
                                    </span>
                                    <div class="w-8 h-8 rounded-full bg-cinema-800 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                                        <i data-lucide="arrow-right" class="w-4 h-4 text-cinema-gold"></i>
                                    </div>
                                </div>

                                <h3 class="font-display text-2xl font-semibold text-white mb-3 group-hover:text-cinema-gold transition-colors">${s.title}</h3>

                                <p class="font-body text-gray-400 text-sm mb-6 line-clamp-2 leading-relaxed">${s.description || '暂无描述'}</p>

                                <div class="flex items-center justify-between pt-4 border-t border-cinema-500/20">
                                    <span class="font-mono text-xs text-gray-500 flex items-center gap-2">
                                        <i data-lucide="calendar" class="w-3 h-3"></i>
                                        ${new Date(s.created_at).toLocaleDateString('zh-CN', { year: 'numeric', month: 'short', day: 'numeric' })}
                                    </span>
                                    <span class="font-mono text-xs text-cinema-gold/60">#${s.id.slice(-4).toUpperCase()}</span>
                                </div>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    // Create Story Modal - Cinematic
    createStoryModal() {
        return `
            <div class="fixed inset-0 bg-cinema-950/90 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in" onclick="if(event.target===this)app.closeModal()">
                <div class="glass-cinema rounded-2xl p-8 w-full max-w-xl border border-cinema-gold/20 animate-fade-up relative overflow-hidden" onclick="event.stopPropagation()">
                    <!-- Decorative elements -->
                    <div class="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-cinema-gold to-transparent"></div>
                    <div class="absolute bottom-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-cinema-gold to-transparent"></div>

                    <h3 class="font-display text-3xl font-bold text-cinema-gold mb-2">新故事开篇</h3>
                    <p class="font-body text-gray-400 italic mb-8">"每一个伟大的故事，都始于勇敢的第一笔"</p>

                    <form id="create-story-form" onsubmit="app.handleCreateStory(event)">
                        <div class="mb-6">
                            <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">故事标题 *</label>
                            <input type="text" name="title" required placeholder="给你的故事一个名字..."
                                class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white placeholder-gray-600 focus:border-cinema-gold/50 focus:outline-none focus:ring-1 focus:ring-cinema-gold/30 transition-all">
                        </div>

                        <div class="grid grid-cols-2 gap-4 mb-6">
                            <div>
                                <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">类型</label>
                                <select name="genre" class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white focus:border-cinema-gold/50 focus:outline-none transition-all appearance-none cursor-pointer">
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
                            <div>
                                <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">基调</label>
                                <select name="tone" class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white focus:border-cinema-gold/50 focus:outline-none transition-all appearance-none cursor-pointer">
                                    <option value="">选择基调</option>
                                    <option value="dark">暗黑</option>
                                    <option value="light">轻松</option>
                                    <option value="serious">严肃</option>
                                    <option value="humorous">幽默</option>
                                    <option value="epic">史诗</option>
                                </select>
                            </div>
                        </div>

                        <div class="mb-8">
                            <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">故事简介</label>
                            <textarea name="description" rows="4" placeholder="用几句话描述你的故事世界..."
                                class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white placeholder-gray-600 focus:border-cinema-gold/50 focus:outline-none focus:ring-1 focus:ring-cinema-gold/30 transition-all resize-none"></textarea>
                        </div>

                        <div class="flex justify-end gap-4">
                            <button type="button" onclick="app.closeModal()" class="px-6 py-3 font-body text-gray-400 hover:text-white transition-colors">
                                取消
                            </button>
                            <button type="submit" class="group px-8 py-3 bg-gradient-to-r from-cinema-gold to-cinema-gold-dark rounded-xl font-body font-semibold text-cinema-950 hover:shadow-lg hover:shadow-cinema-gold/20 transition-all duration-300 flex items-center gap-2">
                                <i data-lucide="sparkles" class="w-4 h-4"></i>
                                开始创作
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        `;
    },

    // Characters View - Cinematic
    characters(characters) {
        return `
            <div class="animate-fade-up">
                <div class="flex justify-between items-end mb-10">
                    <div>
                        <h2 class="font-display text-4xl font-bold text-white mb-2">角色画廊</h2>
                        <p class="font-body text-gray-400 italic">"赋予灵魂以形状，赋予声音以生命"</p>
                        <div class="gold-line w-24 mt-4"></div>
                    </div>
                    <button onclick="app.showModal('createCharacter')" class="group px-6 py-3 bg-gradient-to-r from-cinema-velvet to-purple-800 rounded-xl font-body font-semibold text-white hover:shadow-lg hover:shadow-purple-500/20 transition-all duration-300 flex items-center gap-2">
                        <i data-lucide="plus" class="w-5 h-5 group-hover:rotate-90 transition-transform"></i>
                        创造角色
                    </button>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    ${characters.map((c, i) => `
                        <div class="group glass-cinema rounded-2xl p-6 border border-cinema-500/20 hover:border-cinema-velvet/40 transition-all duration-500 animate-fade-up" style="animation-delay: ${i * 0.1}s">
                            <div class="flex items-start justify-between mb-4">
                                <div class="flex items-center gap-4">
                                    <div class="w-14 h-14 rounded-2xl bg-gradient-to-br from-cinema-velvet to-cinema-gold/30 flex items-center justify-center">
                                        <span class="font-display text-2xl text-white">${c.name.charAt(0)}</span>
                                    </div>
                                    <div>
                                        <h3 class="font-display text-xl font-semibold text-white group-hover:text-cinema-gold transition-colors">${c.name}</h3>
                                        <span class="font-mono text-xs text-cinema-velvet">${c.role || '配角'}</span>
                                    </div>
                                </div>
                                <button onclick="app.editCharacter('${c.id}')" class="w-8 h-8 rounded-lg bg-cinema-800 flex items-center justify-center text-gray-400 hover:text-cinema-gold hover:bg-cinema-700 transition-colors">
                                    <i data-lucide="edit-2" class="w-4 h-4"></i>
                                </button>
                            </div>
                            <p class="font-body text-gray-400 text-sm mb-4 line-clamp-2">${c.background || '暂无背景'}</p>
                            ${c.personality ? `
                                <div class="mb-3 p-3 rounded-xl bg-cinema-900/50 border-l-2 border-cinema-velvet">
                                    <span class="font-mono text-xs text-cinema-velvet uppercase tracking-wider">性格</span>
                                    <p class="font-body text-sm text-gray-300 mt-1">${c.personality}</p>
                                </div>
                            ` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    // Chapters View - Cinematic
    chapters(chapters) {
        return `
            <div class="animate-fade-up">
                <div class="flex justify-between items-end mb-10">
                    <div>
                        <h2 class="font-display text-4xl font-bold text-white mb-2">剧本工坊</h2>
                        <p class="font-body text-gray-400 italic">"一字一句，编织梦想的画卷"</p>
                        <div class="gold-line w-24 mt-4"></div>
                    </div>
                    <button onclick="app.showModal('createChapter')" class="group px-6 py-3 bg-gradient-to-r from-cinema-gold to-cinema-gold-dark rounded-xl font-body font-semibold text-cinema-950 hover:shadow-lg hover:shadow-cinema-gold/20 transition-all duration-300 flex items-center gap-2">
                        <i data-lucide="plus" class="w-5 h-5"></i>
                        新章节
                    </button>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <!-- Chapter List -->
                    <div class="lg:col-span-1 glass-cinema rounded-2xl border border-cinema-500/20 overflow-hidden">
                        <div class="p-5 border-b border-cinema-500/20">
                            <h3 class="font-display text-lg font-semibold text-cinema-gold">章节列表</h3>
                        </div>
                        <div class="max-h-[600px] overflow-y-auto">
                            ${chapters.map((c, i) => `
                                <div class="p-4 border-b border-cinema-500/10 hover:bg-cinema-800/50 cursor-pointer transition-all group" onclick="app.selectChapter('${c.id}')">
                                    <div class="flex items-center justify-between mb-2">
                                        <span class="font-mono text-xs text-gray-500">第 ${i + 1} 章</span>
                                        <span class="font-mono text-xs px-2 py-1 rounded-full ${c.status === 'completed' ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'bg-amber-500/20 text-amber-400 border border-amber-500/30'}">${c.status === 'completed' ? '已完成' : '写作中'}</span>
                                    </div>
                                    <h4 class="font-body font-medium text-gray-300 group-hover:text-cinema-gold transition-colors">${c.title || '未命名章节'}</h4>
                                </div>
                            `).join('')}
                        </div>
                    </div>

                    <!-- Editor -->
                    <div class="lg:col-span-2 glass-cinema rounded-2xl border border-cinema-500/20 p-8">
                        <div id="chapter-editor">
                            <div class="text-center text-gray-500 py-20">
                                <div class="w-20 h-20 rounded-full bg-cinema-800 flex items-center justify-center mx-auto mb-6">
                                    <i data-lucide="pen-tool" class="w-10 h-10 text-cinema-gold/50"></i>
                                </div>
                                <p class="font-body text-lg italic">选择一个章节，开始创作之旅</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;
    },

    // Skills View - Cinematic
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

        const categoryIcons = {
            writing: 'pen-tool',
            analysis: 'bar-chart-2',
            character: 'users',
            plot: 'git-branch',
            style: 'palette',
            export: 'download',
            integration: 'plug',
            custom: 'sparkles'
        };

        return `
            <div class="animate-fade-up">
                <div class="flex justify-between items-end mb-10">
                    <div>
                        <h2 class="font-display text-4xl font-bold text-white mb-2">技能工坊</h2>
                        <p class="font-body text-gray-400 italic">"赋能创作，无限可能"</p>
                        <div class="gold-line w-24 mt-4"></div>
                    </div>
                    <div class="flex gap-3">
                        <button onclick="app.showModal('importSkill')" class="group px-5 py-3 glass-cinema rounded-xl font-body font-medium text-gray-300 border border-cinema-500/30 hover:border-cinema-gold/50 hover:text-cinema-gold transition-all duration-300 flex items-center gap-2">
                            <i data-lucide="download" class="w-4 h-4"></i>
                            导入技能
                        </button>
                        <button onclick="app.navigate('mcp')" class="group px-5 py-3 bg-gradient-to-r from-cinema-gold to-cinema-gold-dark rounded-xl font-body font-semibold text-cinema-950 hover:shadow-lg hover:shadow-cinema-gold/20 transition-all duration-300 flex items-center gap-2">
                            <i data-lucide="plug" class="w-4 h-4"></i>
                            MCP 配置
                        </button>
                    </div>
                </div>

                <!-- Category Tabs -->
                <div class="flex gap-2 mb-8 overflow-x-auto pb-2">
                    <button onclick="app.filterSkills('all')" class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-cinema-gold to-cinema-gold-dark text-cinema-950 font-semibold whitespace-nowrap shadow-lg shadow-cinema-gold/10">全部</button>
                    ${Object.entries(categories).map(([key, label]) => `
                        <button onclick="app.filterSkills('${key}')" class="px-5 py-2.5 rounded-xl glass-cinema text-gray-400 border border-cinema-500/20 hover:border-cinema-gold/30 hover:text-cinema-gold transition-all whitespace-nowrap">${label}</button>
                    `).join('')}
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    ${skills.map((s, i) => `
                        <div class="group glass-cinema rounded-2xl p-6 border border-cinema-500/20 hover:border-cinema-gold/30 transition-all duration-500 ${s.is_enabled ? '' : 'opacity-60'} animate-fade-up" style="animation-delay: ${i * 0.05}s">
                            <div class="flex items-start justify-between mb-4">
                                <div class="flex items-center gap-3">
                                    <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-cinema-gold/20 to-cinema-velvet/20 flex items-center justify-center border border-cinema-gold/10 group-hover:border-cinema-gold/30 transition-colors">
                                        <i data-lucide="${categoryIcons[s.manifest.category] || 'zap'}" class="w-5 h-5 text-cinema-gold"></i>
                                    </div>
                                    <div>
                                        <h3 class="font-display text-lg font-semibold text-white group-hover:text-cinema-gold transition-colors">${s.manifest.name}</h3>
                                        <span class="font-mono text-xs text-cinema-velvet">${categories[s.manifest.category] || s.manifest.category}</span>
                                    </div>
                                </div>
                                <label class="relative inline-flex items-center cursor-pointer">
                                    <input type="checkbox" ${s.is_enabled ? 'checked' : ''} onchange="app.toggleSkill('${s.manifest.id}', this.checked)" class="sr-only peer">
                                    <div class="w-11 h-6 bg-cinema-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-cinema-gold"></div>
                                </label>
                            </div>
                            <p class="font-body text-gray-400 text-sm mb-4 line-clamp-2">${s.manifest.description}</p>
                            <div class="flex items-center justify-between font-mono text-xs text-gray-500">
                                <span class="px-2 py-1 rounded bg-cinema-800">v${s.manifest.version}</span>
                                <span>${s.manifest.author}</span>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    }

    // MCP Configuration View - Cinematic
    mcpConfig(servers) {
        return `
            <div class="animate-fade-up">
                <div class="flex justify-between items-end mb-10">
                    <div>
                        <h2 class="font-display text-4xl font-bold text-white mb-2">外部连接</h2>
                        <p class="font-body text-gray-400 italic">"连接世界的每一个角落"</p>
                        <div class="gold-line w-24 mt-4"></div>
                    </div>
                    <button onclick="app.showModal('addMcpServer')" class="group px-6 py-3 bg-gradient-to-r from-cinema-gold to-cinema-gold-dark rounded-xl font-body font-semibold text-cinema-950 hover:shadow-lg hover:shadow-cinema-gold/20 transition-all duration-300 flex items-center gap-2">
                        <i data-lucide="plus" class="w-5 h-5"></i>
                        添加服务器
                    </button>
                </div>

                <div class="glass-cinema rounded-2xl border border-cinema-gold/10 p-8 mb-8">
                    <h3 class="font-display text-xl font-semibold mb-3 text-cinema-gold">关于 MCP</h3>
                    <p class="font-body text-gray-400">Model Context Protocol (MCP) 允许你连接外部工具和服务到 CINEMA-AI。配置 MCP 服务器后，你可以在技能系统中使用这些外部能力。</p>
                </div>

                <div class="space-y-4">
                    ${servers.map((s, i) => `
                        <div class="glass-cinema rounded-2xl p-6 border border-cinema-500/20 hover:border-cinema-gold/20 transition-all animate-fade-up" style="animation-delay: ${i * 0.1}s">
                            <div class="flex items-start justify-between">
                                <div class="flex items-center gap-4">
                                    <div class="w-14 h-14 rounded-2xl bg-gradient-to-br from-cinema-gold/20 to-cinema-amber/20 flex items-center justify-center border border-cinema-gold/10">
                                        <i data-lucide="server" class="w-7 h-7 text-cinema-gold"></i>
                                    </div>
                                    <div>
                                        <h3 class="font-display text-xl font-semibold text-white">${s.name}</h3>
                                        <p class="font-mono text-sm text-gray-500">${s.command} ${s.args.join(' ')}</p>
                                    </div>
                                </div>
                                <div class="flex gap-2">
                                    <button onclick="app.testMcpServer('${s.id}')" class="w-10 h-10 rounded-xl bg-cinema-800 flex items-center justify-center text-gray-400 hover:text-cinema-gold hover:bg-cinema-700 transition-colors" title="测试连接">
                                        <i data-lucide="activity" class="w-5 h-5"></i>
                                    </button>
                                    <button onclick="app.deleteMcpServer('${s.id}')" class="w-10 h-10 rounded-xl bg-cinema-800 flex items-center justify-center text-gray-400 hover:text-red-400 hover:bg-red-500/10 transition-colors" title="删除">
                                        <i data-lucide="trash-2" class="w-5 h-5"></i>
                                    </button>
                                </div>
                            </div>
                            ${s.tools?.length ? `
                                <div class="mt-4 pt-4 border-t border-cinema-500/20">
                                    <span class="font-mono text-xs text-cinema-gold uppercase tracking-wider">可用工具</span>
                                    <div class="flex flex-wrap gap-2 mt-2">
                                        ${s.tools.map(t => `<span class="px-3 py-1 rounded-full bg-cinema-800 text-xs text-gray-400 border border-cinema-500/20">${t.name}</span>`).join('')}
                                    </div>
                                </div>
                            ` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    // Settings View - Cinematic
    settings(config) {
        return `
            <div class="animate-fade-up">
                <div class="mb-10">
                    <h2 class="font-display text-4xl font-bold text-white mb-2">工作室配置</h2>
                    <p class="font-body text-gray-400 italic">"精心调校您的创作环境"</p>
                    <div class="gold-line w-24 mt-4"></div>
                </div>

                <div class="max-w-2xl space-y-8">
                    <!-- LLM Settings -->
                    <div class="glass-cinema rounded-2xl p-8 border border-cinema-gold/10">
                        <h3 class="font-display text-2xl font-semibold mb-6 text-cinema-gold flex items-center gap-3">
                            <div class="w-10 h-10 rounded-xl bg-cinema-gold/10 flex items-center justify-center">
                                <i data-lucide="brain" class="w-5 h-5"></i>
                            </div>
                            AI 引擎配置
                        </h3>
                        <form onsubmit="app.saveSettings(event)">
                            <div class="mb-6">
                                <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">提供商</label>
                                <select name="provider" class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white focus:border-cinema-gold/50 focus:outline-none transition-all">
                                    <option value="openai" ${config?.provider === 'openai' ? 'selected' : ''}>OpenAI</option>
                                    <option value="anthropic" ${config?.provider === 'anthropic' ? 'selected' : ''}>Anthropic</option>
                                    <option value="ollama" ${config?.provider === 'ollama' ? 'selected' : ''}>Ollama (本地)</option>
                                </select>
                            </div>

                            <div class="mb-6">
                                <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">API Key</label>
                                <input type="password" name="api_key" value="${config?.api_key || ''}" placeholder="sk-..."
                                    class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white placeholder-gray-600 focus:border-cinema-gold/50 focus:outline-none transition-all">
                            </div>

                            <div class="mb-6">
                                <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">模型</label>
                                <select name="model" class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white focus:border-cinema-gold/50 focus:outline-none transition-all">
                                    <option value="gpt-4" ${config?.model === 'gpt-4' ? 'selected' : ''}>GPT-4</option>
                                    <option value="gpt-4-turbo" ${config?.model === 'gpt-4-turbo' ? 'selected' : ''}>GPT-4 Turbo</option>
                                    <option value="claude-3-opus" ${config?.model === 'claude-3-opus' ? 'selected' : ''}>Claude 3 Opus</option>
                                    <option value="claude-3-sonnet" ${config?.model === 'claude-3-sonnet' ? 'selected' : ''}>Claude 3 Sonnet</option>
                                </select>
                            </div>

                            <div class="grid grid-cols-2 gap-4 mb-8">
                                <div>
                                    <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">Temperature</label>
                                    <input type="number" name="temperature" value="${config?.temperature || 0.7}" min="0" max="2" step="0.1"
                                        class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white focus:border-cinema-gold/50 focus:outline-none transition-all">
                                </div>
                                <div>
                                    <label class="font-mono text-xs text-cinema-gold uppercase tracking-wider mb-2 block">Max Tokens</label>
                                    <input type="number" name="max_tokens" value="${config?.max_tokens || 4096}"
                                        class="w-full bg-cinema-900/50 border border-cinema-500/30 rounded-xl px-5 py-3 font-body text-white focus:border-cinema-gold/50 focus:outline-none transition-all">
                                </div>
                            </div>

                            <button type="submit" class="w-full group px-8 py-4 bg-gradient-to-r from-cinema-gold to-cinema-gold-dark rounded-xl font-body font-semibold text-cinema-950 hover:shadow-lg hover:shadow-cinema-gold/20 transition-all duration-300 flex items-center justify-center gap-2">
                                <i data-lucide="save" class="w-5 h-5"></i>
                                保存配置
                            </button>
                        </form>
                    </div>
                </div>
            </div>
        `;
    },

    // Cinematic Sidebar
    sidebar(currentView) {
        const items = [
            { id: 'dashboard', icon: 'layout-dashboard', label: '创作大厅' },
            { id: 'stories', icon: 'book-open', label: '故事管理' },
            { id: 'characters', icon: 'users', label: '角色管理' },
            { id: 'chapters', icon: 'file-text', label: '章节创作' },
            { id: 'skills', icon: 'zap', label: '技能管理' },
            { id: 'settings', icon: 'settings', label: '系统设置' },
        ];

        return `
            <aside class="w-72 glass-cinema border-r border-cinema-gold/10 flex flex-col relative overflow-hidden">
                <!-- Decorative gradient -->
                <div class="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-cinema-gold to-transparent opacity-50"></div>

                <div class="p-8 border-b border-cinema-500/20">
                    <div class="flex items-center gap-3 mb-2">
                        <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-cinema-gold to-cinema-gold-dark flex items-center justify-center shadow-lg shadow-cinema-gold/20">
                            <i data-lucide="film" class="w-5 h-5 text-cinema-950"></i>
                        </div>
                        <div>
                            <h1 class="font-display text-xl font-bold text-cinema-gold tracking-wider">CINEMA</h1>
                            <span class="font-mono text-xs text-gray-500">AI STUDIO</span>
                        </div>
                    </div>
                </div>

                <nav class="flex-1 p-6 space-y-2">
                    <div class="font-mono text-xs text-gray-500 uppercase tracking-widest mb-4 px-4">导航</div>
                    ${items.map((item, i) => `
                        <button onclick="app.navigate('${item.id}')"
                            class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 group ${currentView === item.id ? 'bg-gradient-to-r from-cinema-gold/20 to-transparent text-cinema-gold border-l-2 border-cinema-gold' : 'text-gray-400 hover:bg-cinema-800/50 hover:text-gray-200'}"
                            style="animation-delay: ${i * 0.05}s">
                            <div class="w-8 h-8 rounded-lg ${currentView === item.id ? 'bg-cinema-gold/20' : 'bg-cinema-800 group-hover:bg-cinema-700'} flex items-center justify-center transition-colors">
                                <i data-lucide="${item.icon}" class="w-4 h-4"></i>
                            </div>
                            <span class="font-body text-sm">${item.label}</span>
                            ${currentView === item.id ? '<div class="ml-auto w-1.5 h-1.5 rounded-full bg-cinema-gold"></div>' : ''}
                        </button>
                    `).join('')}
                </nav>

                <div class="p-6 border-t border-cinema-500/20">
                    <div class="font-mono text-xs text-gray-500 uppercase tracking-widest mb-3">当前项目</div>
                    <div class="glass-cinema rounded-xl p-4 border border-cinema-gold/10 hover:border-cinema-gold/30 transition-colors cursor-pointer group">
                        <div class="flex items-center gap-3">
                            <div class="w-2 h-2 rounded-full bg-cinema-gold animate-pulse"></div>
                            <div class="font-body text-sm text-gray-300 truncate group-hover:text-cinema-gold transition-colors" id="current-story-name">未选择故事</div>
                        </div>
                    </div>
                </div>
            </aside>
        `;
    },

    // Cinematic Toast Notification
    toast(message, type = 'info') {
        const styles = {
            info: { bg: 'bg-cinema-850', border: 'border-cinema-gold/30', icon: 'info', color: 'text-cinema-gold' },
            success: { bg: 'bg-cinema-850', border: 'border-green-500/30', icon: 'check-circle', color: 'text-green-400' },
            warning: { bg: 'bg-cinema-850', border: 'border-amber-500/30', icon: 'alert-triangle', color: 'text-amber-400' },
            error: { bg: 'bg-cinema-850', border: 'border-red-500/30', icon: 'x-circle', color: 'text-red-400' }
        };

        const style = styles[type];
        const id = 'toast-' + Date.now();
        const el = document.createElement('div');
        el.id = id;
        el.className = `${style.bg} border ${style.border} rounded-xl shadow-2xl shadow-black/50 flex items-center gap-4 px-6 py-4 min-w-[300px] animate-slide-left`;
        el.innerHTML = `
            <i data-lucide="${style.icon}" class="w-5 h-5 ${style.color}"></i>
            <span class="font-body text-gray-200">${message}</span>
        `;

        document.getElementById('toast-container').appendChild(el);
        lucide.createIcons();

        setTimeout(() => {
            el.style.opacity = '0';
            el.style.transform = 'translateX(100%)';
            el.style.transition = 'all 0.4s cubic-bezier(0.4, 0, 0.2, 1)';
            setTimeout(() => el.remove(), 400);
        }, 4000);
    }
};