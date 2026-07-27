import React, { useState } from 'react';
import { ChevronDown } from 'lucide-react';

interface PanelProps {
  title: string;
  children: React.ReactNode;
  collapsible?: boolean;
  defaultOpen?: boolean;
}

export const Panel: React.FC<PanelProps> = ({
  title,
  children,
  collapsible,
  defaultOpen = true,
}) => {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="bg-cinema-850 border border-white/[0.06] rounded-panel shadow-panel overflow-hidden">
      <div
        className={[
          'flex items-center justify-between px-4 py-3',
          'border-b border-white/[0.06]',
          collapsible ? 'cursor-pointer hover:bg-cinema-800/50' : '',
        ].join(' ')}
        onClick={collapsible ? () => setOpen(v => !v) : undefined}
      >
        <h3 className="text-xs font-bold uppercase tracking-wider text-cinema-gold/80">{title}</h3>
        {collapsible && (
          <ChevronDown
            className={[
              'w-4 h-4 text-cinema-gold/80 transition-transform duration-300 ease-spring',
              open ? 'rotate-180' : '',
            ].join(' ')}
          />
        )}
      </div>
      <div
        className={[
          'transition-all duration-300 ease-spring overflow-hidden',
          open ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0',
        ].join(' ')}
      >
        <div className="p-4">{children}</div>
      </div>
    </div>
  );
};
