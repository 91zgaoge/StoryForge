/**
 * FrontStage 类型定义
 */

export interface HintPosition {
  line: number;
  column: number;
  offset: number;
}

export interface AiHint {
  id: string;
  text: string;
  position: HintPosition;
  duration: number;
  isPreview?: boolean;
}

export interface FrontstageEvent {
  type: 'ContentUpdate' | 'AiHint' | 'AiPreview' | 'ChapterSwitch' | 'SaveStatus';
  payload: any;
}

export interface BackstageEvent {
  type: 'ContentChanged' | 'GenerationRequested' | 'FrontstageClosed' | 'FrontstageFocused';
  payload: any;
}

export interface ChapterInfo {
  id: string;
  title: string;
  storyTitle?: string;
}