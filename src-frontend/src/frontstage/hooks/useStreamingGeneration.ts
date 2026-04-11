/**
 * useStreamingGeneration - AI 流式生成 Hook
 * 
 * 功能：
 * - 管理 AI 生成状态
 * - 逐字流式输出效果
 * - 接受/拒绝/重新生成功能
 * - 暂停/继续控制
 */

import { useState, useCallback, useRef, useEffect } from 'react';

export type GenerationState = 'idle' | 'generating' | 'paused' | 'completed' | 'accepted' | 'rejected';

interface UseStreamingGenerationOptions {
  /** 打字速度（每字间隔 ms） */
  typingSpeed?: { min: number; max: number };
  /** 生成完成后的回调 */
  onComplete?: (text: string) => void;
  /** 接受生成时的回调 */
  onAccept?: (text: string) => void;
  /** 拒绝生成时的回调 */
  onReject?: () => void;
}

interface UseStreamingGenerationReturn {
  /** 当前生成状态 */
  state: GenerationState;
  /** 已生成的文本 */
  generatedText: string;
  /** 是否正在生成中 */
  isGenerating: boolean;
  /** 是否已暂停 */
  isPaused: boolean;
  /** 生成进度 (0-100) */
  progress: number;
  /** 开始生成 */
  startGeneration: (fullText: string) => void;
  /** 暂停生成 */
  pauseGeneration: () => void;
  /** 继续生成 */
  resumeGeneration: () => void;
  /** 接受生成 */
  acceptGeneration: () => void;
  /** 拒绝生成 */
  rejectGeneration: () => void;
  /** 重新生成 */
  restartGeneration: (fullText: string) => void;
  /** 清除生成 */
  clearGeneration: () => void;
}

export function useStreamingGeneration(
  options: UseStreamingGenerationOptions = {}
): UseStreamingGenerationReturn {
  const {
    typingSpeed = { min: 30, max: 80 },
    onComplete,
    onAccept,
    onReject,
  } = options;

  const [state, setState] = useState<GenerationState>('idle');
  const [generatedText, setGeneratedText] = useState('');
  const [progress, setProgress] = useState(0);
  
  const fullTextRef = useRef('');
  const currentIndexRef = useRef(0);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isPausedRef = useRef(false);

  const clearTimeoutSafe = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  const typeNextChar = useCallback(() => {
    if (isPausedRef.current) return;

    const fullText = fullTextRef.current;
    const currentIndex = currentIndexRef.current;

    if (currentIndex >= fullText.length) {
      setState('completed');
      onComplete?.(fullText);
      return;
    }

    // 获取下一个字符（处理 Unicode）
    const nextChar = fullText[currentIndex];
    const newText = fullText.slice(0, currentIndex + 1);
    
    setGeneratedText(newText);
    currentIndexRef.current = currentIndex + 1;
    setProgress(Math.round((currentIndex + 1) / fullText.length * 100));

    // 计算下一个字符的延迟（随机模拟真实打字）
    // 标点符号后延迟更长，模拟思考
    const isPunctuation = /[。，！？.!?;；]/.test(nextChar);
    const baseDelay = isPunctuation 
      ? typingSpeed.max * 2 
      : typingSpeed.min + Math.random() * (typingSpeed.max - typingSpeed.min);
    
    // 偶尔添加额外停顿（模拟 AI 思考）
    const thinkPause = Math.random() > 0.95 ? 200 : 0;
    const delay = baseDelay + thinkPause;

    timeoutRef.current = setTimeout(typeNextChar, delay);
  }, [typingSpeed, onComplete]);

  const startGeneration = useCallback((fullText: string) => {
    clearTimeoutSafe();
    fullTextRef.current = fullText;
    currentIndexRef.current = 0;
    isPausedRef.current = false;
    setGeneratedText('');
    setProgress(0);
    setState('generating');
    
    // 稍微延迟后开始，给用户准备时间
    timeoutRef.current = setTimeout(typeNextChar, 300);
  }, [clearTimeoutSafe, typeNextChar]);

  const pauseGeneration = useCallback(() => {
    if (state === 'generating') {
      isPausedRef.current = true;
      clearTimeoutSafe();
      setState('paused');
    }
  }, [state, clearTimeoutSafe]);

  const resumeGeneration = useCallback(() => {
    if (state === 'paused') {
      isPausedRef.current = false;
      setState('generating');
      typeNextChar();
    }
  }, [state, typeNextChar]);

  const acceptGeneration = useCallback(() => {
    clearTimeoutSafe();
    setState('accepted');
    onAccept?.(generatedText);
  }, [clearTimeoutSafe, generatedText, onAccept]);

  const rejectGeneration = useCallback(() => {
    clearTimeoutSafe();
    setState('rejected');
    setGeneratedText('');
    setProgress(0);
    onReject?.();
  }, [clearTimeoutSafe, onReject]);

  const restartGeneration = useCallback((fullText: string) => {
    rejectGeneration();
    // 短暂延迟后重新开始
    setTimeout(() => {
      startGeneration(fullText);
    }, 200);
  }, [rejectGeneration, startGeneration]);

  const clearGeneration = useCallback(() => {
    clearTimeoutSafe();
    setState('idle');
    setGeneratedText('');
    setProgress(0);
    fullTextRef.current = '';
    currentIndexRef.current = 0;
    isPausedRef.current = false;
  }, [clearTimeoutSafe]);

  // 清理
  useEffect(() => {
    return () => {
      clearTimeoutSafe();
    };
  }, [clearTimeoutSafe]);

  return {
    state,
    generatedText,
    isGenerating: state === 'generating',
    isPaused: state === 'paused',
    progress,
    startGeneration,
    pauseGeneration,
    resumeGeneration,
    acceptGeneration,
    rejectGeneration,
    restartGeneration,
    clearGeneration,
  };
}

/**
 * 模拟后端流式生成（用于测试）
 * 实际项目中应该调用 Tauri 命令
 */
export function mockStreamGeneration(
  prompt: string,
  onChunk: (chunk: string) => void,
  onComplete: () => void
): () => void {
  const sampleTexts = [
    '夜风轻轻拂过窗棂，带来远处桂花的香气。她放下手中的笔，望向窗外那轮明月，心中涌起无限思绪。',
    '他的声音低沉而温柔，像是大提琴的最后一个音符，在空气中缓缓消散。',
    '雨点开始敲打屋顶，节奏清晰而有力，仿佛大自然在谱写一首独特的乐章。',
    '那一刻，时间仿佛静止。所有的喧嚣都远去，只剩下心跳的声音在耳畔回响。',
    '烛光摇曳，在墙上投下舞动的影子。她轻抚那本泛黄的书页，指尖传来岁月的温度。',
  ];

  // 随机选择一段文本
  const text = sampleTexts[Math.floor(Math.random() * sampleTexts.length)];
  const chars = text.split('');
  let index = 0;
  
  const interval = setInterval(() => {
    if (index < chars.length) {
      onChunk(chars[index]);
      index++;
    } else {
      clearInterval(interval);
      onComplete();
    }
  }, 50 + Math.random() * 50);

  return () => clearInterval(interval);
}
