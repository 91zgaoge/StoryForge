/**
 * 智能文思 — 三层架构入口
 *
 * 感知层 (Perception): textAnalyzer.ts
 * 决策层 (Decision):   suggestionEngine.ts
 * 表达层 (Presentation): SmartHintSystem.tsx
 */

export * from './types';
export * from './textAnalyzer';
export * from './suggestionEngine';
export { default as SmartHintSystem } from './SmartHintSystem';
export { default } from './SmartHintSystem';
