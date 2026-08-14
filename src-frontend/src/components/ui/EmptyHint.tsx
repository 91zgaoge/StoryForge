import type { ReactNode } from 'react';

export function EmptyHint({ children }: { children: ReactNode }) {
  return (
    <p className="rounded-md border border-dashed border-ai-line px-4 py-6 text-center text-sm text-ai-ink-3">
      {children}
    </p>
  );
}
