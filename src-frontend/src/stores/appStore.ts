import { create } from 'zustand';
import type { Story, Character, Chapter, Skill, ViewType } from '@/types/index';

interface AppState {
  // Navigation
  currentView: ViewType;
  setCurrentView: (view: ViewType) => void;
  
  // Current Story Context
  currentStory: Story | null;
  setCurrentStory: (story: Story | null) => void;
  
  // Data
  stories: Story[];
  setStories: (stories: Story[]) => void;
  addStory: (story: Story) => void;
  updateStoryInList: (story: Story) => void;
  removeStory: (id: string) => void;
  
  characters: Character[];
  setCharacters: (characters: Character[]) => void;
  addCharacter: (character: Character) => void;
  updateCharacterInList: (character: Character) => void;
  removeCharacter: (id: string) => void;
  
  chapters: Chapter[];
  setChapters: (chapters: Chapter[]) => void;
  addChapter: (chapter: Chapter) => void;
  updateChapterInList: (chapter: Chapter) => void;
  removeChapter: (id: string) => void;
  
  skills: Skill[];
  setSkills: (skills: Skill[]) => void;
  updateSkill: (skill: Skill) => void;
  
  // Loading States
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
  
  // Error
  error: string | null;
  setError: (error: string | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Navigation
  currentView: 'dashboard',
  setCurrentView: (view) => set({ currentView: view }),
  
  // Current Story
  currentStory: null,
  setCurrentStory: (story) => set({ currentStory: story }),
  
  // Stories
  stories: [],
  setStories: (stories) => set({ stories }),
  addStory: (story) => set((state) => ({ 
    stories: [...state.stories, story] 
  })),
  updateStoryInList: (story) => set((state) => ({
    stories: state.stories.map((s) => s.id === story.id ? story : s)
  })),
  removeStory: (id) => set((state) => ({
    stories: state.stories.filter((s) => s.id !== id),
    currentStory: state.currentStory?.id === id ? null : state.currentStory,
  })),
  
  // Characters
  characters: [],
  setCharacters: (characters) => set({ characters }),
  addCharacter: (character) => set((state) => ({
    characters: [...state.characters, character]
  })),
  updateCharacterInList: (character) => set((state) => ({
    characters: state.characters.map((c) => c.id === character.id ? character : c)
  })),
  removeCharacter: (id) => set((state) => ({
    characters: state.characters.filter((c) => c.id !== id)
  })),
  
  // Chapters
  chapters: [],
  setChapters: (chapters) => set({ chapters }),
  addChapter: (chapter) => set((state) => ({
    chapters: [...state.chapters, chapter]
  })),
  updateChapterInList: (chapter) => set((state) => ({
    chapters: state.chapters.map((c) => c.id === chapter.id ? chapter : c)
  })),
  removeChapter: (id) => set((state) => ({
    chapters: state.chapters.filter((c) => c.id !== id)
  })),
  
  // Skills
  skills: [],
  setSkills: (skills) => set({ skills }),
  updateSkill: (skill) => set((state) => ({
    skills: state.skills.map((s) => s.id === skill.id ? skill : s)
  })),
  
  // Loading
  isLoading: false,
  setIsLoading: (loading) => set({ isLoading: loading }),
  
  // Error
  error: null,
  setError: (error) => set({ error }),
}));
