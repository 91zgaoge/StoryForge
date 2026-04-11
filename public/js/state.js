// CINEMA-AI State Management (Phase 4)
const AppState = {
  data: {
    currentStory: null,
    stories: [],
    chapters: [],
    characters: [],
    config: null,
    isLoading: false,
  },
  
  listeners: {},
  
  subscribe(key, callback) {
    if (!this.listeners[key]) this.listeners[key] = [];
    this.listeners[key].push(callback);
    return () => {
      this.listeners[key] = this.listeners[key].filter(cb => cb !== callback);
    };
  },
  
  set(key, value) {
    this.data[key] = value;
    if (this.listeners[key]) {
      this.listeners[key].forEach(cb => cb(value));
    }
  },
  
  get(key) {
    return this.data[key];
  },
  
  // API Wrappers
  async loadStories() {
    this.set('isLoading', true);
    try {
      const stories = await invoke('list_stories');
      this.set('stories', stories);
      return stories;
    } finally {
      this.set('isLoading', false);
    }
  },
  
  async loadChapters(storyId) {
    if (!storyId) return [];
    const chapters = await invoke('get_story_chapters', { storyId });
    this.set('chapters', chapters);
    return chapters;
  },
  
  async loadCharacters(storyId) {
    if (!storyId) return [];
    const characters = await invoke('get_story_characters', { storyId });
    this.set('characters', characters);
    return characters;
  },
  
  async switchStory(id) {
    const stories = this.get('stories');
    const story = stories.find(s => s.id === id);
    if (story) {
      this.set('currentStory', story);
      await Promise.all([
        this.loadChapters(id),
        this.loadCharacters(id)
      ]);
    }
  }
};

// Component Registry
const Components = {
  registry: {},
  
  register(name, renderFn) {
    this.registry[name] = renderFn;
  },
  
  render(name, props = {}) {
    const fn = this.registry[name];
    return fn ? fn(props) : '';
  }
};

// Toast notification system
const Toast = {
  show(message, type = 'info', duration = 3000) {
    const colors = {
      success: 'bg-green-600',
      error: 'bg-red-600',
      info: 'bg-blue-600',
      warning: 'bg-yellow-600'
    };
    
    const toast = document.createElement('div');
    toast.className = `fixed top-4 right-4 ${colors[type] || colors.info} text-white px-4 py-2 rounded shadow-lg z-50 transition-all duration-300 transform translate-x-full`;
    toast.innerHTML = `
      <div class="flex items-center gap-2">
        ${type === 'success' ? '✓' : type === 'error' ? '✕' : 'ℹ'} ${message}
      </div>
    `;
    document.body.appendChild(toast);
    
    // Animate in
    requestAnimationFrame(() => {
      toast.classList.remove('translate-x-full');
    });
    
    setTimeout(() => {
      toast.classList.add('opacity-0', 'translate-x-full');
      setTimeout(() => toast.remove(), 300);
    }, duration);
  }
};

// Modal system
const Modal = {
  open(content, options = {}) {
    const backdrop = document.createElement('div');
    backdrop.className = 'fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4';
    backdrop.id = 'modal-backdrop';
    backdrop.innerHTML = `
      <div class="bg-gray-800 rounded-lg max-w-2xl w-full max-h-[90vh] overflow-auto">
        ${content}
      </div>
    `;
    
    backdrop.addEventListener('click', (e) => {
      if (e.target === backdrop && options.closeOnBackdrop !== false) {
        this.close();
      }
    });
    
    document.body.appendChild(backdrop);
    document.body.style.overflow = 'hidden';
  },
  
  close() {
    const backdrop = document.getElementById('modal-backdrop');
    if (backdrop) {
      backdrop.remove();
      document.body.style.overflow = '';
    }
  }
};

// Export for global access
window.AppState = AppState;
window.Components = Components;
window.Toast = Toast;
window.Modal = Modal;
