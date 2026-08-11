export interface MethodologyStep {
  title: string;
  instruction: string;
  checklist: string[];
}

export interface Technique {
  name: string;
  when_to_use: string;
  how: string;
}

export interface AntiPattern {
  what: string;
  why: string;
}

export interface Cheatsheet {
  decision_rules: string[];
  anti_patterns: AntiPattern[];
}

export interface CustomMethodology {
  id: string;
  guidebook_id: string | null;
  name: string;
  description: string | null;
  steps: MethodologyStep[];
  enabled: boolean;
  patterns: Technique[];
  cheatsheet: Cheatsheet;
  created_at: string;
  updated_at: string;
}

export interface Guidebook {
  id: string;
  title: string;
  author: string | null;
  subject: string | null;
  word_count: number | null;
  file_format: string | null;
  file_hash: string | null;
  file_path: string | null;
  methodology_id: string | null;
  status: string;
  progress: number;
  error: string | null;
  task_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface GuidebookListItem {
  id: string;
  title: string;
  author: string | null;
  subject: string | null;
  word_count: number | null;
  file_format: string | null;
  methodology_id: string | null;
  merge_into_methodology_id: string | null;
  status: string;
  progress: number;
  created_at: string;
}

export interface GuidebookStatusResponse {
  guidebook_id: string;
  status: string;
  progress: number;
  current_step: string | null;
  error: string | null;
}

export interface GuidebookResult {
  guidebook: Guidebook;
  methodology: CustomMethodology | null;
}

export interface DistillationProgressEvent {
  guidebook_id: string;
  status: string;
  progress: number;
  current_step: string;
  message: string | null;
  active_threads?: number;
}

export interface MethodologyInfo {
  id: string;
  name: string;
  description: string;
  max_steps: number;
  is_custom: boolean;
  source_book: string | null;
  enabled: boolean;
}
