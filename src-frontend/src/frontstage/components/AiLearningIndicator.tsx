import React from 'react';

export interface LearningPoint {
  category: string;
  observation: string;
  impact: string;
}

interface AiLearningIndicatorProps {
  learnings: LearningPoint[];
  onDismiss: () => void;
  onStrengthen: (index: number) => void;
  onIgnore: (index: number) => void;
}

export const AiLearningIndicator: React.FC<AiLearningIndicatorProps> = ({
  learnings,
  onDismiss,
  onStrengthen,
  onIgnore,
}) => {
  if (learnings.length === 0) return null;

  return (
    <div className="ai-learning-indicator">
      <div className="ai-learning-header">
        <span>我学到了这些</span>
        <button onClick={onDismiss}>×</button>
      </div>
      <div className="ai-learning-list">
        {learnings.map((point, idx) => (
          <div key={idx} className="ai-learning-item">
            <span className="ai-learning-category">{point.category}</span>
            <p className="ai-learning-observation">{point.observation}</p>
            <p className="ai-learning-impact">影响: {point.impact}</p>
            <div className="ai-learning-actions">
              <button onClick={() => onStrengthen(idx)}>强化</button>
              <button onClick={() => onIgnore(idx)}>忽略</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
