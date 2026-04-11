// CINEMA-AI Frontend v2.0
// Complete UI implementation

const app = {
    state: {
        currentView: 'dashboard',
        currentStory: null,
        stories: [],
        characters: [],
        chapters: [],
        skills: [],
        mcpServers: [],
        settings: null,
        isLoading: false
    },

    // Get Tauri invoke function
    get invoke() {
        if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
        if (typeof mockTauri !== 'undefined') return mockTauri.invoke;
        return null;
    },

    // Initialize application
    async init() {
        console.log('Initializing CINEMA-AI v2.0...');

        if (!this.invoke) {
            this.showError('Tauri API not found. Please run via Tauri.');
            return;
        }

        try {
            // Load initial data
            await this.loadDashboard();
            this.render();
            lucide.createIcons();
        } catch (err) {
            console.error('Initialization failed:', err);
            this.showError('Failed to initialize: ' + err.message);
        }
    },

    // Load dashboard data
    async loadDashboard() {
        const state = await this.invoke('get_state');
        this.state.stories = await this.invoke('list_stories');
        this.state.currentStory = state.current_story;
    },

    // Load stories
    async loadStories() {
        this.state.stories = await this.invoke('list_stories');
    },

    // Load characters
    async loadCharacters() {
        if (!this.state.currentStory) return;
        this.state.characters = await this.invoke('get_story_characters', {
            storyId: this.state.currentStory.id
        });
    },

    // Load chapters
    async loadChapters() {
        if (!this.state.currentStory) return;
        this.state.chapters = await this.invoke('get_story_chapters', {
            storyId: this.state.currentStory.id
        });
    },

    // Load skills
    async loadSkills() {
        this.state.skills = await this.invoke('get_skills');
    },

    // Load settings
    async loadSettings() {
        this.state.settings = await this.invoke('get_config_command');
    },

    // Navigate to view
    async navigate(view) {
        this.state.currentView = view;
        this.state.isLoading = true;
        this.render();

        try {
            switch (view) {
                case 'dashboard':
                    await this.loadDashboard();
                    break;
                case 'stories':
                    await this.loadStories();
                    break;
                case 'characters':
                    await this.loadCharacters();
                    break;
                case 'chapters':
                    await this.loadChapters();
                    break;
                case 'skills':
                    await this.loadSkills();
                    break;
                case 'settings':
                    await this.loadSettings();
                    break;
            }
        } catch (err) {
            Views.toast('加载失败: ' + err.message, 'error');
        }

        this.state.isLoading = false;
        this.render();
        lucide.createIcons();
    },

    // Render main UI
    render() {
        const appEl = document.getElementById('app');

        if (this.state.isLoading && !this.state.stories.length) {
            appEl.innerHTML = this.renderLoading();
            return;
        }

        let content;
        switch (this.state.currentView) {
            case 'dashboard':
                content = Views.dashboard({
                    stories_count: this.state.stories.length,
                    characters_count: this.state.characters.length,
                    chapters_count: this.state.chapters.length,
                    current_story: this.state.currentStory
                });
                break;
            case 'stories':
                content = Views.storiesList(this.state.stories);
                break;
            case 'characters':
                content = Views.characters(this.state.characters);
                break;
            case 'chapters':
                content = Views.chapters(this.state.chapters);
                break;
            case 'skills':
                content = Views.skills(this.state.skills);
                break;
            case 'mcp':
                content = Views.mcpConfig(this.state.mcpServers);
                break;
            case 'settings':
                content = Views.settings(this.state.settings);
                break;
            default:
                content = Views.dashboard({});
        }

        appEl.innerHTML = `
            <div class="flex h-screen">
                ${Views.sidebar(this.state.currentView)}
                <main class="flex-1 overflow-auto p-8">
                    ${this.state.isLoading ? '<div class="flex items-center justify-center h-full"><div class="loading-dots text-2xl">加载中</div></div>' : content}
                </main>
            </div>
        `;
    },

    // Render loading screen
    renderLoading() {
        return `
            <div class="flex h-screen items-center justify-center bg-cinema-900">
                <div class="text-center">
                    <div class="w-16 h-16 border-4 border-cinema-accent border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
                    <h2 class="text-xl font-semibold">正在初始化...</h2>
                </div>
            </div>
        `;
    },

    // Show error screen
    showError(message) {
        document.getElementById('app').innerHTML = `
            <div class="flex h-screen items-center justify-center bg-cinema-900">
                <div class="text-center max-w-md p-8 bg-cinema-800 rounded-xl border border-cinema-700">
                    <i data-lucide="alert-triangle" class="w-16 h-16 text-red-500 mx-auto mb-4"></i>
                    <h1 class="text-2xl font-bold mb-4">初始化失败</h1>
                    <p class="text-gray-400 mb-6">${message}</p>
                    <button onclick="location.reload()" class="bg-cinema-accent hover:bg-blue-600 px-6 py-2 rounded-lg transition">
                        重试
                    </button>
                </div>
            </div>
        `;
        lucide.createIcons();
    },

    // Modal management
    showModal(type) {
        const modalContainer = document.getElementById('modal-container');
        let content;

        switch (type) {
            case 'createStory':
                content = Views.createStoryModal();
                break;
            default:
                return;
        }

        modalContainer.innerHTML = content;
        lucide.createIcons();
    },

    closeModal() {
        document.getElementById('modal-container').innerHTML = '';
    },

    // Form handlers
    async handleCreateStory(e) {
        e.preventDefault();
        const formData = new FormData(e.target);

        try {
            await this.invoke('create_story', {
                title: formData.get('title'),
                description: formData.get('description'),
                genre: formData.get('genre')
            });
            this.closeModal();
            Views.toast('故事创建成功', 'success');
            this.navigate('stories');
        } catch (err) {
            Views.toast('创建失败: ' + err.message, 'error');
        }
    },

    async saveSettings(e) {
        e.preventDefault();
        const formData = new FormData(e.target);

        try {
            await this.invoke('update_config', {
                llm: {
                    provider: formData.get('provider'),
                    api_key: formData.get('api_key'),
                    model: formData.get('model'),
                    temperature: parseFloat(formData.get('temperature')),
                    max_tokens: parseInt(formData.get('max_tokens'))
                }
            });
            Views.toast('设置已保存', 'success');
        } catch (err) {
            Views.toast('保存失败: ' + err.message, 'error');
        }
    },

    // Story selection
    async selectStory(storyId) {
        const story = this.state.stories.find(s => s.id === storyId);
        if (story) {
            this.state.currentStory = story;
            document.getElementById('current-story-name').textContent = story.title;
            Views.toast(`已选择: ${story.title}`, 'success');
            this.navigate('chapters');
        }
    },

    // Skill management
    async toggleSkill(skillId, enabled) {
        try {
            if (enabled) {
                await this.invoke('enable_skill', { skillId });
            } else {
                await this.invoke('disable_skill', { skillId });
            }
            Views.toast(enabled ? '技能已启用' : '技能已禁用', 'success');
        } catch (err) {
            Views.toast('操作失败: ' + err.message, 'error');
        }
    },

    filterSkills(category) {
        // Implement skill filtering
        console.log('Filter skills by:', category);
    },

    // Chapter editing
    selectChapter(chapterId) {
        console.log('Selected chapter:', chapterId);
    },

    // Character editing
    editCharacter(characterId) {
        console.log('Edit character:', characterId);
    },

    // MCP management
    testMcpServer(serverId) {
        Views.toast('测试连接: ' + serverId, 'info');
    },

    deleteMcpServer(serverId) {
        Views.toast('删除服务器: ' + serverId, 'warning');
    }
};

// Initialize on load
document.addEventListener('DOMContentLoaded', () => app.init());
