/**
 * FrontStage 状态管理
 */

import { create } from 'zustand';
import type { AiHint, ChapterInfo } from '../types';

interface FrontstageState {
  // Content
  content: string;
  chapterId: string | null;
  chapterTitle: string | null;
  storyTitle: string | null;
  
  // AI Hints
  aiHints: AiHint[];
  
  // Status
  isSaved: boolean;
  lastSavedAt: string | null;
  isGenerating: boolean;
  
  // Actions
  setContent: (content: string) => void;
  setChapterInfo: (id: string, title: string, storyTitle?: string) => void;
  addAiHint: (hint: AiHint) => void;
  removeAiHint: (id: string) => void;
  clearAiHints: () => void;
  setSaveStatus: (saved: boolean, timestamp?: string | null) => void;
  setGenerating: (generating: boolean) => void;
}

export const useFrontstageStore = create<FrontstageState>((set) => ({
  // Initial state
  content: '',
  chapterId: null,
  chapterTitle: null,
  storyTitle: null,
  aiHints: [],
  isSaved: true,
  lastSavedAt: null,
  isGenerating: false,
  
  // Actions
  setContent: (content) => set({ content, isSaved: false }),
  
  setChapterInfo: (id, title, storyTitle) => set({
    chapterId: id,
    chapterTitle: title,
    storyTitle: storyTitle || null,
  }),
  
  addAiHint: (hint) => set((state) => ({
    aiHints: [...state.aiHints, hint],
  })),
  
  removeAiHint: (id) => set((state) => ({
    aiHints: state.aiHints.filter((h) => h.id !== id),
  })),
  
  clearAiHints: () => set({ aiHints: [] }),
  
  setSaveStatus: (saved, timestamp) => set({
    isSaved: saved,
    lastSavedAt: timestamp || null,
  }),
  
  setGenerating: (generating) => set({ isGenerating: generating }),
}));